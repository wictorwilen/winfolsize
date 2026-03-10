#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod scanner;
mod ui;
mod viz;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("WinFolSize — Disk Space Visualizer"),
        ..Default::default()
    };

    eframe::run_native(
        "WinFolSize",
        options,
        Box::new(|cc| Ok(Box::new(app::WinFolSizeApp::new(cc)))),
    )
}

