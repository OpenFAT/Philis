use analog::base::{place_macros, DeviceGroup, DeviceId, DeviceKind, Macro, Orient, Rect, Shape};
use analog::cell::{SeriesParallel, Unitization};
use substrate3::cells::finfet::FinFet;
use substrate3::cells::Cell;

use super::{glue, row_stack, targets, Refusal, Side, Stack};

/// Oracle: mirror geometry. Every pair's halves sit at equal distances from
/// the axis, the left one turned `Mx180`; the self is centred on it.
#[test]
fn pairs_mirror_about_the_axis() {
    let size = [(100, 40), (60, 40), (60, 40), (80, 30), (80, 30)];
    let side = vec![Side::Axis, Side::Left, Side::Right, Side::Right, Side::Left];
    let f = row_stack(&size, &Stack { row: vec![0; 5], side }, 10, 2).unwrap();
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
    let macros: Vec<Macro> = size.iter().map(|&(w, h)| Macro { bbox: Rect { x: 7, y: 3, w, h }, ..Macro::default() }).collect();
    for (i, m) in place_macros(&macros, &f.layout()).iter().enumerate() {
        assert_eq!((m.bbox.x, m.bbox.y), (f.x[i], f.y[i]), "tile {i}");
    }
}

/// Negative control: a pair whose halves differ refuses.
#[test]
fn unequal_halves_refuse() {
    let stack = Stack { row: vec![3, 3], side: vec![Side::Left, Side::Right] };
    assert_eq!(row_stack(&[(60, 40), (62, 40)], &stack, 10, 2), Err(Refusal::PairMismatch { row: 3 }));
    let stack = Stack { row: vec![3, 3], side: vec![Side::Left, Side::Left] };
    assert_eq!(row_stack(&[(60, 40), (60, 40)], &stack, 10, 2), Err(Refusal::PairMismatch { row: 3 }));
}

/// Oracle: stage order. Rows stack bottom to top by stage index, whatever
/// the input order, each above the tallest of the row below plus the gap.
#[test]
fn rows_follow_stage_order() {
    let size = [(50, 30), (40, 70), (40, 70), (90, 20)];
    let stack = Stack { row: vec![2, 0, 0, 1], side: vec![Side::Axis, Side::Left, Side::Right, Side::Axis] };
    let f = row_stack(&size, &stack, 10, 2).unwrap();
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

/// One FinFet tile of `kind` and `nf` fingers, 1680 nm wide, as the engine draws
/// a single-device call.
fn finfet(kind: DeviceKind, nf: u16, pdk: &verify::Pdk) -> Macro {
    let group = DeviceGroup { devices: vec![DeviceId(0)] };
    let mut c = analog::Constraints::default();
    c.unitization.push(Unitization {
        devices: group.devices.clone(),
        device_type: kind,
        dev_nf: vec![nf],
        target_ratio: vec![1],
        unit_w: 1680,
        unit_l: 20,
        series_parallel: SeriesParallel::Parallel,
        dummy_required: false,
        route_matching_required: false,
        class: None,
        kind: None,
        series: Vec::new(),
        style: None,
    });
    FinFet { shared: false }.draw(&group, &c, pdk)
}

/// The glued row's DRC rule ids, density rules aside, with or without fill.
fn glued_findings(tiles: &[Macro], with_fill: bool, pdk: &verify::Pdk) -> Vec<String> {
    let (f, fill) = glue(tiles, pdk, 0, 2).expect("same-height tiles glue");
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
        let tiles = [finfet(kind, 2, &pdk), finfet(kind, 3, &pdk)];
        let clean = glued_findings(&tiles, true, &pdk);
        assert!(clean.is_empty(), "{kind:?}: {clean:?}");
        let bare = glued_findings(&tiles, false, &pdk);
        assert!(bare.iter().any(|r| r == "GATE.S.1"), "{kind:?}: {bare:?}");
    }
}

/// Glued actives sit exactly ACTIVE.S.2A apart, the fill 9 nm clear of
/// each (GATE.ACTIVE.S.4), every gate on the 54 nm lattice.
#[test]
fn glue_spacing_is_the_decks() {
    let pdk = asap7();
    let tiles = [finfet(DeviceKind::Nmos, 2, &pdk), finfet(DeviceKind::Nmos, 2, &pdk)];
    let (f, fill) = glue(&tiles, &pdk, 0, 2).unwrap();
    let placed = place_macros(&tiles, &f.layout());
    let diff = analog::base::Process::layer(&pdk, "diff").unwrap();
    let poly = analog::base::Process::layer(&pdk, "poly").unwrap();
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

/// Negative control: tiles whose gates span different heights refuse.
#[test]
fn unequal_gate_spans_refuse() {
    let pdk = asap7();
    let short = finfet(DeviceKind::Nmos, 2, &pdk);
    let mut tall = finfet(DeviceKind::Nmos, 2, &pdk);
    let poly = analog::base::Process::layer(&pdk, "poly").unwrap();
    tall.shapes.iter_mut().filter(|s| s.layer == poly).for_each(|s| s.rect.h += 27);
    assert_eq!(glue(&[short, tall], &pdk, 0, 2).map(|_| ()), Err(Refusal::GateSpan { tile: 1 }));
}
