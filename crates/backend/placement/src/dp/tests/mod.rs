//! Detailed-placement tests: coarse layouts → `place` and its moves, checked
//! against the analog rules (`SymmetryGroup`, `HeatSeparation`), the full
//! encroachment scan, hand-worked keys and seeded reproducibility.
//!
//! Contents: none public.

mod moves;

use analog::base::geom::Rect;
use analog::base::ids::{AxisId, Target};
use analog::base::DeviceId;
use analog::base::Orient;
use analog::placement::symmetry::{SymMode, Symmetry, SymmetryGroup};
use analog::placement::DtiBand;
use analog::Rule;

use super::moves::accept;
use super::*;
use crate::gp::mechanics::analog_theta;
use crate::gp::mechanics::{analog_violations, encroachment, hpwl};
use crate::gp::VariantSpace;

/// sky130-like placement rules the fixed-geometry tests are written against.
const CLEARANCE_NM: i32 = 2000;

// ---- private ----

fn rules(c: i32) -> crate::gp::PlaceRules {
    crate::gp::PlaceRules::uniform(5, c)
}

/// Layout of devices `(x, y, hw, hh)`, each in its own group.
fn layout(cells: &[(i32, i32, i32, i32)]) -> Layout {
    let n = cells.len();
    Layout {
        x: cells.iter().map(|c| c.0).collect(),
        y: cells.iter().map(|c| c.1).collect(),
        hw: cells.iter().map(|c| c.2).collect(),
        hh: cells.iter().map(|c| c.3).collect(),
        variant: vec![0; n],
        axis: vec![0; n],
        branch: vec![false; n],
        groups: (0..n).map(|i| vec![DeviceId(i as u16)]).collect(),
        orient: vec![Orient::default(); n],
        power_uw: vec![0; n],
        temp_mc: vec![0; n],
        units: Default::default(),
    }
}

fn run(coarse: &Layout, macros: &[Macro], variants: &[VariantSpace], reqs: &Requirements<Layout>, fixed: &[bool], seed: u64) -> Layout {
    let locks = locks::locks(reqs, coarse.x.len(), variants);
    place(coarse, macros, variants, reqs, fixed, &locks, &mut crate::gp::Prices::new(), &rules(CLEARANCE_NM), &[], seed, Schedule::cold()).0
}

fn sym(a: u16, b: u16) -> Symmetry {
    Symmetry { a: Target::Device(DeviceId(a)), b: Target::Device(DeviceId(b)), axis: AxisId(0), mode: SymMode::Perfect }
}

// ---- rotation ----

/// Four tall devices; 0 and 1 form one matched group (`groups`, which dp ignores).
fn rotate_bench() -> Layout {
    let mut l = layout(&[(0, 0, 1_000, 8_000), (20_000, 0, 1_000, 8_000), (40_000, 0, 1_000, 8_000), (60_000, 0, 1_000, 8_000)]);
    l.groups = vec![vec![DeviceId(0), DeviceId(1)], vec![DeviceId(2)], vec![DeviceId(3)]];
    l
}

/// Oracle: the input's symmetry pair: both members, and only they, carry a lock.
#[test]
fn matched_cells_share_one_orient_set() {
    let reqs = Requirements { hard: vec![Box::new(SymmetryGroup(vec![sym(0, 1)]))], ..Default::default() };
    let k = locks::locks(&reqs, 3, &[]);
    assert_eq!(k.orient_of[0], k.orient_of[1]);
    assert!(k.orient_of[0].is_some());
    assert_eq!(k.orient_of[2], None);
}

/// A matched set that is no hard rule and has no units: the lock can only come
/// from `matched_pairs`, and it must actually turn.
/// Oracle: the `OrientationSet` contract: equal orients on every seed, and at least one non-R0.
#[test]
fn mixed_polarity_composite_halves_turn_together() {
    use analog::placement::{OrientCheck, OrientationSet};
    let coarse = rotate_bench();
    let reqs = Requirements {
        budget: vec![Box::new(OrientationSet { members: vec![DeviceId(0), DeviceId(1)], check: OrientCheck::Phi, cell_of: vec![] })],
        ..Default::default()
    };
    let mut turned = false;
    for seed in 1..=20u64 {
        let l = run(&coarse, &[], &[], &reqs, &[false; 4], seed);
        assert_eq!(l.orient[0], l.orient[1], "seed {seed}");
        turned |= l.orient[0] != Orient::R0;
    }
    assert!(turned, "the locked set never turned");
}

/// PLC-21: a Mirror pair is seeded as reflections and every set rotation keeps
/// the partner at `orient[0].then(Mx180)`.
/// Oracle: the orientation group: `orient[1] == orient[0].then(Mx180)` on every seed.
#[test]
fn mirror_pair_keeps_its_reflection_through_rotation() {
    let coarse = rotate_bench();
    let reqs = Requirements { hard: vec![Box::new(SymmetryGroup(vec![Symmetry { mode: SymMode::Mirror, ..sym(0, 1) }]))], ..Default::default() };
    let mut turned = false;
    for seed in 1..=10u64 {
        let l = run(&coarse, &[], &[], &reqs, &[false; 4], seed);
        assert_eq!(l.orient[1], l.orient[0].then(Orient::Mx180), "seed {seed}");
        turned |= l.orient[0] != Orient::R0;
    }
    assert!(turned, "the mirrored set never turned");
}

/// Oracle: the coarse extents swapped by hand for every quarter turn.
#[test]
fn extents_stay_in_lockstep_with_orientation() {
    let coarse = rotate_bench();
    let reqs = Requirements { hard: vec![Box::new(SymmetryGroup(vec![sym(0, 1)]))], ..Default::default() };
    let l = run(&coarse, &[], &[], &reqs, &[false; 4], 42);
    for i in 0..4 {
        let (w, h) = (coarse.hw[i], coarse.hh[i]);
        let want = if l.orient[i].swaps_axes() { (h, w) } else { (w, h) };
        assert_eq!((l.hw[i], l.hh[i]), want, "device {i}");
    }
    assert_eq!(l.orient[0], l.orient[1]);
}

/// Oracle: metamorphic: the same seed twice gives the same layout.
#[test]
fn placement_is_deterministic_for_a_seed() {
    let coarse = rotate_bench();
    let go = || {
        let l = run(&coarse, &[], &[], &Requirements::default(), &[false; 4], 7);
        (l.x, l.y, l.orient)
    };
    assert_eq!(go(), go());
}

/// Oracle: liveness only: some free cell ends off R0. No independent oracle for which orient is right.
#[test]
fn rotation_actually_happens() {
    let l = run(&rotate_bench(), &[], &[], &Requirements::default(), &[false; 4], 0);
    assert!(l.orient[2..].iter().any(|&o| o != Orient::R0), "{:?}", l.orient);
}

// ---- variant reshape ----

fn alt(w: i32, h: i32) -> Macro {
    Macro { shapes: Vec::new(), pins: Vec::new(), bbox: Rect { x: 0, y: 0, w, h }, units: Vec::new(), dummies: Vec::new(), ..Default::default() }
}

/// Three alternatives per cell, not ordered by similarity.
fn spaces() -> Vec<VariantSpace> {
    let s = || VariantSpace { alternatives: vec![alt(2_000, 16_000), alt(16_000, 2_000), alt(6_000, 6_000)] };
    vec![s(), s()]
}

fn drawn(variants: &[VariantSpace]) -> Vec<Macro> {
    variants.iter().map(|v| v.alternatives[0].clone()).collect()
}

fn variant_bench() -> Layout {
    layout(&[(0, 0, 1_000, 8_000), (200_000, 0, 1_000, 8_000)])
}

fn assert_extents(l: &Layout, variants: &[VariantSpace], seed: u64) {
    for (i, v) in variants.iter().enumerate().take(2) {
        let (w, h) = crate::gp::mechanics::variant_extents(&v.alternatives[l.variant[i] as usize]);
        let want = if l.orient[i].swaps_axes() { (h, w) } else { (w, h) };
        assert_eq!((l.hw[i], l.hh[i]), want, "seed {seed}: cell {i} sized off-variant");
    }
}

/// Oracle: the variant space: each cell's extent equals its chosen alternative's.
#[test]
fn reshape_changes_the_variant_and_keeps_the_extent_invariant() {
    let coarse = variant_bench();
    let variants = spaces();
    let mut fired = false;
    for seed in 0..8u64 {
        let l = run(&coarse, &drawn(&variants), &variants, &Requirements::default(), &[false; 2], seed);
        fired |= l.variant != coarse.variant;
        assert_extents(&l, &variants, seed);
    }
    assert!(fired, "the reshape move never fired");
}

/// Oracle: the input: a fixed cell keeps variant 0 and its coarse extent.
#[test]
fn a_fixed_cell_never_reshapes() {
    let coarse = variant_bench();
    let variants = spaces();
    for seed in 0..8u64 {
        let l = run(&coarse, &drawn(&variants), &variants, &Requirements::default(), &[true, false], seed);
        assert_eq!(l.variant[0], 0, "seed {seed}");
        assert_eq!((l.hw[0], l.hh[0]), (coarse.hw[0], coarse.hh[0]), "seed {seed}");
    }
}

/// Oracle: the symmetry pair: equal variants on every seed.
#[test]
fn mirror_partners_reshape_together() {
    let s = || VariantSpace { alternatives: vec![alt(1_000, 4_000), alt(2_000, 2_000), alt(4_000, 1_000)] };
    let variants = vec![s(), s()];
    let coarse = layout(&[(-20_000, 0, 500, 2_000), (20_000, 0, 500, 2_000)]);
    let reqs = Requirements { hard: vec![Box::new(SymmetryGroup(vec![sym(0, 1)]))], ..Default::default() };
    let mut reshaped = false;
    for seed in 1..=20u64 {
        let l = run(&coarse, &drawn(&variants), &variants, &reqs, &[false; 2], seed);
        assert_eq!(l.variant[0], l.variant[1], "seed {seed}");
        reshaped |= l.variant[0] != 0;
    }
    assert!(reshaped, "the locked pair never reshaped");
}

/// Fixed means drawn as given, not pinned: two stacked fixed cells separate.
/// Oracle: `encroachment`'s full scan reads zero; variant, orient and extent equal the input's.
#[test]
fn fixed_cells_separate_but_keep_shape() {
    let variants = spaces();
    let coarse = layout(&[(0, 0, 1_000, 8_000), (0, 0, 1_000, 8_000)]);
    for seed in 0..4u64 {
        let l = run(&coarse, &drawn(&variants), &variants, &Requirements::default(), &[true, true], seed);
        assert_eq!(encroachment(&l, 0), 0.0, "seed {seed}");
        assert_eq!(l.variant, vec![0, 0], "seed {seed}");
        assert_eq!(l.orient, vec![Orient::R0, Orient::R0], "seed {seed}");
        assert_eq!((&l.hw, &l.hh), (&coarse.hw, &coarse.hh), "seed {seed}");
    }
}

/// Oracle: liveness: some seed gives two different variants.
#[test]
fn unmatched_cells_reshape_independently() {
    let coarse = variant_bench();
    let variants = spaces();
    assert!((0..16u64).any(|seed| {
        let l = run(&coarse, &drawn(&variants), &variants, &Requirements::default(), &[false; 2], seed);
        l.variant[0] != l.variant[1]
    }));
}

// ---- prices ----

/// T6: the dual step is the flow's, once per epoch on the scored layout; gp
/// and dp only bind, so a violated budget leaves a fresh `Prices` unstepped.
/// Oracle: the input: a fresh `Prices` must still read λ0 after `place`.
#[test]
fn place_does_not_settle() {
    #[derive(Clone, Copy)]
    struct Over;
    impl Rule for Over {
        type On = Layout;
        fn cost(self, _: &Layout) -> f32 {
            0.0
        }
        fn satisfied(self, _: &Layout) -> bool {
            false
        }
    }
    let variants = spaces();
    let macros = drawn(&variants);
    let reqs = Requirements { hard: Vec::new(), budget: vec![Box::new(vec![Over])], cost: Vec::new() };
    let mut prices = crate::gp::Prices::new();
    let inp = crate::gp::GpInput {
        macros: &macros,
        variants: &variants,
        assignment: &[0, 0],
        reqs: &reqs,
        rules: &rules(CLEARANCE_NM),
        net_weight: &[],
        n_axes: 1,
        power_uw: &[],
        units: Default::default(),
        iterate: true,
    };
    let (coarse, _) = crate::gp::place(&inp, &mut prices, 3);
    let locks = locks::locks(&reqs, coarse.x.len(), &variants);
    place(&coarse, &macros, &variants, &reqs, &[false; 2], &locks, &mut prices, &rules(CLEARANCE_NM), &[], 3, Schedule::cold());
    assert_eq!(prices.drift(), f64::INFINITY, "no dual step inside place");
    assert_eq!(prices.steps(), 0);
}

// ---- exact projection ----

fn sym_bench() -> (Requirements<Layout>, Layout) {
    let reqs = Requirements {
        hard: vec![Box::new(SymmetryGroup(vec![Symmetry {
            a: Target::Device(DeviceId(0)),
            b: Target::Device(DeviceId(1)),
            axis: AxisId(0),
            mode: SymMode::Perfect,
        }]))],
        budget: Vec::new(),
        cost: Vec::new(),
    };
    (reqs, layout(&[(-30_000, 0, 1_000, 1_000), (30_500, 3_500, 1_000, 1_000)]))
}

/// Oracle: the analog `SymmetryGroup` rule reports zero violations.
#[test]
fn symmetry_holds_at_exit_with_room_to_move() {
    let (reqs, coarse) = sym_bench();
    let l = run(&coarse, &[], &[], &reqs, &[false; 2], 11);
    assert_eq!(analog_violations(&reqs, &l), 0, "x = {:?}, y = {:?}, axis = {:?}", l.x, l.y, l.axis);
}

/// Oracle: the analog `SymmetryGroup` rule, and the fixed cell's input extent and orient.
#[test]
fn projection_moves_fixed_cells_but_not_their_shape() {
    let (reqs, coarse) = sym_bench();
    let mut moved = false;
    for seed in 0..4u64 {
        let l = run(&coarse, &[], &[], &reqs, &[true, false], seed);
        assert_eq!(analog_violations(&reqs, &l), 0, "seed {seed}");
        assert_eq!((l.hw[0], l.hh[0], l.orient[0]), (coarse.hw[0], coarse.hh[0], Orient::R0), "seed {seed}");
        moved |= (l.x[0], l.y[0]) != (coarse.x[0], coarse.y[0]);
    }
    assert!(moved, "fixed cell 0 never left its coarse position");
}

/// A mirror pair stacked on its own axis satisfies `Symmetry` with zero
/// separation; single-cell moves must still pull it apart (the partner follows
/// by projection) until the pair is clearance-legal and still mirrored.
/// Oracle: the analog `SymmetryGroup` rule and `encroachment`'s full scan.
#[test]
fn a_stacked_symmetric_pair_separates_and_stays_mirrored() {
    let (reqs, _) = sym_bench();
    let coarse = layout(&[(0, 0, 1_000, 1_000), (0, 0, 1_000, 1_000), (5_000, 0, 1_000, 1_000)]);
    let l = run(&coarse, &[], &[], &reqs, &[false; 3], 3);
    assert_eq!(analog_violations(&reqs, &l), 0, "x = {:?}, y = {:?}", l.x, l.y);
    assert_eq!(encroachment(&l, CLEARANCE_NM), 0.0, "x = {:?}, y = {:?}", l.x, l.y);
}

/// A diff stage: input pair and load pair mirrored about ONE shared axis, plus
/// a self-symmetric tail cell, all starting piled on top of each other.
/// Oracle: the analog `SymmetryGroup` rule and `encroachment`'s full scan, every seed.
#[test]
fn two_pairs_and_a_tail_share_one_axis_legally() {
    let sym = |a: u16, b: u16| Symmetry { a: Target::Device(DeviceId(a)), b: Target::Device(DeviceId(b)), axis: AxisId(0), mode: SymMode::Perfect };
    let group = || SymmetryGroup(vec![sym(0, 1), sym(2, 3), sym(4, 4)]);
    let reqs = Requirements { hard: vec![Box::new(group())], budget: Vec::new(), cost: vec![Box::new(group())] };
    for seed in 0..4 {
        let coarse = layout(&[(0, 0, 3_000, 2_000), (100, 50, 3_000, 2_000), (0, 0, 4_000, 1_500), (-50, 20, 4_000, 1_500), (30, 30, 2_500, 2_500)]);
        let l = run(&coarse, &[], &[], &reqs, &[false; 5], seed);
        assert_eq!(analog_violations(&reqs, &l), 0, "seed {seed}: x = {:?}, y = {:?}", l.x, l.y);
        assert_eq!(encroachment(&l, CLEARANCE_NM), 0.0, "seed {seed}: x = {:?}, y = {:?}", l.x, l.y);
    }
}

/// Oracle: metamorphic: the same seed twice gives the same layout.
#[test]
fn projection_is_deterministic_for_a_seed() {
    let go = || {
        let (reqs, coarse) = sym_bench();
        let l = run(&coarse, &[], &[], &reqs, &[false; 2], 7);
        (l.x, l.y, l.axis)
    };
    assert_eq!(go(), go());
}

// ---- report ----

/// Oracle: hand arithmetic: a 100 nm gap under a 270 nm clearance on 500 nm cells.
#[test]
fn report_counts_clearance_only_residue() {
    let l = layout(&[(0, 0, 500, 500), (1_100, 0, 500, 500)]);
    let rep = report(&Nets::from_macros(&[]), &Requirements::default(), &l, &crate::gp::Prices::new(), &rules(270));
    let rules: Vec<&str> = rep.hard_violations.iter().map(|v| v.rule.as_str()).collect();
    let clearance_rows: Vec<_> = rep.hard_violations.iter().filter(|v| v.rule == "clearance encroachment").collect();
    assert_eq!(clearance_rows.len(), 1, "{rules:?}");
    assert_eq!(clearance_rows[0].margin, 215_900);
    assert!(!rules.contains(&"device overlap"), "{rules:?}");
}

/// PLC-10 step 0: `Schedule::cold()` is exactly the constants it replaced.
/// Oracle: golden: the layout the replaced constants produced. Pins behaviour, not correctness.
#[test]
fn cold_schedule_reproduces_todays_layout() {
    let l = run(&rotate_bench(), &[], &[], &Requirements::default(), &[false; 4], 7);
    assert_eq!(l.x, vec![26615, 8990, 44550, 53905]);
    assert_eq!(l.y, vec![-975, -2545, 2190, 265]);
    assert_eq!(l.orient, vec![Orient::R270, Orient::R0, Orient::R180, Orient::R0]);
}

// ---- PLC-18: dimensionless PEX ----

/// Two cells with every converted cost term live; all lengths ×`k`.
fn scene(k: i32) -> (Nets, Layout, Requirements<Layout>) {
    let mut l = layout(&[(0, 0, 5_000 * k, 5_000 * k), (40_000 * k, 3_000 * k, 5_000 * k, 5_000 * k)]);
    l.axis = vec![10_000 * k; 2];
    l.branch = vec![true];
    let pin = |x: i32| Macro {
        pins: vec![analog::base::Pin {
            name: "G".to_string(),
            net: analog::base::NetId(0),
            at: Rect { x: x * k, y: 4_950 * k, w: 100 * k, h: 100 * k },
            layer: analog::base::LayerId(0),
        }],
        bbox: Rect { x: 0, y: 0, w: 10_000 * k, h: 10_000 * k },
        ..Default::default()
    };
    let nets = Nets::from_macros(&[pin(0), pin(9_900)]);
    let (d0, d1) = (Target::Device(DeviceId(0)), Target::Device(DeviceId(1)));
    let reqs = Requirements {
        hard: Vec::new(),
        budget: Vec::new(),
        cost: vec![
            Box::new(vec![analog::placement::Proximity { a: d0, b: d1, max_distance_nm: 10_000 * k }]),
            Box::new(vec![analog::placement::Isolation { a: d0, b: d1, min_distance_nm: 60_000 * k }]),
            Box::new(vec![DtiBand { a: d0, b: d1, s_max_nm: 200 * k, d_dti_nm: 50_000 * k, branch: BranchId(0), seed_isolate: true }]),
            Box::new(SymmetryGroup(vec![Symmetry { a: d0, b: d1, axis: AxisId(0), mode: SymMode::Perfect }])),
        ],
    };
    (nets, l, reqs)
}

/// PEX reads no absolute length: scaling every length by 2 (exact in binary
/// float) leaves it unchanged. Breaks if a cost keeps nm units or HPWL loses `/L_ref`.
/// Oracle: metamorphic: every length ×2 (exact in binary float) leaves PEX unchanged.
#[test]
fn energy_is_invariant_to_scaling_all_lengths() {
    let (nets1, l1, reqs1) = scene(1);
    let (nets2, l2, reqs2) = scene(2);
    for b in &reqs1.cost {
        assert!(b.cost(&l1) > 0.0, "{:?} must be live", b.kind());
    }
    assert!(hpwl(&nets1, &l1) > 0.0);
    let prices = crate::gp::Prices::new();
    let sa1 = Sa::new(nets1, 2, &reqs1, &prices, &[], &rules(0));
    let sa2 = Sa::new(nets2, 2, &reqs2, &prices, &[], &rules(0));
    let (e1, e2) = (sa1.pex(&l1), sa2.pex(&l2));
    assert!((e2 - e1).abs() <= 1e-6 * e1.abs(), "{e1} vs {e2}");
}

// ---- thermal separation (PLC-14) ----

/// A matched pair seeded against a hot cell: dp must open both gaps to the
/// 1 µm/mW floor.
/// Oracle: the analog `HeatSeparation` rule's residual reads zero.
#[test]
fn dp_pushes_a_matched_pair_away_from_a_hot_cell() {
    use analog::placement::HeatSeparation;
    let coarse = layout(&[(0, 0, 1_000, 1_000), (2_000, 0, 1_000, 1_000), (4_000, 0, 1_000, 1_000)]);
    let d = |i: u16| Target::Device(DeviceId(i));
    let heat =
        vec![HeatSeparation { victim: d(0), source: d(2), min_gap_nm: 2_000 }, HeatSeparation { victim: d(1), source: d(2), min_gap_nm: 2_000 }];
    let reqs = Requirements { budget: vec![Box::new(heat.clone())], cost: vec![Box::new(heat.clone())], ..Default::default() };
    for seed in 1..=10u64 {
        let l = run(&coarse, &[], &[], &reqs, &[false; 3], seed);
        for h in &heat {
            assert_eq!(h.residual(&l), 0.0, "seed {seed}: gap {}", l.edge_gap(h.victim, h.source));
        }
    }
}

// ---- performance rows (PLC-16) ----

/// 10 µm square cell: a pin on `own` at the left face, one on the shared net 2
/// at the right face.
fn perf_cell(own: u16) -> Macro {
    let pin = |net: u16, x: i32| analog::base::Pin {
        name: format!("P{net}"),
        net: analog::base::NetId(net),
        at: Rect { x, y: 4_950, w: 100, h: 100 },
        layer: analog::base::LayerId(0),
    };
    Macro { pins: vec![pin(own, 0), pin(2, 9_900)], bbox: Rect { x: 0, y: 0, w: 10_000, h: 10_000 }, ..Default::default() }
}

/// T5 (unit): net 0 (cells 0, 1) and net 1 (cells 2, 3) compete with the
/// shared net 2 for adjacency. A row on net 0 alone, priced with
/// residual > 0 at the no-row MST (w·mst·1e-3 / 0.8 > 1 for mst > 800 nm),
/// must shorten net 0's mean MST over seeds 1..=10. Fresh `Prices` (λ0) each
/// run: `place` does not settle prices (`place_does_not_settle`).
/// Oracle: metamorphic: the same scene with and without the row; net 0's MST from `PlacePerf` must shrink.
#[test]
fn a_row_shortens_its_sensitive_net() {
    use analog::placement::PlacePerf;
    use analog::routing::PerformanceBudget;
    let macros: Vec<Macro> = [0, 0, 1, 1].into_iter().map(perf_cell).collect();
    let variants: Vec<VariantSpace> = macros.iter().map(|m| VariantSpace { alternatives: vec![m.clone()] }).collect();
    let alts: Vec<&[Macro]> = variants.iter().map(|v| &v.alternatives[..]).collect();
    let row = PerformanceBudget::ground_c("t".into(), vec![analog::base::NetId(0)], vec![1.0], 1e-3);
    let perf = PlacePerf::new(&row, &alts, analog::placement::perf::RESERVE);
    assert_eq!(perf.items.len(), 1, "net 0 joins two cells");
    let coarse = layout(&[(0, 0, 5_000, 5_000), (40_000, 30_000, 5_000, 5_000), (40_000, 0, 5_000, 5_000), (0, 30_000, 5_000, 5_000)]);
    let with = Requirements { budget: vec![Box::new(perf.clone())], ..Default::default() };
    let mean = |reqs: &Requirements<Layout>| {
        (1..=10u64).map(|s| perf.mst_len(0, &run(&coarse, &macros, &variants, reqs, &[false; 4], s)) as f64).sum::<f64>() / 10.0
    };
    let (without, with) = (mean(&Requirements::default()), mean(&with));
    assert!(without > 800.0, "the row must bind at the no-row MST: {without}");
    assert!(with < without, "mean MST with the row {with} !< without {without}");
}
