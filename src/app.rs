use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Instant;

use crate::delete;
use crate::scanner::types::{FileNode, ScanMessage, ScanProgress};
use crate::scanner::walker;
use crate::ui::{sidebar, status_bar, toolbar};
use crate::viz::{sunburst, treemap};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Treemap,
    Sunburst,
}

pub struct AppState {
    pub selected_path: Option<PathBuf>,
    pub view_mode: ViewMode,
    pub is_scanning: bool,
    pub start_scan: bool,
    pub scan_progress: Option<ScanProgress>,
    pub scan_error: Option<String>,
    pub estimated_total_bytes: Option<u64>,
}

pub struct WinFolSizeApp {
    pub state: AppState,
    scan_receiver: Option<mpsc::Receiver<ScanMessage>>,
    scan_root: Option<FileNode>,
    scan_start_time: Option<Instant>,
    scan_duration: Option<f64>,
    // Cached layouts
    treemap_rects: Vec<treemap::TreemapRect>,
    sunburst_arcs: Vec<sunburst::SunburstArc>,
    last_viz_size: egui::Vec2,
    // Navigation
    drill_path: Vec<usize>,
    drill_depths: Vec<usize>, // how many indices each drill-down step pushed
    breadcrumb: Vec<String>,
    show_about: bool,
    // Context menu for right-click delete
    context_menu: Option<ContextMenuItem>,
    delete_error: Option<String>,
}

#[derive(Clone)]
struct ContextMenuItem {
    name: String,
    path: PathBuf,
    node_index: Vec<usize>,
    is_dir: bool,
    size: u64,
}

impl WinFolSizeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState {
                selected_path: Some(std::path::PathBuf::from("C:\\")),
                view_mode: ViewMode::Treemap,
                is_scanning: false,
                start_scan: false,
                scan_progress: None,
                scan_error: None,
                estimated_total_bytes: None,
            },
            scan_receiver: None,
            scan_root: None,
            scan_start_time: None,
            scan_duration: None,
            treemap_rects: Vec::new(),
            sunburst_arcs: Vec::new(),
            last_viz_size: egui::Vec2::ZERO,
            drill_path: Vec::new(),
            drill_depths: Vec::new(),
            breadcrumb: Vec::new(),
            show_about: false,
            context_menu: None,
            delete_error: None,
        }
    }

    fn current_node(&self) -> Option<&FileNode> {
        let root = self.scan_root.as_ref()?;
        let mut node = root;
        for &idx in &self.drill_path {
            if idx < node.children.len() {
                node = &node.children[idx];
            } else {
                return Some(root);
            }
        }
        Some(node)
    }

    /// Resolve a node_index (relative to current view) to its full filesystem path.
    fn resolve_path(&self, node_index: &[usize]) -> Option<String> {
        let root = self.scan_root.as_ref()?;
        let mut node = root;
        // Walk drill_path first
        for &idx in &self.drill_path {
            if idx < node.children.len() {
                node = &node.children[idx];
            } else {
                return None;
            }
        }
        // Then walk node_index
        for &idx in node_index {
            if idx < node.children.len() {
                node = &node.children[idx];
            } else {
                return None;
            }
        }
        Some(node.path.to_string_lossy().to_string())
    }

    /// Resolve a node_index (relative to current view) to its PathBuf.
    fn resolve_pathbuf(&self, node_index: &[usize]) -> Option<PathBuf> {
        let root = self.scan_root.as_ref()?;
        let mut node = root;
        for &idx in &self.drill_path {
            if idx < node.children.len() {
                node = &node.children[idx];
            } else {
                return None;
            }
        }
        for &idx in node_index {
            if idx < node.children.len() {
                node = &node.children[idx];
            } else {
                return None;
            }
        }
        Some(node.path.clone())
    }

    /// Remove a node from the scan tree by its full index path (drill_path + node_index)
    /// and recalculate sizes.
    fn remove_node(&mut self, node_index: &[usize]) {
        let Some(root) = self.scan_root.as_mut() else { return };
        // Build full path: drill_path + node_index
        let full_path: Vec<usize> = self.drill_path.iter().chain(node_index.iter()).copied().collect();
        if full_path.is_empty() {
            return;
        }
        // Navigate to the parent
        let parent_indices = &full_path[..full_path.len() - 1];
        let child_idx = full_path[full_path.len() - 1];
        let mut node = &mut *root;
        for &idx in parent_indices {
            if idx < node.children.len() {
                node = &mut node.children[idx];
            } else {
                return;
            }
        }
        if child_idx < node.children.len() {
            node.children.remove(child_idx);
        }
        // Recalculate sizes from root
        root.recalculate_sizes();
        self.invalidate_layout();
    }

    fn start_scanning(&mut self) {
        if let Some(ref path) = self.state.selected_path {
            // Estimate total bytes from disk used space for progress bar
            self.state.estimated_total_bytes = disk_used_bytes(path);

            let (tx, rx) = mpsc::channel();
            walker::start_scan(path, tx);
            self.scan_receiver = Some(rx);
            self.state.is_scanning = true;
            self.state.scan_progress = None;
            self.state.scan_error = None;
            self.scan_root = None;
            self.scan_start_time = Some(Instant::now());
            self.scan_duration = None;
            self.drill_path.clear();
            self.drill_depths.clear();
            self.breadcrumb.clear();
            self.treemap_rects.clear();
            self.sunburst_arcs.clear();
        }
    }

    fn poll_scan(&mut self) {
        let mut completed_root = None;
        let mut error_msg = None;

        if let Some(ref receiver) = self.scan_receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    ScanMessage::Progress(progress) => {
                        self.state.scan_progress = Some(progress);
                    }
                    ScanMessage::Complete(root) => {
                        completed_root = Some(root);
                    }
                    ScanMessage::Error(err) => {
                        error_msg = Some(err);
                    }
                }
            }
        }

        if let Some(root) = completed_root {
            self.scan_root = Some(root);
            self.state.is_scanning = false;
            self.scan_duration = self.scan_start_time.map(|t| t.elapsed().as_secs_f64());
            self.invalidate_layout();
        }
        if let Some(err) = error_msg {
            self.state.scan_error = Some(err);
            self.state.is_scanning = false;
        }
    }

    fn invalidate_layout(&mut self) {
        self.last_viz_size = egui::Vec2::ZERO;
    }
}

impl eframe::App for WinFolSizeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start scan if requested
        if self.state.start_scan {
            self.state.start_scan = false;
            self.start_scanning();
        }

        // Poll scan channel
        self.poll_scan();

        // Request repaint while scanning
        if self.state.is_scanning {
            ctx.request_repaint();
        }

        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            toolbar::draw(ui, &mut self.state, &mut self.show_about);
            ui.add_space(4.0);
        });

        // About dialog
        if self.show_about {
            egui::Window::new("About WinFolSize")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .fixed_size([340.0, 220.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("📊").size(48.0));
                        ui.add_space(4.0);
                        ui.heading(egui::RichText::new("WinFolSize").strong());
                        ui.label("Disk Space Visualizer");
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Built with ❤️ by Wictor Wilén").strong());
                        ui.hyperlink_to("www.wictorwilen.se", "https://www.wictorwilen.se");
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("© 2026 Wictor Wilén · MIT License").weak());
                        ui.add_space(12.0);
                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }

        // Context menu for right-click delete
        let mut delete_action: Option<(Vec<usize>, PathBuf, bool)> = None;
        if let Some(ref item) = self.context_menu {
            let mut open = true;
            egui::Window::new("context_menu")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .fixed_size([220.0, 0.0])
                .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                .current_pos(ctx.input(|i| i.pointer.latest_pos().unwrap_or_default()))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(&item.name).strong());
                    ui.label(egui::RichText::new(format_size(item.size)).weak().size(11.0));
                    ui.separator();
                    if ui.button("🗑 Move to Recycle Bin").clicked() {
                        delete_action = Some((item.node_index.clone(), item.path.clone(), false));
                    }
                    if ui.button("❌ Delete permanently").clicked() {
                        delete_action = Some((item.node_index.clone(), item.path.clone(), true));
                    }
                });
            if !open {
                self.context_menu = None;
            }
        }
        // Handle delete action outside the borrow
        if let Some((node_index, path, permanent)) = delete_action {
            let result = if permanent {
                delete::permanent_delete(&path)
            } else {
                delete::recycle(&path)
            };
            match result {
                Ok(()) => {
                    self.remove_node(&node_index);
                    self.delete_error = None;
                }
                Err(e) => {
                    self.delete_error = Some(e);
                }
            }
            self.context_menu = None;
        }
        // Close context menu on any click outside it
        if self.context_menu.is_some()
            && ctx.input(|i| i.pointer.primary_clicked())
        {
            self.context_menu = None;
        }

        // Show delete error briefly
        if self.delete_error.is_some() {
            let err_text = self.delete_error.clone().unwrap();
            let mut dismiss = false;
            egui::Window::new("Delete Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, &err_text);
                    if ui.button("OK").clicked() {
                        dismiss = true;
                    }
                });
            if dismiss {
                self.delete_error = None;
            }
        }
        if self.scan_root.is_some() {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                if let Some(ref root) = self.scan_root {
                    status_bar::draw(
                        ui,
                        root.total_size(),
                        root.file_count(),
                        root.dir_count(),
                        self.scan_duration,
                    );
                }
            });
        }

        // Right sidebar
        let mut hovered_info: Option<sidebar::HoveredInfo> = None;

        egui::SidePanel::right("sidebar")
            .min_width(200.0)
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    sidebar::draw(
                        ui,
                        hovered_info.as_ref(),
                        self.scan_root.as_ref(),
                        &self.breadcrumb,
                    );
                });
            });

        // Central panel with visualization
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref error) = self.state.scan_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                return;
            }

            if self.scan_root.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.heading("Select a folder and click Scan to begin");
                });
                return;
            }

            // Get current node data (clone to avoid borrow issues)
            let current_data = self.current_node().map(|n| (n.name.clone(), n.size, n.children.is_empty()));
            if current_data.is_none() {
                return;
            }
            let (current_name, current_size, children_empty) = current_data.unwrap();

            let _available = ui.available_rect_before_wrap();

            // Navigation: back button
            if !self.drill_path.is_empty() {
                if ui.button("⬅ Back").clicked() {
                    let depth = self.drill_depths.pop().unwrap_or(1);
                    self.drill_path.truncate(self.drill_path.len().saturating_sub(depth));
                    self.breadcrumb.pop();
                    self.invalidate_layout();
                }
            }

            let viz_rect = ui.available_rect_before_wrap();

            // Recompute layout if size changed
            let size_changed = (viz_rect.size() - self.last_viz_size).length() > 1.0;
            if size_changed || self.treemap_rects.is_empty() && self.sunburst_arcs.is_empty() {
                self.last_viz_size = viz_rect.size();
                if let Some(current) = self.current_node() {
                    match self.state.view_mode {
                        ViewMode::Treemap => {
                            self.treemap_rects = treemap::layout(current, viz_rect);
                        }
                        ViewMode::Sunburst => {
                            let center = viz_rect.center();
                            let max_radius = viz_rect.width().min(viz_rect.height()) / 2.0 * 0.95;
                            self.sunburst_arcs = sunburst::layout(current, center, max_radius, 6);
                        }
                    }
                }
            }

            // Also recompute if view mode doesn't match cached data
            match self.state.view_mode {
                ViewMode::Treemap => {
                    if self.treemap_rects.is_empty() && !children_empty {
                        if let Some(current) = self.current_node() {
                            self.treemap_rects = treemap::layout(current, viz_rect);
                        }
                    }

                    let hovered_rect = treemap::draw(ui, &self.treemap_rects, viz_rect);

                    if let Some(ref tr) = hovered_rect {
                        hovered_info = Some(sidebar::HoveredInfo {
                            name: tr.name.clone(),
                            size: tr.size,
                            is_dir: tr.is_dir,
                            extension: tr.extension.clone(),
                            path: None,
                        });

                        // Show tooltip
                        egui::show_tooltip_at_pointer(
                            ctx,
                            ui.layer_id(),
                            egui::Id::new("treemap_tooltip"),
                            |ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                ui.label(egui::RichText::new(&tr.name).strong());
                                if let Some(path) = self.resolve_path(&tr.node_index) {
                                    ui.label(egui::RichText::new(path).weak().size(11.0));
                                }
                                ui.label(format_size(tr.size));
                                if tr.is_dir {
                                    ui.label("Directory — click to drill in");
                                }
                            },
                        );

                        // Click to drill into directories
                        if ui.input(|i| i.pointer.primary_clicked()) && tr.is_dir {
                            if !tr.node_index.is_empty() {
                                self.drill_depths.push(tr.node_index.len());
                                self.drill_path.extend_from_slice(&tr.node_index);
                                self.breadcrumb.push(tr.name.clone());
                                self.invalidate_layout();
                            }
                        }

                        // Right-click to open context menu
                        if ui.input(|i| i.pointer.secondary_clicked()) {
                            if let Some(path) = self.resolve_pathbuf(&tr.node_index) {
                                self.context_menu = Some(ContextMenuItem {
                                    name: tr.name.clone(),
                                    path,
                                    node_index: tr.node_index.clone(),
                                    is_dir: tr.is_dir,
                                    size: tr.size,
                                });
                            }
                        }
                    }
                }
                ViewMode::Sunburst => {
                    let center = viz_rect.center();
                    let max_radius = viz_rect.width().min(viz_rect.height()) / 2.0 * 0.95;

                    if self.sunburst_arcs.is_empty() && !children_empty {
                        if let Some(current) = self.current_node() {
                            self.sunburst_arcs = sunburst::layout(current, center, max_radius, 6);
                        }
                    }

                    let hovered_arc = sunburst::draw(
                        ui,
                        &self.sunburst_arcs,
                        center,
                        max_radius,
                        &current_name,
                        current_size,
                    );

                    if let Some(ref arc) = hovered_arc {
                        hovered_info = Some(sidebar::HoveredInfo {
                            name: arc.name.clone(),
                            size: arc.size,
                            is_dir: arc.is_dir,
                            extension: arc.extension.clone(),
                            path: None,
                        });

                        egui::show_tooltip_at_pointer(
                            ctx,
                            ui.layer_id(),
                            egui::Id::new("sunburst_tooltip"),
                            |ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                ui.label(egui::RichText::new(&arc.name).strong());
                                if let Some(path) = self.resolve_path(&arc.node_index) {
                                    ui.label(egui::RichText::new(path).weak().size(11.0));
                                }
                                ui.label(format_size(arc.size));
                                if arc.is_dir {
                                    ui.label("Directory — click to drill in");
                                }
                            },
                        );

                        if ui.input(|i| i.pointer.primary_clicked()) && arc.is_dir {
                            if !arc.node_index.is_empty() {
                                self.drill_depths.push(arc.node_index.len());
                                self.drill_path.extend_from_slice(&arc.node_index);
                                self.breadcrumb.push(arc.name.clone());
                                self.invalidate_layout();
                            }
                        }

                        // Right-click to open context menu
                        if ui.input(|i| i.pointer.secondary_clicked()) {
                            if let Some(path) = self.resolve_pathbuf(&arc.node_index) {
                                self.context_menu = Some(ContextMenuItem {
                                    name: arc.name.clone(),
                                    path,
                                    node_index: arc.node_index.clone(),
                                    is_dir: arc.is_dir,
                                    size: arc.size,
                                });
                            }
                        }
                    }
                }
            }

            // Update sidebar hovered info (we drew the sidebar before the central panel,
            // so this won't update until next frame — acceptable for tooltips)
            let _ = _available;
        });
    }
}

fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}

/// Get used bytes on the disk containing `path` (total - free).
fn disk_used_bytes(path: &std::path::Path) -> Option<u64> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes_available,
                &mut total_bytes,
                &mut total_free_bytes,
            )
        };
        if ok != 0 {
            Some(total_bytes.saturating_sub(total_free_bytes))
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}
