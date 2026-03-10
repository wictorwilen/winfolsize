use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

use crate::scanner::types::FileNode;
use crate::viz::colors;

/// A laid-out rectangle for a file/folder in the treemap.
#[derive(Debug, Clone)]
pub struct TreemapRect {
    pub rect: Rect,
    pub node_index: Vec<usize>, // path of child indices to reach this node
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub color: Color32,
    pub depth: usize,
}

/// Compute squarified treemap layout for the children of `node` within `bounds`.
/// Recursively lays out all levels — directories contain their children.
pub fn layout(node: &FileNode, bounds: Rect) -> Vec<TreemapRect> {
    let mut result = Vec::new();
    if node.children.is_empty() || node.size == 0 {
        return result;
    }
    layout_recursive(&node.children, bounds, &[], 0, &mut result);
    result
}

const MAX_LAYOUT_DEPTH: usize = 8;
const DIR_PADDING: f32 = 2.0;

fn layout_recursive(
    children: &[FileNode],
    bounds: Rect,
    parent_path: &[usize],
    depth: usize,
    result: &mut Vec<TreemapRect>,
) {
    if depth > MAX_LAYOUT_DEPTH || bounds.width() < 3.0 || bounds.height() < 3.0 {
        return;
    }
    squarify(children, bounds, parent_path, depth, result);
}

fn squarify(
    children: &[FileNode],
    bounds: Rect,
    parent_path: &[usize],
    depth: usize,
    result: &mut Vec<TreemapRect>,
) {
    let total_size: u64 = children.iter().map(|c| c.size).sum();
    if total_size == 0 || bounds.width() < 1.0 || bounds.height() < 1.0 {
        return;
    }

    let area = bounds.width() as f64 * bounds.height() as f64;

    let items: Vec<(usize, f64)> = children
        .iter()
        .enumerate()
        .filter(|(_, c)| c.size > 0)
        .map(|(i, c)| (i, (c.size as f64 / total_size as f64) * area))
        .collect();

    if items.is_empty() {
        return;
    }

    layout_strip(&items, children, bounds, parent_path, depth, result);
}

fn layout_strip(
    items: &[(usize, f64)],
    children: &[FileNode],
    bounds: Rect,
    parent_path: &[usize],
    depth: usize,
    result: &mut Vec<TreemapRect>,
) {
    if items.is_empty() || bounds.width() < 1.0 || bounds.height() < 1.0 {
        return;
    }

    if items.len() == 1 {
        let (idx, _) = items[0];
        let child = &children[idx];
        let mut path = parent_path.to_vec();
        path.push(idx);

        let color = if child.is_dir {
            dir_color(child)
        } else {
            colors::color_for_extension(child.extension.as_deref())
        };

        result.push(TreemapRect {
            rect: bounds,
            node_index: path.clone(),
            name: child.name.clone(),
            size: child.size,
            is_dir: child.is_dir,
            extension: child.extension.clone(),
            color,
            depth,
        });

        // Recurse into directory children
        if child.is_dir && !child.children.is_empty() {
            let inner = bounds.shrink(DIR_PADDING);
            if inner.width() > 4.0 && inner.height() > 4.0 {
                layout_recursive(&child.children, inner, &path, depth + 1, result);
            }
        }
        return;
    }

    let total_area: f64 = items.iter().map(|(_, a)| a).sum();
    let is_wide = bounds.width() >= bounds.height();
    let side = if is_wide {
        bounds.height() as f64
    } else {
        bounds.width() as f64
    };

    // Find best split using squarified algorithm
    let mut best_split = 1;
    let mut best_worst_ratio = f64::MAX;

    for split in 1..=items.len() {
        let strip_area: f64 = items[..split].iter().map(|(_, a)| a).sum();
        let strip_side = strip_area / side;
        if strip_side <= 0.0 {
            continue;
        }

        let worst_ratio = items[..split]
            .iter()
            .map(|(_, a)| {
                let other_side = a / strip_side;
                if other_side > strip_side {
                    other_side / strip_side
                } else {
                    strip_side / other_side
                }
            })
            .fold(0.0_f64, f64::max);

        if worst_ratio <= best_worst_ratio {
            best_worst_ratio = worst_ratio;
            best_split = split;
        } else {
            break;
        }
    }

    let strip_items = &items[..best_split];
    let rest_items = &items[best_split..];
    let strip_area: f64 = strip_items.iter().map(|(_, a)| a).sum();
    let strip_fraction = strip_area / total_area;

    let (strip_bounds, rest_bounds) = if is_wide {
        let split_x = bounds.left() + bounds.width() * strip_fraction as f32;
        (
            Rect::from_min_max(bounds.left_top(), Pos2::new(split_x, bounds.bottom())),
            Rect::from_min_max(Pos2::new(split_x, bounds.top()), bounds.right_bottom()),
        )
    } else {
        let split_y = bounds.top() + bounds.height() * strip_fraction as f32;
        (
            Rect::from_min_max(bounds.left_top(), Pos2::new(bounds.right(), split_y)),
            Rect::from_min_max(Pos2::new(bounds.left(), split_y), bounds.right_bottom()),
        )
    };

    // Place items in the strip
    let mut offset = 0.0_f32;
    for &(idx, item_area) in strip_items {
        let fraction = item_area / strip_area;
        let child = &children[idx];
        let mut path = parent_path.to_vec();
        path.push(idx);

        let item_rect = if is_wide {
            let h = strip_bounds.height() * fraction as f32;
            let r = Rect::from_min_size(
                Pos2::new(strip_bounds.left(), strip_bounds.top() + offset),
                Vec2::new(strip_bounds.width(), h),
            );
            offset += h;
            r
        } else {
            let w = strip_bounds.width() * fraction as f32;
            let r = Rect::from_min_size(
                Pos2::new(strip_bounds.left() + offset, strip_bounds.top()),
                Vec2::new(w, strip_bounds.height()),
            );
            offset += w;
            r
        };

        let color = if child.is_dir {
            dir_color(child)
        } else {
            colors::color_for_extension(child.extension.as_deref())
        };

        result.push(TreemapRect {
            rect: item_rect,
            node_index: path.clone(),
            name: child.name.clone(),
            size: child.size,
            is_dir: child.is_dir,
            extension: child.extension.clone(),
            color,
            depth,
        });

        // Recurse into directory children
        if child.is_dir && !child.children.is_empty() {
            let inner = item_rect.shrink(DIR_PADDING);
            if inner.width() > 4.0 && inner.height() > 4.0 {
                layout_recursive(&child.children, inner, &path, depth + 1, result);
            }
        }
    }

    // Recurse for remaining items
    if !rest_items.is_empty() && rest_bounds.width() >= 1.0 && rest_bounds.height() >= 1.0 {
        layout_strip(rest_items, children, rest_bounds, parent_path, depth, result);
    }
}

fn dir_color(node: &FileNode) -> Color32 {
    // Find the dominant file type in this directory
    let mut category_sizes = std::collections::HashMap::new();
    accumulate_categories(node, &mut category_sizes);

    category_sizes
        .into_iter()
        .max_by_key(|(_, size)| *size)
        .map(|(cat, _)| {
            let base = cat.color();
            // Darken slightly for directories
            Color32::from_rgb(
                (base.r() as u16 * 3 / 4) as u8,
                (base.g() as u16 * 3 / 4) as u8,
                (base.b() as u16 * 3 / 4) as u8,
            )
        })
        .unwrap_or(Color32::from_rgb(80, 80, 80))
}

fn accumulate_categories(
    node: &FileNode,
    categories: &mut std::collections::HashMap<colors::FileCategory, u64>,
) {
    if node.is_dir {
        for child in &node.children {
            accumulate_categories(child, categories);
        }
    } else {
        let cat = colors::category_for_extension(node.extension.as_deref());
        *categories.entry(cat).or_insert(0) += node.size;
    }
}

/// Draw the treemap onto the egui UI. Returns the hovered TreemapRect if any.
pub fn draw(
    ui: &mut egui::Ui,
    rects: &[TreemapRect],
    available_rect: Rect,
) -> Option<TreemapRect> {
    let painter = ui.painter_at(available_rect);
    let mut hovered: Option<TreemapRect> = None;

    // Draw in depth order: deeper items first (they are smaller and sit inside parents)
    // The layout already produces them in a reasonable order (parent before children),
    // so we draw all fills first, then all borders on top.

    // Pass 1: fill all rectangles
    for tr in rects {
        if tr.rect.width() < 1.0 || tr.rect.height() < 1.0 {
            continue;
        }
        painter.rect_filled(tr.rect, 0.0, tr.color);
    }

    // Pass 2: draw borders — thicker for top-level (depth 0)
    for tr in rects {
        if tr.rect.width() < 1.0 || tr.rect.height() < 1.0 {
            continue;
        }
        let border_width = match tr.depth {
            0 => 2.0,
            1 => 1.0,
            _ => 0.5,
        };
        let border_color = match tr.depth {
            0 => Color32::from_gray(20),
            1 => Color32::from_gray(40),
            _ => Color32::from_gray(50),
        };
        painter.rect_stroke(
            tr.rect,
            0.0,
            Stroke::new(border_width, border_color),
            egui::StrokeKind::Inside,
        );
    }

    // Pass 3: draw labels (only on leaf nodes or large-enough items)
    for tr in rects {
        if tr.rect.width() < 1.0 || tr.rect.height() < 1.0 {
            continue;
        }
        // Only label items that are large enough and are either files or small dirs
        let min_label_w = if tr.depth == 0 { 60.0 } else { 40.0 };
        let min_label_h = if tr.depth == 0 { 20.0 } else { 14.0 };
        if tr.rect.width() > min_label_w && tr.rect.height() > min_label_h {
            let text = if tr.rect.width() > 100.0 && tr.rect.height() > 28.0 {
                format!("{}\n{}", tr.name, format_size(tr.size))
            } else {
                tr.name.clone()
            };

            let text_color = if is_dark_color(tr.color) {
                Color32::WHITE
            } else {
                Color32::BLACK
            };

            let font_size = match tr.depth {
                0 => 12.0,
                1 => 11.0,
                _ => 10.0,
            };

            painter.text(
                tr.rect.center(),
                egui::Align2::CENTER_CENTER,
                &text,
                egui::FontId::proportional(font_size),
                text_color,
            );
        }
    }

    // Check hover — find the deepest (smallest) rect under cursor
    let response = ui.allocate_rect(available_rect, Sense::hover());
    if let Some(pointer_pos) = response.hover_pos() {
        let mut best: Option<&TreemapRect> = None;
        let mut best_area = f64::MAX;
        for tr in rects {
            if tr.rect.contains(pointer_pos) {
                let area = tr.rect.width() as f64 * tr.rect.height() as f64;
                if area < best_area {
                    best_area = area;
                    best = Some(tr);
                }
            }
        }
        if let Some(tr) = best {
            painter.rect_stroke(
                tr.rect,
                0.0,
                Stroke::new(2.0, Color32::WHITE),
                egui::StrokeKind::Inside,
            );
            hovered = Some(tr.clone());
        }
    }

    hovered
}

fn is_dark_color(c: Color32) -> bool {
    let luminance = 0.299 * c.r() as f64 + 0.587 * c.g() as f64 + 0.114 * c.b() as f64;
    luminance < 128.0
}

fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}
