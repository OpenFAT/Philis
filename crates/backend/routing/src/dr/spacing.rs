//! Drawn spacing: the routed nets' metal, beside the fixed metal they run
//! past (cells, rings, an earlier routing pass) → every gap under the deck's
//! spacing between two nets, as [`Gap`]s that `route` re-lands and repairs,
//! and as hard rows. Cites: the deck's `spacing` table ([`DetailedCfg::space`]).
//!
//! Contents:
//! - `pub(super) const NOBODY: usize`
//! - `pub(super) struct Gap`
//! - `pub(super) fn spacing_pairs(wires: &[Vec<Shape>], fixed: &[(Option<u32>, Shape)], cfg: &DetailedCfg, metals: &[LayerId], side: i32) -> Vec<Gap>`
//! - `pub(super) fn spacing_violations(gaps: &[Gap]) -> Vec<Violation>`

use std::collections::BTreeMap;

use analog::base::geom::{LayerId, Rect, Shape};
use analog::base::report::Violation;

use super::{rect_gap, DetailedCfg, ShapeIndex};

/// The net of fixed metal no pin reaches: another net to every routed one.
pub(super) const NOBODY: usize = usize::MAX;

/// Routed net `a`'s shape `sa` closer than `need` to `sb` without touching (a
/// touch is a short, `cross_net_shorts`'). `sb` is net `b`'s routed metal, or
/// fixed metal when `fixed`: then `b` is the net a pin of its macro reaches,
/// else [`NOBODY`], and only `a` can move.
#[derive(Clone, Copy, Debug)]
pub(super) struct Gap {
    pub a: usize,
    pub sa: Shape,
    pub b: usize,
    pub sb: Shape,
    pub need: i32,
    pub fixed: bool,
}

/// Every [`Gap`] on one of `metals`: between two routed nets once, and between
/// a routed net and each `fixed` shape another net owns (`fixed` as
/// `cross_net_shorts` reads it: the owner, `None` for nobody). Layers without
/// a stated spacing are not checked. Candidates come from a `ShapeIndex` of
/// `side` nm buckets.
pub(super) fn spacing_pairs(wires: &[Vec<Shape>], fixed: &[(Option<u32>, Shape)], cfg: &DetailedCfg, metals: &[LayerId], side: i32) -> Vec<Gap> {
    let md = |r: Rect| r.w.min(r.h);
    let routed = wires.iter().enumerate().flat_map(|(n, w)| w.iter().map(move |s| (n, *s, false)));
    let pinned = fixed.iter().map(|&(o, s)| (o.map_or(NOBODY, |o| o as usize), s, true));
    let flat: Vec<(usize, Shape, bool)> = routed.chain(pinned).filter(|(_, s, _)| metals.contains(&s.layer)).collect();
    let idx = ShapeIndex::new(side, flat.iter().map(|(_, s, _)| (s.layer, s.rect)));
    let mut out = Vec::new();
    for (k0, &(a, sa, _)) in flat.iter().enumerate().take_while(|(_, f)| !f.2) {
        let halo = cfg.space(sa.layer, md(sa.rect), i32::MAX / 2, 0);
        // Routed pairs once (`k > k0`), every fixed shape near.
        let mut near: Vec<u32> = idx.near(sa.layer, sa.rect, halo).filter(|&k| k as usize > k0).collect();
        near.sort_unstable();
        near.dedup();
        for k in near {
            let (b, sb, fixed) = flat[k as usize];
            let (gap, need) = (rect_gap(sa.rect, sb.rect), cfg.space(sa.layer, md(sa.rect), md(sb.rect), 0));
            if a != b && gap > 0 && gap < need {
                out.push(Gap { a, sa, b, sb, need, fixed });
            }
        }
    }
    debug_assert!(out.iter().all(|g| g.a != NOBODY && g.a != g.b));
    out
}

/// `gaps` as hard rows, one per net pair (`drawn spacing nets a/b`) or routed
/// net beside fixed metal (`drawn spacing net a/fixed`) with its worst
/// shortfall: no gap the drawing leaves is silent.
pub(super) fn spacing_violations(gaps: &[Gap]) -> Vec<Violation> {
    let mut worst: BTreeMap<(usize, usize), i32> = BTreeMap::new();
    for g in gaps {
        let key = if g.fixed { (g.a, NOBODY) } else { (g.a.min(g.b), g.a.max(g.b)) };
        let e = worst.entry(key).or_insert(0);
        *e = (*e).max(g.need - rect_gap(g.sa.rect, g.sb.rect));
    }
    let rule = |a: usize, b: usize| if b == NOBODY { format!("drawn spacing net {a}/fixed") } else { format!("drawn spacing nets {a}/{b}") };
    worst.into_iter().map(|((a, b), m)| Violation { rule: rule(a, b), margin: i64::from(m) }).collect()
}

#[cfg(test)]
mod tests {
    use analog::base::{Macro, NetId, Pin, Report, Routes};
    use analog::Requirements;

    use super::super::{Cut, DetailedRoute};
    use super::*;
    use crate::gr::LayerSpec;

    const LAYERS: [LayerId; 2] = [LayerId(0), LayerId(1)];
    const SPACE: i32 = 140;

    /// DRC's spacing between two rects: Euclidean at a corner, else the
    /// larger axis gap; `0` when they touch.
    fn drc_gap(a: Rect, b: Rect) -> f64 {
        let dx = f64::from((b.x - (a.x + a.w)).max(a.x - (b.x + b.w)).max(0));
        let dy = f64::from((b.y - (a.y + a.h)).max(a.y - (b.y + b.h)).max(0));
        dx.hypot(dy)
    }

    /// li, the access cut, metal 1, the via, metal 2: `cell_metal` credits
    /// the tile's li to the pins on it.
    fn stack() -> &'static analog::routing::Stack {
        use analog::routing::stack::Layer;
        let layers = [3, 4, 0, 2, 1].map(|id| Layer { id, cut: id == 2 || id == 4, ..Layer::default() }).to_vec();
        Box::leak(Box::new(analog::routing::Stack { layers, antenna_cumulative: false, diode: None }))
    }

    /// sky130's lattice, cuts, layer specs and spacing as the engine builds
    /// them, pins on li (`LayerId(3)`) through the access cut.
    fn sky130() -> DetailedCfg {
        let spec = |id, horizontal, wire, pad_along| LayerSpec {
            id,
            horizontal,
            stride: 1,
            wire,
            space: SPACE,
            pad_across: wire,
            pad_along,
            halo_wire: 0,
            halo_via: 1,
        };
        DetailedCfg {
            pitch: 420,
            grid: 5,
            wire_width: 260,
            layers: vec![spec(LAYERS[0], true, 260, 320), spec(LAYERS[1], false, 280, 370)],
            spacing: vec![(LAYERS[0], SPACE, Vec::new()), (LAYERS[1], SPACE, Vec::new())],
            min_width: vec![(LAYERS[0], 140), (LAYERS[1], 140), (LayerId(3), 290)],
            pin_access: Some((LayerId(3), (LayerId(4), 170, 300, 300))),
            pin_access_spacing: 170,
            stack: Some(stack()),
            ..DetailedCfg::default()
        }
    }

    const CUTS: [Cut; 1] = [(LayerId(2), 150, 320, 320)];

    /// sky130's `MosCc1d` row (n=2, nf=4, shared source) in `route_rest`:
    /// the drains, routed first, are fixed metal-1 (a pinless ring); the
    /// shared source's four pins on the tile's li route around them.
    fn cc1d_source(fixed: &[Rect]) -> (Report, Vec<(usize, Rect, f64)>) {
        let li = LayerId(3);
        // The tile: its li (gate, S/D and tap straps) and the source's pins.
        let r = |x, y, w, h| Rect { x, y, w, h };
        let sd = [190, 980, 1_760, 2_540, 3_320, 4_100, 4_880, 5_660, 6_450].map(|x| r(x, 770, 250, 330));
        let straps = [r(590, 20, 5_710, 330), r(1_370, 1_530, 4_150, 330), r(130, 2_060, 6_550, 330)];
        let pin = |x| Pin { name: "d0:S".into(), net: NetId(0), at: r(x, 850, 170, 170), layer: li };
        let tile = Macro {
            bbox: r(0, 0, 6_800, 2_400),
            shapes: sd.iter().chain(&straps).map(|&rect| Shape { layer: li, rect }).collect(),
            pins: [980, 2_540, 4_100, 5_660].map(pin).to_vec(),
            ..Macro::default()
        };
        let shapes = fixed.iter().map(|&rect| Shape { layer: LAYERS[0], rect }).collect();
        let ring = Macro { bbox: r(80, 80, 6_560, 1_520), shapes, ..Macro::default() };
        let neg = &mut crate::gr::Negotiation::new();
        let (routes, report, _) =
            DetailedRoute { cfg: sky130() }.route(&[], &[tile], &[ring], &Requirements::<Routes>::default(), &LAYERS, &CUTS, neg);
        let close = routes.wires[0]
            .iter()
            .filter(|s| s.layer == LAYERS[0])
            .flat_map(|s| fixed.iter().map(move |&f| (0, s.rect, drc_gap(s.rect, f))))
            .filter(|&(_, _, g)| g < f64::from(SPACE))
            .collect();
        (report, close)
    }

    /// Oracle: a Euclidean gap check of the source's drawn metal-1 against
    /// the fixed drains with the deck's 140 nm, independent of
    /// `spacing_pairs`. The pin at x = 4100 sits between drain 0's riser and
    /// drain 1's trunk a track above: a landing that climbs to metal 2 there
    /// draws a 320 nm square pad 130 nm from both (the bug).
    #[test]
    fn access_keeps_spacing_from_fixed_metal() {
        let r = |x, y, w, h| Rect { x, y, w, h };
        let drains = [
            // drain 0: risers, the bottom trunk, pin access.
            r(80, 80, 260, 1_100),
            r(80, 80, 3_620, 260),
            r(3_440, 80, 260, 1_100),
            r(3_440, 80, 3_200, 260),
            r(6_380, 80, 260, 1_100),
            r(80, 920, 325, 260),
            r(125, 785, 300, 300),
            r(3_275, 920, 425, 260),
            r(3_255, 785, 300, 300),
            r(6_380, 920, 285, 260),
            r(6_385, 785, 300, 300),
            // drain 1: risers, the top trunk, pin access.
            r(1_760, 920, 260, 680),
            r(1_760, 1_340, 3_200, 260),
            r(4_700, 920, 260, 680),
            r(1_695, 785, 300, 300),
            r(4_700, 920, 395, 260),
            r(4_815, 785, 300, 300),
        ];
        let (report, close) = cc1d_source(&drains);
        let rows: Vec<&String> = report.hard_violations.iter().map(|v| &v.rule).collect();
        assert!(close.is_empty() || rows.iter().any(|x| *x == "drawn spacing net 0/fixed"), "unreported: {close:?} {rows:?}");
        assert!(close.is_empty(), "source metal-1 within spacing of the drains: {close:?}");
        assert!(rows.is_empty(), "{rows:?}");
    }

    /// A row of S/D pins on li, `(x, net)`, at `y`, over the tile's li
    /// `shapes`, routed on [`sky130`]. Returns every two nets' metal-1 pair
    /// under spacing, every metal-1 shape under 140 nm wide, and the report's
    /// hard rows.
    fn pin_row(pins: &[(i32, u16)], y: i32, shapes: &[Rect]) -> (Vec<(usize, Rect, usize, Rect)>, Vec<Rect>, Vec<String>) {
        let li = LayerId(3);
        let at = |x| Rect { x, y, w: 170, h: 170 };
        let tile = Macro {
            bbox: shapes.iter().copied().reduce(super::super::bbox).unwrap_or_default(),
            shapes: shapes.iter().map(|&rect| Shape { layer: li, rect }).collect(),
            pins: pins.iter().map(|&(x, n)| Pin { name: format!("p{x}"), net: NetId(n), at: at(x), layer: li }).collect(),
            ..Macro::default()
        };
        let neg = &mut crate::gr::Negotiation::new();
        let (routes, report, _) = DetailedRoute { cfg: sky130() }.route(&[], &[tile], &[], &Requirements::<Routes>::default(), &LAYERS, &CUTS, neg);
        let m1 = |n: usize| routes.wires[n].iter().filter(|s| s.layer == LAYERS[0]).map(|s| s.rect).collect::<Vec<_>>();
        let mut bad = Vec::new();
        for a in 0..routes.wires.len() {
            for b in a + 1..routes.wires.len() {
                for p in m1(a) {
                    bad.extend(m1(b).into_iter().filter(|&q| drc_gap(p, q) < f64::from(SPACE)).map(|q| (a, p, b, q)));
                }
            }
        }
        let thin = (0..routes.wires.len()).flat_map(m1).filter(|r| r.w.min(r.h) < 140).collect();
        (bad, thin, report.hard_violations.iter().map(|v| v.rule.clone()).collect())
    }

    /// Oracle: a Euclidean gap and width check over the drawn metal-1
    /// ([`pin_row`]), independent of `spacing_pairs`: two nets' shapes keep
    /// 140 nm or touch, and every shape is 140 nm wide. The row is sky130's
    /// `MosCc1d` over XM0..2 (`nfet_01v8` W=1u L=0.15u nf=1, m = 2, 4, 8;
    /// gate, source and bulk shared): fifteen S/D pins 440 nm apart over a
    /// 420 nm lattice, `d0 s d2 s d1 s d2 s d2 s d1 s d2 s d0` (nets 0, 3, 2,
    /// 1), between the gate and tap rails. Before: m1.2 between a drain's
    /// narrow jog and the source's node pad, and a sacrificed stitch pad
    /// (signoff: m1.1, m1.2, m1.4).
    #[test]
    fn a_ratioed_pin_row_keeps_metal_spacing() {
        let r = |x, y, w, h| Rect { x, y, w, h };
        let nets = [0u16, 3, 2, 3, 1, 3, 2, 3, 2, 3, 1, 3, 2, 3, 0];
        let pins: Vec<(i32, u16)> = (0..15).map(|k| 190 + 440 * k as i32).zip(nets).collect();
        let mut li: Vec<Rect> = pins.iter().map(|&(x, _)| r(x, 520, 250, 1_010)).collect();
        li.extend([r(410, 20, 5_970, 330), r(130, 2_150, 6_450, 330)]);
        let (bad, thin, rows) = pin_row(&pins, 940, &li);
        assert!(bad.is_empty(), "two nets' metal-1 under 140 nm: {bad:?}\n{rows:?}");
        assert!(thin.is_empty(), "metal-1 under 140 nm wide: {thin:?}");
        assert!(rows.is_empty(), "{rows:?}");
    }

    /// The same oracle on `MosRatioed` 1:2 (`nfet_01v8` W=1u and 2u L=0.15u
    /// nf=1, gate, source and bulk shared): `d0 s d1 s d1 s d0` (nets 0, 2,
    /// 4, the engine's ids: the source lands before d1) between dummy strips
    /// tied to the tap rail. Before: a drain's jog
    /// up between its pin and the source's next pin, 20 nm off the source's
    /// stitch pad (m1.2).
    #[test]
    fn a_ratioed_pair_row_keeps_metal_spacing() {
        let r = |x, y, w, h| Rect { x, y, w, h };
        let pins: Vec<(i32, u16)> = [0u16, 2, 4, 2, 4, 2, 0].iter().enumerate().map(|(k, &n)| (4_900 + 440 * k as i32, n)).collect();
        let mut li: Vec<Rect> = pins.iter().map(|&(x, _)| r(x, 1_560, 250, 330)).collect();
        li.extend((1..=7).flat_map(|k| [r(4_900 - 440 * k, 1_560, 250, 1_560), r(7_540 + 440 * k, 1_560, 250, 1_560)]));
        li.extend([r(5_120, 20, 2_450, 330), r(1_740, 3_110, 3_190, 460), r(7_680, 3_110, 3_190, 460), r(130, 3_560, 12_350, 330)]);
        let (bad, thin, rows) = pin_row(&pins, 1_640, &li);
        assert!(bad.is_empty(), "two nets' metal-1 under 140 nm: {bad:?}\n{rows:?}");
        assert!(thin.is_empty(), "metal-1 under 140 nm wide: {thin:?}");
        assert!(rows.is_empty(), "{rows:?}");
    }
}
