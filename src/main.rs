#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod delete;
mod scanner;
mod ui;
mod viz;

fn make_icon() -> eframe::egui::IconData {
    let size = 64u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    // Draw a mini treemap icon with colored rectangles
    let blocks: &[(u32, u32, u32, u32, [u8; 3])] = &[
        (0, 0, 38, 38, [66, 133, 244]),   // blue (top-left)
        (40, 0, 24, 18, [234, 67, 53]),    // red (top-right upper)
        (40, 20, 24, 18, [251, 188, 4]),   // yellow (top-right lower)
        (0, 40, 20, 24, [52, 168, 83]),    // green (bottom-left)
        (22, 40, 42, 24, [171, 71, 188]),  // purple (bottom-right)
    ];
    for &(bx, by, bw, bh, color) in blocks {
        for y in by..by + bh {
            for x in bx..bx + bw {
                let i = ((y * size + x) * 4) as usize;
                rgba[i] = color[0];
                rgba[i + 1] = color[1];
                rgba[i + 2] = color[2];
                rgba[i + 3] = 255;
            }
        }
    }
    eframe::egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("WinFolSize — Disk Space Visualizer")
            .with_icon(std::sync::Arc::new(make_icon())),
        ..Default::default()
    };

    eframe::run_native(
        "WinFolSize",
        options,
        Box::new(|cc| Ok(Box::new(app::WinFolSizeApp::new(cc)))),
    )
}
