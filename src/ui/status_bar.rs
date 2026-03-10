use eframe::egui;

pub fn draw(ui: &mut egui::Ui, total_size: u64, file_count: u64, dir_count: u64, scan_duration: Option<f64>) {
    ui.horizontal(|ui| {
        ui.label(format!(
            "Total: {}",
            humansize::format_size(total_size, humansize::BINARY)
        ));
        ui.separator();
        ui.label(format!("{} files", file_count));
        ui.separator();
        ui.label(format!("{} folders", dir_count));
        if let Some(duration) = scan_duration {
            ui.separator();
            ui.label(format!("Scanned in {:.1}s", duration));
        }
    });
}
