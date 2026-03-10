use eframe::egui;

use crate::scanner::types::FileNode;
use crate::viz::colors::FileCategory;

pub fn draw(ui: &mut egui::Ui, hovered: Option<&HoveredInfo>, root: Option<&FileNode>, breadcrumb: &[String]) {
    ui.vertical(|ui| {
        ui.heading("Details");
        ui.separator();

        // Breadcrumb navigation
        if !breadcrumb.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label("📍");
                for (i, part) in breadcrumb.iter().enumerate() {
                    if i > 0 {
                        ui.label("›");
                    }
                    ui.label(part);
                }
            });
            ui.separator();
        }

        // Hovered item details
        if let Some(info) = hovered {
            ui.label(egui::RichText::new(&info.name).strong());
            ui.label(format!("Size: {}", format_size(info.size)));
            if info.is_dir {
                ui.label("Type: Directory");
            } else if let Some(ref ext) = info.extension {
                let cat = crate::viz::colors::categorize_extension(ext);
                ui.label(format!("Type: {} (.{})", cat.label(), ext));
            }
            if let Some(ref path) = info.path {
                ui.label(format!("Path: {}", path));
            }
        } else {
            ui.label("Hover over the chart to see details");
        }

        ui.add_space(16.0);
        ui.separator();
        ui.heading("Legend");
        ui.add_space(4.0);

        // File type legend
        for cat in FileCategory::all() {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(14.0, 14.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, cat.color());
                ui.label(cat.label());
            });
        }

        // Summary stats
        if let Some(root) = root {
            ui.add_space(16.0);
            ui.separator();
            ui.heading("Summary");
            ui.label(format!("Total size: {}", format_size(root.total_size())));
            ui.label(format!("Files: {}", root.file_count()));
            ui.label(format!("Directories: {}", root.dir_count()));
        }
    });
}

#[derive(Debug, Clone)]
pub struct HoveredInfo {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub path: Option<String>,
}

fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}
