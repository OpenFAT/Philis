//! Polygon, text and legend tables → GPU vertex lists: filled triangles
//! (earcut), outlines as line pairs, stroked glyphs, and the camera that
//! fits a layout to the view.
//!
//! Contents:
//! - `pub(crate) struct Vertex`
//! - `pub(crate) struct CamUni`
//! - `pub(crate) fn triangulate(polys: &[Poly], names: &LayerMap) -> Vec<Vertex>`
//! - `pub(crate) fn outline_vertices(polys: &[Poly], names: &LayerMap) -> Vec<Vertex>`
//! - `pub(crate) fn text_vertices(texts: &[TextEntry], polys: &[Poly]) -> Vec<Vertex>`
//! - `pub(crate) fn build_legend(polys: &[Poly], layer_names: &LayerMap) -> Vec<Vertex>`
//! - `pub(crate) fn fit_view(polys: &[Poly], aspect: f32) -> CamUni`

use bytemuck::{Pod, Zeroable};

use crate::font::{glyph, PEN_UP};
use crate::gds::{bounds, Poly, TextEntry};
use crate::style::{layer_label, layer_style, outline_color, LayerMap};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct CamUni {
    pub(crate) offset: [f32; 2],
    pub(crate) scale: f32,
    pub(crate) aspect: f32,
}

pub(crate) fn triangulate(polys: &[Poly], names: &LayerMap) -> Vec<Vertex> {
    let mut verts = Vec::new();
    for p in polys {
        let (color, fill) = layer_style(names, p.key());
        if !fill {
            continue;
        }
        let coords: Vec<f64> = p.pts.iter().flat_map(|v| [f64::from(v[0]), f64::from(v[1])]).collect();
        for i in earcutr::earcut(&coords, &[], 2).unwrap_or_default() {
            verts.push(Vertex { pos: [p.pts[i][0] as f32, p.pts[i][1] as f32], color });
        }
    }
    verts
}

pub(crate) fn outline_vertices(polys: &[Poly], names: &LayerMap) -> Vec<Vertex> {
    let mut verts = Vec::new();
    for p in polys {
        let color = outline_color(layer_style(names, p.key()).0);
        for (i, a) in p.pts.iter().enumerate() {
            let b = p.pts[(i + 1) % p.pts.len()];
            verts.push(Vertex { pos: [a[0] as f32, a[1] as f32], color });
            verts.push(Vertex { pos: [b[0] as f32, b[1] as f32], color });
        }
    }
    verts
}

/// World-space text labels, sized to the layout's span.
pub(crate) fn text_vertices(texts: &[TextEntry], polys: &[Poly]) -> Vec<Vertex> {
    let span = bounds(polys).map_or(7000.0, |(x0, y0, x1, y1)| (x1 - x0).max(y1 - y0).max(1) as f32);
    let mut verts = Vec::new();
    for t in texts {
        push_text(&mut verts, &t.text, t.x as f32, t.y as f32, span * 0.025 / 7.0, [1.0, 1.0, 1.0, 0.95]);
    }
    verts
}

/// Screen-space (NDC) legend in the top-right corner: a swatch and name per layer.
pub(crate) fn build_legend(polys: &[Poly], layer_names: &LayerMap) -> Vec<Vertex> {
    let mut layers: Vec<(u16, u16)> = polys.iter().map(Poly::key).collect();
    layers.sort_unstable();
    layers.dedup();
    if layers.is_empty() {
        return Vec::new();
    }
    let (row, margin) = (0.04_f32, 0.02_f32);
    let (x0, y_top) = (1.0 - margin - 0.3, 1.0 - margin);
    let mut verts = Vec::new();
    let bottom = y_top - layers.len() as f32 * row - margin;
    push_rect(&mut verts, x0 - margin, bottom, 1.0 - margin / 2.0, y_top + margin / 2.0, [0.08, 0.08, 0.10, 0.75]);
    for (i, &layer) in layers.iter().enumerate() {
        let y = y_top - (i as f32 + 0.5) * row;
        let c = layer_style(layer_names, layer).0;
        push_rect(&mut verts, x0, y - row * 0.35, x0 + 0.03, y + row * 0.35, [c[0], c[1], c[2], 1.0]);
        let label = layer_label(layer_names, layer);
        push_text(&mut verts, &label, x0 + 0.05, y - row * 0.35, row * 0.11, [0.9, 0.9, 0.9, 1.0]);
    }
    verts
}

pub(crate) fn fit_view(polys: &[Poly], aspect: f32) -> CamUni {
    let Some((x0, y0, x1, y1)) = bounds(polys) else {
        return CamUni { offset: [0.0; 2], scale: 1.0, aspect };
    };
    let span = (x1 - x0).max(y1 - y0).max(1) as f32;
    CamUni { offset: [-(x0 + x1) as f32 / 2.0, -(y0 + y1) as f32 / 2.0], scale: 1.8 / span, aspect }
}

// ---- private ----

/// A thick line as two triangles.
fn stroke_quad(verts: &mut Vec<Vertex>, a: [f32; 2], b: [f32; 2], half_w: f32, color: [f32; 4]) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt().max(1e-6);
    let (nx, ny) = (-dy / len * half_w, dx / len * half_w);
    for pos in [
        [a[0] - nx, a[1] - ny],
        [b[0] - nx, b[1] - ny],
        [b[0] + nx, b[1] + ny],
        [a[0] - nx, a[1] - ny],
        [b[0] + nx, b[1] + ny],
        [a[0] + nx, a[1] + ny],
    ] {
        verts.push(Vertex { pos, color });
    }
}

/// `text` as stroked glyphs from `(x, y)`, one glyph unit = `s`.
fn push_text(verts: &mut Vec<Vertex>, text: &str, x: f32, y: f32, s: f32, color: [f32; 4]) {
    for (ci, ch) in text.chars().enumerate() {
        let cx = x + ci as f32 * 6.0 * s;
        for seg in glyph(ch).as_chunks::<2>().0 {
            if seg.contains(&PEN_UP) {
                continue;
            }
            let at = |(gx, gy): (u8, u8)| [cx + f32::from(gx) * s, y + f32::from(gy) * s];
            stroke_quad(verts, at(seg[0]), at(seg[1]), s * 0.25, color);
        }
    }
}

/// A solid axis-aligned rectangle as two triangles.
fn push_rect(verts: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]) {
    for pos in [[x0, y0], [x1, y0], [x1, y1], [x0, y0], [x1, y1], [x0, y1]] {
        verts.push(Vertex { pos, color });
    }
}
