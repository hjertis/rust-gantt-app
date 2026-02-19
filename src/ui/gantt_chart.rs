use crate::model::{Task, TimelineScale, TimelineViewport};
use crate::model::task::{Dependency, DependencyKind};
use crate::ui::theme;
use chrono::{Datelike, NaiveDate};
use egui::{Color32, Id, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use uuid::Uuid;

fn header_height() -> f32 { theme::header_height() }

#[derive(Debug, Clone)]
struct DragSnapshot {
    start: NaiveDate,
    end: NaiveDate,
    start_pointer_x: f32,
    start_pointer_y: f32,
}

/// Result details from interactions in the Gantt chart.
#[derive(Debug, Clone)]
pub struct ChartInteraction {
    pub changed: bool,
    /// A new dependency to add (created via Shift+drag).
    pub new_dependency: Option<Dependency>,
    /// A dependency to remove (right-clicked on arrow).
    pub remove_dependency: Option<(Uuid, Uuid)>,
    /// A parent task whose collapsed state should be toggled.
    pub toggle_collapse: Option<Uuid>,
    /// Request to add a subtask under this parent id.
    pub add_subtask: Option<Uuid>,
    /// Request to delete this task.
    pub delete_task: Option<Uuid>,
}

impl Default for ChartInteraction {
    fn default() -> Self {
        Self {
            changed: false,
            new_dependency: None,
            remove_dependency: None,
            toggle_collapse: None,
            add_subtask: None,
            delete_task: None,
        }
    }
}

/// State for creating a dependency link via Shift+drag.
#[derive(Debug, Clone)]
struct LinkDragState {
    from_task: Uuid,
    from_point: Pos2,
}

/// Render the Gantt chart area (right panel).
pub fn show_gantt_chart(
    tasks: &mut [Task],
    dependencies: &[Dependency],
    viewport: &mut TimelineViewport,
    selected_task: &mut Option<Uuid>,
    ui: &mut Ui,
) -> ChartInteraction {
    let mut interaction = ChartInteraction::default();
    let available = ui.available_size();
    let row_height = scaled_row_height(viewport);
    let row_padding = scaled_row_padding(viewport);
    let chart_width = viewport.total_width().max(available.x);
    let hh = header_height();

    // Build the list of visible task indices, skipping children of collapsed parents.
    let visible_rows: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            if let Some(pid) = t.parent_id {
                let parent_collapsed = tasks.iter().find(|p| p.id == pid).map(|p| p.collapsed).unwrap_or(false);
                if parent_collapsed { return None; }
            }
            Some(i)
        })
        .collect();

    let chart_height = hh + (visible_rows.len() as f32 * (row_height + row_padding)) + 40.0;

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                Vec2::new(chart_width, chart_height.max(available.y)),
                Sense::click(),
            );

            // Handle Ctrl+Scroll zoom when pointer is over chart.
            if response.hovered() && ui.input(|i| i.modifiers.ctrl) {
                let raw_scroll_y = ui.input(|i| i.raw_scroll_delta.y);
                let smooth_scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
                let scroll_y = if raw_scroll_y.abs() > smooth_scroll_y.abs() {
                    raw_scroll_y
                } else {
                    smooth_scroll_y
                };

                if scroll_y > 0.0 {
                    viewport.zoom_in();
                } else if scroll_y < 0.0 {
                    viewport.zoom_out();
                }
            }

            let row_height = scaled_row_height(viewport);
            let row_padding = scaled_row_padding(viewport);
            let handle_width = scaled_handle_width(viewport);

            let origin = response.rect.min;
            let mut consumed_click = false;
            let shift_held = ui.input(|i| i.modifiers.shift);
            let mut reorder_request: Option<(usize, usize)> = None;
            let mut reorder_preview_target: Option<usize> = None;

            // Fill entire canvas with dark background
            painter.rect_filled(
                response.rect,
                0.0,
                theme::bg_dark(),
            );

            // Draw alternating row backgrounds (only for visible rows)
            for (vis_i, &task_i) in visible_rows.iter().enumerate() {
                let _task = &tasks[task_i];
                let y = origin.y + hh + vis_i as f32 * (row_height + row_padding);
                let row_bg = if vis_i % 2 == 0 {
                    theme::bg_panel()
                } else {
                    theme::bg_dark()
                };
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(origin.x, y),
                        Vec2::new(chart_width, row_height + row_padding),
                    ),
                    0.0,
                    row_bg,
                );
                // Row bottom border
                painter.line_segment(
                    [
                        Pos2::new(origin.x, y + row_height + row_padding),
                        Pos2::new(origin.x + chart_width, y + row_height + row_padding),
                    ],
                    Stroke::new(0.5, theme::border_subtle()),
                );
            }

            // Shade weekends in the gantt body so they stand out clearly.
            draw_weekend_bands(
                &painter,
                origin,
                viewport,
                chart_width,
                origin.y + chart_height,
            );

            // Draw timeline header in content space
            draw_timeline_header(
                &painter,
                origin,
                viewport,
                chart_width,
                origin.y + chart_height,
            );

            // Animated row Y positions for smooth reorder transitions.
            // Only visible rows get a Y slot; collapsed children are not assigned a Y.
            let anim_dur = theme::reorder_anim_duration();
            let mut animated_row_y: std::collections::HashMap<Uuid, f32> =
                std::collections::HashMap::with_capacity(visible_rows.len());
            let mut animating_rows = false;
            for (vis_i, &task_i) in visible_rows.iter().enumerate() {
                let task = &tasks[task_i];
                let target_y =
                    origin.y + hh + vis_i as f32 * (row_height + row_padding) + row_padding;
                let anim_id = Id::new(("row-y", task.id));
                let animated_y = ui.ctx().animate_value_with_time(anim_id, target_y, anim_dur);
                if (animated_y - target_y).abs() > 0.25 {
                    animating_rows = true;
                }
                animated_row_y.insert(task.id, animated_y);
            }
            if animating_rows {
                ui.ctx().request_repaint();
            }

            // ── Calculate task positions (for dependencies) ──────────
            // Only visible rows get a position entry.
            let task_positions: std::collections::HashMap<Uuid, (usize, Rect)> = visible_rows
                .iter()
                .enumerate()
                .map(|(vis_i, &task_i)| {
                    let task = &tasks[task_i];
                    let y = *animated_row_y.get(&task.id).unwrap_or(
                        &(origin.y + hh + vis_i as f32 * (row_height + row_padding) + row_padding),
                    );
                    let inset = theme::bar_inset();
                    if task.is_milestone {
                        let x = origin.x + viewport.date_to_x(task.start);
                        let size = (row_height / 2.0 - 3.0).max(6.0);
                        let center = Pos2::new(x, y + row_height / 2.0);
                        (task.id, (vis_i, Rect::from_center_size(center, Vec2::splat(size * 2.0))))
                    } else {
                        let x_start = origin.x + viewport.date_to_x(task.start);
                        let x_end = origin.x + viewport.date_to_x(task.end);
                        let bar_width = (x_end - x_start).max(6.0);
                        let bar_rect = Rect::from_min_size(
                            Pos2::new(x_start, y + inset),
                            Vec2::new(bar_width, row_height - inset * 2.0),
                        );
                        (task.id, (vis_i, bar_rect))
                    }
                })
                .collect();

            // ── Draw dependency arrows (BEHIND bars) ─────────────────
            for dep in dependencies {
                if let (Some(&(_, from_rect)), Some(&(_, to_rect))) =
                    (task_positions.get(&dep.from_task), task_positions.get(&dep.to_task))
                {
                    let (start_pt, end_pt) = dependency_endpoints(from_rect, to_rect, dep.kind);
                    draw_dependency_arrow(&painter, start_pt, end_pt, dep.kind, with_alpha(theme::dep_arrow(), 180), 1.4);
                }
            }

            let mut hovered_task: Option<Uuid> = None;

            // Draw task bars — iterate only visible rows.
            let vis_count = visible_rows.len();
            for (vis_i, &task_i) in visible_rows.iter().enumerate() {
                // Safety: split borrow so we can read siblings while mutating task.
                let task_id = tasks[task_i].id;
                let task_parent_id = tasks[task_i].parent_id;
                let is_parent_task = tasks[task_i].has_children(tasks);

                let y = *animated_row_y.get(&task_id).unwrap_or(
                    &(origin.y + hh + vis_i as f32 * (row_height + row_padding) + row_padding),
                );
                let is_selected = *selected_task == Some(task_id);

                if is_parent_task {
                    // ── Summary / parent bar ─────────────────────────
                    let task = &tasks[task_i];
                    let summary_rect = draw_summary_bar(&painter, origin, viewport, task, y, row_height, is_selected);

                    // Collapse/expand toggle button (small triangle to the left of bar)
                    let toggle_x = summary_rect.left() - 14.0;
                    let toggle_center = Pos2::new(toggle_x, y + row_height / 2.0);
                    let toggle_rect = Rect::from_center_size(toggle_center, Vec2::splat(14.0));
                    let tri = if task.collapsed { egui_phosphor::regular::CARET_RIGHT } else { egui_phosphor::regular::CARET_DOWN };
                    painter.text(
                        toggle_center,
                        egui::Align2::CENTER_CENTER,
                        tri,
                        egui::FontId::proportional(9.0),
                        theme::text_dim(),
                    );
                    let toggle_resp = ui.interact(
                        toggle_rect,
                        ui.make_persistent_id(("collapse-toggle", task_id)),
                        Sense::click(),
                    );
                    if toggle_resp.clicked() {
                        interaction.toggle_collapse = Some(task_id);
                        consumed_click = true;
                    }

                    // Click on bar selects it; right-click shows context menu
                    let summary_resp = ui.interact(
                        summary_rect,
                        ui.make_persistent_id(("summary-bar", task_id)),
                        Sense::click_and_drag(),
                    );
                    if summary_resp.clicked() {
                        *selected_task = Some(task_id);
                        consumed_click = true;
                    }
                    if summary_resp.secondary_clicked() {
                        let open_pos = ui.input(|i| i.pointer.interact_pos().unwrap_or(summary_rect.center()));
                        ui.ctx().data_mut(|d| d.insert_temp(Id::new(("ctx-menu", task_id)), open_pos));
                    }
                    // Context menu for parent tasks
                    let ctx_pos: Option<Pos2> = ui.ctx().data_mut(|d| d.get_temp(Id::new(("ctx-menu", task_id))));
                    if let Some(open_pos) = ctx_pos {
                        let mut close_menu = false;
                        egui::Area::new(Id::new(("ctx-area", task_id)))
                            .fixed_pos(open_pos)
                            .order(egui::Order::Foreground)
                            .show(ui.ctx(), |ui| {
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    if ui.button(egui_phosphor::regular::PLUS.to_string() + "  Add Subtask").clicked() {
                                        interaction.add_subtask = Some(task_id);
                                        close_menu = true;
                                    }
                                    if ui.button(egui_phosphor::regular::TRASH.to_string() + "  Delete Group").clicked() {
                                        interaction.delete_task = Some(task_id);
                                        close_menu = true;
                                    }
                                });
                            });
                        if close_menu || ui.input(|i| i.pointer.secondary_pressed()) {
                            ui.ctx().data_mut(|d| d.remove::<Pos2>(Id::new(("ctx-menu", task_id))));
                        }
                    }

                    if summary_resp.hovered() {
                        hovered_task = Some(task_id);
                        egui::show_tooltip_at_pointer(
                            ui.ctx(),
                            ui.layer_id(),
                            egui::Id::new(("summary-tip", task_id)),
                            |ui| {
                                let task = &tasks[task_i];
                                ui.strong(&task.name);
                                ui.label(format!(
                                    "{} → {}",
                                    task.start.format("%d/%m/%Y"),
                                    task.end.format("%d/%m/%Y"),
                                ));
                                ui.label(format!("Progress: {}%", (task.progress * 100.0) as i32));
                                ui.label(egui::RichText::new("Right-click for options").size(9.0).color(theme::text_dim()));
                            },
                        );
                    }
                } else if tasks[task_i].is_milestone {
                    let task = &mut tasks[task_i];
                    let task_rect = draw_milestone(&painter, origin, viewport, task, y, row_height, is_selected);
                    let response = ui.interact(
                        task_rect.expand(6.0),
                        ui.make_persistent_id(("milestone", task.id)),
                        Sense::click_and_drag(),
                    );

                    if response.clicked() {
                        *selected_task = Some(task.id);
                        consumed_click = true;
                    }
                    // Right-click context menu for milestones
                    if response.secondary_clicked() {
                        let open_pos = ui.input(|i| i.pointer.interact_pos().unwrap_or(task_rect.center()));
                        ui.ctx().data_mut(|d| d.insert_temp(Id::new(("ctx-menu", task.id)), open_pos));
                    }
                    let ctx_pos: Option<Pos2> = ui.ctx().data_mut(|d| d.get_temp(Id::new(("ctx-menu", task.id))));
                    if let Some(open_pos) = ctx_pos {
                        let mut close_menu = false;
                        let tid = task.id;
                        let is_child = task_parent_id.is_some();
                        egui::Area::new(Id::new(("ctx-area", tid)))
                            .fixed_pos(open_pos)
                            .order(egui::Order::Foreground)
                            .show(ui.ctx(), |ui| {
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    if !is_child {
                                        if ui.button(egui_phosphor::regular::PLUS.to_string() + "  Add Subtask").clicked() {
                                            interaction.add_subtask = Some(tid);
                                            close_menu = true;
                                        }
                                    }
                                    if ui.button(egui_phosphor::regular::TRASH.to_string() + "  Delete Task").clicked() {
                                        interaction.delete_task = Some(tid);
                                        close_menu = true;
                                    }
                                });
                            });
                        if close_menu || ui.input(|i| i.pointer.secondary_pressed()) {
                            ui.ctx().data_mut(|d| d.remove::<Pos2>(Id::new(("ctx-menu", tid))));
                        }
                    }

                    if response.drag_started() && !shift_held {
                        let ptr = response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "milestone"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr.x,
                                    start_pointer_y: ptr.y,
                                },
                            );
                        });
                    }

                    if response.dragged() && !shift_held {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                        let ptr = response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                        let snapshot = ui.ctx().data_mut(|data| {
                            data.get_persisted::<DragSnapshot>(drag_id(task.id, "milestone"))
                        });
                        if let Some(snapshot) = snapshot {
                            let delta_x = ptr.x - snapshot.start_pointer_x;
                            let delta_y = ptr.y - snapshot.start_pointer_y;
                            let is_reorder_drag =
                                delta_y.abs() > row_height * 0.45 && delta_y.abs() > delta_x.abs();

                            if is_reorder_drag {
                                if let Some(target_vis) = row_index_from_pointer_y(
                                    ptr.y,
                                    origin,
                                    row_height,
                                    row_padding,
                                    vis_count,
                                ) {
                                    reorder_preview_target = Some(target_vis);
                                    if target_vis != vis_i {
                                        let target_task_i = visible_rows[target_vis];
                                        reorder_request = Some((task_i, target_task_i));
                                    }
                                }
                            } else {
                                let day_delta = drag_days(delta_x, viewport);
                                task.start = snapshot.start + chrono::Duration::days(day_delta);
                                task.end = task.start;
                                interaction.changed = true;
                                *selected_task = Some(task.id);
                            }
                        }
                    }

                    if response.drag_stopped() {
                        ui.ctx().data_mut(|data| {
                            data.remove::<DragSnapshot>(drag_id(task.id, "milestone"));
                        });
                    }

                    if response.hovered() {
                        hovered_task = Some(task.id);
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
                    let task = &mut tasks[task_i];
                    let bar_rect = draw_task_bar(&painter, origin, viewport, task, y, row_height, is_selected);

                    let bar_response = ui.interact(
                        bar_rect,
                        ui.make_persistent_id(("task-bar", task.id)),
                        Sense::click_and_drag(),
                    );
                    let left_handle_rect = Rect::from_min_max(
                        Pos2::new(bar_rect.left() - handle_width * 0.5, bar_rect.top()),
                        Pos2::new(bar_rect.left() + handle_width * 0.5, bar_rect.bottom()),
                    );
                    let right_handle_rect = Rect::from_min_max(
                        Pos2::new(bar_rect.right() - handle_width * 0.5, bar_rect.top()),
                        Pos2::new(bar_rect.right() + handle_width * 0.5, bar_rect.bottom()),
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

                    // Right-click context menu for regular tasks
                    if bar_response.secondary_clicked() {
                        let open_pos = ui.input(|i| i.pointer.interact_pos().unwrap_or(bar_rect.center()));
                        ui.ctx().data_mut(|d| d.insert_temp(Id::new(("ctx-menu", task.id)), open_pos));
                    }
                    let ctx_pos: Option<Pos2> = ui.ctx().data_mut(|d| d.get_temp(Id::new(("ctx-menu", task.id))));
                    if let Some(open_pos) = ctx_pos {
                        let mut close_menu = false;
                        let tid = task.id;
                        let is_child = task_parent_id.is_some();
                        egui::Area::new(Id::new(("ctx-area", tid)))
                            .fixed_pos(open_pos)
                            .order(egui::Order::Foreground)
                            .show(ui.ctx(), |ui| {
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    if !is_child {
                                        if ui.button(egui_phosphor::regular::PLUS.to_string() + "  Add Subtask").clicked() {
                                            interaction.add_subtask = Some(tid);
                                            close_menu = true;
                                        }
                                    }
                                    if ui.button(egui_phosphor::regular::TRASH.to_string() + "  Delete Task").clicked() {
                                        interaction.delete_task = Some(tid);
                                        close_menu = true;
                                    }
                                });
                            });
                        if close_menu || ui.input(|i| i.pointer.secondary_pressed()) {
                            ui.ctx().data_mut(|d| d.remove::<Pos2>(Id::new(("ctx-menu", tid))));
                        }
                    }

                    if left_response.drag_started() && !shift_held {
                        let ptr = left_response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "left"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr.x,
                                    start_pointer_y: ptr.y,
                                },
                            );
                        });
                    }
                    if right_response.drag_started() && !shift_held {
                        let ptr = right_response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "right"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr.x,
                                    start_pointer_y: ptr.y,
                                },
                            );
                        });
                    }
                    if bar_response.drag_started() && !shift_held {
                        let ptr = bar_response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                        ui.ctx().data_mut(|data| {
                            data.insert_persisted(
                                drag_id(task.id, "move"),
                                DragSnapshot {
                                    start: task.start,
                                    end: task.end,
                                    start_pointer_x: ptr.x,
                                    start_pointer_y: ptr.y,
                                },
                            );
                        });
                    }

                    if bar_response.drag_started() || left_response.drag_started() || right_response.drag_started() {
                        *selected_task = Some(task.id);
                        consumed_click = true;
                    }

                    if left_response.dragged() && !shift_held {
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
                    } else if right_response.dragged() && !shift_held {
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
                    } else if bar_response.dragged() && !shift_held {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                        let ptr = bar_response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                        let snapshot = ui
                            .ctx()
                            .data_mut(|data| data.get_persisted::<DragSnapshot>(drag_id(task.id, "move")));
                        if let Some(snapshot) = snapshot {
                            let delta_x = ptr.x - snapshot.start_pointer_x;
                            let delta_y = ptr.y - snapshot.start_pointer_y;
                            let is_reorder_drag =
                                delta_y.abs() > row_height * 0.45 && delta_y.abs() > delta_x.abs();

                            if is_reorder_drag {
                                if let Some(target_vis) = row_index_from_pointer_y(
                                    ptr.y,
                                    origin,
                                    row_height,
                                    row_padding,
                                    vis_count,
                                ) {
                                    reorder_preview_target = Some(target_vis);
                                    if target_vis != vis_i {
                                        let target_task_i = visible_rows[target_vis];
                                        reorder_request = Some((task_i, target_task_i));
                                    }
                                }
                            } else {
                                let day_delta = drag_days(delta_x, viewport);
                                task.start = snapshot.start + chrono::Duration::days(day_delta);
                                task.end = snapshot.end + chrono::Duration::days(day_delta);
                                interaction.changed = true;
                            }
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
                        painter.rect_filled(lh, Rounding::same(2.0), theme::handle_color());
                        painter.rect_filled(rh, Rounding::same(2.0), theme::handle_color());
                    }

                    // Tooltip on hover
                    if bar_response.hovered() || left_response.hovered() || right_response.hovered() {
                        hovered_task = Some(task.id);
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

            // Visual drop target while dragging tasks vertically to reorder.
            if let Some(target_vis) = reorder_preview_target {
                let y = origin.y + hh + target_vis as f32 * (row_height + row_padding);
                let row_rect = Rect::from_min_size(
                    Pos2::new(origin.x, y),
                    Vec2::new(chart_width, row_height + row_padding),
                );
                let ba = theme::border_accent();
                painter.rect_filled(
                    row_rect,
                    0.0,
                    Color32::from_rgba_premultiplied(ba.r(), ba.g(), ba.b(), 26),
                );
                painter.line_segment(
                    [
                        Pos2::new(row_rect.left(), row_rect.top() + 1.0),
                        Pos2::new(row_rect.right(), row_rect.top() + 1.0),
                    ],
                    Stroke::new(1.5, with_alpha(ba, 170)),
                );
            }

            // Apply pending reorder after drawing/interactions for this frame.
            if let Some((from, to)) = reorder_request {
                move_task_by_swapping(tasks, from, to);
                interaction.changed = true;
            }

            // Draw today marker in header (no full-height line through tasks)
            draw_today_line(&painter, origin, viewport);



            // Sticky header overlay when vertically scrolled past the content header.
            let clip_rect = ui.clip_rect();
            if origin.y < clip_rect.top() {
                let sticky_origin = Pos2::new(origin.x, clip_rect.top());
                draw_timeline_header(
                    &painter,
                    sticky_origin,
                    viewport,
                    chart_width,
                    sticky_origin.y + hh,
                );
                draw_today_line(&painter, sticky_origin, viewport);

                // Soft shadow under pinned header for separation.
                let r = theme::rendering();
                let shadow_rect = Rect::from_min_size(
                    Pos2::new(sticky_origin.x, sticky_origin.y + hh),
                    Vec2::new(chart_width, r.sticky_shadow_height),
                );
                painter.rect_filled(
                    shadow_rect,
                    0.0,
                    Color32::from_rgba_premultiplied(0, 0, 0, r.sticky_shadow_alpha),
                );
            }

            // Add arrow interaction + focus mode for dependencies
            let focus_task = hovered_task.or(*selected_task);
            let pointer_pos = ui.input(|i| i.pointer.hover_pos());

            for dep in dependencies {
                if let (Some(&(_, from_rect)), Some(&(_, to_rect))) =
                    (task_positions.get(&dep.from_task), task_positions.get(&dep.to_task))
                {
                    let (start_pt, end_pt) = dependency_endpoints(from_rect, to_rect, dep.kind);
                    let route = dependency_route_points(start_pt, end_pt, dep.kind);

                    let is_related = focus_task
                        .map(|task_id| dep.from_task == task_id || dep.to_task == task_id)
                        .unwrap_or(false);

                    // Focus mode: show related links bright, unrelated links stay subtle.
                    if focus_task.is_some() && is_related {
                        draw_dependency_polyline(
                            &painter,
                            &route,
                            theme::dep_arrow_hover(),
                            1.8,
                            3.0,
                        );
                        if route.len() >= 2 {
                            let last_from = route[route.len() - 2];
                            let last_to = route[route.len() - 1];
                            draw_arrowhead(&painter, last_from, last_to, theme::dep_arrow_hover());
                        }
                    }

                    let is_hovered_arrow = pointer_pos
                        .map(|p| is_point_near_polyline(p, &route, 6.0))
                        .unwrap_or(false);

                    if is_hovered_arrow {
                        draw_dependency_polyline(
                            &painter,
                            &route,
                            theme::dep_arrow_hover(),
                            2.2,
                            3.0,
                        );
                        if route.len() >= 2 {
                            let last_from = route[route.len() - 2];
                            let last_to = route[route.len() - 1];
                            draw_arrowhead(&painter, last_from, last_to, theme::dep_arrow_hover());
                        }

                        let dep_hit = ui.interact(
                            Rect::from_center_size(
                                pointer_pos.unwrap_or(start_pt),
                                Vec2::splat(16.0),
                            ),
                            ui.make_persistent_id(("dep-arrow", dep.from_task, dep.to_task)),
                            Sense::click(),
                        );
                        if dep_hit.secondary_clicked() {
                            interaction.remove_dependency = Some((dep.from_task, dep.to_task));
                        }

                        egui::show_tooltip_at_pointer(
                            ui.ctx(),
                            ui.layer_id(),
                            egui::Id::new(("dep-tip", dep.from_task, dep.to_task)),
                            |ui| {
                                let from_name = tasks
                                    .iter()
                                    .find(|t| t.id == dep.from_task)
                                    .map(|t| t.name.as_str())
                                    .unwrap_or("?");
                                let to_name = tasks
                                    .iter()
                                    .find(|t| t.id == dep.to_task)
                                    .map(|t| t.name.as_str())
                                    .unwrap_or("?");
                                ui.label(format!("{} → {}", from_name, to_name));
                                ui.label(
                                    egui::RichText::new("Right-click to remove")
                                        .size(10.0)
                                        .color(theme::text_dim()),
                                );
                            },
                        );
                    }
                }
            }

            // Empty click on background clears selection
            if response.clicked() && !consumed_click {
                *selected_task = None;
            }

            // ── Shift+Drag link creation ─────────────────────────────
            let link_id = Id::new("dep-link-drag");
            let pointer_pos = ui.input(|i| i.pointer.interact_pos());
            let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
            let primary_down = ui.input(|i| i.pointer.primary_down());
            let primary_released = ui.input(|i| i.pointer.primary_released());

            // Check if user started Shift+clicking on a bar
            if shift_held && primary_pressed {
                if let Some(ptr) = pointer_pos {
                    // Find which bar the pointer is over
                    for task in tasks.iter() {
                        if let Some(&(_, rect)) = task_positions.get(&task.id) {
                            if rect.contains(ptr) {
                                let state = LinkDragState {
                                    from_task: task.id,
                                    from_point: Pos2::new(rect.left(), rect.center().y),
                                };
                                ui.ctx().data_mut(|d| d.insert_temp(link_id, state));
                                break;
                            }
                        }
                    }
                }
            }

            // Draw the in-progress link line
            let link_state: Option<LinkDragState> =
                ui.ctx().data_mut(|d| d.get_temp(link_id));
            if let Some(ref state) = link_state {
                if let Some(ptr) = pointer_pos {
                    if primary_down {
                        draw_dependency_arrow(
                            &painter,
                            state.from_point,
                            ptr,
                            DependencyKind::FinishToStart,
                            theme::dep_creating(),
                            1.5,
                        );
                    }
                }
            }

            // On release, check if we landed on a target bar
            if primary_released {
                if let Some(state) = link_state {
                    if let Some(ptr) = pointer_pos {
                        for task in tasks.iter() {
                            if task.id != state.from_task {
                                if let Some(&(_, rect)) = task_positions.get(&task.id) {
                                    if rect.contains(ptr) {
                                        interaction.new_dependency = Some(Dependency {
                                            from_task: state.from_task,
                                            to_task: task.id,
                                            kind: DependencyKind::FinishToStart,
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    ui.ctx().data_mut(|d| d.remove::<LinkDragState>(link_id));
                }
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

fn row_index_from_pointer_y(
    pointer_y: f32,
    origin: Pos2,
    row_height: f32,
    row_padding: f32,
    task_count: usize,
) -> Option<usize> {
    if task_count == 0 {
        return None;
    }

    let row_top = origin.y + header_height() + row_padding;
    let row_span = row_height + row_padding;
    if row_span <= 0.0 {
        return None;
    }

    let raw = ((pointer_y - row_top) / row_span).floor() as isize;
    let clamped = raw.clamp(0, task_count as isize - 1) as usize;
    Some(clamped)
}

fn move_task_by_swapping(tasks: &mut [Task], from: usize, to: usize) {
    if from == to || from >= tasks.len() || to >= tasks.len() {
        return;
    }

    if from < to {
        for idx in from..to {
            tasks.swap(idx, idx + 1);
        }
    } else {
        for idx in (to + 1..=from).rev() {
            tasks.swap(idx - 1, idx);
        }
    }
}

fn vertical_zoom_scale(viewport: &TimelineViewport) -> f32 {
    let z = theme::zoom();
    (viewport.pixels_per_day / z.default_pixels_per_day).clamp(z.vertical_scale_min, z.vertical_scale_max)
}

fn scaled_row_height(viewport: &TimelineViewport) -> f32 {
    theme::row_height() * vertical_zoom_scale(viewport)
}

fn scaled_row_padding(viewport: &TimelineViewport) -> f32 {
    (theme::row_gap() * vertical_zoom_scale(viewport)).clamp(1.0, 10.0)
}

fn scaled_handle_width(viewport: &TimelineViewport) -> f32 {
    (theme::handle_width() * vertical_zoom_scale(viewport).sqrt()).clamp(5.0, 12.0)
}

fn draw_timeline_header(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    width: f32,
    grid_bottom_y: f32,
) {
    let hh = header_height();
    // Background for header
    painter.rect_filled(
        Rect::from_min_size(origin, Vec2::new(width, hh)),
        0.0,
        theme::bg_header(),
    );

    // Bottom border of header
    painter.line_segment(
        [
            Pos2::new(origin.x, origin.y + hh),
            Pos2::new(origin.x + width, origin.y + hh),
        ],
        Stroke::new(1.0, theme::border_subtle()),
    );

    // Subtle weekend tint in header (especially useful in Weeks view).
    draw_weekend_header_bands(painter, origin, viewport, width);

    let mut date = viewport.start;
    let end = viewport.end;

    match viewport.scale {
        TimelineScale::Days => {
            while date <= end {
                let x = origin.x + viewport.date_to_x(date);

                painter.line_segment(
                    [
                        Pos2::new(x, origin.y + hh),
                        Pos2::new(x, grid_bottom_y),
                    ],
                    Stroke::new(0.5, theme::grid_line()),
                );

                if viewport.pixels_per_day >= 20.0 {
                    let is_weekend = date.weekday().num_days_from_monday() >= 5;
                    let day_color = if is_weekend {
                        theme::text_dim()
                    } else {
                        theme::text_secondary()
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
                        theme::text_primary(),
                    );
                }

                date += chrono::Duration::days(1);
            }
        }
        TimelineScale::Weeks => {
            let weekday = date.weekday().num_days_from_monday();
            date -= chrono::Duration::days(weekday as i64);
            let show_weekdays = viewport.pixels_per_day >= 18.0 * 1.4;
            let weekday_labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

            while date <= end {
                let x = origin.x + viewport.date_to_x(date);
                let week_band_top = origin.y + 22.0;

                painter.line_segment(
                    [
                        Pos2::new(x, week_band_top),
                        Pos2::new(x, grid_bottom_y),
                    ],
                    Stroke::new(0.5, theme::grid_line()),
                );

                painter.text(
                    Pos2::new(x + 3.0, origin.y + 26.0),
                    egui::Align2::LEFT_CENTER,
                    date.format("W%V").to_string(),
                    theme::font_sub(),
                    theme::text_secondary(),
                );

                if show_weekdays {
                    for (day_offset, label) in weekday_labels.iter().enumerate() {
                        let day_date = date + chrono::Duration::days(day_offset as i64);
                        if day_date > end {
                            break;
                        }
                        let day_x = origin.x + viewport.date_to_x(day_date) + viewport.pixels_per_day * 0.5;
                        painter.text(
                            Pos2::new(day_x, origin.y + 39.0),
                            egui::Align2::CENTER_CENTER,
                            *label,
                            theme::font_small(),
                            theme::text_dim(),
                        );
                    }
                }

                if date.day() <= 7 {
                    painter.text(
                        Pos2::new(x + 3.0, origin.y + 12.0),
                        egui::Align2::LEFT_CENTER,
                        date.format("%b %Y").to_string(),
                        theme::font_header(),
                        theme::text_primary(),
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
                        Pos2::new(x, origin.y + hh),
                        Pos2::new(x, grid_bottom_y),
                    ],
                    Stroke::new(0.5, theme::grid_line()),
                );

                painter.text(
                    Pos2::new(x + 5.0, origin.y + 18.0),
                    egui::Align2::LEFT_CENTER,
                    date.format("%b %Y").to_string(),
                    theme::font_header(),
                    theme::text_primary(),
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

fn draw_weekend_bands(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    width: f32,
    bottom_y: f32,
) {
    // Skip when days are too compressed to avoid noise.
    if viewport.pixels_per_day < 5.0 || viewport.scale == TimelineScale::Months {
        return;
    }

    let mut date = viewport.start;
    let end = viewport.end;
    let right = origin.x + width;
    while date <= end {
        let weekday = date.weekday().num_days_from_monday();
        if weekday >= 5 {
            let x0 = origin.x + viewport.date_to_x(date);
            let next_day = date + chrono::Duration::days(1);
            let x1 = (origin.x + viewport.date_to_x(next_day)).min(right);
            if x1 > x0 {
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(x0, origin.y + header_height()),
                        Pos2::new(x1, bottom_y),
                    ),
                    0.0,
                    theme::weekend_shade(),
                );

                // Crisp separator at weekend start (Saturday) for quick scanning.
                if weekday == 5 {
                    painter.line_segment(
                        [
                            Pos2::new(x0, origin.y + header_height()),
                            Pos2::new(x0, bottom_y),
                        ],
                        Stroke::new(1.0, with_alpha(theme::border_subtle(), theme::rendering().weekend_sep_alpha)),
                    );
                }
            }
        }
        date += chrono::Duration::days(1);
    }
}

fn draw_weekend_header_bands(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    width: f32,
) {
    if viewport.pixels_per_day < 7.0 || viewport.scale == TimelineScale::Months {
        return;
    }

    let mut date = viewport.start;
    let end = viewport.end;
    let right = origin.x + width;
    let y_min = origin.y + 22.0;
    let y_max = origin.y + header_height();

    while date <= end {
        let weekday = date.weekday().num_days_from_monday();
        if weekday >= 5 {
            let x0 = origin.x + viewport.date_to_x(date);
            let next_day = date + chrono::Duration::days(1);
            let x1 = (origin.x + viewport.date_to_x(next_day)).min(right);
            if x1 > x0 {
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(x0, y_min), Pos2::new(x1, y_max)),
                    0.0,
                    theme::weekend_header_shade(),
                );
            }
        }
        date += chrono::Duration::days(1);
    }
}

fn draw_today_line(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
) {
    let today = chrono::Local::now().date_naive();
    let x = origin.x + viewport.date_to_x(today);

    // Dedicated header marker: diamond + compact label.
    let center = Pos2::new(x, origin.y + 11.0);
    let size = 5.5;
    let points = vec![
        Pos2::new(center.x, center.y - size),
        Pos2::new(center.x + size, center.y),
        Pos2::new(center.x, center.y + size),
        Pos2::new(center.x - size, center.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        points,
        theme::today_line(),
        Stroke::new(1.0, Color32::from_white_alpha(30)),
    ));

}

/// Draw a summary / parent task bar (bracket style, spans all children).
/// Returns the interaction rect for click handling.
fn draw_summary_bar(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    task: &Task,
    y: f32,
    row_height: f32,
    is_selected: bool,
) -> Rect {
    let x_start = origin.x + viewport.date_to_x(task.start);
    let x_end   = origin.x + viewport.date_to_x(task.end);
    let width   = (x_end - x_start).max(8.0);

    // Draw a thin horizontal bar in the middle of the row (bracket body)
    let bar_h    = (row_height * 0.35).max(5.0);
    let bar_y    = y + (row_height - bar_h) * 0.5;
    let bar_rect = Rect::from_min_size(Pos2::new(x_start, bar_y), Vec2::new(width, bar_h));

    // Muted color based on task color
    let body_color = Color32::from_rgba_premultiplied(
        task.color.r() / 2,
        task.color.g() / 2,
        task.color.b() / 2,
        200,
    );
    let tick_color = with_alpha(task.color, 220);

    // Body
    painter.rect_filled(bar_rect, Rounding::same(2.0), body_color);

    // Progress fill
    if task.progress > 0.0 {
        let prog_rect = Rect::from_min_size(
            bar_rect.min,
            Vec2::new(width * task.progress, bar_h),
        );
        painter.rect_filled(prog_rect, Rounding::same(2.0), with_alpha(task.color, 180));
    }

    // Left downward tick
    let tick_h = row_height * 0.5;
    painter.line_segment(
        [Pos2::new(x_start, bar_y), Pos2::new(x_start, bar_y + tick_h)],
        Stroke::new(3.0, tick_color),
    );
    // Right downward tick
    painter.line_segment(
        [Pos2::new(x_start + width, bar_y), Pos2::new(x_start + width, bar_y + tick_h)],
        Stroke::new(3.0, tick_color),
    );

    // Selection highlight
    if is_selected {
        painter.rect_stroke(
            bar_rect.expand(2.0),
            Rounding::same(3.0),
            Stroke::new(1.5, with_alpha(task.color, 200)),
        );
    }

    // Label to the right of the bar
    let label_x = x_start + width + 6.0;
    let label_y = y + row_height / 2.0;
    painter.text(
        Pos2::new(label_x, label_y),
        egui::Align2::LEFT_CENTER,
        format!("{} ({:.0}%)", task.name, task.progress * 100.0),
        egui::FontId::proportional(11.0),
        with_alpha(Color32::WHITE, 180),
    );

    // Return a slightly expanded rect so clicking near the bar registers
    bar_rect.expand(4.0)
}

fn draw_task_bar(
    painter: &egui::Painter,
    origin: Pos2,
    viewport: &TimelineViewport,
    task: &Task,
    y: f32,
    row_height: f32,
    is_selected: bool,
) -> Rect {
    let x_start = origin.x + viewport.date_to_x(task.start);
    let x_end = origin.x + viewport.date_to_x(task.end);
    let bar_width = (x_end - x_start).max(6.0);
    let inset = theme::bar_inset();

    let bar_rect = Rect::from_min_size(
        Pos2::new(x_start, y + inset),
        Vec2::new(bar_width, row_height - inset * 2.0),
    );
    let br = theme::bar_rounding();
    let rounding = Rounding::same(br);
    let r = theme::rendering();

    // Layered shadow (skipped when alpha is 0 for flat themes)
    if r.bar_shadow_alpha_1 > 0 {
        let shadow_rect_1 = bar_rect.translate(Vec2::new(0.0, r.bar_shadow_offset_y_1));
        painter.rect_filled(shadow_rect_1, rounding, Color32::from_black_alpha(r.bar_shadow_alpha_1));
    }
    if r.bar_shadow_alpha_2 > 0 {
        let shadow_rect_2 = bar_rect.translate(Vec2::new(r.bar_shadow_offset_x_2, r.bar_shadow_offset_y_2));
        painter.rect_filled(shadow_rect_2, rounding, Color32::from_black_alpha(r.bar_shadow_alpha_2));
    }

    // Main bar — flat fill when darken_factor is 1.0
    let base_color = darken_color(task.color, r.bar_darken_factor);
    painter.rect_filled(bar_rect, rounding, base_color);

    // Mid-body glaze (skipped for flat themes)
    if r.bar_glaze_alpha > 0 {
        let body_glaze = Rect::from_min_size(
            Pos2::new(bar_rect.left(), bar_rect.top() + bar_rect.height() * r.bar_glaze_top_frac),
            Vec2::new(bar_width, bar_rect.height() * r.bar_glaze_height_frac),
        );
        painter.rect_filled(
            body_glaze,
            Rounding::same((br - 1.0).max(1.0)),
            with_alpha(task.color, r.bar_glaze_alpha),
        );
    }

    // Top specular highlight (skipped for flat themes)
    if r.bar_highlight_alpha > 0 {
        let highlight_rect = Rect::from_min_size(
            bar_rect.min,
            Vec2::new(bar_width, (bar_rect.height() * r.bar_highlight_height_frac).max(3.0)),
        );
        painter.rect_filled(
            highlight_rect,
            Rounding {
                nw: br,
                ne: br,
                sw: 0.0,
                se: 0.0,
            },
            Color32::from_white_alpha(r.bar_highlight_alpha),
        );
    }

    // Bottom contrast edge (skipped for flat themes)
    if r.bar_bottom_edge_alpha > 0 {
        painter.line_segment(
            [
                Pos2::new(bar_rect.left() + 1.0, bar_rect.bottom() - 1.0),
                Pos2::new(bar_rect.right() - 1.0, bar_rect.bottom() - 1.0),
            ],
            Stroke::new(1.0, Color32::from_black_alpha(r.bar_bottom_edge_alpha)),
        );
    }

    // Progress fill (darkened overlay)
    if task.progress > 0.0 {
        let progress_width = bar_width * task.progress.clamp(0.0, 1.0);
        let progress_rect = Rect::from_min_size(
            bar_rect.min,
            Vec2::new(progress_width, bar_rect.height()),
        );
        painter.rect_filled(progress_rect, rounding, theme::progress_overlay());

        // Progress divider tick
        if task.progress < 0.98 {
            let tick_x = bar_rect.left() + progress_width;
            painter.line_segment(
                [
                    Pos2::new(tick_x, bar_rect.top() + 2.0),
                    Pos2::new(tick_x, bar_rect.bottom() - 2.0),
                ],
                Stroke::new(1.0, Color32::from_white_alpha(r.progress_tick_alpha)),
            );
        }
    }

    // Selection glow
    if is_selected {
        painter.rect_stroke(
            bar_rect.expand(r.selection_glow_outer_expand),
            Rounding::same(br + r.selection_glow_outer_expand),
            Stroke::new(2.0, with_alpha(theme::border_accent(), r.selection_glow_outer_alpha)),
        );
        painter.rect_stroke(
            bar_rect.expand(r.selection_glow_inner_expand),
            Rounding::same(br + r.selection_glow_inner_expand),
            Stroke::new(2.0, theme::border_accent()),
        );
    }

    // Overdue indicator — red border when past due and not complete
    let today = chrono::Local::now().date_naive();
    if !task.is_milestone && task.end < today && task.progress < 1.0 {
        painter.rect_stroke(
            bar_rect.expand(1.0),
            Rounding::same(br + 1.0),
            Stroke::new(2.0, Color32::from_rgb(220, 60, 60)),
        );
    }

    // Task name on bar (single line, clipped to bar bounds)
    if bar_width > 30.0 {
        let galley = painter.layout_no_wrap(
            task.name.clone(),
            theme::font_bar(),
            theme::text_on_bar(),
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
    row_height: f32,
    is_selected: bool,
) -> Rect {
    let x = origin.x + viewport.date_to_x(task.start);
    let center = Pos2::new(x, y + row_height / 2.0);
    let size = (row_height / 2.0 - 3.0).max(6.0);

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
        Color32::from_black_alpha(theme::rendering().milestone_shadow_alpha),
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
            Stroke::new(2.0, theme::border_accent()),
        ));
    }

    // Label
    painter.text(
        Pos2::new(x + size + 6.0, y + row_height / 2.0),
        egui::Align2::LEFT_CENTER,
        &task.name,
        theme::font_bar(),
        theme::text_secondary(),
    );

    Rect::from_center_size(center, Vec2::splat(size * 2.0 + 2.0))
}

fn dependency_endpoints(from_rect: Rect, to_rect: Rect, kind: DependencyKind) -> (Pos2, Pos2) {
    // Route endpoints based on dependency type:
    // FS (Finish→Start):  exit from right of from, enter left of to
    // SS (Start→Start):   exit from left of from, enter left of to
    // FF (Finish→Finish): exit from right of from, enter right of to
    // SF (Start→Finish):  exit from left of from, enter right of to
    let (start_x, end_x) = match kind {
        DependencyKind::FinishToStart  => (from_rect.right(), to_rect.left()),
        DependencyKind::StartToStart   => (from_rect.left(),  to_rect.left()),
        DependencyKind::FinishToFinish => (from_rect.right(), to_rect.right()),
        DependencyKind::StartToFinish  => (from_rect.left(),  to_rect.right()),
    };
    let from = Pos2::new(start_x, from_rect.center().y);
    let to   = Pos2::new(end_x,   to_rect.center().y);
    (from, to)
}

fn dependency_route_points(from: Pos2, to: Pos2, kind: DependencyKind) -> Vec<Pos2> {
    // Short fixed stub length — exits/enters each bar by this amount before turning.
    const STUB: f32 = 10.0;

    match kind {
        DependencyKind::FinishToStart => {
            // Exit right side, enter left side.
            let exit_x = from.x + STUB;
            let enter_x = to.x - STUB;

            if exit_x <= enter_x {
                // Normal forward case: exit stub → drop straight → enter stub.
                if (from.y - to.y).abs() < 1.0 {
                    // Same row, straight line.
                    vec![from, to]
                } else {
                    vec![
                        from,
                        Pos2::new(exit_x, from.y),
                        Pos2::new(exit_x, to.y),
                        to,
                    ]
                }
            } else {
                // Backtrack / overlap: bars too close or reversed.
                // Route horizontal travel through the gutter (midpoint Y) between the two rows
                // so the line runs in the visual gap between bars rather than through them.
                let gutter_y = (from.y + to.y) / 2.0;
                if (from.y - to.y).abs() < 1.0 {
                    // Same row — loop below the row.
                    let loop_y = from.y + 8.0;
                    vec![
                        from,
                        Pos2::new(exit_x, from.y),
                        Pos2::new(exit_x, loop_y),
                        Pos2::new(enter_x, loop_y),
                        Pos2::new(enter_x, to.y),
                        to,
                    ]
                } else {
                    vec![
                        from,
                        Pos2::new(exit_x, from.y),
                        Pos2::new(exit_x, gutter_y),
                        Pos2::new(enter_x, gutter_y),
                        Pos2::new(enter_x, to.y),
                        to,
                    ]
                }
            }
        }

        DependencyKind::StartToStart => {
            // Both exit/enter the LEFT side — lane to the left of both bars.
            let lane_x = (from.x.min(to.x) - STUB).max(2.0);
            if (from.y - to.y).abs() < 1.0 {
                vec![from, Pos2::new(lane_x, from.y), to]
            } else {
                vec![
                    from,
                    Pos2::new(lane_x, from.y),
                    Pos2::new(lane_x, to.y),
                    to,
                ]
            }
        }

        DependencyKind::FinishToFinish => {
            // Both exit/enter the RIGHT side — lane to the right of both bars.
            let lane_x = from.x.max(to.x) + STUB;
            if (from.y - to.y).abs() < 1.0 {
                vec![from, Pos2::new(lane_x, from.y), to]
            } else {
                vec![
                    from,
                    Pos2::new(lane_x, from.y),
                    Pos2::new(lane_x, to.y),
                    to,
                ]
            }
        }

        DependencyKind::StartToFinish => {
            // Exit left, enter right.
            let exit_x = from.x - STUB;
            let enter_x = to.x + STUB;
            if exit_x >= enter_x {
                // Normal: from bar starts after to bar ends.
                let mid_x = to.x + (from.x - to.x) / 2.0;
                if (from.y - to.y).abs() < 1.0 {
                    vec![from, to]
                } else {
                    vec![
                        from,
                        Pos2::new(mid_x, from.y),
                        Pos2::new(mid_x, to.y),
                        to,
                    ]
                }
            } else {
                // Backtrack — route horizontal travel through the gutter between rows.
                let gutter_y = (from.y + to.y) / 2.0;
                if (from.y - to.y).abs() < 1.0 {
                    let loop_y = from.y + 8.0;
                    vec![
                        from,
                        Pos2::new(exit_x, from.y),
                        Pos2::new(exit_x, loop_y),
                        Pos2::new(enter_x, loop_y),
                        Pos2::new(enter_x, to.y),
                        to,
                    ]
                } else {
                    vec![
                        from,
                        Pos2::new(exit_x, from.y),
                        Pos2::new(exit_x, gutter_y),
                        Pos2::new(enter_x, gutter_y),
                        Pos2::new(enter_x, to.y),
                        to,
                    ]
                }
            }
        }
    }
}

fn draw_dependency_arrow(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    kind: DependencyKind,
    color: Color32,
    width: f32,
) {
    let route = dependency_route_points(from, to, kind);
    draw_dependency_polyline(painter, &route, color, width, 3.0);
    if route.len() >= 2 {
        let last_from = route[route.len() - 2];
        let last_to = route[route.len() - 1];
        draw_arrowhead(painter, last_from, last_to, color);
    }
}

fn draw_dependency_polyline(
    painter: &egui::Painter,
    points: &[Pos2],
    color: Color32,
    width: f32,
    corner_radius: f32,
) {
    if points.len() < 2 {
        return;
    }

    let stroke = Stroke::new(width, color);
    let mut draw_points: Vec<Pos2> = Vec::with_capacity(points.len() * 3);
    draw_points.push(points[0]);

    for i in 1..points.len() - 1 {
        let prev = points[i - 1];
        let curr = points[i];
        let next = points[i + 1];

        let in_vec = curr - prev;
        let out_vec = next - curr;
        let in_len = in_vec.length();
        let out_len = out_vec.length();

        if in_len < 0.001 || out_len < 0.001 {
            draw_points.push(curr);
            continue;
        }

        let r = corner_radius.min(in_len * 0.45).min(out_len * 0.45);
        let in_dir = in_vec / in_len;
        let out_dir = out_vec / out_len;

        let p_in = curr - in_dir * r;
        let p_out = curr + out_dir * r;

        draw_points.push(p_in);

        let arc_steps = 5;
        for s in 1..arc_steps {
            let t = s as f32 / arc_steps as f32;
            let q1 = p_in + (curr - p_in) * t;
            let q2 = curr + (p_out - curr) * t;
            let arc = q1 + (q2 - q1) * t;
            draw_points.push(arc);
        }

        draw_points.push(p_out);
    }

    draw_points.push(*points.last().unwrap_or(&points[0]));

    for seg in draw_points.windows(2) {
        painter.line_segment([seg[0], seg[1]], stroke);
    }
}

fn is_point_near_polyline(point: Pos2, points: &[Pos2], threshold: f32) -> bool {
    if points.len() < 2 {
        return false;
    }
    points
        .windows(2)
        .any(|seg| distance_to_segment(point, seg[0], seg[1]) <= threshold)
}

fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ab_len_sq = ab.length_sq();
    if ab_len_sq <= f32::EPSILON {
        return (p - a).length();
    }
    let ap = p - a;
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha)
}

fn darken_color(color: Color32, factor: f32) -> Color32 {
    let f = factor.clamp(0.0, 1.0);
    Color32::from_rgb(
        (color.r() as f32 * f) as u8,
        (color.g() as f32 * f) as u8,
        (color.b() as f32 * f) as u8,
    )
}

/// Draw a small triangular arrowhead pointing from `from` toward `to`.
fn draw_arrowhead(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32) {
    let dir = (to - from).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let arrow_len = 6.0;
    let arrow_w = 3.4;
    let tip = to;
    let left = tip - dir * arrow_len + perp * arrow_w;
    let right = tip - dir * arrow_len - perp * arrow_w;
    painter.add(egui::Shape::convex_polygon(
        vec![tip, left, right],
        color,
        Stroke::NONE,
    ));
}
