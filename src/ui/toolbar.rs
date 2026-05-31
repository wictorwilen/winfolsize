use eframe::egui;

use crate::app::{AppState, ViewMode};

pub fn draw(ui: &mut egui::Ui, state: &mut AppState, show_about: &mut bool) {
    ui.horizontal(|ui| {
        // App logo
        ui.label(egui::RichText::new("📊").size(20.0));
        ui.label(egui::RichText::new("WinFolSize").strong().size(16.0));
        ui.separator();

        if ui.button("📁 Select Folder").clicked() {
            let mut dialog = rfd::FileDialog::new();
            if let Some(ref current) = state.selected_path {
                if current.is_dir() {
                    dialog = dialog.set_directory(current);
                } else if let Some(parent) = current.parent() {
                    dialog = dialog.set_directory(parent);
                }
            }
            if let Some(path) = dialog.pick_folder() {
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
                // Show progress bar if we have an estimate
                if let Some(total) = state.estimated_total_bytes {
                    if total > 0 {
                        let fraction = (progress.bytes_scanned as f32 / total as f32).min(1.0);
                        let bar = egui::ProgressBar::new(fraction)
                            .text(format!(
                                "{} / {} ({:.0}%) — {} files",
                                humansize::format_size(progress.bytes_scanned, humansize::BINARY),
                                humansize::format_size(total, humansize::BINARY),
                                fraction * 100.0,
                                progress.files_scanned,
                            ));
                        ui.add_sized([220.0, 18.0], bar);
                    }
                } else {
                    ui.label(format!(
                        "{} files scanned ({})",
                        progress.files_scanned,
                        humansize::format_size(progress.bytes_scanned, humansize::BINARY),
                    ));
                }
            }
        }

        ui.add_space(16.0);

        ui.label("View:");
        ui.selectable_value(&mut state.view_mode, ViewMode::Treemap, "🗺 Treemap");
        ui.selectable_value(&mut state.view_mode, ViewMode::Sunburst, "☀ Sunburst");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("ℹ About").clicked() {
                *show_about = true;
            }
        });
    });
}
