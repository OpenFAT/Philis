//! Floorplan recipe tests: tiles and a `Stack` → `row_stack`, `glue` and
//! `targets` output, checked against mirror geometry, the input's own shape,
//! and GPurify DRC on the sky130 and asap7 decks (docs/TESTING.md, geometry rows).
//!
//! Contents: none public.

use analog::base::{place_macros, DeviceGroup, DeviceId, DeviceKind, LayerId, Macro, Orient, Process, Rect, Shape};
use analog::cell::{SeriesParallel, Unitization};
use substrate3::cells::finfet::FinFet;
use substrate3::cells::mosfet::Mosfet;
use substrate3::cells::{Cell, Pattern};

use super::{glue, row_stack, targets, Refusal, Side, Spacing, Stack};

/// The roles a FinFet tile draws.
const ASAP7_ROLES: [&str; 13] = ["fin", "poly", "diff", "sdt", "lisd", "lig", "licon", "li", "mcon", "met1", "nsdm", "psdm", "nwell"];

/// sky130's drawn front-end and first-metal layers.
const SKY130_ROLES: [&str; 11] = ["nwell", "diff", "tap", "poly", "licon", "li", "mcon", "met1", "nsdm", "psdm", "npc"];

// ---- private ----

/// Solid tiles of `size` on one layer that keeps 10 nm from itself: every
/// facing gap is 10.
fn solid(size: &[(i32, i32)]) -> (Vec<Macro>, Spacing) {
    let l = LayerId(1);
    let tiles = size
        .iter()
        .map(|&(w, h)| {
            let r = Rect { x: 7, y: 3, w, h };
            Macro { shapes: vec![Shape { layer: l, rect: r }], bbox: r, ..Macro::default() }
        })
        .collect();
    (tiles, Spacing { a: vec![l], b: vec![l], nm: vec![10] })
}

/// Oracle: mirror geometry. Every pair's halves sit at equal distances from
/// the axis, the left one turned `Mx180`; the self is centred on it.
#[test]
fn pairs_mirror_about_the_axis() {
    let size = [(100, 40), (60, 40), (60, 40), (80, 30), (80, 30)];
    let side = vec![Side::Axis, Side::Left, Side::Right, Side::Right, Side::Left];
    let (tiles, spacing) = solid(&size);
    let f = row_stack(&tiles, &Stack { row: vec![0; 5], side }, &spacing, 2).unwrap();
    let axis = f.axis[0];
    for (l, r) in [(1, 2), (4, 3)] {
        assert_eq!(axis - (f.x[l] + f.w[l]), f.x[r] - axis, "pair {l},{r}");
        assert_eq!((f.orient[l], f.orient[r]), (Orient::Mx180, Orient::R0));
        assert_eq!(f.y[l], f.y[r]);
    }
    assert_eq!(f.x[0] + f.w[0] / 2, axis);
    // No overlaps in the row: sorted by x, each ends before the next starts.
    let mut xs: Vec<(i32, i32)> = (0..5).map(|i| (f.x[i], f.x[i] + f.w[i])).collect();
    xs.sort_unstable();
    assert!(xs.windows(2).all(|w| w[0].1 + 10 <= w[1].0), "{xs:?}");
    // The Layout columns stamp each lower-left where the table says.
    for (i, m) in place_macros(&tiles, &f.layout()).iter().enumerate() {
        assert_eq!((m.bbox.x, m.bbox.y), (f.x[i], f.y[i]), "tile {i}");
    }
}

/// Negative control: a pair whose halves differ refuses.
/// Oracle: the input: halves 60 and 62 wide, or two `Left` halves, are a mismatch by construction.
#[test]
fn unequal_halves_refuse() {
    let stack = Stack { row: vec![3, 3], side: vec![Side::Left, Side::Right] };
    let (tiles, spacing) = solid(&[(60, 40), (62, 40)]);
    assert_eq!(row_stack(&tiles, &stack, &spacing, 2), Err(Refusal::PairMismatch { row: 3 }));
    let stack = Stack { row: vec![3, 3], side: vec![Side::Left, Side::Left] };
    let (tiles, spacing) = solid(&[(60, 40), (60, 40)]);
    assert_eq!(row_stack(&tiles, &stack, &spacing, 2), Err(Refusal::PairMismatch { row: 3 }));
}

/// Oracle: stage order. Rows stack bottom to top by stage index, whatever
/// the input order, each above the tallest of the row below plus the gap.
#[test]
fn rows_follow_stage_order() {
    let (tiles, spacing) = solid(&[(50, 30), (40, 70), (40, 70), (90, 20)]);
    let stack = Stack { row: vec![2, 0, 0, 1], side: vec![Side::Axis, Side::Left, Side::Right, Side::Axis] };
    let f = row_stack(&tiles, &stack, &spacing, 2).unwrap();
    assert_eq!((f.y[1], f.y[2]), (0, 0));
    assert_eq!(f.y[3], 80);
    assert_eq!(f.y[0], 110);
}

/// Oracle: arithmetic on the bounds. Rows split the height, a row's tiles
/// its width; every set's axis is the block's centre line.
#[test]
fn targets_split_the_block() {
    let stack = Stack { row: vec![0, 0, 0, 1], side: vec![Side::Axis, Side::Left, Side::Right, Side::Axis] };
    let t = targets(&stack, (1000, 410), 10);
    assert_eq!(t.w, [(1000 - 20) / 3; 3].into_iter().chain([1000]).collect::<Vec<_>>());
    assert_eq!(t.h, [200; 4]);
    assert_eq!(t.axis, [500; 4]);
}

fn asap7() -> verify::Pdk {
    verify::Pdk::builtin("asap7").expect("asap7 loads")
}

/// One FinFet tile of `kind`, `nf` fingers of `w` nm, 1680 nm wide, as the engine draws
/// a single-device call.
fn finfet(kind: DeviceKind, nf: u16, w: i32, pdk: &verify::Pdk) -> Macro {
    let group = DeviceGroup { devices: vec![DeviceId(0)] };
    let c = analog::Constraints { unitization: vec![unit(kind, nf, w, 20)], ..analog::Constraints::default() };
    FinFet { shared: false }.draw(&group, &c, pdk)
}

/// Device 0 alone, `nf` fingers of `w` by `l` nm.
fn unit(kind: DeviceKind, nf: u16, w: i32, l: i32) -> Unitization {
    Unitization {
        devices: vec![DeviceId(0)],
        device_type: kind,
        dev_nf: vec![nf],
        target_ratio: vec![1],
        unit_w: w,
        unit_l: l,
        series_parallel: SeriesParallel::Parallel,
        dummy_required: false,
        route_matching_required: false,
        class: None,
        kind: None,
        series: Vec::new(),
        style: None,
    }
}

/// The glued row's DRC rule ids, density rules aside, with or without fill.
fn glued_findings(tiles: &[Macro], with_fill: bool, pdk: &verify::Pdk) -> Vec<String> {
    let (f, fill) = glue(tiles, pdk, &Spacing::of(pdk, &ASAP7_ROLES), 2).expect("the gates share a band");
    let mut shapes: Vec<Shape> = place_macros(tiles, &f.layout()).into_iter().flat_map(|m| m.shapes).collect();
    if with_fill {
        assert!(!fill.shapes.is_empty(), "a break between tiles needs field gates");
        shapes.extend(fill.shapes);
    }
    let mut rules: Vec<String> = verify::drc(&shapes, &[], pdk).into_iter().map(|f| f.rule).filter(|r| !r.ends_with("_density")).collect();
    rules.sort_unstable();
    rules.dedup();
    rules
}

/// Oracle: the asap7 deck through GPurify. Two FinFet tiles glued with
/// their fill are clean of the exact-pitch and active-spacing rules; the
/// negative control drops the fill and GATE.S.1 fires.
#[test]
fn glued_finfets_keep_the_pitches_on_asap7() {
    let pdk = asap7();
    for kind in [DeviceKind::Nmos, DeviceKind::Pmos] {
        let tiles = [finfet(kind, 2, 1680, &pdk), finfet(kind, 3, 1680, &pdk)];
        let clean = glued_findings(&tiles, true, &pdk);
        assert!(clean.is_empty(), "{kind:?}: {clean:?}");
        let bare = glued_findings(&tiles, false, &pdk);
        assert!(bare.iter().any(|r| r == "GATE.S.1"), "{kind:?}: {bare:?}");
    }
}

/// Glued actives sit exactly ACTIVE.S.2A apart, the fill 9 nm clear of
/// each (GATE.ACTIVE.S.4), every gate on the 54 nm lattice.
/// Oracle: the asap7 DRM numbers (ACTIVE.S.2A 92 nm, GATE.ACTIVE.S.4 9 nm, GATE.S.1 54 nm pitch), measured
/// on the placed shapes.
#[test]
fn glue_spacing_is_the_decks() {
    let pdk = asap7();
    let tiles = [finfet(DeviceKind::Nmos, 2, 1680, &pdk), finfet(DeviceKind::Nmos, 2, 1680, &pdk)];
    let (f, fill) = glue(&tiles, &pdk, &Spacing::of(&pdk, &ASAP7_ROLES), 2).unwrap();
    let placed = place_macros(&tiles, &f.layout());
    let diff = Process::layer(&pdk, "diff").unwrap();
    let poly = Process::layer(&pdk, "poly").unwrap();
    let dev = |m: &Macro| m.shapes.iter().filter(|s| s.layer == diff).map(|s| s.rect).min_by_key(|r| r.y).unwrap();
    let (a, b) = (dev(&placed[0]), dev(&placed[1]));
    assert_eq!(a.y, b.y);
    assert_eq!(b.x - (a.x + a.w), 92);
    assert!(fill.shapes.iter().all(|s| s.rect.x - (a.x + a.w) >= 9 && b.x - (s.rect.x + s.rect.w) >= 9));
    let mut xs: Vec<i32> = placed.iter().flat_map(|m| m.shapes.iter()).chain(&fill.shapes).filter(|s| s.layer == poly).map(|s| s.rect.x).collect();
    xs.sort_unstable();
    xs.dedup();
    assert!(xs.windows(2).all(|w| w[1] - w[0] == 54), "{xs:?}");
}

/// Oracle: the asap7 deck. Tiles of different fin counts (gates of
/// different heights, taps on different fin rows) glue with no refusal,
/// DRC clean, NMOS and PMOS, in both orders and short between tall; without the fill GATE.S.1
/// fires.
#[test]
fn different_fin_counts_glue_on_asap7() {
    let pdk = asap7();
    for kind in [DeviceKind::Nmos, DeviceKind::Pmos] {
        let (few, many) = (finfet(kind, 2, 54, &pdk), finfet(kind, 3, 270, &pdk));
        let poly = Process::layer(&pdk, "poly").unwrap();
        let height = |m: &Macro| m.shapes.iter().filter(|s| s.layer == poly).map(|s| s.rect.y + s.rect.h).max();
        assert_ne!(height(&few), height(&many), "the fixture needs unequal gate heights");
        for tiles in [vec![few.clone(), many.clone()], vec![many.clone(), few.clone()], vec![many.clone(), few.clone(), many.clone()]] {
            let clean = glued_findings(&tiles, true, &pdk);
            assert!(clean.is_empty(), "{kind:?}: {clean:?}");
            let bare = glued_findings(&tiles, false, &pdk);
            assert!(bare.iter().any(|r| r == "GATE.S.1"), "{kind:?}: {bare:?}");
        }
    }
}

/// Negative control: gates that share no height refuse.
/// Oracle: the input: gates moved 10 µm up share no height with the left tile's by construction.
#[test]
fn disjoint_gate_spans_refuse() {
    let pdk = asap7();
    let short = finfet(DeviceKind::Nmos, 2, 1680, &pdk);
    let mut far = short.clone();
    let poly = Process::layer(&pdk, "poly").unwrap();
    far.shapes.iter_mut().filter(|s| s.layer == poly).for_each(|s| s.rect.y += 10_000);
    assert_eq!(glue(&[short, far], &pdk, &Spacing::of(&pdk, &ASAP7_ROLES), 2).map(|_| ()), Err(Refusal::GateSpan { tile: 1 }));
}

/// One planar single-row tile of `kind`, 2 fingers of 1 µm by 150 nm.
fn mosfet(kind: DeviceKind, pdk: &verify::Pdk) -> Macro {
    let group = DeviceGroup { devices: vec![DeviceId(0)] };
    let c = analog::Constraints { unitization: vec![unit(kind, 2, 1000, 150)], ..analog::Constraints::default() };
    Mosfet::enumerate(&group, &c, pdk)
        .into_iter()
        .filter(|v| v.style == Pattern::Single && v.rows == 1)
        .map(|v| v.draw(&group, &c, pdk))
        .min_by_key(|m| i64::from(m.bbox.w) * i64::from(m.bbox.h))
        .expect("a single-row variant")
}

/// Oracle: the sky130 deck through GPurify. Two NMOS tiles, whose facing
/// shapes are diffusion, poly, li and met1, glue closer than the nwell's
/// 1270 nm; two PMOS tiles keep their wells 1270 nm apart. Both DRC clean.
#[test]
fn only_facing_layers_set_the_gap_on_sky130() {
    let pdk = verify::Pdk::builtin("sky130").expect("sky130 loads");
    let spacing = Spacing::of(&pdk, &SKY130_ROLES);
    let nwell = Process::layer(&pdk, "nwell").unwrap();
    for (kind, wells) in [(DeviceKind::Nmos, false), (DeviceKind::Pmos, true)] {
        let tiles = [mosfet(kind, &pdk), mosfet(kind, &pdk)];
        let (f, _) = glue(&tiles, &pdk, &spacing, 10).unwrap();
        let placed = place_macros(&tiles, &f.layout());
        let shapes: Vec<Shape> = placed.iter().flat_map(|m| m.shapes.iter().copied()).collect();
        let rules: Vec<String> = verify::drc(&shapes, &[], &pdk).into_iter().map(|f| f.rule).filter(|r| !r.ends_with("_density")).collect();
        assert!(rules.is_empty(), "{kind:?}: {rules:?}");
        let gap = f.x[1] - (f.x[0] + f.w[0]);
        if wells {
            let well = |m: &Macro| m.shapes.iter().filter(|s| s.layer == nwell).map(|s| s.rect).next().unwrap();
            let (a, b) = (well(&placed[0]), well(&placed[1]));
            assert!(b.x - (a.x + a.w) >= 1270, "{kind:?}: wells {} apart", b.x - (a.x + a.w));
        } else {
            assert!(gap < 1270, "{kind:?}: gap {gap}");
        }
    }
}

/// Oracle: the sky130 deck through GPurify. An NMOS beside a PMOS (its
/// n-well facing the NMOS diffusion) glued at the facing gap is DRC clean,
/// and the gap is narrower than the widest drawn-layer spacing (nwell,
/// 1270 nm) the engine's one-row floorplan uses. Negative control: with no
/// spacing rows the tiles abut and DRC fires.
#[test]
fn facing_gap_is_clean_and_tighter_on_sky130() {
    let pdk = verify::Pdk::builtin("sky130").expect("sky130 loads");
    let tiles = [mosfet(DeviceKind::Nmos, &pdk), mosfet(DeviceKind::Pmos, &pdk)];
    let drc = |spacing: &Spacing| {
        let (f, _) = glue(&tiles, &pdk, spacing, 10).unwrap();
        let shapes: Vec<Shape> = place_macros(&tiles, &f.layout()).into_iter().flat_map(|m| m.shapes).collect();
        let mut rules: Vec<String> = verify::drc(&shapes, &[], &pdk).into_iter().map(|f| f.rule).filter(|r| !r.ends_with("_density")).collect();
        rules.sort_unstable();
        rules.dedup();
        (f.x[1] - (f.x[0] + f.w[0]), rules)
    };
    let (gap, rules) = drc(&Spacing::of(&pdk, &SKY130_ROLES));
    assert!(rules.is_empty(), "gap {gap}: {rules:?}");
    let widest = SKY130_ROLES.iter().filter_map(|r| pdk.space(r)).max().unwrap();
    assert!(gap < widest, "gap {gap} vs widest {widest}");
    eprintln!("sky130 facing gap {gap} nm, widest {widest} nm");
    let (gap, rules) = drc(&Spacing::default());
    assert!(gap < 10 && !rules.is_empty(), "abutted, gap {gap}: {rules:?}");
}

/// Oracle: the sky130 deck. A mirrored NMOS pair under a PMOS on the axis,
/// every gap from the facing shapes, is DRC clean, and the pair's halves
/// sit closer than the widest drawn-layer spacing.
#[test]
fn row_stack_signs_off_on_sky130() {
    let pdk = verify::Pdk::builtin("sky130").expect("sky130 loads");
    let (n, p) = (mosfet(DeviceKind::Nmos, &pdk), mosfet(DeviceKind::Pmos, &pdk));
    let tiles = [n.clone(), n, p];
    let stack = Stack { row: vec![0, 0, 1], side: vec![Side::Left, Side::Right, Side::Axis] };
    let f = row_stack(&tiles, &stack, &Spacing::of(&pdk, &SKY130_ROLES), 10).unwrap();
    let shapes: Vec<Shape> = place_macros(&tiles, &f.layout()).into_iter().flat_map(|m| m.shapes).collect();
    let mut rules: Vec<String> = verify::drc(&shapes, &[], &pdk).into_iter().map(|f| f.rule).filter(|r| !r.ends_with("_density")).collect();
    rules.sort_unstable();
    rules.dedup();
    assert!(rules.is_empty(), "{rules:?}");
    assert!(f.x[1] - (f.x[0] + f.w[0]) < 1270);
}
