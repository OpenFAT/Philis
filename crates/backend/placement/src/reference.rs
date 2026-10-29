//! The reference floorplan (UW-ASIC/Philis#78, Q61): a prior design's rows,
//! extents and aspect, the sidecar's `Reference` entry, as the floorplan's
//! targets in place of the stage order (Q21, Q22).
//!
//! - [`assign`]: `(Reference, Intent, tile members) → Assignment`: each
//!   tile's block and the conflicts that degrade, or the refusal naming the
//!   constraint and the entry (Q18, Q19).
//! - [`fit`]: `(Reference, Assignment, tile sizes) → Fit`: the outline, each
//!   block's target box, each tile's lower-left corner.
//!
//! What M0 honours: the row order, the extents' positions and the outline's
//! aspect. Tiles are drawn before the floorplan and never resized, so a
//! tile's size against its target box is a deviation the report shows, not
//! a target met: set recipes answer a target from M1 on (Q22), capacitor
//! arrays from M3. `unit_area` and `area_ratio` reach no recipe yet.

use analog::base::ids::DeviceId;
use analog::base::Rect;
use analog::intent::{ArrayStyle, AxisDir, ClassSource, Intent, MatchClass, Origin, Reference};

/// How close to an outline edge an extent counts as touching it. The
/// fractions are `f32`: past about 8 mm an outline's rounding reaches 1 nm.
const EDGE_EPS: f32 = 1e-6;

/// Each tile's block, and the conflicts the floorplan degrades through.
#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    /// Per tile: its block, an index into `Reference::blocks`: the block of
    /// its first referenced member. `None`: no member is referenced.
    pub block: Vec<Option<u16>>,
    /// One report line per conflict that degrades instead of refusing.
    pub degraded: Vec<String>,
}

/// Where the reference floorplan puts every tile.
#[derive(Clone, Debug, PartialEq)]
pub struct Fit {
    /// Outline width and height, nm, multiples of the lattice.
    pub outline: (i32, i32),
    /// Per block: its target box, nm, the extent scaled to the outline.
    pub target: Vec<Rect>,
    /// Per tile: lower-left corner, nm, on the lattice.
    pub x: Vec<i32>,
    pub y: Vec<i32>,
}

/// Which block each tile sits in (`members[t]`: tile `t`'s devices; `names`:
/// every device's name by id), and what the reference does to the intent.
///
/// # Errors
/// The refusal's text, naming the constraint and the entry, when the
/// reference puts the members of a common-centroid, Precise or User matched
/// set in two blocks (a set recipe draws one tile), or contradicts a user
/// `Order` (a sidecar entry, so User).
pub fn assign(reference: &Reference, intent: &Intent, members: &[&[DeviceId]], names: &[&str]) -> Result<Assignment, String> {
    let mut of = vec![None; names.len()];
    for (b, k) in reference.blocks.iter().enumerate() {
        for d in &k.devices {
            assert!(usize::from(d.0) < names.len(), "block {}: device {} past the netlist", k.name, d.0);
            of[usize::from(d.0)] = Some(b as u16);
        }
    }
    let entry = format!("the Reference (sidecar entry {})", reference.entry);
    let at = |b: u16| {
        let k = &reference.blocks[usize::from(b)];
        format!("{} (row {})", k.name, k.row)
    };
    let list = |ds: &[DeviceId]| ds.iter().map(|d| names[usize::from(d.0)]).collect::<Vec<_>>().join(", ");
    let joined = |bs: &[u16]| bs.iter().map(|&b| at(b)).collect::<Vec<_>>().join(" and ");
    // A set recipe draws one tile, so two blocks cannot share a set.
    for s in &intent.sets {
        let devices: Vec<DeviceId> = s.members.iter().map(|m| m.device).collect();
        let split = distinct(&of, &devices);
        if split.len() < 2 {
            continue;
        }
        let why = if matches!(s.style, ArrayStyle::CommonCentroid1d | ArrayStyle::CommonCentroid2d) {
            "common-centroid"
        } else if s.class == MatchClass::Exceptional {
            "Precise"
        } else if s.class_source == ClassSource::User || matches!(s.origin, Origin::User { .. }) {
            "User"
        } else {
            continue; // degrades below, with the tile it splits
        };
        return Err(format!("{entry} puts the {why} matched set {} [{}] in {}: a set recipe draws one tile", s.id.0, list(&devices), joined(&split)));
    }
    // Only a user order is not reversible; an extracted one yields to the reference.
    for o in intent.order.iter().filter(|o| !o.reversible) {
        for w in o.steps.windows(2) {
            for &a in &w[0] {
                for &b in &w[1] {
                    let (Some(ba), Some(bb)) = (of[usize::from(a.0)], of[usize::from(b.0)]) else { continue };
                    let (ka, kb) = (&reference.blocks[usize::from(ba)], &reference.blocks[usize::from(bb)]);
                    let (against, side) = match o.dir {
                        AxisDir::V => (kb.row < ka.row, "below"),
                        AxisDir::H => (kb.extent.x + kb.extent.w <= ka.extent.x + EDGE_EPS, "left of"),
                    };
                    if against {
                        return Err(format!(
                            "{entry} puts {} in {} {side} {} in {}, against the user Order [{}] then [{}]",
                            names[usize::from(b.0)],
                            at(bb),
                            names[usize::from(a.0)],
                            at(ba),
                            list(&w[0]),
                            list(&w[1])
                        ));
                    }
                }
            }
        }
    }
    let mut degraded = Vec::new();
    let block = members
        .iter()
        .enumerate()
        .map(|(t, ms)| {
            let bs = distinct(&of, ms);
            if bs.len() > 1 {
                degraded.push(format!("call {t} [{}] spans {}: it sits in {}", list(ms), joined(&bs), at(bs[0])));
            }
            bs.first().copied()
        })
        .collect();
    Ok(Assignment { block, degraded })
}

/// The blocks `devices` are in, each once, first seen first.
fn distinct(of: &[Option<u16>], devices: &[DeviceId]) -> Vec<u16> {
    let mut out = Vec::new();
    for b in devices.iter().filter_map(|d| of[usize::from(d.0)]) {
        if !out.contains(&b) {
            out.push(b);
        }
    }
    out
}

/// Lays the tiles out (`sizes[t]`: tile `t`'s width and height, nm). Each
/// block's tiles form one row, in tile order and `gap` apart, anchored where
/// the block's extent says: flush with an outline edge the extent touches
/// (the low one first), else centred on the extent. The outline is the
/// smallest of the reference's aspect at which every block stays inside it
/// and keeps `gap` plus a lattice step from the blocks beside it in its row
/// and from every block in the rows next to it, so rows keep their order
/// and tiles of different blocks never come closer than `gap`. An extent is
/// a target (Q22), not a fence: a block whose tiles outgrow it overflows it,
/// and the report shows by how much. Tiles of no block form a row `gap`
/// above the outline. Corners land on multiples of `lattice`.
///
/// # Errors
/// No outline within `i32` nm keeps the blocks apart: anchors too close
/// for their tiles.
///
/// # Panics
/// `block` and `sizes` of different lengths, `lattice` < 1, a negative size
/// or `gap`: caller bugs, all three come from one tile table.
pub fn fit(reference: &Reference, block: &[Option<u16>], sizes: &[(i32, i32)], gap: i32, lattice: i32) -> Result<Fit, String> {
    assert_eq!(block.len(), sizes.len(), "one block per tile");
    assert!(lattice >= 1, "lattice {lattice} < 1");
    assert!(gap >= 0, "gap {gap} < 0");
    let l = i64::from(lattice);
    let up = |v: i64| (v + l - 1).div_euclid(l) * l;
    let n = reference.blocks.len();
    // Each block's row: every tile's offset in it, the row's width and height.
    let (mut offset, mut span, mut cursor, mut used) = (vec![0i64; sizes.len()], vec![(0i64, 0i64); n], vec![0i64; n], vec![false; n]);
    for (t, (&b, &(w, h))) in block.iter().zip(sizes).enumerate() {
        assert!(w >= 0 && h >= 0, "tile {t}: size {w} x {h}");
        let Some(b) = b.map(usize::from) else { continue };
        offset[t] = cursor[b];
        span[b] = (cursor[b] + i64::from(w), span[b].1.max(i64::from(h)));
        cursor[b] += up(i64::from(w) + i64::from(gap));
        used[b] = true;
    }
    // A block's corner along one axis as `coef · outline + konst`.
    let anchor = |lo: f32, len: f32, row: i64| -> (f64, f64) {
        if lo <= EDGE_EPS {
            (0.0, 0.0)
        } else if lo + len >= 1.0 - EDGE_EPS {
            (1.0, -(row as f64))
        } else {
            (f64::from(lo) + f64::from(len) / 2.0, -(row as f64) / 2.0)
        }
    };
    let aspect = f64::from(reference.aspect);
    // Per block with tiles: its x anchor over the width, its y anchor over the height.
    let anchors: Vec<Option<((f64, f64), (f64, f64))>> = (0..n)
        .map(|b| {
            let e = reference.blocks[b].extent;
            used[b].then(|| (anchor(e.x, e.w, span[b].0), anchor(e.y, e.h, span[b].1)))
        })
        .collect();
    // Every constraint is `coef · W + konst ≥ min` (H = W / aspect), coef ≥ 0
    // for every extent the sidecar admits; W is the least that meets them all.
    let mut least = 0f64;
    let mut at_least = |coef: f64, konst: f64, min: f64| {
        least = least.max(if coef > 1e-12 { (min - konst) / coef } else if konst >= min { 0.0 } else { f64::INFINITY });
    };
    let separation = (i64::from(gap) + l) as f64;
    for (b, a) in anchors.iter().enumerate() {
        let Some(((cx, kx), (cy, ky))) = *a else { continue };
        let (w, h) = (span[b].0 as f64, span[b].1 as f64);
        at_least(cx, kx, 0.0);
        at_least(1.0 - cx, -kx - w, 0.0);
        at_least(cy / aspect, ky, 0.0);
        at_least((1.0 - cy) / aspect, -ky - h, 0.0);
    }
    // The blocks with tiles, row by row, left to right; empty rows dropped.
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); reference.blocks.iter().map(|k| usize::from(k.row) + 1).max().unwrap_or(0)];
    for (b, k) in reference.blocks.iter().enumerate().filter(|&(b, _)| used[b]) {
        rows[usize::from(k.row)].push(b);
    }
    rows.retain(|r| !r.is_empty());
    for row in &rows {
        for w in row.windows(2) {
            let ((ca, ka), _) = anchors[w[0]].expect("a block with tiles");
            let ((cb, kb), _) = anchors[w[1]].expect("a block with tiles");
            at_least(cb - ca, kb - ka - span[w[0]].0 as f64, separation);
        }
    }
    for pair in rows.windows(2) {
        for &a in &pair[0] {
            for &b in &pair[1] {
                let (_, (ca, ka)) = anchors[a].expect("a block with tiles");
                let (_, (cb, kb)) = anchors[b].expect("a block with tiles");
                at_least((cb - ca) / aspect, kb - ka - span[a].1 as f64, separation);
            }
        }
    }
    let limit = f64::from(i32::MAX) / 2.0;
    if !(least <= limit && least / aspect <= limit) {
        return Err(format!("no outline under {limit:.0} nm keeps the blocks apart: their extents' anchors are too close for their tiles"));
    }
    let (wo, ho) = (up(least.ceil() as i64), up((least / aspect).ceil() as i64));
    let snap = |coef: f64, konst: f64, outline: i64| ((coef * outline as f64 + konst).floor() as i64).div_euclid(l) * l;
    let corners: Vec<(i64, i64)> = anchors
        .iter()
        .map(|a| a.map_or((0, 0), |((cx, kx), (cy, ky))| (snap(cx, kx, wo), snap(cy, ky, ho))))
        .collect();
    let (mut x, mut y) = (Vec::with_capacity(sizes.len()), Vec::with_capacity(sizes.len()));
    let (mut free, free_y) = (0i64, up(ho + i64::from(gap)));
    for (t, b) in block.iter().enumerate() {
        let (px, py) = match b.map(usize::from) {
            Some(b) => (corners[b].0 + offset[t], corners[b].1),
            None => {
                let at = (free, free_y);
                free += up(i64::from(sizes[t].0) + i64::from(gap));
                at
            }
        };
        let fits = |v: i64| i32::try_from(v).map_err(|_| format!("tile {t} at {px}, {py} nm: past i32"));
        x.push(fits(px)?);
        y.push(fits(py)?);
    }
    let scale = |f: f32, outline: i64| (f64::from(f) * outline as f64).round() as i32;
    let target = reference.blocks.iter().map(|k| Rect { x: scale(k.extent.x, wo), y: scale(k.extent.y, ho), w: scale(k.extent.w, wo), h: scale(k.extent.h, ho) }).collect();
    debug_assert!(x.iter().chain(&y).all(|v| v % lattice == 0), "a corner off the lattice");
    Ok(Fit { outline: (wo as i32, ho as i32), target, x, y })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::mechanics::SplitMix64;
    use analog::intent::{ArrayStyle, AxisDir, ClassSource, ConstraintId, Extent, Family, MatchClass, MatchKind, MatchSpec, Member, Order, Origin, RefBlock};

    fn block(name: &str, devices: &[u16], row: u16, (x, y, w, h): (f32, f32, f32, f32)) -> RefBlock {
        RefBlock { name: name.into(), devices: devices.iter().map(|&d| DeviceId(d)).collect(), row, extent: Extent { x, y, w, h }, unit_area: None }
    }

    fn reference(aspect: f32, blocks: Vec<RefBlock>) -> Reference {
        Reference { entry: 3, source: String::new(), aspect, blocks, area_ratio: None }
    }

    fn set(id: u32, devices: &[u16], class: MatchClass, style: ArrayStyle) -> MatchSpec {
        MatchSpec {
            id: ConstraintId(id),
            origin: Origin::Pattern { template: "diff_pair" },
            members: devices.iter().map(|&d| Member { device: DeviceId(d), parallel: 1, series: 1, half: None }).collect(),
            reference: None,
            family: Family::Mos,
            kind: MatchKind::Voltage,
            class,
            class_source: ClassSource::Role,
            unit: None,
            allowance: None,
            weight: None,
            style,
            compound: None,
        }
    }

    const NAMES: [&str; 4] = ["XM1", "XM2", "XM3", "XM4"];
    const ONE: [DeviceId; 1] = [DeviceId(0)];

    /// Two rows: `low` holds XM1, `high` holds XM2.
    fn two_rows() -> Reference {
        reference(1.0, vec![block("low", &[0], 0, (0.0, 0.0, 1.0, 0.5)), block("high", &[1], 1, (0.0, 0.5, 1.0, 0.5))])
    }

    /// The gate's third check: a common-centroid set split across rows is
    /// refused, naming the set and the entry; inside one block it is not.
    #[test]
    fn a_split_common_centroid_set_is_refused() {
        let pair: [DeviceId; 2] = [DeviceId(0), DeviceId(1)];
        let intent = Intent { sets: vec![set(7, &[0, 1], MatchClass::Moderate, ArrayStyle::CommonCentroid1d)], ..Intent::default() };
        let err = assign(&two_rows(), &intent, &[&pair], &NAMES).expect_err("a split centroid set is refused");
        for part in ["set 7", "XM1, XM2", "common-centroid", "sidecar entry 3", "low (row 0)", "high (row 1)"] {
            assert!(err.contains(part), "{part:?} missing: {err}");
        }
        let one_block = reference(1.0, vec![block("both", &[0, 1], 0, (0.0, 0.0, 1.0, 1.0))]);
        assert_eq!(assign(&one_block, &intent, &[&pair], &NAMES).map(|a| a.block), Ok(vec![Some(0)]));
    }

    /// Precise (Exceptional) and User sets refuse whatever their style; a
    /// Minimal Adjacent one degrades: its tile follows its first member's
    /// block, and a report line says so.
    #[test]
    fn a_split_set_refuses_by_class_and_source_else_degrades() {
        let pair: [DeviceId; 2] = [DeviceId(1), DeviceId(0)];
        let exceptional = set(1, &[0, 1], MatchClass::Exceptional, ArrayStyle::Adjacent);
        let user = MatchSpec { class_source: ClassSource::User, ..set(2, &[0, 1], MatchClass::Minimal, ArrayStyle::Adjacent) };
        for (s, why) in [(exceptional, "Precise"), (user, "User")] {
            let intent = Intent { sets: vec![s], ..Intent::default() };
            let err = assign(&two_rows(), &intent, &[&pair], &NAMES).expect_err(why);
            assert!(err.contains(why), "{why}: {err}");
        }
        let intent = Intent { sets: vec![set(3, &[0, 1], MatchClass::Minimal, ArrayStyle::Adjacent)], ..Intent::default() };
        let a = assign(&two_rows(), &intent, &[&pair], &NAMES).expect("a Minimal set degrades");
        assert_eq!(a.block, [Some(1)], "the tile follows XM2, its first member");
        assert_eq!(a.degraded.len(), 1);
        assert!(a.degraded[0].contains("XM2, XM1") && a.degraded[0].contains("high (row 1)"), "{}", a.degraded[0]);
    }

    /// A user `Order` (not reversible) against the rows refuses; the same
    /// order the right way up, or an extracted (reversible) one, does not.
    #[test]
    fn a_user_order_against_the_rows_is_refused() {
        let tiles: [&[DeviceId]; 2] = [&ONE, &[DeviceId(1)]];
        let order = |steps: [u16; 2], dir, reversible| Order { steps: steps.iter().map(|&d| vec![DeviceId(d)]).collect(), dir, reversible, weight: 1.0 };
        let with = |o: Order| Intent { order: vec![o], ..Intent::default() };
        let err = assign(&two_rows(), &with(order([1, 0], AxisDir::V, false)), &tiles, &NAMES).expect_err("XM2 above XM1, the user asks below");
        assert!(err.contains("Order") && err.contains("sidecar entry 3"), "{err}");
        assert!(assign(&two_rows(), &with(order([0, 1], AxisDir::V, false)), &tiles, &NAMES).is_ok());
        assert!(assign(&two_rows(), &with(order([1, 0], AxisDir::V, true)), &tiles, &NAMES).is_ok(), "an extracted order yields");
        let side = reference(1.0, vec![block("left", &[0], 0, (0.0, 0.0, 0.5, 1.0)), block("right", &[1], 0, (0.5, 0.0, 0.5, 1.0))]);
        assert!(assign(&side, &with(order([1, 0], AxisDir::H, false)), &tiles, &NAMES).is_err(), "XM2 right of XM1, the user asks left");
        assert!(assign(&side, &with(order([0, 1], AxisDir::H, false)), &tiles, &NAMES).is_ok());
    }

    /// Oracle: worked by hand. Aspect 2, gap 100, lattice 10 (separation
    /// 110); `a` across the bottom half, `b` and `c` the top quarters. Anchors:
    /// `a` flush low on both axes; `b` flush left and top, y = H − 200; `c`
    /// flush right and top. The binding bound: `c` keeps 110 above `a`,
    /// H − 300 − 400 ≥ 110, so H = 810 and W = 1620. Then `b` sits at
    /// (0, 610), `c` at (1620 − 400, 810 − 300), the unreferenced tile at
    /// 810 + 100. The drawn layout fills the outline: aspect exactly 2.
    #[test]
    fn fit_anchors_each_block_and_keeps_the_gap() {
        let r = reference(
            2.0,
            vec![block("a", &[0], 0, (0.0, 0.0, 1.0, 0.5)), block("b", &[1], 1, (0.0, 0.5, 0.5, 0.5)), block("c", &[2], 1, (0.5, 0.5, 0.5, 0.5))],
        );
        let f = fit(&r, &[Some(0), Some(1), Some(2), None], &[(1000, 400), (300, 200), (400, 300), (250, 250)], 100, 10).expect("fits");
        assert_eq!(f.outline, (1620, 810));
        assert_eq!((f.x, f.y), (vec![0, 0, 1220, 0], vec![0, 610, 510, 910]));
        assert_eq!(f.target[2], Rect { x: 810, y: 405, w: 810, h: 405 });
    }

    /// One block, two tiles: a row `gap` apart on the lattice; the outline
    /// takes the aspect, the tiles cannot (the deviation M0 reports).
    #[test]
    fn a_block_holds_its_tiles_as_one_row() {
        let r = reference(1.0, vec![block("a", &[0, 1], 0, (0.0, 0.0, 1.0, 1.0))]);
        let f = fit(&r, &[Some(0), Some(0)], &[(300, 100), (200, 150)], 100, 10).expect("fits");
        assert_eq!((f.outline, f.x, f.y), ((600, 600), vec![0, 400], vec![0, 0]));
    }

    /// Two blocks whose centres sit 1e-7 of the outline apart need a 10 m
    /// outline to keep their tiles apart.
    #[test]
    fn an_outline_past_i32_is_refused() {
        let r = reference(1.0, vec![block("a", &[0], 0, (0.4, 0.0, 1e-7, 1.0)), block("b", &[1], 0, (0.400_000_1, 0.0, 1e-7, 1.0))]);
        assert!(fit(&r, &[Some(0), Some(1)], &[(500_000, 10), (500_000, 10)], 0, 1).is_err());
    }

    /// Property, 400 seeded cases: random rows of random blocks (extents
    /// inside their cells, some flush with an edge, some not), 0–3 tiles
    /// each, a few unreferenced. Oracle: geometry alone. Every corner on the
    /// lattice; every block anchored as its extent says (flush with the
    /// edge it touches, else centred on it, within a lattice step); rows in
    /// order, `gap` apart; every two tiles at least `gap` apart; referenced
    /// tiles inside the outline; the outline's aspect within two lattice
    /// steps.
    #[test]
    fn fit_keeps_anchors_rows_gaps_and_aspect() {
        let mut rng = SplitMix64::new(78);
        for case in 0..400 {
            let (gap, lattice) = (rng.below(500) as i32, 1 + rng.below(50) as i32);
            let aspect = (0.2 + 4.8 * rng.f64()) as f32;
            let rows = 1 + rng.below(3);
            let mut blocks = Vec::new();
            let (mut y0, mut tiles, mut sizes) = (0.0f64, Vec::new(), Vec::new());
            for r in 0..rows {
                let y1 = if r + 1 == rows { 1.0 } else { y0 + (1.0 - y0) * (0.2 + 0.6 * rng.f64()) };
                let n = 1 + rng.below(3);
                let mut x0 = 0.0f64;
                for k in 0..n {
                    let x1 = if k + 1 == n { 1.0 } else { x0 + (1.0 - x0) * (0.2 + 0.6 * rng.f64()) };
                    // Shrink inside the cell half the time, so some extents touch no edge.
                    let (sx, sy) = if rng.below(2) == 0 { (0.0, 0.0) } else { (0.1 * rng.f64(), 0.1 * rng.f64()) };
                    let (ex, ey) = (x0 + sx * (x1 - x0), y0 + sy * (y1 - y0));
                    let (ew, eh) = ((x1 - x0) * (1.0 - 2.0 * sx), (y1 - y0) * (1.0 - 2.0 * sy));
                    let b = blocks.len() as u16;
                    blocks.push(block(&format!("b{b}"), &[], r as u16, (ex as f32, ey as f32, ew as f32, eh as f32)));
                    for _ in 0..rng.below(4) {
                        tiles.push(Some(b));
                        sizes.push((50 + rng.below(5000) as i32, 50 + rng.below(5000) as i32));
                    }
                    x0 = x1;
                }
                y0 = y1;
            }
            for _ in 0..rng.below(3) {
                tiles.push(None);
                sizes.push((50 + rng.below(5000) as i32, 50 + rng.below(5000) as i32));
            }
            let r = reference(aspect, blocks);
            let f = fit(&r, &tiles, &sizes, gap, lattice).unwrap_or_else(|e| panic!("case {case}: {e}"));
            let (w, h) = f.outline;
            let at = |t: usize| (f.x[t], f.y[t], f.x[t] + sizes[t].0, f.y[t] + sizes[t].1);
            for t in 0..tiles.len() {
                let (x0, y0, x1, y1) = at(t);
                assert!(x0 % lattice == 0 && y0 % lattice == 0, "case {case}: tile {t} off the lattice");
                if tiles[t].is_some() {
                    assert!(x0 >= 0 && y0 >= 0 && x1 <= w && y1 <= h, "case {case}: tile {t} outside the outline");
                }
                for u in t + 1..tiles.len() {
                    let (a, b) = (at(t), at(u));
                    let apart = (b.0 - a.2).max(a.0 - b.2).max(b.1 - a.3).max(a.1 - b.3);
                    assert!(apart >= gap, "case {case}: tiles {t} and {u} {apart} nm apart, gap {gap}");
                    if let (Some(p), Some(q)) = (tiles[t], tiles[u]) {
                        let (rp, rq) = (r.blocks[usize::from(p)].row, r.blocks[usize::from(q)].row);
                        let (lo, hi) = if rp < rq { (a, b) } else { (b, a) };
                        assert!(rp == rq || hi.1 - lo.3 >= gap, "case {case}: rows {rp} and {rq} out of order");
                    }
                }
            }
            for (b, k) in r.blocks.iter().enumerate() {
                let mine: Vec<usize> = (0..tiles.len()).filter(|&t| tiles[t] == Some(b as u16)).collect();
                let Some(&first) = mine.first() else { continue };
                let (x0, y0) = (f.x[first], mine.iter().map(|&t| f.y[t]).min().unwrap());
                let x1 = mine.iter().map(|&t| at(t).2).max().unwrap();
                let y1 = mine.iter().map(|&t| at(t).3).max().unwrap();
                let anchored = |lo: f32, len: f32, p0: i32, p1: i32, outline: i32| {
                    let (lo, len, l) = (f64::from(lo), f64::from(len), f64::from(lattice));
                    if lo <= 1e-6 {
                        p0 == 0
                    } else if lo + len >= 1.0 - 1e-6 {
                        p1 <= outline && f64::from(outline - p1) < l
                    } else {
                        (f64::from(p0 + p1) / 2.0 - (lo + len / 2.0) * f64::from(outline)).abs() <= l
                    }
                };
                assert!(anchored(k.extent.x, k.extent.w, x0, x1, w), "case {case}: block {b} x {x0}..{x1} not anchored in {w}");
                assert!(anchored(k.extent.y, k.extent.h, y0, y1, h), "case {case}: block {b} y {y0}..{y1} not anchored in {h}");
            }
            if tiles.iter().any(Option::is_some) {
                let got = f64::from(w) / f64::from(h);
                let slack = 2.0 * f64::from(lattice) / f64::from(w.min(h)) + 1e-9;
                assert!((got / f64::from(aspect) - 1.0).abs() <= slack, "case {case}: aspect {got} for {aspect}");
            }
        }
    }
}
