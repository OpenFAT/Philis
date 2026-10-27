//! Oracles: a mirror pair's shapes against their exact reflection (geometry,
//! not the router's own stats); a star's branch R from `Stack`'s port graph.

use super::*;
use crate::dr::DetailedCfg;
use analog::base::Pin;
use analog::routing::Differential;

const LAYERS: [LayerId; 2] = [LayerId(0), LayerId(1)];
const CUTS: [Cut; 1] = [(LayerId(2), 100, 140, 140)];
const METALS: Metals = Metals { layers: &LAYERS, cuts: &CUTS };
/// The pair's axis, a vertical-track centre at p0 = 420.
const AXIS: i32 = 10_290;

/// sky130-like metals joined by layer 2's cut.
fn stack() -> &'static analog::routing::Stack {
    use analog::routing::stack::Layer;
    Box::leak(Box::new(analog::routing::Stack {
        layers: [0, 2, 1].map(|id| Layer { id, cut: id == 2, sheet_ohm: if id == 2 { 4.5 } else { 0.125 }, ..Layer::default() }).to_vec(),
        antenna_cumulative: false,
        diode: None,
    }))
}

fn router() -> DetailedRoute {
    DetailedRoute { cfg: DetailedCfg { pitch: 420, grid: 5, wire_width: 260, stack: Some(stack()), ..DetailedCfg::default() } }
}

fn at(x: i32, y: i32) -> Rect {
    Rect { x, y, w: 170, h: 170 }
}

/// A tile: one metal-0 pad per pin, `bbox` around them.
fn tile(pins: &[(u16, Rect)]) -> Macro {
    Macro {
        shapes: pins.iter().map(|&(_, rect)| Shape { layer: LAYERS[0], rect }).collect(),
        pins: pins.iter().map(|&(n, rect)| Pin { name: format!("d0:{n}"), net: NetId(n), at: rect, layer: LAYERS[0] }).collect(),
        bbox: pins.iter().map(|p| p.1).reduce(bbox).expect("pins"),
        ..Macro::default()
    }
}

fn opens_or_shorts(r: &Report) -> Vec<&String> {
    r.hard_violations.iter().map(|v| &v.rule).filter(|r| r.starts_with("open net") || r.contains("short")).collect()
}

/// Two tiles mirrored about [`AXIS`]: net 0's pins in the left one and a
/// loose third, net 1's their images (its second pin `off` nm off), under a
/// `Differential`; net 2 crosses the floorplan on the axis, and an obstacle
/// sits on net 0's side only.
fn two_tiles(off: i32) -> (Routes, Report, Vec<RequestRow>, Prerouted) {
    let a = [(6_000, 3_000), (8_000, 9_000), (5_000, 12_000)];
    let image = |i: usize| at(2 * AXIS - a[i].0 - 170 + if i == 1 { off } else { 0 }, a[i].1);
    let left = tile(&[(0, at(a[0].0, a[0].1)), (0, at(a[1].0, a[1].1))]);
    let right = tile(&[(1, image(0)), (1, image(1))]);
    let wall = Macro {
        shapes: LAYERS.map(|layer| Shape { layer, rect: Rect { x: 5_500, y: 5_500, w: 3_500, h: 1_000 } }).to_vec(),
        bbox: Rect { x: 5_500, y: 5_500, w: 3_500, h: 1_000 },
        ..Macro::default()
    };
    let pins = [
        (NetId(0), at(a[2].0, a[2].1), LAYERS[0]),
        (NetId(1), image(2), LAYERS[0]),
        (NetId(2), at(AXIS - 85, 500), LAYERS[0]),
        (NetId(2), at(AXIS - 85, 15_000), LAYERS[0]),
    ];
    let mut reqs = Requirements::<Routes>::default();
    reqs.budget.push(Box::new(vec![Differential {
        pos: NetId(0),
        neg: NetId(1),
        max_len_delta_pct10: 0,
        same_layer_required: true,
        stack: Some(stack()),
        aggressor_weight: None,
    }]));
    let requests = from_rules(&reqs);
    assert_eq!(requests.len(), 1);
    let (dr, placed) = (router(), [left, right, wall]);
    let pre = route_requests(&dr, &pins, &placed, &reqs, &requests, METALS, 2);
    let (routes, report, _) = route_rest(&dr, &pins, &placed, &reqs, &requests, &pre, METALS, 2);
    let rows = rows(&dr, &requests, &pre, &routes, METALS);
    (routes, report, rows, pre)
}

/// The pair routes first and comes out mirror-exact about the axis; the
/// crossing net routes after it, around it, and leaves it as drawn.
#[test]
fn a_two_tile_pair_routes_mirror_exact_first() {
    let (routes, report, rows, pre) = two_tiles(0);
    assert_eq!(pre.nets, [NetId(0), NetId(1)]);
    assert_eq!(pre.stats.pairs_exact.len(), 1, "{:?}", pre.stats.pairs_fallback);
    assert_eq!(rows, [RequestRow { request: 0, kind: RequestKind::MirrorPair, met: true, measured: 0.0, limit: 0.0 }]);
    assert_eq!(unmirrored(&routes, MirrorPair { a: NetId(0), b: NetId(1), axis_x2: Some(2 * AXIS) }), 0);
    assert!(opens_or_shorts(&report).is_empty(), "{:?}", report.hard_violations.iter().map(|v| &v.rule).collect::<Vec<_>>());
    assert_eq!(routes.wires[0], pre.routes.wires[0]);
    assert_eq!(routes.wires[1], pre.routes.wires[1]);
    assert!(pre.routes.wires.get(2).is_none_or(Vec::is_empty), "net 2 waits for the rest");
    for y in [500, 15_000] {
        assert!(routes.wires[2].iter().any(|s| s.rect.touches(&at(AXIS - 85, y))), "net 2 pin at y {y} unrouted");
    }
}

/// Negative control: one of net 1's pins 5 nm off the image. No exact map
/// exists, the row reports the pair violated, and the report carries it.
#[test]
fn asymmetric_pins_report_the_pair_violated() {
    let (_, _, rows, pre) = two_tiles(5);
    assert!(pre.stats.pairs_exact.is_empty());
    assert_eq!(rows.len(), 1);
    assert!(!rows[0].met && rows[0].measured > 0.0, "{rows:?}");
    assert_eq!(violations(&rows).len(), 1);
}

/// A common node fed from the left between two members mirrored about the
/// feed's row: the branches' R from the feed agree within the tolerance.
/// Negative control: a tolerance no drawn route meets reads violated.
///
/// ponytail: `star: true` (branches meet only at the root) routes as a star
/// but `dr`'s balance repair does not run on it (ΔR 4.3 Ω here): v2.
#[test]
fn a_common_node_routes_equal_branch_r() {
    let (feed, a, b) = (at(4_000, 8_000), at(12_000, 2_000), at(12_000, 14_000));
    let run = |max_delta_ohm| {
        let node = CommonNode { net: NetId(0), groups: vec![vec![a], vec![b]], feeds: vec![feed], max_delta_ohm, star: false };
        let pins = [feed, a, b].map(|r| (NetId(0), r, LAYERS[0]));
        let (_, report, rows) = route(&router(), &pins, &[], &Requirements::default(), &[RoutingRequest::Star(node)], METALS, 2);
        assert!(opens_or_shorts(&report).is_empty(), "{:?}", report.hard_violations.iter().map(|v| &v.rule).collect::<Vec<_>>());
        assert_eq!(rows.len(), 1);
        rows[0]
    };
    let row = run(0.2);
    assert!(row.met && row.measured <= 0.2, "{row:?}");
    let row = run(1e-3);
    assert!(!row.met && row.measured > 1e-3, "{row:?}");
}
