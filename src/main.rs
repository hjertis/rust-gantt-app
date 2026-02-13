#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod io;
mod model;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 400.0])
            .with_title("Rust Gantt App"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Gantt App",
        options,
        Box::new(|cc| Ok(Box::new(app::GanttApp::new(cc)))),
    )
}
