use crate::model::Task;
use crate::model::task::Dependency;
use crate::ui::theme;
use egui::{Color32, RichText, Ui};
use uuid::Uuid;

/// Actions the editor can request.
pub enum EditorAction {
    None,
    Changed,
    RemoveDependency(Uuid, Uuid),
}

/// Render an inline task editor for the selected task.
/// Also shows dependencies involving this task.
pub fn show_task_editor(
    task: &mut Task,
    all_tasks: &[Task],
    dependencies: &[Dependency],
    ui: &mut Ui,
) -> EditorAction {
    let mut action = EditorAction::None;

    // Section header
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Edit Task")
                .strong()
                .size(13.0)
                .color(theme::text_primary()),
        );
    });
    ui.add_space(4.0);

    let frame = egui::Frame {
        fill: theme::bg_dark(),
        rounding: egui::Rounding::same(theme::widget_rounding_val()),
        inner_margin: egui::Margin::same(theme::layout().editor_inner_margin),
        outer_margin: egui::Margin::ZERO,
        stroke: egui::Stroke::new(1.0, theme::border_subtle()),
        shadow: egui::epaint::Shadow::NONE,
    };

    frame.show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        // Force dark text-field backgrounds
        ui.visuals_mut().extreme_bg_color = theme::bg_field();

        // ── Task Name ──────────────────────────────────────────────────
        ui.label(
            RichText::new("Name")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        let name_edit = ui.add_sized(
            [ui.available_width(), 24.0],
            egui::TextEdit::singleline(&mut task.name)
                .font(egui::FontId::proportional(12.0))
                .text_color(theme::text_primary()),
        );
        if name_edit.changed() {
            action = EditorAction::Changed;
        }

        ui.add_space(2.0);

        // ── Dates ──────────────────────────────────────────────
        if !task.is_milestone {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Start")
                            .size(10.0)
                            .color(theme::text_dim())
                            .strong(),
                    );
                    let mut start_str = task.start.format("%Y-%m-%d").to_string();
                    let resp = ui.add_sized(
                        [140.0, 24.0],
                        egui::TextEdit::singleline(&mut start_str)
                            .font(egui::FontId::proportional(11.0))
                            .text_color(theme::text_secondary()),
                    );
                    if resp.changed() {
                        if let Ok(d) = chrono::NaiveDate::parse_from_str(&start_str, "%Y-%m-%d") {
                            task.start = d;
                            if task.start > task.end {
                                task.end = task.start;
                            }
                            action = EditorAction::Changed;
                        }
                    }
                });

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("End")
                            .size(10.0)
                            .color(theme::text_dim())
                            .strong(),
                    );
                    let mut end_str = task.end.format("%Y-%m-%d").to_string();
                    let resp = ui.add_sized(
                        [140.0, 24.0],
                        egui::TextEdit::singleline(&mut end_str)
                            .font(egui::FontId::proportional(11.0))
                            .text_color(theme::text_secondary()),
                    );
                    if resp.changed() {
                        if let Ok(d) = chrono::NaiveDate::parse_from_str(&end_str, "%Y-%m-%d") {
                            task.end = d;
                            if task.end < task.start {
                                task.start = task.end;
                            }
                            action = EditorAction::Changed;
                        }
                    }
                });
            });
        } else {
            // Milestone: single date
            ui.label(
                RichText::new("Date")
                    .size(10.0)
                    .color(theme::text_dim())
                    .strong(),
            );
            let mut date_str = task.start.format("%Y-%m-%d").to_string();
            let resp = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(&mut date_str)
                    .font(egui::FontId::proportional(11.0))
                    .text_color(theme::text_secondary()),
            );
            if resp.changed() {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    task.start = d;
                    task.end = d;
                    action = EditorAction::Changed;
                }
            }
        }

        ui.add_space(2.0);

        // ── Progress ───────────────────────────────────────────
        ui.label(
            RichText::new("Progress")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        ui.horizontal(|ui| {
            let slider = egui::Slider::new(&mut task.progress, 0.0..=1.0)
                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                .custom_parser(|s| {
                    let s = s.trim().trim_end_matches('%');
                    s.parse::<f64>().ok().map(|v| v / 100.0)
                });
            let resp = ui.add_sized([ui.available_width(), 20.0], slider);
            if resp.changed() {
                action = EditorAction::Changed;
            }
        });

        ui.add_space(2.0);

        // ── Color ──────────────────────────────────────────────
        ui.label(
            RichText::new("Color")
                .size(10.0)
                .color(theme::text_dim())
                .strong(),
        );
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
            let palette = theme::task_palette();
            for color in &palette {
                let is_current = task.color == *color;
                let size = if is_current { 20.0 } else { 16.0 };
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

                let rounding = egui::Rounding::same(3.0);
                ui.painter().rect_filled(rect, rounding, *color);

                if is_current {
                    ui.painter().rect_stroke(
                        rect.expand(1.0),
                        egui::Rounding::same(4.0),
                        egui::Stroke::new(2.0, Color32::WHITE),
                    );
                }

                if resp.on_hover_text("Click to set color").clicked() {
                    task.color = *color;
                    action = EditorAction::Changed;
                }
            }
        });

        ui.add_space(2.0);

        // ── Milestone toggle ───────────────────────────────────
        ui.horizontal(|ui| {
            let mut is_milestone = task.is_milestone;
            let resp = ui.checkbox(&mut is_milestone, "");
            ui.label(
                RichText::new("Milestone")
                    .size(11.0)
                    .color(theme::text_secondary()),
            );
            if resp.changed() {
                task.is_milestone = is_milestone;
                if is_milestone {
                    task.end = task.start;
                }
                action = EditorAction::Changed;
            }
        });

        ui.add_space(4.0);

        // ── Dependencies ───────────────────────────────────────
        let task_deps: Vec<&Dependency> = dependencies
            .iter()
            .filter(|d| d.from_task == task.id || d.to_task == task.id)
            .collect();

        if !task_deps.is_empty() {
            ui.separator();
            ui.add_space(2.0);
            ui.label(
                RichText::new("Dependencies")
                    .size(10.0)
                    .color(theme::text_dim())
                    .strong(),
            );
            ui.add_space(2.0);

            for dep in &task_deps {
                let (label, other_id) = if dep.from_task == task.id {
                    let other_name = all_tasks
                        .iter()
                        .find(|t| t.id == dep.to_task)
                        .map(|t| t.name.as_str())
                        .unwrap_or("?");
                    (format!("→ {}", other_name), dep.to_task)
                } else {
                    let other_name = all_tasks
                        .iter()
                        .find(|t| t.id == dep.from_task)
                        .map(|t| t.name.as_str())
                        .unwrap_or("?");
                    (format!("← {}", other_name), dep.from_task)
                };
                let _ = other_id; // used for identification

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&label)
                            .size(11.0)
                            .color(theme::text_secondary()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let del = ui.add(
                            egui::Button::new(
                                RichText::new("✕").size(9.0).color(theme::text_dim()),
                            )
                            .frame(false),
                        );
                        if del.on_hover_text("Remove dependency").clicked() {
                            action = EditorAction::RemoveDependency(dep.from_task, dep.to_task);
                        }
                    });
                });
            }
        } else {
            ui.add_space(2.0);
            ui.label(
                RichText::new("Shift+drag between bars to link tasks")
                    .size(9.5)
                    .color(theme::text_dim()),
            );
        }
    });

    action
}
