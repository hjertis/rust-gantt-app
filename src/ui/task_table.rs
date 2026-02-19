use crate::model::Task;
use crate::model::task::TaskPriority;
use crate::ui::{filter_bar, theme};
use egui::{Color32, RichText, Ui};
use uuid::Uuid;

/// Actions that the task table can request.
pub enum TaskTableAction {
    None,
    Select(Uuid),
    Delete(Uuid),
    Add,
    ToggleCollapse(Uuid),
}

/// Render the left-side task table panel.
/// `search_query` and `filter_priority` are used to hide non-matching tasks.
pub fn show_task_table(
    tasks: &[Task],
    selected_task: Option<Uuid>,
    search_query: &str,
    filter_priority: Option<TaskPriority>,
    ui: &mut Ui,
) -> TaskTableAction {
    let mut action = TaskTableAction::None;

    // Determine which tasks are visible after filtering
    // A parent task is shown if it or any of its children pass the filter.
    let passes_filter = |t: &Task| -> bool {
        if !filter_bar::task_matches(
            &t.name,
            &t.description,
            t.priority,
            search_query,
            filter_priority,
        ) {
            // Check if any child passes the filter
            let child_passes = tasks.iter().any(|child| {
                child.parent_id == Some(t.id)
                    && filter_bar::task_matches(
                        &child.name,
                        &child.description,
                        child.priority,
                        search_query,
                        filter_priority,
                    )
            });
            if !child_passes {
                return false;
            }
        }
        true
    };

    let visible_count = tasks.iter().filter(|t| passes_filter(t)).count();

    // Header area
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Tasks")
                .strong()
                .size(15.0)
                .color(theme::text_primary()),
        );
        ui.add_space(4.0);
        let count_label = if visible_count != tasks.len() {
            format!("({} / {})", visible_count, tasks.len())
        } else {
            format!("({})", tasks.len())
        };
        ui.label(
            RichText::new(count_label)
                .size(11.0)
                .color(theme::text_dim()),
        );
    });
    ui.add_space(4.0);

    // Add task button
    let btn = egui::Button::new(
        RichText::new("＋  Add Task").color(Color32::WHITE).size(12.0),
    )
    .fill(theme::accent())
    .rounding(egui::Rounding::same(5.0));
    if ui.add_sized([ui.available_width(), 30.0], btn).clicked() {
        action = TaskTableAction::Add;
    }

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(2.0);

    // Column headers
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        let w = ui.available_width();
        ui.allocate_ui_with_layout(
            egui::vec2(w, 16.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let hdr = |ui: &mut Ui, text: &str, width: f32| {
                    ui.allocate_ui(egui::vec2(width, 16.0), |ui| {
                        ui.label(RichText::new(text).size(9.0).color(theme::text_dim()).strong());
                    });
                };
                hdr(ui, "", 14.0);   // color dot
                hdr(ui, "!", 10.0);  // priority icon column
                hdr(ui, "TASK", 100.0);
                hdr(ui, "START", 50.0);
                hdr(ui, "END", 50.0);
                hdr(ui, "DONE", 55.0);
            },
        );
    });

    ui.add_space(2.0);

    // Task rows
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let today = chrono::Local::now().date_naive();

            for (i, task) in tasks.iter().enumerate() {
                // Skip if filtered out
                if !passes_filter(task) {
                    continue;
                }

                // Skip children of collapsed parents
                if let Some(pid) = task.parent_id {
                    if let Some(parent) = tasks.iter().find(|t| t.id == pid) {
                        if parent.collapsed {
                            continue;
                        }
                    }
                }

                let is_selected = selected_task == Some(task.id);
                let is_parent = task.has_children(tasks);
                let is_child = task.parent_id.is_some();
                let is_overdue =
                    !task.is_milestone && task.end < today && task.progress < 1.0;

                // Row background
                let row_bg = if is_selected {
                    theme::bg_selected()
                } else if i % 2 == 0 {
                    theme::bg_panel()
                } else {
                    theme::bg_dark()
                };

                let frame = egui::Frame {
                    fill: row_bg,
                    rounding: egui::Rounding::same(4.0),
                    inner_margin: egui::Margin::symmetric(6.0, 4.0),
                    outer_margin: if is_child {
                        egui::Margin { left: 12.0, ..egui::Margin::ZERO }
                    } else {
                        egui::Margin::ZERO
                    },
                    stroke: if is_selected {
                        egui::Stroke::new(1.0, theme::row_selected_stroke())
                    } else if is_overdue {
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 60, 60))
                    } else {
                        egui::Stroke::new(1.0, theme::row_unselected_stroke())
                    },
                    shadow: egui::epaint::Shadow::NONE,
                };

                let frame_resp = frame.show(ui, |ui| {
                    ui.set_min_height(theme::row_height() - 8.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        // Expand/collapse for parent tasks
                        if is_parent {
                            let tri = if task.collapsed { egui_phosphor::regular::CARET_RIGHT } else { egui_phosphor::regular::CARET_DOWN };
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(tri).size(9.0).color(theme::text_dim()),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                action = TaskTableAction::ToggleCollapse(task.id);
                            }
                        } else if is_child {
                            ui.add_space(12.0);
                        } else {
                            ui.add_space(12.0);
                        }

                        // Color dot
                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 3.0, task.color);

                        // Priority icon
                        let pri_icon = task.priority.icon();
                        let pri_color = match task.priority {
                            TaskPriority::Critical => egui::Color32::from_rgb(220, 60, 60),
                            TaskPriority::High => egui::Color32::from_rgb(220, 140, 40),
                            TaskPriority::Medium => egui::Color32::from_rgb(200, 180, 40),
                            TaskPriority::Low => egui::Color32::from_rgb(80, 160, 80),
                            TaskPriority::None => theme::text_dim(),
                        };
                        ui.label(
                            RichText::new(pri_icon).size(9.0).color(pri_color).strong(),
                        );

                        // Task name
                        let name = if task.is_milestone {
                            format!("◆ {}", task.name)
                        } else if is_overdue {
                            format!("⚠ {}", task.name)
                        } else {
                            task.name.clone()
                        };
                        let name_color = if is_selected {
                            Color32::WHITE
                        } else if is_overdue {
                            egui::Color32::from_rgb(230, 100, 100)
                        } else if is_parent {
                            theme::text_primary()
                        } else {
                            theme::text_secondary()
                        };
                        let name_text = RichText::new(name).size(12.0).color(name_color);
                        ui.add(egui::Label::new(name_text).truncate());

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;

                                let del_btn = ui.add(
                                    egui::Button::new(
                                        RichText::new(egui_phosphor::regular::X)
                                            .size(10.0)
                                            .color(theme::text_dim()),
                                    )
                                    .frame(false),
                                );
                                if del_btn.on_hover_text("Delete task").clicked() {
                                    action = TaskTableAction::Delete(task.id);
                                }

                                let pbar = egui::ProgressBar::new(task.progress)
                                    .desired_width(48.0)
                                    .fill(task.color)
                                    .rounding(egui::Rounding::same(3.0));
                                ui.add(pbar);

                                ui.label(
                                    RichText::new(task.end.format("%m/%d").to_string())
                                        .size(10.0)
                                        .color(theme::text_secondary()),
                                );
                                ui.label(
                                    RichText::new(egui_phosphor::regular::ARROW_RIGHT)
                                        .size(9.0)
                                        .color(theme::text_dim()),
                                );
                                ui.label(
                                    RichText::new(task.start.format("%m/%d").to_string())
                                        .size(10.0)
                                        .color(theme::text_secondary()),
                                );
                            },
                        );
                    });
                });

                // Make entire row clickable
                let row_rect = frame_resp.response.rect;
                let row_click = ui.interact(
                    row_rect,
                    egui::Id::new(("task-row", task.id)),
                    egui::Sense::click(),
                );
                if row_click.clicked() && matches!(action, TaskTableAction::None) {
                    action = TaskTableAction::Select(task.id);
                }

                ui.add_space(theme::row_gap());
            }
        });

    action
}
#[allow(dead_code)]
pub fn task_colors() -> Vec<Color32> { theme::task_palette() }
