use crate::app::GanttApp;
use crate::ui::theme;
use egui::{Color32, Context, RichText, Window};

const FIELD_BG: Color32 = Color32::from_rgb(20, 20, 28);

/// Render the "Add Task" dialog.
pub fn show_add_task_dialog(app: &mut GanttApp, ctx: &Context) {
    let mut should_close = false;
    let resp = Window::new(RichText::new("Add Task").strong().size(14.0))
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([320.0, 0.0])
        .show(ctx, |ui| {
            // Force dark backgrounds inside this dialog
            ui.visuals_mut().extreme_bg_color = FIELD_BG;
            ui.visuals_mut().faint_bg_color = Color32::TRANSPARENT;
            ui.visuals_mut().striped = false;

            ui.add_space(4.0);

            egui::Grid::new("add_task_grid")
                .num_columns(2)
                .striped(false)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Name").color(theme::TEXT_SECONDARY));
                    ui.add_sized(
                        [220.0, 24.0],
                        egui::TextEdit::singleline(&mut app.new_task_name)
                            .hint_text("Task name...")
                            .text_color(theme::TEXT_PRIMARY),
                    );
                    ui.end_row();

                    ui.label(RichText::new("Start").color(theme::TEXT_SECONDARY));
                    ui.add_sized(
                        [220.0, 24.0],
                        egui::TextEdit::singleline(&mut app.new_task_start)
                            .hint_text("YYYY-MM-DD")
                            .text_color(theme::TEXT_PRIMARY),
                    );
                    ui.end_row();

                    ui.label(RichText::new("End").color(theme::TEXT_SECONDARY));
                    ui.add_sized(
                        [220.0, 24.0],
                        egui::TextEdit::singleline(&mut app.new_task_end)
                            .hint_text("YYYY-MM-DD")
                            .text_color(theme::TEXT_PRIMARY),
                    );
                    ui.end_row();

                    ui.label("");
                    ui.checkbox(&mut app.new_task_is_milestone, "Milestone");
                    ui.end_row();
                });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                let create_btn = egui::Button::new(
                    RichText::new("Create").color(Color32::WHITE),
                )
                .fill(theme::ACCENT)
                .rounding(egui::Rounding::same(4.0));
                if ui.add_sized([80.0, 28.0], create_btn).clicked() {
                    app.create_task_from_dialog();
                    should_close = true;
                }
                if ui.add_sized([80.0, 28.0], egui::Button::new("Cancel")).clicked() {
                    should_close = true;
                }
            });
            ui.add_space(2.0);
        });

    // Close on button click or if user clicked outside the window
    if should_close {
        app.show_add_task = false;
    }
    // Also handle Escape key to close
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_add_task = false;
    }
    // If the window title bar close button was clicked
    if let Some(resp) = resp {
        if resp.response.clicked_elsewhere() && !resp.response.has_focus() {
            // Don't close on click elsewhere — only buttons and Escape
        }
    }
}

/// Render the "About" dialog.
pub fn show_about_dialog(app: &mut GanttApp, ctx: &Context) {
    let mut should_close = false;
    Window::new("About")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([300.0, 160.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.heading(RichText::new("Rust Gantt App").strong());
                ui.add_space(2.0);
                ui.label(RichText::new("Version 0.1.0").color(theme::TEXT_SECONDARY));
                ui.add_space(10.0);
                ui.label("A Gantt chart application");
                ui.label("built with Rust and egui.");
                ui.add_space(14.0);
                if ui.add_sized([100.0, 28.0], egui::Button::new("Close")).clicked() {
                    should_close = true;
                }
            });
        });
    if should_close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_about = false;
    }
}
