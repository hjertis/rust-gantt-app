use crate::app::GanttApp;
use egui::{menu, RichText, Ui};

/// Render the top toolbar / menu bar.
pub fn show_toolbar(app: &mut GanttApp, ui: &mut Ui) {
    menu::bar(ui, |ui| {
        ui.menu_button(RichText::new("  File  ").size(12.0), |ui| {
            if ui.button("  New Project").clicked() {
                app.new_project();
                ui.close_menu();
            }
            if ui.button("  Open...").clicked() {
                app.open_project();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("  Save          Ctrl+S").clicked() {
                app.save_project();
                ui.close_menu();
            }
            if ui.button("  Save As...").clicked() {
                app.save_project_as();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("  Import CSV...").clicked() {
                app.import_csv();
                ui.close_menu();
            }
            if ui.button("  Export CSV...").clicked() {
                app.export_csv();
                ui.close_menu();
            }
        });

        ui.menu_button(RichText::new("  View  ").size(12.0), |ui| {
            if ui.button("  Zoom In        Ctrl+Scroll ↑").clicked() {
                app.viewport.zoom_in();
                ui.close_menu();
            }
            if ui.button("  Zoom Out      Ctrl+Scroll ↓").clicked() {
                app.viewport.zoom_out();
                ui.close_menu();
            }
            ui.separator();
            ui.label(RichText::new("Timeline Scale").small().weak());
            if ui
                .radio_value(
                    &mut app.viewport.scale,
                    crate::model::TimelineScale::Days,
                    "Days",
                )
                .clicked()
            {
                ui.close_menu();
            }
            if ui
                .radio_value(
                    &mut app.viewport.scale,
                    crate::model::TimelineScale::Weeks,
                    "Weeks",
                )
                .clicked()
            {
                ui.close_menu();
            }
            if ui
                .radio_value(
                    &mut app.viewport.scale,
                    crate::model::TimelineScale::Months,
                    "Months",
                )
                .clicked()
            {
                ui.close_menu();
            }
        });

        ui.menu_button(RichText::new("  Help  ").size(12.0), |ui| {
            if ui.button("About").clicked() {
                app.show_about = true;
                ui.close_menu();
            }
        });

        // Right-aligned project name
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let modified = if app.file_path.is_some() { "" } else { " (unsaved)" };
            ui.label(
                RichText::new(format!("{}{}", app.project.name, modified))
                    .size(11.0)
                    .weak(),
            );
        });
    });
}
