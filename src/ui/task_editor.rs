use crate::model::Task;
use crate::ui::theme;
use egui::{Color32, RichText, Ui};

/// Actions the editor can request.
pub enum EditorAction {
    None,
    Changed,
}

/// Render an inline task editor for the selected task.
/// Returns whether the task was modified.
pub fn show_task_editor(task: &mut Task, ui: &mut Ui) -> EditorAction {
    let mut action = EditorAction::None;

    // Section header
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Edit Task")
                .strong()
                .size(13.0)
                .color(theme::TEXT_PRIMARY),
        );
    });
    ui.add_space(4.0);

    let frame = egui::Frame {
        fill: theme::BG_DARK,
        rounding: egui::Rounding::same(6.0),
        inner_margin: egui::Margin::same(10.0),
        outer_margin: egui::Margin::ZERO,
        stroke: egui::Stroke::new(1.0, theme::BORDER_SUBTLE),
        shadow: egui::epaint::Shadow::NONE,
    };

    frame.show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        // Force dark text-field backgrounds
        ui.visuals_mut().extreme_bg_color = egui::Color32::from_rgb(20, 20, 28);

        // ── Task Name ──────────────────────────────────────────
        ui.label(
            RichText::new("Name")
                .size(10.0)
                .color(theme::TEXT_DIM)
                .strong(),
        );
        let name_edit = ui.add_sized(
            [ui.available_width(), 24.0],
            egui::TextEdit::singleline(&mut task.name)
                .font(egui::FontId::proportional(12.0))
                .text_color(theme::TEXT_PRIMARY),
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
                            .color(theme::TEXT_DIM)
                            .strong(),
                    );
                    let mut start_str = task.start.format("%Y-%m-%d").to_string();
                    let resp = ui.add_sized(
                        [140.0, 24.0],
                        egui::TextEdit::singleline(&mut start_str)
                            .font(egui::FontId::proportional(11.0))
                            .text_color(theme::TEXT_SECONDARY),
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
                            .color(theme::TEXT_DIM)
                            .strong(),
                    );
                    let mut end_str = task.end.format("%Y-%m-%d").to_string();
                    let resp = ui.add_sized(
                        [140.0, 24.0],
                        egui::TextEdit::singleline(&mut end_str)
                            .font(egui::FontId::proportional(11.0))
                            .text_color(theme::TEXT_SECONDARY),
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
                    .color(theme::TEXT_DIM)
                    .strong(),
            );
            let mut date_str = task.start.format("%Y-%m-%d").to_string();
            let resp = ui.add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(&mut date_str)
                    .font(egui::FontId::proportional(11.0))
                    .text_color(theme::TEXT_SECONDARY),
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
                .color(theme::TEXT_DIM)
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
                .color(theme::TEXT_DIM)
                .strong(),
        );
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
            for &color in theme::TASK_COLORS {
                let is_current = task.color == color;
                let size = if is_current { 20.0 } else { 16.0 };
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

                let rounding = egui::Rounding::same(3.0);
                ui.painter().rect_filled(rect, rounding, color);

                if is_current {
                    ui.painter().rect_stroke(
                        rect.expand(1.0),
                        egui::Rounding::same(4.0),
                        egui::Stroke::new(2.0, Color32::WHITE),
                    );
                }

                if resp.on_hover_text("Click to set color").clicked() {
                    task.color = color;
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
                    .color(theme::TEXT_SECONDARY),
            );
            if resp.changed() {
                task.is_milestone = is_milestone;
                if is_milestone {
                    task.end = task.start;
                }
                action = EditorAction::Changed;
            }
        });
    });

    action
}
