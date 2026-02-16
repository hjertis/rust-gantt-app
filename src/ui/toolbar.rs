use crate::app::GanttApp;
use crate::ui::theme;
use egui::{menu, RichText, Ui};

/// Render the top toolbar / menu bar.
pub fn show_toolbar(app: &mut GanttApp, ui: &mut Ui) {
    menu::bar(ui, |ui| {
        ui.menu_button(RichText::new("  File  ").font(theme::font_menu()), |ui| {
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

        ui.menu_button(RichText::new("  View  ").font(theme::font_menu()), |ui| {
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
            ui.separator();
            ui.label(RichText::new("Theme").small().weak());
            let themes = app.theme_manager.list();
            let active_idx = app.theme_manager.active_index();
            for (idx, name) in &themes {
                let selected = *idx == active_idx;
                if ui.radio(selected, name).clicked() {
                    app.theme_manager.set_active(*idx);
                    ui.close_menu();
                }
            }
            ui.separator();
            if ui.button("  Reload Themes").clicked() {
                app.theme_manager.reload_user_themes();
                ui.close_menu();
            }
            if ui.button("  Open Themes Folder").clicked() {
                let dir = app.theme_manager.themes_dir().clone();
                let _ = open::that(&dir);
                ui.close_menu();
            }
        });

        ui.menu_button(RichText::new("  Help  ").font(theme::font_menu()), |ui| {
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
