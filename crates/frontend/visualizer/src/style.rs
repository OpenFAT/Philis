//! GDS `(layer, datatype)` → a layer name and a fill colour: the names a
//! caller supplies from its deck, conventional colours for known names,
//! outline-only for wells, implants and markers.
//!
//! Contents:
//! - `pub type LayerMap`
//! - `pub fn layer_names(pairs: &[(String, (u16, u16))]) -> LayerMap`
//! - `pub(crate) fn layer_label(names: &LayerMap, (l, d): (u16, u16)) -> String`
//! - `pub(crate) fn layer_style(names: &LayerMap, key: (u16, u16)) -> ([f32; 4], bool)`
//! - `pub(crate) fn outline_color(fill: [f32; 4]) -> [f32; 4]`

use std::collections::HashMap;

/// GDS (layer, datatype) → layer name.
pub type LayerMap = HashMap<(i32, i32), String>;

/// Layer names from `(name, (gds layer, datatype))` pairs — a caller with a
/// deck builds them from `Pdk::layers` and `Pdk::layer_gds` (this crate does
/// not depend on `verify`). A later pair on the same GDS key wins.
#[must_use]
pub fn layer_names(pairs: &[(String, (u16, u16))]) -> LayerMap {
    pairs.iter().map(|(n, (l, d))| ((i32::from(*l), i32::from(*d)), n.clone())).collect()
}

/// The name for GDS `(layer, datatype)`, else `L{layer}/{datatype}`. Datatype
/// matters: sky130 poly and licon are both layer 66.
pub(crate) fn layer_label(names: &LayerMap, (l, d): (u16, u16)) -> String {
    names.get(&(i32::from(l), i32::from(d))).cloned().unwrap_or_else(|| format!("L{l}/{d}"))
}

/// Fill colour and whether to fill at all. Known layer names get the
/// conventional colours; wells, implants and marker layers are outline-only so
/// they never hide the devices under them; anything else hashes (layer, datatype).
pub(crate) fn layer_style(names: &LayerMap, key: (u16, u16)) -> ([f32; 4], bool) {
    let name = names.get(&(i32::from(key.0), i32::from(key.1))).map(String::as_str).unwrap_or("");
    let c = |r: f32, g: f32, b: f32| [r, g, b, 0.55];
    let known = match name {
        "diff" | "comp" | "activ" => Some(c(0.20, 0.75, 0.25)),
        "tap" => Some(c(0.60, 0.75, 0.20)),
        "poly" | "poly2" | "gatpoly" => Some(c(0.90, 0.20, 0.20)),
        "licon" | "mcon" | "via" | "via1" | "via2" | "via3" | "via4" | "cont" | "contact" => Some([0.95, 0.95, 0.95, 0.85]),
        "li1" | "li" => Some(c(0.60, 0.45, 0.90)),
        "met1" | "m1" | "metal1" => Some(c(0.25, 0.50, 1.00)),
        "met2" | "m2" | "metal2" => Some(c(0.90, 0.40, 0.85)),
        "met3" | "m3" | "metal3" => Some(c(0.20, 0.85, 0.85)),
        "met4" | "m4" | "metal4" => Some(c(1.00, 0.65, 0.20)),
        "met5" | "m5" | "metal5" => Some(c(0.85, 0.85, 0.30)),
        "rpoly" | "poly_res" => Some(c(1.00, 0.55, 0.22)),
        _ => None,
    };
    if let Some(col) = known {
        return (col, true);
    }
    const P: [[f32; 4]; 10] = [
        [0.22, 0.60, 1.00, 0.60],
        [1.00, 0.33, 0.33, 0.60],
        [0.30, 0.85, 0.40, 0.60],
        [1.00, 0.80, 0.20, 0.60],
        [0.75, 0.35, 0.85, 0.60],
        [0.30, 0.88, 0.88, 0.60],
        [1.00, 0.55, 0.22, 0.60],
        [0.55, 0.78, 0.25, 0.60],
        [0.85, 0.45, 0.55, 0.60],
        [0.50, 0.50, 0.80, 0.60],
    ];
    let outline_only = name.contains("well")
        || name.contains("sdm")
        || name.ends_with("_mk")
        || matches!(name, "nplus" | "pplus" | "nsd" | "psd" | "npc" | "hvtp" | "lvtn" | "rpm" | "diom" | "dualgate" | "sab" | "salblock");
    (P[(usize::from(key.0) * 7 + usize::from(key.1)) % P.len()], !outline_only)
}

pub(crate) fn outline_color(fill: [f32; 4]) -> [f32; 4] {
    [(fill[0] * 1.5).min(1.0), (fill[1] * 1.5).min(1.0), (fill[2] * 1.5).min(1.0), 1.0]
}
