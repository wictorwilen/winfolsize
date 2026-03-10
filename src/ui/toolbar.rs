use eframe::egui;

use crate::app::{AppState, ViewMode};

pub fn draw(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui.button("📁 Select Folder").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.selected_path = Some(path);
            }
        }

        if let Some(ref path) = state.selected_path {
            ui.label(format!("📂 {}", path.display()));
        }

        ui.add_space(16.0);

        let can_scan = state.selected_path.is_some() && !state.is_scanning;
        if ui.add_enabled(can_scan, egui::Button::new("🔍 Scan")).clicked() {
            state.start_scan = true;
        }

        if state.is_scanning {
            ui.spinner();
            if let Some(ref progress) = state.scan_progress {
                ui.label(format!(
                    "{} files scanned ({})",
                    progress.files_scanned,
                    humansize::format_size(progress.bytes_scanned, humansize::BINARY),
                ));
            }
        }

        ui.add_space(16.0);

        ui.label("View:");
        ui.selectable_value(&mut state.view_mode, ViewMode::Treemap, "🗺 Treemap");
        ui.selectable_value(&mut state.view_mode, ViewMode::Sunburst, "☀ Sunburst");
    });
}
