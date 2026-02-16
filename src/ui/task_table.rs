use crate::model::Task;
use crate::ui::theme;
use egui::{Color32, RichText, Ui};
use uuid::Uuid;

/// Actions that the task table can request.
pub enum TaskTableAction {
    None,
    Select(Uuid),
    Delete(Uuid),
    Add,
}

/// Render the left-side task table panel.
pub fn show_task_table(
    tasks: &[Task],
    selected_task: Option<Uuid>,
    ui: &mut Ui,
) -> TaskTableAction {
    let mut action = TaskTableAction::None;

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
        ui.label(
            RichText::new(format!("({})", tasks.len()))
                .size(11.0)
                .color(theme::text_dim()),
        );
    });
    ui.add_space(4.0);

    // Add task button — accent styled
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
                hdr(ui, "", 14.0);            // color dot
                hdr(ui, "TASK", 110.0);
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
            for (i, task) in tasks.iter().enumerate() {
                let is_selected = selected_task == Some(task.id);

                // Row frame — use solid dark colors so no light fill bleeds through
                let row_bg = if is_selected {
                    theme::bg_selected()
                } else if i % 2 == 0 {
                    theme::bg_panel()       // solid slightly lighter dark
                } else {
                    theme::bg_dark()        // solid base dark
                };

                let frame = egui::Frame {
                    fill: row_bg,
                    rounding: egui::Rounding::same(4.0),
                    inner_margin: egui::Margin::symmetric(6.0, 4.0),
                    outer_margin: egui::Margin::ZERO,
                    stroke: if is_selected {
                        egui::Stroke::new(1.0, theme::row_selected_stroke())
                    } else {
                        egui::Stroke::new(1.0, theme::row_unselected_stroke())
                    },
                    shadow: egui::epaint::Shadow::NONE,
                };

                let frame_resp = frame.show(ui, |ui| {
                    ui.set_min_height(theme::row_height() - 8.0); // 8.0 accounts for inner margin
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        // Color dot
                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 3.0, task.color);

                        // Task name
                        let name = if task.is_milestone {
                            format!("◆ {}", task.name)
                        } else {
                            task.name.clone()
                        };
                        let name_text = RichText::new(name).size(12.0).color(if is_selected {
                            Color32::WHITE
                        } else {
                            theme::text_primary()
                        });
                        ui.add(
                            egui::Label::new(name_text)
                                .truncate(),
                        );

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;

                                // Delete button
                                let del_btn = ui.add(
                                    egui::Button::new(
                                        RichText::new("✕")
                                            .size(10.0)
                                            .color(theme::text_dim()),
                                    )
                                    .frame(false),
                                );
                                if del_btn.on_hover_text("Delete task").clicked() {
                                    action = TaskTableAction::Delete(task.id);
                                }

                                // Progress
                                let pbar = egui::ProgressBar::new(task.progress)
                                    .desired_width(48.0)
                                    .fill(task.color)
                                    .rounding(egui::Rounding::same(3.0));
                                ui.add(pbar);

                                // Dates (compact)
                                ui.label(
                                    RichText::new(task.end.format("%m/%d").to_string())
                                        .size(10.0)
                                        .color(theme::text_secondary()),
                                );
                                ui.label(
                                    RichText::new("→")
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

                // Small gap between rows
                ui.add_space(theme::row_gap());
            }
        });

    action
}

/// Color palette for auto-assigning task colors (re-exported from theme).
#[allow(dead_code)]
pub fn task_colors() -> Vec<Color32> { theme::task_palette() }
