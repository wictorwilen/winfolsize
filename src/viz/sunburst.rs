use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke};
use std::f32::consts::TAU;

use crate::scanner::types::FileNode;
use crate::viz::colors;

#[derive(Debug, Clone)]
pub struct SunburstArc {
    pub center: Pos2,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub node_index: Vec<usize>,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub color: Color32,
    pub depth: usize,
}

/// Layout the sunburst chart for the children of `node`.
pub fn layout(node: &FileNode, center: Pos2, max_radius: f32, max_depth: usize) -> Vec<SunburstArc> {
    let mut result = Vec::new();
    if node.children.is_empty() || node.size == 0 {
        return result;
    }

    let ring_width = max_radius / (max_depth as f32 + 1.0);

    layout_ring(
        &node.children,
        node.size,
        center,
        ring_width,       // inner radius (skip center)
        ring_width,
        0.0,
        TAU,
        0,
        max_depth,
        &[],
        &mut result,
    );

    result
}

#[allow(clippy::too_many_arguments)]
fn layout_ring(
    children: &[FileNode],
    parent_size: u64,
    center: Pos2,
    inner_radius: f32,
    ring_width: f32,
    start_angle: f32,
    end_angle: f32,
    depth: usize,
    max_depth: usize,
    parent_path: &[usize],
    result: &mut Vec<SunburstArc>,
) {
    if depth >= max_depth || inner_radius + ring_width < 1.0 {
        return;
    }

    let angle_span = end_angle - start_angle;
    if angle_span < 0.001 {
        return;
    }

    let outer_radius = inner_radius + ring_width;
    let mut current_angle = start_angle;

    for (i, child) in children.iter().enumerate() {
        if child.size == 0 {
            continue;
        }

        let fraction = child.size as f32 / parent_size as f32;
        let child_angle_span = angle_span * fraction;

        // Skip arcs that are too thin to see
        if child_angle_span < 0.002 {
            current_angle += child_angle_span;
            continue;
        }

        let mut path = parent_path.to_vec();
        path.push(i);

        let color = if child.is_dir {
            dir_color_sunburst(child)
        } else {
            colors::color_for_extension(child.extension.as_deref())
        };

        result.push(SunburstArc {
            center,
            inner_radius,
            outer_radius,
            start_angle: current_angle,
            end_angle: current_angle + child_angle_span,
            node_index: path.clone(),
            name: child.name.clone(),
            size: child.size,
            is_dir: child.is_dir,
            extension: child.extension.clone(),
            color,
            depth,
        });

        // Recurse into children
        if child.is_dir && !child.children.is_empty() {
            layout_ring(
                &child.children,
                child.size,
                center,
                outer_radius,
                ring_width,
                current_angle,
                current_angle + child_angle_span,
                depth + 1,
                max_depth,
                &path,
                result,
            );
        }

        current_angle += child_angle_span;
    }
}

fn dir_color_sunburst(node: &FileNode) -> Color32 {
    let mut category_sizes = std::collections::HashMap::new();
    accumulate_categories(node, &mut category_sizes);

    category_sizes
        .into_iter()
        .max_by_key(|(_, size)| *size)
        .map(|(cat, _)| {
            let base = cat.color();
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

/// Draw the sunburst chart. Returns hovered arc if any.
pub fn draw(
    ui: &mut egui::Ui,
    arcs: &[SunburstArc],
    center: Pos2,
    max_radius: f32,
    root_name: &str,
    root_size: u64,
) -> Option<SunburstArc> {
    let bounds = Rect::from_center_size(center, egui::Vec2::splat(max_radius * 2.0));
    let painter = ui.painter_at(bounds);

    // Draw arcs
    for arc in arcs {
        draw_arc(&painter, arc);
    }

    // Draw center label
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        format!("{}\n{}", root_name, format_size(root_size)),
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );

    // Check hover
    let response = ui.allocate_rect(bounds, Sense::hover());
    let mut hovered: Option<SunburstArc> = None;

    if let Some(pointer_pos) = response.hover_pos() {
        let dx = pointer_pos.x - center.x;
        let dy = pointer_pos.y - center.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx).rem_euclid(TAU);

        // Find the arc under the cursor (innermost matching)
        let mut best: Option<&SunburstArc> = None;
        for arc in arcs {
            if dist >= arc.inner_radius
                && dist <= arc.outer_radius
                && angle_in_range(angle, arc.start_angle, arc.end_angle)
            {
                match best {
                    None => best = Some(arc),
                    Some(prev) => {
                        if arc.depth >= prev.depth {
                            best = Some(arc);
                        }
                    }
                }
            }
        }

        if let Some(arc) = best {
            // Highlight
            draw_arc_highlight(&painter, arc);
            hovered = Some(arc.clone());
        }
    }

    hovered
}

fn draw_arc(painter: &egui::Painter, arc: &SunburstArc) {
    let segments = ((arc.end_angle - arc.start_angle) * arc.outer_radius / 4.0)
        .max(4.0)
        .min(64.0) as usize;

    let mut points = Vec::with_capacity(segments * 2 + 2);

    // Outer arc
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = arc.start_angle + t * (arc.end_angle - arc.start_angle);
        points.push(Pos2::new(
            arc.center.x + arc.outer_radius * angle.cos(),
            arc.center.y + arc.outer_radius * angle.sin(),
        ));
    }

    // Inner arc (reversed)
    for i in (0..=segments).rev() {
        let t = i as f32 / segments as f32;
        let angle = arc.start_angle + t * (arc.end_angle - arc.start_angle);
        points.push(Pos2::new(
            arc.center.x + arc.inner_radius * angle.cos(),
            arc.center.y + arc.inner_radius * angle.sin(),
        ));
    }

    let shape = egui::Shape::convex_polygon(points, arc.color, Stroke::new(0.5, Color32::from_gray(30)));
    painter.add(shape);
}

fn draw_arc_highlight(painter: &egui::Painter, arc: &SunburstArc) {
    let segments = ((arc.end_angle - arc.start_angle) * arc.outer_radius / 4.0)
        .max(4.0)
        .min(64.0) as usize;

    let mut points = Vec::with_capacity(segments * 2 + 2);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = arc.start_angle + t * (arc.end_angle - arc.start_angle);
        points.push(Pos2::new(
            arc.center.x + arc.outer_radius * angle.cos(),
            arc.center.y + arc.outer_radius * angle.sin(),
        ));
    }
    for i in (0..=segments).rev() {
        let t = i as f32 / segments as f32;
        let angle = arc.start_angle + t * (arc.end_angle - arc.start_angle);
        points.push(Pos2::new(
            arc.center.x + arc.inner_radius * angle.cos(),
            arc.center.y + arc.inner_radius * angle.sin(),
        ));
    }

    let shape = egui::Shape::convex_polygon(
        points,
        Color32::from_rgba_premultiplied(255, 255, 255, 40),
        Stroke::new(2.0, Color32::WHITE),
    );
    painter.add(shape);
}

fn angle_in_range(angle: f32, start: f32, end: f32) -> bool {
    let a = angle.rem_euclid(TAU);
    let s = start.rem_euclid(TAU);
    let e = end.rem_euclid(TAU);
    if s <= e {
        a >= s && a <= e
    } else {
        a >= s || a <= e
    }
}

fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}
