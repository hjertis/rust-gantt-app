use crate::app::GanttApp;
use crate::ui::theme;
use egui::{Color32, Context, RichText, Window};

/// Render the "Add Task" dialog.
pub fn show_add_task_dialog(app: &mut GanttApp, ctx: &Context) {
    let mut should_close = false;
    let layout = theme::layout();
    let resp = Window::new(RichText::new("Add Task").strong().size(14.0))
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([layout.dialog_width, 0.0])
        .show(ctx, |ui| {
            // Force dark backgrounds inside this dialog
            ui.visuals_mut().extreme_bg_color = theme::bg_field();
            ui.visuals_mut().faint_bg_color = Color32::TRANSPARENT;
            ui.visuals_mut().striped = false;

            ui.add_space(4.0);

            egui::Grid::new("add_task_grid")
                .num_columns(2)
                .striped(false)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Name").color(theme::text_secondary()));
                    ui.add_sized(
                        [220.0, 24.0],
                        egui::TextEdit::singleline(&mut app.new_task_name)
                            .hint_text("Task name...")
                            .text_color(theme::text_primary()),
                    );
                    ui.end_row();

                    ui.label(RichText::new("Start").color(theme::text_secondary()));
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut app.new_task_start_date)
                            .id_salt("dlg_dp_start"),
                    );
                    ui.end_row();

                    ui.label(RichText::new("End").color(theme::text_secondary()));
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut app.new_task_end_date)
                            .id_salt("dlg_dp_end"),
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
                .fill(theme::accent())
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
    let layout = theme::layout();
    Window::new("About")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([layout.about_dialog_width, layout.about_dialog_height])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.heading(RichText::new("Rust Gantt App").strong());
                ui.add_space(2.0);
                ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).color(theme::text_secondary()));
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

/// Render the "CSV Import Format" help dialog.
pub fn show_csv_help_dialog(app: &mut GanttApp, ctx: &Context) {
    let mut should_close = false;

    Window::new(RichText::new("CSV Import Format").strong().size(14.0))
        .resizable(true)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([560.0, 500.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);

                // ── Delimiters ───────────────────────────────────────────
                ui.label(RichText::new("Delimiters").strong());
                ui.label("The delimiter is auto-detected: comma (,), semicolon (;), or tab.");
                ui.add_space(8.0);

                // ── Required columns ─────────────────────────────────────
                ui.label(RichText::new("Required Columns").strong());
                ui.add_space(2.0);
                egui::Grid::new("csv_required")
                    .num_columns(2)
                    .striped(true)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Column").underline());
                        ui.label(RichText::new("Accepted headers (case-insensitive)").underline());
                        ui.end_row();

                        ui.label(RichText::new("Task Name").strong());
                        ui.label("Name, Task, Task Label, Task Name, Label, Title, Activity");
                        ui.end_row();

                        ui.label(RichText::new("Start Date").strong());
                        ui.label("Start, Start Date, From, Begin, Begin Date");
                        ui.end_row();

                        ui.label(RichText::new("End Date").strong());
                        ui.label("End, End Date, To, Finish, Finish Date, Due, Due Date");
                        ui.end_row();
                    });
                ui.add_space(8.0);

                // ── Optional columns ─────────────────────────────────────
                ui.label(RichText::new("Optional Columns").strong());
                ui.add_space(2.0);
                egui::Grid::new("csv_optional")
                    .num_columns(3)
                    .striped(true)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Column").underline());
                        ui.label(RichText::new("Accepted headers").underline());
                        ui.label(RichText::new("Accepted values").underline());
                        ui.end_row();

                        ui.label(RichText::new("Status").strong());
                        ui.label("Status, State, Progress, Stage");
                        ui.label("Finished / Done / In Progress / Released / Planned / Not Started");
                        ui.end_row();

                        ui.label(RichText::new("Priority").strong());
                        ui.label("Priority, Pri, Importance");
                        ui.label("Critical / High / Medium / Low");
                        ui.end_row();

                        ui.label(RichText::new("Description").strong());
                        ui.label("Description, Notes, Note, Details, Comment");
                        ui.label("Any text");
                        ui.end_row();

                        ui.label(RichText::new("Parent").strong());
                        ui.label("Parent, Parent Task, Parent Name, Subtask Of");
                        ui.label("Name of the parent task (must exist in this file)");
                        ui.end_row();

                        ui.label(RichText::new("Milestone").strong());
                        ui.label("Milestone, Is Milestone, Type");
                        ui.label("true / false / yes / no / 1 / milestone");
                        ui.end_row();
                    });
                ui.add_space(8.0);

                // ── Date formats ─────────────────────────────────────────
                ui.label(RichText::new("Supported Date Formats").strong());
                ui.add_space(2.0);
                for fmt in &[
                    "YYYY-MM-DD   (e.g. 2025-06-15)",
                    "DD/MM/YYYY   (e.g. 15/06/2025)",
                    "MM/DD/YYYY   (e.g. 06/15/2025)",
                    "DD-MM-YYYY   (e.g. 15-06-2025)",
                    "DD.MM.YYYY   (e.g. 15.06.2025)",
                    "YYYY/MM/DD   (e.g. 2025/06/15)",
                ] {
                    ui.label(RichText::new(*fmt).monospace().size(11.0));
                }
                ui.add_space(8.0);

                // ── Notes ────────────────────────────────────────────────
                ui.label(RichText::new("Notes").strong());
                ui.add_space(2.0);
                let notes = [
                    "• Header matching is case-insensitive and ignores spaces, hyphens and underscores.",
                    "• Parent tasks are matched by name — the parent must appear somewhere in the same CSV.",
                    "• A task whose start date equals its end date is automatically treated as a milestone.",
                    "• If a parent name is not found a warning is logged and the task is imported without a parent.",
                    "• Rows with a missing or invalid name, start, or end date are skipped.",
                ];
                for note in &notes {
                    ui.label(RichText::new(*note).small());
                }
                ui.add_space(10.0);

                // ── Example ──────────────────────────────────────────────
                ui.label(RichText::new("Minimal Example (semicolon-delimited)").strong());
                ui.add_space(2.0);
                let example = "Task Label;Start Date;End Date;Status;Priority;Parent\n\
                               Phase 1;01/01/2025;28/02/2025;In Progress;High;\n\
                               Design;01/01/2025;31/01/2025;Finished;Medium;Phase 1\n\
                               Build;01/02/2025;28/02/2025;Not Started;High;Phase 1\n\
                               Launch;01/03/2025;01/03/2025;Not Started;Critical;";
                egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut example.to_string())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
                ui.add_space(8.0);
            });

            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.add_sized([80.0, 28.0], egui::Button::new("Close")).clicked() {
                    should_close = true;
                }
            });
            ui.add_space(2.0);
        });

    if should_close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_csv_help = false;
    }
}
