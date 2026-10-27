//! Seeded violations (`docs/TESTING.md`, "geometry violates a claim"): per
//! [`analog::RuleKind`] a claim can name, one layout that meets the rule and one that
//! violates it by construction. Oracle: the construction itself (an ABBA row
//! is common-centroid, a well 1 µm from one member is nearer than the class
//! clearance); the rule must agree with it, from geometry alone.

use std::sync::Arc;

use analog::base::{AxisId, DeviceId, Layout, Orient, Rect, Target, Unit, UnitLib};
use analog::matching::class::{Family, MatchClass};
use analog::matching::mismatch::{Budget, Coeffs, MatchKind};
use analog::matching::moments::{cancelled_order, Pt};
use analog::placement::symmetry::SymmetryGroup;
use analog::placement::{EnvGeo, LiveEnvironment, MatchedSet, OrientCheck, OrientationSet, SymMode, Symmetry};
use analog::RuleBatch;

// ---- private ----

/// Cells at `xs` (y 0), each `2·half` square, drawing `cells[c]`'s units for
/// devices `members[c]`.
fn layout(xs: &[i32], half: i32, members: &[Vec<DeviceId>], cells: &[Vec<Unit>]) -> Layout {
    let n = xs.len();
    let mut cell_of = vec![0u16; members.iter().flatten().map(|d| usize::from(d.0) + 1).max().unwrap_or(0)];
    for (c, m) in members.iter().enumerate() {
        m.iter().for_each(|d| cell_of[usize::from(d.0)] = c as u16);
    }
    let bbox = Rect { x: 0, y: 0, w: 2 * half, h: 2 * half };
    let alts: Vec<[(Rect, &[Unit]); 1]> = cells.iter().map(|u| [(bbox, &u[..])]).collect();
    let units = UnitLib::build(cell_of, members, alts.iter().map(|a| &a[..]));
    Layout {
        x: xs.to_vec(),
        y: vec![0; n],
        hw: vec![half; n],
        hh: vec![half; n],
        orient: vec![Orient::R0; n],
        variant: vec![0; n],
        axis: vec![0],
        branch: Vec::new(),
        groups: Vec::new(),
        power_uw: vec![0; n],
        temp_mc: vec![0; n],
        units: Arc::new(units),
    }
}

fn unit(owner: u8, x: i32, y: i32, phi: (i8, i8)) -> Unit {
    Unit { owner, x, y, weight: 1_000_000, phi, sa: 0, sb: 0 }
}

/// Devices 0 and 1 merged in one 8 µm cell, one unit per letter of `order`
/// (rows of four, 1 µm pitch), every channel along x.
fn merged(order: &[[u8; 4]]) -> Layout {
    let units = order
        .iter()
        .enumerate()
        .flat_map(|(r, row)| row.iter().zip([500, 1_500, 2_500, 3_500]).map(move |(&o, x)| unit(o, x, 500 + 1_000 * r as i32, (1, 0))));
    layout(&[4_000], 4_000, &[vec![DeviceId(0), DeviceId(1)]], &[units.collect()])
}

/// Oracle: the construction, ABBA is common-centroid and AABB is 400 tolerances off.
#[test]
fn matched_set_coincidence() {
    let set = MatchedSet::for_family(
        vec![DeviceId(0), DeviceId(1)],
        Family::Mos,
        MatchKind::Current,
        MatchClass::Moderate,
        Coeffs { avt_mv_um: Some(9.5), svt_uv_per_um: Some(1.63), ..Coeffs::default() },
        Budget::Eta(0.3),
        vec![2.0, 2.0],
        5.0,
    );
    let abba = set.ledger(&merged(&[[0, 1, 1, 0]]), 1);
    assert_eq!(abba.coincidence, Some(0.0));
    assert_eq!(set.violations(&merged(&[[0, 1, 1, 0]])), 0);
    // AABB: centroids 2 µm apart, 400 × the 5 nm tolerance.
    let aabb = set.ledger(&merged(&[[0, 0, 1, 1]]), 1);
    assert_eq!(aabb.coincidence, Some(400.0));
    assert_eq!(set.violations(&merged(&[[0, 0, 1, 1]])), 1);
}

/// Oracle: the construction, ±2000 mirrors about 0 and 2040 does not.
#[test]
fn symmetry_mirror_about_the_axis() {
    let rules =
        SymmetryGroup(vec![Symmetry { a: Target::Device(DeviceId(0)), b: Target::Device(DeviceId(1)), axis: AxisId(0), mode: SymMode::Perfect }]);
    let empty = || vec![Vec::new(), Vec::new()];
    let members = [vec![DeviceId(0)], vec![DeviceId(1)]];
    // Axis at x = 0: cells at ∓2 µm mirror, a cell shifted 40 nm does not.
    assert_eq!(rules.violations(&layout(&[-2_000, 2_000], 500, &members, &empty())), 0);
    assert_eq!(rules.violations(&layout(&[-2_000, 2_040], 500, &members, &empty())), 1);
}

/// Oracle: the construction, ABBA cancels order 1 and AABB none.
#[test]
fn centroid_group_moments_through_order_two() {
    let pts = |l: &Layout, d: u16| l.units.of_device(l, DeviceId(d)).map(Pt::from).collect::<Vec<_>>();
    let order = |l: &Layout| cancelled_order(&[&pts(l, 0), &pts(l, 1)], 2, 1e-3).0;
    // ABBA over BAAB cancels orders 1 and 2; ABBA alone only order 1; AABB none.
    assert_eq!(order(&merged(&[[0, 1, 1, 0], [1, 0, 0, 1]])), 2);
    assert_eq!(order(&merged(&[[0, 1, 1, 0]])), 1);
    assert_eq!(order(&merged(&[[0, 0, 1, 1]])), 0);
}

/// Oracle: the construction, a well 1 µm from one member only.
#[test]
fn environment_equal_wpe() {
    // Devices 0 and 1 alone in 1 µm cells at x = 500 and 3500; cell 2 a
    // 2 µm n-well at `well_x`. The Moderate clearance is 3 µm.
    let sq = Rect { x: 0, y: 0, w: 1_000, h: 1_000 };
    let wide = Rect { x: 0, y: 0, w: 2_000, h: 1_000 };
    let env = LiveEnvironment {
        pairs: vec![(DeviceId(0), DeviceId(1), 0, 1)],
        geo: Arc::new(EnvGeo {
            bbox: vec![vec![sq], vec![sq], vec![wide]],
            wells: vec![vec![vec![]], vec![vec![]], vec![vec![wide]]],
            diffs: vec![vec![vec![sq]], vec![vec![sq]], vec![vec![]]],
        }),
        wpe_min_nm: 3_000.0,
        ose_range_nm: 0.0,
    };
    let at = |well_x: i32| {
        let mut l = layout(
            &[500, 3_500, well_x],
            500,
            &[vec![DeviceId(0)], vec![DeviceId(1)], vec![]],
            &[vec![unit(0, 500, 500, (1, 0))], vec![unit(0, 500, 500, (1, 0))], vec![]],
        );
        l.hw[2] = 1_000;
        l
    };
    assert_eq!(env.violations(&at(41_000)), 0, "a well 36 µm away");
    assert_eq!(env.unknown(&at(41_000)), 0);
    assert_eq!(env.violations(&at(5_500)), 1, "a well 1 µm from one member only");
}

/// Oracle: the construction, a partner turned or reflected by hand.
#[test]
fn orientation_set_axis_and_phi() {
    let pair = |phi1: (i8, i8)| {
        layout(&[0, 10_000], 500, &[vec![DeviceId(0)], vec![DeviceId(1)]], &[vec![unit(0, 500, 500, (1, 0))], vec![unit(0, 500, 500, phi1)]])
    };
    let set = |check| OrientationSet { members: vec![DeviceId(0), DeviceId(1)], check, cell_of: Vec::new() };
    assert_eq!(set(OrientCheck::Phi).violations(&pair((1, 0))), 0);
    assert_eq!(set(OrientCheck::Phi).violations(&pair((-1, 0))), 1, "member 1 mirrored: Φ flips");
    assert_eq!(set(OrientCheck::Axis).violations(&pair((-1, 0))), 0, "a flip keeps the channels parallel");
    assert_eq!(set(OrientCheck::Axis).violations(&pair((0, 1))), 1, "member 1 turned: channels cross");
    // No units: unknown, never met.
    let bare = layout(&[0, 10_000], 500, &[vec![DeviceId(0)], vec![DeviceId(1)]], &[vec![], vec![]]);
    assert_eq!((set(OrientCheck::Phi).violations(&bare), set(OrientCheck::Phi).unknown(&bare)), (0, 1));
}
