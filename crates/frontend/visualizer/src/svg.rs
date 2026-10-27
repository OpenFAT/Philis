//! GDS bytes → a standalone SVG document, one `<g>` per layer: the headless
//! export the bench writes per fixture (TESTING.md, Q49).
//!
//! Contents:
//! - `pub fn export_svg(gds_bytes: &[u8], layer_names: &LayerMap) -> String`

use crate::gds::{bounds, parse_gds, Poly};
use crate::style::{layer_label, layer_style, outline_color, LayerMap};

/// The flattened GDS as one SVG document, 800 px square, y flipped upright,
/// one `<g id="layer name">` per `(layer, datatype)`; an empty or unreadable
/// GDS gives the empty `<svg/>`.
#[must_use]
pub fn export_svg(gds_bytes: &[u8], layer_names: &LayerMap) -> String {
    let (polys, texts) = parse_gds(gds_bytes);
    let Some((x0, y0, x1, y1)) = bounds(&polys) else {
        return String::from(r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#);
    };
    let pad = (x1 - x0).max(y1 - y0) / 40;
    let (vx, vy, vw, vh) = (x0 - pad, y0 - pad, x1 - x0 + 2 * pad, y1 - y0 + 2 * pad);
    let mut layers: Vec<(u16, u16)> = polys.iter().map(Poly::key).collect();
    layers.sort_unstable();
    layers.dedup();

    let byte = |v: f32| (v * 255.0) as u8;
    let mut svg =
        format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vx} {vy} {vw} {vh}" width="800" height="800" style="background:#14141a">"#);
    // GDS is y-up, SVG y-down.
    let flip = vy * 2 + vh;
    svg.push_str(&format!(r#"<g transform="translate(0,{flip}) scale(1,-1)">"#));
    for &layer in &layers {
        let (c, fill) = layer_style(layer_names, layer);
        let o = outline_color(c);
        let fill = if fill { format!("rgba({},{},{},{})", byte(c[0]), byte(c[1]), byte(c[2]), c[3]) } else { "none".into() };
        svg.push_str(&format!(
            r#"<g id="{}" fill="{fill}" stroke="rgba({},{},{},1)" stroke-width="{}">"#,
            layer_label(layer_names, layer),
            byte(o[0]),
            byte(o[1]),
            byte(o[2]),
            (vw.max(vh) as f32 * 0.001) as i32,
        ));
        for p in polys.iter().filter(|p| p.key() == layer) {
            let pts: Vec<String> = p.pts.iter().map(|[x, y]| format!("{x},{y}")).collect();
            svg.push_str(&format!(r#"<polygon points="{}"/>"#, pts.join(" ")));
        }
        svg.push_str("</g>");
    }
    svg.push_str("</g>");
    // Labels outside the flip so the glyphs read upright.
    let size = vw.max(vh) / 60;
    for t in &texts {
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" font-size="{size}" font-family="monospace" fill="white">{}</text>"#,
            t.x,
            flip - t.y,
            t.text.replace('&', "&amp;").replace('<', "&lt;"),
        ));
    }
    svg.push_str("</svg>");
    svg
}
