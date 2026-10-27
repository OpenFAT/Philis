//! Tests of the flat anneal's moves and gate (`dp/moves.rs`): one move on a
//! hand-built layout → kept or refused, checked against the full
//! encroachment scan, the analog rules and the lexicographic key by hand.
//!
//! Contents: none public.

use super::*;

/// Cost pulls devices 0 and 1 together with no floor.
#[derive(Clone, Copy)]
struct Attract;

impl Rule for Attract {
    type On = Layout;
    fn cost(self, l: &Layout) -> f32 {
        ((l.x[0] - l.x[1]).abs() + (l.y[0] - l.y[1]).abs()) as f32
    }
}

// ---- private ----

/// Same bbox, one pin on the face at `x`.
fn pin_alt(x: i32) -> Macro {
    Macro {
        shapes: Vec::new(),
        pins: vec![analog::base::Pin {
            name: "G".to_string(),
            net: analog::base::NetId(0),
            at: Rect { x, y: 4_950, w: 100, h: 100 },
            layer: analog::base::LayerId(0),
        }],
        bbox: Rect { x: 0, y: 0, w: 10_000, h: 10_000 },
        units: Vec::new(),
        dummies: Vec::new(),
        ..Default::default()
    }
}

/// Equal bboxes, different pin faces: the reshape must be priced on pins.
/// `temp = 0` makes the refusal of the lengthening move unfakeable.
/// Oracle: HPWL recomputed from the pins before and after the move.
#[test]
fn reshape_is_priced_on_where_the_pins_land() {
    let space = || VariantSpace { alternatives: vec![pin_alt(0), pin_alt(9_900)] };
    let variants = vec![space(), space()];
    let mut l = layout(&[(0, 0, 5_000, 5_000), (100_000, 0, 5_000, 5_000)]);
    let reqs = Requirements::default();
    let prices = crate::gp::Prices::new();
    let nets = Nets::from_macros(&[pin_alt(0), pin_alt(0)]);
    assert_eq!(nets.count(), 1);
    let mut sa = Sa::new(nets, 2, &reqs, &prices, &[], &rules(0));
    let mut rng = SplitMix64::new(1);
    let free = |c: i32, _half: i32| c;

    let before = hpwl(&sa.nets, &l);
    assert!(try_reshape(&mut sa, &mut l, &mut rng, 0.0, &[0], &variants, &free, &free));
    assert_eq!(l.variant[0], 1);
    let after = hpwl(&sa.nets, &l);
    assert!(after < before, "{before} -> {after}");

    assert!(!try_reshape(&mut sa, &mut l, &mut rng, 0.0, &[0], &variants, &free, &free));
    assert_eq!(l.variant[0], 1);
    assert_eq!(hpwl(&sa.nets, &l), after, "refusal must revert the pin geometry too");
}

// ---- lexicographic acceptance ----

/// PEX change for moving device 0 to `(x, 0)`.
fn dpex(sa: &Sa, l: &mut Layout, x: i32) -> f64 {
    let before = sa.pex(l);
    let ox = l.x[0];
    l.x[0] = x;
    let d = sa.pex(l) - before;
    l.x[0] = ox;
    d
}

fn attract_bench() -> (Requirements<Layout>, Layout) {
    let reqs = Requirements { hard: Vec::new(), budget: Vec::new(), cost: vec![Box::new(vec![Attract])] };
    (reqs, layout(&[(0, 0, 1_000, 1_000), (4_000, 0, 1_000, 1_000)]))
}

/// At `temp = 1e12` Metropolis takes anything, so a refusal is the Φ tier's.
/// Oracle: `encroachment`'s full scan: the refused move would have stacked the cells.
#[test]
fn phi_rejects_a_cost_lowering_move_that_stacks_geometry() {
    let (reqs, mut l) = attract_bench();
    let prices = crate::gp::Prices::new();
    let mut sa = Sa::new(Nets::from_macros(&[]), 2, &reqs, &prices, &[], &rules(0));
    let mut rng = SplitMix64::new(1);
    assert!(dpex(&sa, &mut l, 4_000) < 0.0);
    assert!(!try_move(&mut sa, &mut l, &mut rng, 1e12, 0, 4_000, 0));
    assert_eq!(l.x[0], 0);
    assert_eq!(encroachment(&l, 0), 0.0);
}

/// Oracle: hand-built move that lowers PEX and raises Θ; the gate's lexicographic order says refuse.
#[test]
fn theta_outranks_pex_so_a_budget_is_never_traded_for_parasitics() {
    #[derive(Clone, Copy)]
    struct Separation;
    impl Rule for Separation {
        type On = Layout;
        fn cost(self, _: &Layout) -> f32 {
            0.0
        }
        fn residual(self, l: &Layout) -> f32 {
            ((2_000 - (l.x[0] - l.x[1]).abs()).max(0) as f32) / 2_000.0
        }
    }
    let (mut reqs, mut l) = attract_bench();
    reqs.budget = vec![Box::new(vec![Separation])];
    let prices = crate::gp::Prices::new();
    let mut sa = Sa::new(Nets::from_macros(&[]), 2, &reqs, &prices, &[], &rules(0));
    let mut rng = SplitMix64::new(1);
    assert_eq!(analog_theta(&reqs, &l), 0.0);
    assert!(dpex(&sa, &mut l, 3_000) < 0.0);
    assert!(!try_move(&mut sa, &mut l, &mut rng, 1e12, 0, 3_000, 0));
    assert_eq!(l.x[0], 0);
}

/// Oracle: at T = 1e12 Metropolis accepts any PEX increase with probability ≈ 1.
#[test]
fn metropolis_still_accepts_an_uphill_move_inside_the_pex_tier() {
    let (reqs, mut l) = attract_bench();
    let prices = crate::gp::Prices::new();
    let mut sa = Sa::new(Nets::from_macros(&[]), 2, &reqs, &prices, &[], &rules(0));
    let mut rng = SplitMix64::new(1);
    assert!(dpex(&sa, &mut l, -20_000) > 0.0);
    assert!(try_move(&mut sa, &mut l, &mut rng, 1e12, 0, -20_000, 0));
    assert_eq!(l.x[0], -20_000);
}

/// At the same hard-violation count, raising clearance encroachment must never
/// win against a cheaper PEX, even at an extreme temperature.
/// Oracle: the lexicographic order of the key, applied by hand to the two tuples.
#[test]
fn gate_never_trades_a_broken_pair_for_overlap() {
    let e = 1_000.0;
    let mut rng = SplitMix64::new(1);
    for draw in 0..100 {
        assert!(!accept((1, 0.002, e, 0.0), (1, 0.003, e - 2.0, 0.0), -1.0, 1e12, &mut rng), "draw {draw}");
    }
}

// ---- DTI branch flip ----

fn band(id: u16, seed_isolate: bool) -> Requirements<Layout> {
    let b = || {
        vec![DtiBand {
            a: Target::Device(DeviceId(0)),
            b: Target::Device(DeviceId(1)),
            s_max_nm: 200,
            d_dti_nm: 2_000,
            branch: BranchId(id),
            seed_isolate,
        }]
    };
    Requirements { hard: vec![Box::new(b())], budget: Vec::new(), cost: vec![Box::new(b())] }
}

fn gap_bench(gap: i32, branch: Vec<bool>) -> Layout {
    let mut l = layout(&[(0, 0, 1_000, 1_000), (2_000 + gap, 0, 1_000, 1_000)]);
    l.branch = branch;
    l
}

/// Abutting pair committed to `isolate`: the flip to `share` strictly lowers
/// PEX and is taken; the reverse is refused.
/// Oracle: the PEX of share against isolate on an abutting pair, ordered by hand.
#[test]
fn branch_flip_fires_and_is_priced() {
    let reqs = band(0, true);
    let mut l = gap_bench(100, vec![true]);
    let prices = crate::gp::Prices::new();
    let mut sa = Sa::new(Nets::from_macros(&[]), 2, &reqs, &prices, &[], &rules(0));
    let mut rng = SplitMix64::new(1);
    assert!(try_branch(&mut sa, &mut l, &mut rng, 0.0, 0));
    assert!(!l.branch[0]);
    assert!(!try_branch(&mut sa, &mut l, &mut rng, 0.0, 0));
    assert!(!l.branch[0]);
}

/// Both cells pinned, so the returned branch table is exactly what seeding wrote.
/// Oracle: the input's band names branch 3, so exactly that row is set.
#[test]
fn seeds_are_written_and_table_resized() {
    let reqs = band(3, true);
    let mut b = Vec::new();
    assert_eq!(seed_branches(&reqs, &mut b), vec![BranchId(3)]);
    assert_eq!(b, vec![false, false, false, true]);
    let mut b = vec![false; 6];
    seed_branches(&reqs, &mut b);
    assert_eq!(b, vec![false, false, false, true, false, false]);
}

/// `encroach_moved`'s before/after difference must equal the full scan's for
/// every one- and two-cell move, or the gate compares the wrong numbers.
/// Oracle: differential: `encroachment`'s full scan against `encroach_moved`.
#[test]
fn incident_encroachment_difference_matches_full_scan() {
    let cells = [(0, 0, 100, 100), (150, 0, 100, 100), (60, 60, 100, 100), (900, 900, 50, 50), (120, 30, 80, 40)];
    let reqs = Requirements::default();
    let prices = crate::gp::Prices::new();
    for c in 0..cells.len() {
        for o in 0..cells.len() {
            let mut sa = Sa::new(Nets::from_macros(&[]), cells.len(), &reqs, &prices, &[], &rules(300));
            sa.moved = if c == o { vec![c] } else { vec![c, o] };
            let mut l = layout(&cells);
            let (full0, inc0) = (encroachment(&l, 300), sa.encroach_moved(&l));
            for &m in &sa.moved {
                l.x[m] += 37;
                l.y[m] -= 11;
            }
            let full = encroachment(&l, 300) - full0;
            let inc = sa.encroach_moved(&l) - inc0;
            assert!((full - inc).abs() < 1e-6, "moved {:?}: {full} vs {inc}", sa.moved);
        }
    }
}

/// Each compound move keeps every mirror equation of a stage exactly, with no
/// projection: Φ never rises, so `trial` never reaches for `project`.
/// Oracle: the analog `SymmetryGroup` rule after every compound move.
#[test]
fn compound_moves_keep_a_mirrored_stage_mirrored() {
    let sym = |a: u16, b: u16| Symmetry { a: Target::Device(DeviceId(a)), b: Target::Device(DeviceId(b)), axis: AxisId(0), mode: SymMode::Perfect };
    let reqs = Requirements { hard: vec![Box::new(SymmetryGroup(vec![sym(0, 1), sym(2, 3), sym(4, 4)]))], budget: Vec::new(), cost: Vec::new() };
    let mut l = layout(&[
        (-6_000, 0, 1_000, 1_000),
        (6_000, 0, 1_000, 1_000),
        (-6_000, 8_000, 1_000, 1_000),
        (6_000, 8_000, 1_000, 1_000),
        (0, -8_000, 1_000, 1_000),
    ]);
    l.axis = vec![0];
    assert_eq!(analog_violations(&reqs, &l), 0);
    let groups = sym_groups(&reqs, 5);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].members.len(), 5);

    let prices = crate::gp::Prices::new();
    let mut sa = Sa::new(Nets::from_macros(&[]), 5, &reqs, &prices, &[], &rules(CLEARANCE_NM));
    let mut rng = SplitMix64::new(5);
    let free = |c: i32, _: i32| c;
    // Metropolis at huge temperature takes every tie, so each move applies.
    assert!(try_group_shift(&mut sa, &mut l, &mut rng, 1e12, &groups[0], 2_000, -1_000, &free, &free));
    assert_eq!((l.axis[0], l.x[4]), (2_000, 2_000));
    assert!(try_pair_expand(&mut sa, &mut l, &mut rng, 1e12, 0, 1, 1_000, &free));
    assert_eq!((l.x[0], l.x[1]), (-5_000, 9_000));
    assert!(try_pair_swap(&mut sa, &mut l, &mut rng, 1e12, 2, 3));
    assert_eq!(analog_violations(&reqs, &l), 0, "x = {:?}, y = {:?}, axis = {:?}", l.x, l.y, l.axis);
}
