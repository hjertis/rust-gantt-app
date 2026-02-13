use crate::model::{Task, TimelineScale, TimelineViewport};
use crate::ui::theme;
use chrono::{Datelike, NaiveDate};
use egui::{Color32, Id, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use uuid::Uuid;

const ROW_HEIGHT: f32 = theme::ROW_HEIGHT;
const ROW_PADDING: f32 = theme::ROW_GAP;
const HEADER_HEIGHT: f32 = theme::HEADER_HEIGHT;
const HANDLE_WIDTH: f32 = theme::HANDLE_WIDTH;

#[derive(Debug, Clone)]
struct DragSnapshot {
    start: NaiveDate,
    end: NaiveDate,
    start_pointer_x: f32,
}

/// Result details from interactions in the Gantt chart.
#[derive(Debug, Clone)]
pub struct ChartInteraction {
    pub changed: bool,
}

impl Default for ChartInteraction {
    fn default() -> Self {
        Self { changed: false }
    }
}

/// Render the Gantt chart area (right panel).
pub fn show_gantt_chart(
    tasks: &mut [Task],
    viewport: &mut TimelineViewport,
    selected_task: &mut Option<Uuid>,
    ui: &mut Ui,
) -> ChartInteraction {
    let mut interaction = ChartInteraction::default();
    let available = ui.available_size();
    let chart_width = viewport.total_width().max(available.x);
    let chart_height = HEADER_HEIGHT + (tasks.len() as f32 * (ROW_HEIGHT + ROW_PADDING)) + 40.0;

    // Handle zoom with scroll wheel
    let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
    if ui.rect_contains_pointer(ui.max_rect()) {
        if ui.input(|i| i.modifiers.ctrl) {
            if scroll_delta.y > 0.0 {
                viewport.zoom_in();
            } else if scroll_delta.y < 0.0 {
                viewport.zoom_out();
            }
        }
    }

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                Vec2::new(chart_width, chart_height.max(available.y)),
                Sense::click(),
            );
            let origin = response.rect.min;
            let mut consumed_click = false;

            // Fill entire canvas with dark background
            painter.rect_filled(
                response.rect,
                0.0,
                theme::BG_DARK,
            );

            // Draw timeline header
            draw_timeline_header(&painter, origin, viewport, chart_width);

            // Draw today line
            draw_today_line(&painter, origin, viewport, chart_height);

            // Draw alternating row backgrounds
            for (i, _task) in tasks.iter().enumerate() {
                let y = origin.y + HEADER_HEIGHT + i as f32 * (ROW_HEIGHT + ROW_PADDING);
                let row_bg = if i % 2 == 0 {
                    theme::BG_PANEL  // slightly lighter dark
                } else {
                    theme::BG_DARK   // base dark
                };
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(origin.x, y),
                        Vec2::new(chart_width, ROW_HEIGHT + ROW_PADDING),
                    ),
                    0.0,
                    row_bg,
                );
                // Row bottom border
                painter.line_segment(
                    [
                        Pos2::new(origin.x, y + ROW_HEIGHT + ROW_PADDING),
                        Pos2::new(origin.x + chart_width, y + ROW_HEIGHT + ROW_PADDING),
                    ],
                    Stroke::new(0.5, theme::BORDER_SUBTLE),
                );
            }

            // Draw task bars
            for (i, task) in tasks.iter_mut().enumerate() {
                let y =
                    origin.y + HEADER_HEIGHT + i as f32 * (ROW_HEIGHT + ROW_PADDING) + ROW_PADDING;
                let is_selected = *selected_task == Some(task.id);

                if task.is_milestone {
                    let task_rect = draw_milestone(&painter, origin, viewport, task, y, is_selected);
                    let response = ui.interact(
                        task_rect.expand(6.0),
                        ui.make_persistent_id(("milestone", task.id)),
                        Sense::click_and_drag(),
                    );

                    if response.clicked() {
                        *selected_task = Some(task.id);
                        consumed_click = true;
                    }

                    if response.drag_started() {
                        let ptr_x = response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "milestone"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr_x,
                                },
                            );
                        });
                    }

                    if response.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                        let ptr_x = response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        let snapshot = ui.ctx().data_mut(|data| {
                            data.get_persisted::<DragSnapshot>(drag_id(task.id, "milestone"))
                        });
                        if let Some(snapshot) = snapshot {
                            let total_delta_x = ptr_x - snapshot.start_pointer_x;
                            let day_delta = drag_days(total_delta_x, viewport);
                            task.start = snapshot.start + chrono::Duration::days(day_delta);
                            task.end = task.start;
                            interaction.changed = true;
                            *selected_task = Some(task.id);
                        }
                    }

                    if response.drag_stopped() {
                        ui.ctx().data_mut(|data| {
                            data.remove::<DragSnapshot>(drag_id(task.id, "milestone"));
                        });
                    }

                    // Tooltip on hover
                    if response.hovered() {
                        egui::show_tooltip_at_pointer(
                            ui.ctx(),
                            ui.layer_id(),
                            egui::Id::new(("milestone-tip", task.id)),
                            |ui| {
                                ui.strong(&task.name);
                                ui.label(task.start.format("%d/%m/%Y").to_string());
                                ui.label(format!("Progress: {}%", (task.progress * 100.0) as i32));
                            },
                        );
                    }
                } else {
                    let bar_rect = draw_task_bar(&painter, origin, viewport, task, y, is_selected);

                    let bar_response = ui.interact(
                        bar_rect,
                        ui.make_persistent_id(("task-bar", task.id)),
                        Sense::click_and_drag(),
                    );
                    let left_handle_rect = Rect::from_min_max(
                        Pos2::new(bar_rect.left() - HANDLE_WIDTH * 0.5, bar_rect.top()),
                        Pos2::new(bar_rect.left() + HANDLE_WIDTH * 0.5, bar_rect.bottom()),
                    );
                    let right_handle_rect = Rect::from_min_max(
                        Pos2::new(bar_rect.right() - HANDLE_WIDTH * 0.5, bar_rect.top()),
                        Pos2::new(bar_rect.right() + HANDLE_WIDTH * 0.5, bar_rect.bottom()),
                    );

                    let left_response = ui.interact(
                        left_handle_rect.expand(4.0),
                        ui.make_persistent_id(("task-resize-left", task.id)),
                        Sense::drag(),
                    );
                    let right_response = ui.interact(
                        right_handle_rect.expand(4.0),
                        ui.make_persistent_id(("task-resize-right", task.id)),
                        Sense::drag(),
                    );

                    if bar_response.clicked() {
                        *selected_task = Some(task.id);
                        consumed_click = true;
                    }

                    if left_response.drag_started() {
                        let ptr_x = left_response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "left"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr_x,
                                },
                            );
                        });
                    }
                    if right_response.drag_started() {
                        let ptr_x = right_response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "right"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr_x,
                                },
                            );
                        });
                    }
                    if bar_response.drag_started() {
                        let ptr_x = bar_response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "move"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr_x,
                                },
                            );
                        });
                    }

                    if bar_response.drag_started() || left_response.drag_started() || right_response.drag_started() {
                        *selected_task = Some(task.id);
                        consumed_click = true;
                    }

                    if left_response.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        let ptr_x = left_response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        let snapshot = ui
                            .ctx()
                            .data_mut(|data| data.get_persisted::<DragSnapshot>(drag_id(task.id, "left")));
                        if let Some(snapshot) = snapshot {
                            let total_delta_x = ptr_x - snapshot.start_pointer_x;
                            let day_delta = drag_days(total_delta_x, viewport);
                            let new_start = snapshot.start + chrono::Duration::days(day_delta);
                            task.start = new_start.min(snapshot.end);
                            task.end = snapshot.end.max(task.start);
                            interaction.changed = true;
                        }
                    } else if right_response.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        let ptr_x = right_response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        let snapshot = ui
                            .ctx()
                            .data_mut(|data| data.get_persisted::<DragSnapshot>(drag_id(task.id, "right")));
                        if let Some(snapshot) = snapshot {
                            let total_delta_x = ptr_x - snapshot.start_pointer_x;
                            let day_delta = drag_days(total_delta_x, viewport);
                            let new_end = snapshot.end + chrono::Duration::days(day_delta);
                            task.end = new_end.max(snapshot.start);
                            interaction.changed = true;
                        }
                    } else if bar_response.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                        let ptr_x = bar_response.interact_pointer_pos().map(|p| p.x).unwrap_or(0.0);
                        let snapshot = ui
                            .ctx()
                            .data_mut(|data| data.get_persisted::<DragSnapshot>(drag_id(task.id, "move")));
                        if let Some(snapshot) = snapshot {
                            let total_delta_x = ptr_x - snapshot.start_pointer_x;
                            let day_delta = drag_days(total_delta_x, viewport);
                            task.start = snapshot.start + chrono::Duration::days(day_delta);
                            task.end = snapshot.end + chrono::Duration::days(day_delta);
                            interaction.changed = true;
                        }
                    }

                    if left_response.drag_stopped() {
                        ui.ctx().data_mut(|data| {
                            data.remove::<DragSnapshot>(drag_id(task.id, "left"));
                        });
                    }
                    if right_response.drag_stopped() {
                        ui.ctx().data_mut(|data| {
                            data.remove::<DragSnapshot>(drag_id(task.id, "right"));
                        });
                    }
                    if bar_response.drag_stopped() {
                        ui.ctx().data_mut(|data| {
                            data.remove::<DragSnapshot>(drag_id(task.id, "move"));
                        });
                    }

                    // Handle affordances
                    if is_selected || left_response.hovered() || right_response.hovered() {
                        if left_response.hovered() || right_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        } else if bar_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        // Draw rounded pill handles
                        let handle_h = bar_rect.height() * 0.55;
                        let handle_y = bar_rect.center().y - handle_h / 2.0;
                        let lh = Rect::from_min_size(
                            Pos2::new(bar_rect.left() - 1.5, handle_y),
                            Vec2::new(4.0, handle_h),
                        );
                        let rh = Rect::from_min_size(
                            Pos2::new(bar_rect.right() - 2.5, handle_y),
                            Vec2::new(4.0, handle_h),
                        );
                        painter.rect_filled(lh, Rounding::same(2.0), theme::HANDLE_COLOR);
                        painter.rect_filled(rh, Rounding::same(2.0), theme::HANDLE_COLOR);
                    }

                    // Tooltip on hover
                    if bar_response.hovered() || left_response.hovered() || right_response.hovered() {
                        egui::show_tooltip_at_pointer(
                            ui.ctx(),
                            ui.layer_id(),
                            egui::Id::new(("task-tip", task.id)),
                            |ui| {
                                ui.strong(&task.name);
                                ui.label(format!(
                                    "{} → {}",
                                    task.start.format("%d/%m/%Y"),
                                    task.end.format("%d/%m/%Y"),
                                ));
                                ui.label(format!("Progress: {}%", (task.progress * 100.0) as i32));
                            },
                        );
                    }
                }
            }

            // Empty click on background clears selection
            if response.clicked() && !consumed_click {
                *selected_task = None;
            }
        });

    interaction
}

fn drag_id(task_id: Uuid, mode: &'static str) -> Id {
    Id::new(("drag", task_id, mode))
}

fn drag_days(delta_x: f32, viewport: &TimelineViewport) -> i64 {
    (delta_x / viewport.pixels_per_day).round() as i64
}

fn draw_timeline_header(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    width: f32,
) {
    // Background for header
    painter.rect_filled(
        Rect::from_min_size(origin, Vec2::new(width, HEADER_HEIGHT)),
        0.0,
        theme::BG_HEADER,
    );

    // Bottom border of header
    painter.line_segment(
        [
            Pos2::new(origin.x, origin.y + HEADER_HEIGHT),
            Pos2::new(origin.x + width, origin.y + HEADER_HEIGHT),
        ],
        Stroke::new(1.0, theme::BORDER_SUBTLE),
    );

    let mut date = viewport.start;
    let end = viewport.end;

    match viewport.scale {
        TimelineScale::Days => {
            while date <= end {
                let x = origin.x + viewport.date_to_x(date);

                painter.line_segment(
                    [
                        Pos2::new(x, origin.y + HEADER_HEIGHT),
                        Pos2::new(x, origin.y + 2000.0),
                    ],
                    Stroke::new(0.5, theme::GRID_LINE),
                );

                if viewport.pixels_per_day >= 20.0 {
                    let is_weekend = date.weekday().num_days_from_monday() >= 5;
                    let day_color = if is_weekend {
                        theme::TEXT_DIM
                    } else {
                        theme::TEXT_SECONDARY
                    };
                    painter.text(
                        Pos2::new(x + 3.0, origin.y + 28.0),
                        egui::Align2::LEFT_CENTER,
                        date.format("%d").to_string(),
                        theme::font_sub(),
                        day_color,
                    );
                }

                if date.day() == 1 {
                    painter.text(
                        Pos2::new(x + 3.0, origin.y + 12.0),
                        egui::Align2::LEFT_CENTER,
                        date.format("%b %Y").to_string(),
                        theme::font_header(),
                        theme::TEXT_PRIMARY,
                    );
                }

                date += chrono::Duration::days(1);
            }
        }
        TimelineScale::Weeks => {
            let weekday = date.weekday().num_days_from_monday();
            date -= chrono::Duration::days(weekday as i64);

            while date <= end {
                let x = origin.x + viewport.date_to_x(date);

                painter.line_segment(
                    [
                        Pos2::new(x, origin.y + HEADER_HEIGHT),
                        Pos2::new(x, origin.y + 2000.0),
                    ],
                    Stroke::new(0.5, theme::GRID_LINE),
                );

                painter.text(
                    Pos2::new(x + 3.0, origin.y + 28.0),
                    egui::Align2::LEFT_CENTER,
                    date.format("W%V").to_string(),
                    theme::font_sub(),
                    theme::TEXT_SECONDARY,
                );

                if date.day() <= 7 {
                    painter.text(
                        Pos2::new(x + 3.0, origin.y + 12.0),
                        egui::Align2::LEFT_CENTER,
                        date.format("%b %Y").to_string(),
                        theme::font_header(),
                        theme::TEXT_PRIMARY,
                    );
                }

                date += chrono::Duration::days(7);
            }
        }
        TimelineScale::Months => {
            date = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);

            while date <= end {
                let x = origin.x + viewport.date_to_x(date);

                painter.line_segment(
                    [
                        Pos2::new(x, origin.y + HEADER_HEIGHT),
                        Pos2::new(x, origin.y + 2000.0),
                    ],
                    Stroke::new(0.5, theme::GRID_LINE),
                );

                painter.text(
                    Pos2::new(x + 5.0, origin.y + 18.0),
                    egui::Align2::LEFT_CENTER,
                    date.format("%b %Y").to_string(),
                    theme::font_header(),
                    theme::TEXT_PRIMARY,
                );

                let (y, m) = if date.month() == 12 {
                    (date.year() + 1, 1)
                } else {
                    (date.year(), date.month() + 1)
                };
                date = NaiveDate::from_ymd_opt(y, m, 1)
                    .unwrap_or(date + chrono::Duration::days(30));
            }
        }
    }
}

fn draw_scale_badge(painter: &egui::Painter, origin: Pos2, _viewport: &TimelineViewport) {
    let text = "Drag bars to move · Drag edges to resize · Ctrl+Scroll to zoom";

    let galley = painter.layout_no_wrap(
        text.to_string(),
        theme::font_small(),
        theme::TEXT_DIM,
    );
    let text_width = galley.size().x;

    let badge_rect = Rect::from_min_size(
        Pos2::new(origin.x + 8.0, origin.y + HEADER_HEIGHT - 18.0),
        Vec2::new(text_width + 16.0, 16.0),
    );
    painter.rect_filled(
        badge_rect,
        Rounding::same(8.0),
        Color32::from_rgba_premultiplied(20, 20, 28, 200),
    );
    painter.galley(
        Pos2::new(badge_rect.left() + 8.0, badge_rect.top()),
        galley,
        Color32::TRANSPARENT,
    );
}

fn draw_today_line(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    height: f32,
) {
    let today = chrono::Local::now().date_naive();
    let x = origin.x + viewport.date_to_x(today);

    // Dashed-feel: slightly transparent line with a brighter cap
    painter.line_segment(
        [
            Pos2::new(x, origin.y + HEADER_HEIGHT),
            Pos2::new(x, origin.y + height),
        ],
        Stroke::new(1.5, theme::TODAY_LINE),
    );

    // Top badge
    let badge_w = 42.0;
    let badge_rect = Rect::from_min_size(
        Pos2::new(x - badge_w / 2.0, origin.y + HEADER_HEIGHT - 1.0),
        Vec2::new(badge_w, 14.0),
    );
    painter.rect_filled(badge_rect, Rounding::same(3.0), theme::TODAY_LINE);
    painter.text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        "Today",
        theme::font_small(),
        Color32::WHITE,
    );
}

fn draw_task_bar(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    task: &Task,
    y: f32,
    is_selected: bool,
) -> Rect {
    let x_start = origin.x + viewport.date_to_x(task.start);
    let x_end = origin.x + viewport.date_to_x(task.end);
    let bar_width = (x_end - x_start).max(6.0);
    let inset = theme::BAR_INSET;

    let bar_rect = Rect::from_min_size(
        Pos2::new(x_start, y + inset),
        Vec2::new(bar_width, ROW_HEIGHT - inset * 2.0),
    );
    let rounding = Rounding::same(theme::BAR_ROUNDING);

    // Soft shadow
    let shadow_rect = bar_rect.translate(Vec2::new(1.0, 2.0));
    painter.rect_filled(shadow_rect, rounding, Color32::from_black_alpha(35));

    // Main bar — slight gradient effect via two overlapping rects
    painter.rect_filled(bar_rect, rounding, task.color);
    // Lighter top highlight
    let highlight_rect = Rect::from_min_size(
        bar_rect.min,
        Vec2::new(bar_width, (bar_rect.height() * 0.45).max(4.0)),
    );
    painter.rect_filled(
        highlight_rect,
        Rounding {
            nw: theme::BAR_ROUNDING,
            ne: theme::BAR_ROUNDING,
            sw: 0.0,
            se: 0.0,
        },
        Color32::from_white_alpha(25),
    );

    // Progress fill (darkened overlay)
    if task.progress > 0.0 {
        let progress_width = bar_width * task.progress.clamp(0.0, 1.0);
        let progress_rect = Rect::from_min_size(
            bar_rect.min,
            Vec2::new(progress_width, bar_rect.height()),
        );
        painter.rect_filled(progress_rect, rounding, theme::PROGRESS_OVERLAY);

        // Progress divider tick
        if task.progress < 0.98 {
            let tick_x = bar_rect.left() + progress_width;
            painter.line_segment(
                [
                    Pos2::new(tick_x, bar_rect.top() + 2.0),
                    Pos2::new(tick_x, bar_rect.bottom() - 2.0),
                ],
                Stroke::new(1.0, Color32::from_white_alpha(60)),
            );
        }
    }

    // Selection glow
    if is_selected {
        painter.rect_stroke(
            bar_rect.expand(1.5),
            Rounding::same(theme::BAR_ROUNDING + 1.5),
            Stroke::new(2.0, theme::BORDER_ACCENT),
        );
    }

    // Task name on bar (single line, clipped to bar bounds)
    if bar_width > 30.0 {
        let galley = painter.layout_no_wrap(
            task.name.clone(),
            theme::font_bar(),
            theme::TEXT_ON_BAR,
        );
        let clipped = painter.with_clip_rect(bar_rect);
        let text_y = y + inset + (bar_rect.height() - galley.size().y) / 2.0;
        clipped.galley(
            Pos2::new(bar_rect.left() + 6.0, text_y),
            galley,
            Color32::TRANSPARENT,
        );
    }

    bar_rect
}

fn draw_milestone(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    task: &Task,
    y: f32,
    is_selected: bool,
) -> Rect {
    let x = origin.x + viewport.date_to_x(task.start);
    let center = Pos2::new(x, y + ROW_HEIGHT / 2.0);
    let size = (ROW_HEIGHT / 2.0 - 3.0).max(6.0);

    // Shadow diamond
    let shadow_offset = Vec2::new(1.0, 1.5);
    let shadow_pts = vec![
        center + shadow_offset + Vec2::new(0.0, -size),
        center + shadow_offset + Vec2::new(size, 0.0),
        center + shadow_offset + Vec2::new(0.0, size),
        center + shadow_offset + Vec2::new(-size, 0.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        shadow_pts,
        Color32::from_black_alpha(40),
        Stroke::NONE,
    ));

    // Main diamond
    let points = vec![
        Pos2::new(center.x, center.y - size),
        Pos2::new(center.x + size, center.y),
        Pos2::new(center.x, center.y + size),
        Pos2::new(center.x - size, center.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        points.clone(),
        task.color,
        Stroke::NONE,
    ));

    if is_selected {
        painter.add(egui::Shape::convex_polygon(
            points,
            Color32::TRANSPARENT,
            Stroke::new(2.0, theme::BORDER_ACCENT),
        ));
    }

    // Label
    painter.text(
        Pos2::new(x + size + 6.0, y + ROW_HEIGHT / 2.0),
        egui::Align2::LEFT_CENTER,
        &task.name,
        theme::font_bar(),
        theme::TEXT_SECONDARY,
    );

    Rect::from_center_size(center, Vec2::splat(size * 2.0 + 2.0))
}
