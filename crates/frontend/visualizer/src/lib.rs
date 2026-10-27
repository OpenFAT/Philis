//! GDS viewer: GDS bytes (or `analog::base` shapes) → a wgpu window over a
//! GDS file that reloads on change, an in-process [`Probe`] window fed
//! polygons over a channel, and headless [`export_svg`]. Links `analog` only,
//! never the GPurify engine (ARCHITECTURE.md, Q36).
//!
//! Contents:
//! - `pub use gds::{parse_gds, polys_from_shapes, Poly, TextEntry}`
//! - `pub use style::{layer_names, LayerMap}`
//! - `pub use svg::export_svg`
//! - `pub use window::{run_viewer, show_macro, Probe}`

#![warn(missing_docs)]

mod font;
mod gds;
mod gpu;
mod mesh;
mod style;
mod svg;
mod window;

pub use gds::{parse_gds, polys_from_shapes, Poly, TextEntry};
pub use style::{layer_names, LayerMap};
pub use svg::export_svg;
pub use window::{run_viewer, show_macro, Probe};

#[cfg(test)]
mod tests {
    use super::*;
    use gds::bounds;
    use style::{layer_label, layer_style};

    /// Oracle: hand-written tables: the names passed in, the `L{l}/{d}` fallback, extremes of three points
    /// worked by hand, and the GDS rows given.
    #[test]
    fn layer_names_and_bounds() {
        let names = layer_names(&[("met1".into(), (68, 20)), ("via".into(), (68, 44))]);
        assert_eq!(names[&(68, 20)], "met1");
        assert_eq!(layer_label(&names, (68, 44)), "via");
        assert_eq!(layer_label(&names, (68, 5)), "L68/5");
        assert!(layer_style(&names, (68, 20)) != layer_style(&names, (68, 44)));
        let p = |pts: Vec<[i32; 2]>| Poly { layer: 0, datatype: 0, pts };
        assert_eq!(bounds(&[]), None);
        assert_eq!(bounds(&[p(vec![[1, 5], [3, -2]]), p(vec![[0, 0]])]), Some((0, -2, 3, 5)));
        let s = analog::base::Shape { layer: analog::base::LayerId(1), rect: analog::base::Rect { x: 0, y: 0, w: 1, h: 1 } };
        let q = &polys_from_shapes(&[s], &[(0, 0), (68, 20)])[0];
        assert_eq!((q.layer, q.datatype), (68, 20));
        let q = &polys_from_shapes(&[s], &[])[0];
        assert_eq!((q.layer, q.datatype), (1, 0), "no table row: layer id, datatype 0");
    }

    /// Oracle: the spec of the empty document: no polygons, so no viewBox and no layers.
    #[test]
    fn empty_gds_is_an_empty_svg() {
        assert_eq!(export_svg(&[], &LayerMap::new()), r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#);
    }
}
