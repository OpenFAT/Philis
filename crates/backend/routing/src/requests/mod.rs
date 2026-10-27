//! Routing recipes (Q23, `docs/RECIPES.md` §4): the request table recipes
//! write and this crate drains. [`route`] is [`route_requests`] — the
//! constrained nets alone, by construction — then [`route_rest`], `dr`'s
//! negotiation of every other net around that geometry as a fixed obstacle.
//! One router: both halves are [`DetailedRoute::route`]; a mirror pair is its
//! exact-image pairing (RTE-15: one net searched with two-sided legality, the
//! other drawn as its image), a star its common-node balance.

use analog::base::geom::{LayerId, Rect, Shape};
use analog::base::report::Violation;
use analog::base::{Macro, NetId, Report, Routes};
use analog::routing::{CommonNode, CommonNodes};
use analog::{RepairKind, Requirements, RuleBatch};

use crate::dr::{Blockage, Cut, DetailedRoute, RouteStats};
use crate::gr::Negotiation;

#[cfg(test)]
mod tests;

/// One row of the request table (API.md: `RoutingRequest`, Q58).
#[derive(Clone, Debug)]
pub enum RoutingRequest {
    MirrorPair(MirrorPair),
    /// A shared node: branches from `feeds` to each group, equal R within
    /// `max_delta_ohm` (`> 0`), meeting only at the root when `star`.
    Star(CommonNode),
    Shield(Shield),
    /// Region no foreign net crosses (matched gate, resistor body, plate).
    Keepout(Blockage),
}

/// The tag a table that stores only kinds keeps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RequestKind {
    MirrorPair,
    Star,
    Shield,
    Keepout,
}

/// What every request answers.
pub trait Requested {
    /// Append the nets this request routes or protects.
    fn nets(&self, out: &mut Vec<NetId>);
    /// Lower drains first: `0` is routed by [`route_requests`], above it
    /// shapes [`route_rest`].
    fn priority(&self) -> u8;
    fn kind(&self) -> RequestKind;
}

/// Nets `a` and `b` routed as mirror images about the vertical axis
/// `x = axis_x2 / 2` (`None`: the axis their pins mirror about). `dr` ties a
/// pair only under a `Mirror`-repaired rule in `reqs` (`Differential`), which
/// is where [`from_rules`] reads them; a row without one routes unmirrored and
/// reports violated.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MirrorPair {
    pub a: NetId,
    pub b: NetId,
    /// Twice the axis x, nm, so a half-nm axis stays exact.
    pub axis_x2: Option<i32>,
}

/// `reference` metal beside `victim`, no farther than `max_gap_nm`; drawn by
/// `dr`'s shield repair after every other wire (the `Shield` rule in `reqs`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shield {
    pub victim: NetId,
    pub reference: NetId,
    pub max_gap_nm: i32,
}

impl Requested for MirrorPair {
    fn nets(&self, out: &mut Vec<NetId>) {
        out.extend([self.a, self.b]);
    }
    fn priority(&self) -> u8 {
        0
    }
    fn kind(&self) -> RequestKind {
        RequestKind::MirrorPair
    }
}

impl Requested for CommonNode {
    fn nets(&self, out: &mut Vec<NetId>) {
        out.push(self.net);
    }
    fn priority(&self) -> u8 {
        0
    }
    fn kind(&self) -> RequestKind {
        RequestKind::Star
    }
}

impl Requested for Shield {
    fn nets(&self, out: &mut Vec<NetId>) {
        out.extend([self.victim, self.reference]);
    }
    fn priority(&self) -> u8 {
        2
    }
    fn kind(&self) -> RequestKind {
        RequestKind::Shield
    }
}

impl Requested for Blockage {
    fn nets(&self, _: &mut Vec<NetId>) {}
    fn priority(&self) -> u8 {
        1
    }
    fn kind(&self) -> RequestKind {
        RequestKind::Keepout
    }
}

impl Requested for RoutingRequest {
    fn nets(&self, out: &mut Vec<NetId>) {
        match self {
            Self::MirrorPair(r) => r.nets(out),
            Self::Star(r) => r.nets(out),
            Self::Shield(r) => r.nets(out),
            Self::Keepout(r) => r.nets(out),
        }
    }
    fn priority(&self) -> u8 {
        match self {
            Self::MirrorPair(r) => r.priority(),
            Self::Star(r) => r.priority(),
            Self::Shield(r) => r.priority(),
            Self::Keepout(r) => r.priority(),
        }
    }
    fn kind(&self) -> RequestKind {
        match self {
            Self::MirrorPair(r) => r.kind(),
            Self::Star(r) => r.kind(),
            Self::Shield(r) => r.kind(),
            Self::Keepout(r) => r.kind(),
        }
    }
}

/// The rows the annotator's routing rules ask for: a [`MirrorPair`] per
/// `Mirror`-repaired pair (`Differential`), a [`Shield`] per shield pair.
/// Stars and keepouts need placed pins and cells: the caller adds them.
#[must_use]
pub fn from_rules(reqs: &Requirements<Routes>) -> Vec<RoutingRequest> {
    let mut out = Vec::new();
    for b in reqs.hard.iter().chain(&reqs.budget) {
        if b.repair_kind() == RepairKind::Mirror {
            let mut ids = Vec::new();
            b.touched(&mut ids);
            for p in ids.chunks_exact(2) {
                out.push(RoutingRequest::MirrorPair(MirrorPair { a: NetId(p[0] as u16), b: NetId(p[1] as u16), axis_x2: None }));
            }
        }
        let mut sh = Vec::new();
        b.shield_pairs(&mut sh);
        out.extend(
            sh.into_iter().map(|(v, r, gap)| RoutingRequest::Shield(Shield { victim: NetId(v as u16), reference: NetId(r as u16), max_gap_nm: gap })),
        );
    }
    out
}

/// A drained request's verdict: `measured` against `limit`. A mirror pair
/// measures the shapes without an exact image (limit 0); a star its branches'
/// ΔR, Ω (limit `max_delta_ohm`; `NaN` = no balance asked, the star topology
/// alone decides). Shields and keepouts are scored by their rules in the
/// report, not here.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RequestRow {
    /// Index into the request table.
    pub request: u16,
    pub kind: RequestKind,
    pub met: bool,
    pub measured: f32,
    pub limit: f32,
}

/// What [`route_requests`] drew: the constrained nets' routes, which
/// [`route_rest`] keeps as fixed geometry.
pub struct Prerouted {
    pub nets: Vec<NetId>,
    pub routes: Routes,
    pub report: Report,
    pub stats: RouteStats,
}

/// The routing stack a drain runs over (`route`'s `layers`, `cuts`).
#[derive(Clone, Copy)]
pub struct Metals<'a> {
    pub layers: &'a [LayerId],
    pub cuts: &'a [Cut],
}

/// The requests that apply: every net of a priority-0 row has a pin to route
/// (in `pins` or on a macro of `placed`).
fn routed_first(requests: &[RoutingRequest], pins: &[(NetId, Rect, LayerId)], placed: &[Macro]) -> Vec<NetId> {
    let pinned = |x: NetId| pins.iter().any(|p| p.0 == x) || placed.iter().flat_map(|m| &m.pins).any(|p| p.net == x);
    let mut nets = Vec::new();
    for r in requests.iter().filter(|r| r.priority() == 0) {
        let mut n = Vec::new();
        r.nets(&mut n);
        if n.iter().all(|&x| pinned(x)) {
            nets.extend(n);
        }
    }
    nets.sort_unstable_by_key(|n| n.0);
    nets.dedup();
    nets
}

/// `placed` with only the pins whose net `keep` passes (`dr` routes the
/// macros' pins as well as `pins`).
fn with_pins(placed: &[Macro], keep: impl Fn(NetId) -> bool) -> Vec<Macro> {
    placed.iter().map(|m| Macro { pins: m.pins.iter().filter(|p| keep(p.net)).cloned().collect(), ..m.clone() }).collect()
}

/// `dr` with the requests' stars as its common nodes and keepouts as blockages.
fn router(dr: &DetailedRoute, requests: &[RoutingRequest]) -> DetailedRoute {
    let mut cfg = dr.cfg.clone();
    for r in requests {
        match r {
            RoutingRequest::Star(n) => cfg.common.push(n.clone()),
            RoutingRequest::Keepout(b) => cfg.blockages.push(*b),
            RoutingRequest::MirrorPair(_) | RoutingRequest::Shield(_) => {}
        }
    }
    DetailedRoute { cfg }
}

/// The best of `epochs` routes (lexicographic report), negotiation history
/// carried between them.
fn best_of(epochs: u32, mut once: impl FnMut(&mut Negotiation) -> (Routes, Report, RouteStats)) -> (Routes, Report, RouteStats) {
    assert!(epochs > 0);
    let mut neg = Negotiation::new();
    let mut best: Option<(Routes, Report, RouteStats)> = None;
    for _ in 0..epochs {
        let r = once(&mut neg);
        if best.as_ref().is_none_or(|b| r.1.lex() < b.1.lex()) {
            best = Some(r);
        }
    }
    best.expect("epochs > 0")
}

/// Route the priority-0 requests' nets alone — mirror pairs as exact images,
/// stars balanced from their feeds — before any other net exists. A request
/// with a net that has no pin to route (whole inside its tile) is not drained.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn route_requests(
    dr: &DetailedRoute,
    pins: &[(NetId, Rect, LayerId)],
    placed: &[Macro],
    reqs: &Requirements<Routes>,
    requests: &[RoutingRequest],
    metals: Metals,
    epochs: u32,
) -> Prerouted {
    let nets = routed_first(requests, pins, placed);
    // Every other pin rides along as a one-pin net of its own (ids past every
    // real net): `dr` lands it and keeps its access jog clear, routes nothing
    // to it, and the first pass's metal leaves room for the rest to land.
    let base = pins.iter().map(|p| p.0).chain(placed.iter().flat_map(|m| m.pins.iter().map(|p| p.net))).map(|n| n.0 as usize + 1).max().unwrap_or(0);
    let base = base.max(dr.cfg.n_nets);
    let mut alone: Vec<Rect> = Vec::new();
    let mut relabel = |n: NetId, at: Rect| {
        if nets.contains(&n) {
            return n;
        }
        let k = alone.iter().position(|&r| r == at).unwrap_or_else(|| {
            alone.push(at);
            alone.len() - 1
        });
        NetId(u16::try_from(base + k).expect("nets within u16"))
    };
    let mine: Vec<_> = pins.iter().map(|&(n, at, l)| (relabel(n, at), at, l)).collect();
    let placed: Vec<Macro> = placed
        .iter()
        .map(|m| Macro { pins: m.pins.iter().map(|p| analog::base::Pin { net: relabel(p.net, p.at), ..p.clone() }).collect(), ..m.clone() })
        .collect();
    let dr = router(dr, requests);
    let (mut routes, report, stats) =
        best_of(
            epochs,
            |neg| if nets.is_empty() { Default::default() } else { dr.route(&mine, &placed, &[], reqs, metals.layers, metals.cuts, neg) },
        );
    for v in [&mut routes.wires, &mut routes.cell] {
        v.truncate(base);
    }
    routes.gates.truncate(base);
    routes.terms.truncate(base);
    Prerouted { nets, routes, report, stats }
}

/// Route every net [`route_requests`] did not, around its routes as fixed
/// metal, and merge: the constrained nets' shapes as drawn first, the report's
/// router rows from both passes and its rule tiers rescored on the whole.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn route_rest(
    dr: &DetailedRoute,
    pins: &[(NetId, Rect, LayerId)],
    placed: &[Macro],
    reqs: &Requirements<Routes>,
    requests: &[RoutingRequest],
    pre: &Prerouted,
    metals: Metals,
    epochs: u32,
) -> (Routes, Report, RouteStats) {
    let rest: Vec<_> = pins.iter().copied().filter(|p| !pre.nets.contains(&p.0)).collect();
    let placed = with_pins(placed, |n| !pre.nets.contains(&n));
    // The first pass's metal, pinless: unattributed, so every routed net
    // keeps its spacing from it (`dr`'s fixed metal).
    let shapes: Vec<Shape> = pre.nets.iter().flat_map(|&n| pre.routes.shapes(n).iter().copied()).collect();
    let fixed = shapes.iter().map(|s| s.rect).reduce(bbox).map(|bbox| Macro { shapes, bbox, ..Macro::default() });
    let dr = router(dr, requests);
    let (mut routes, report, stats) = best_of(epochs, |neg| dr.route(&rest, &placed, fixed.as_slice(), reqs, metals.layers, metals.cuts, neg));
    let n = routes.wires.len().max(pre.routes.wires.len());
    for v in [&mut routes.wires, &mut routes.cell] {
        v.resize(n, Vec::new());
    }
    routes.gates.resize(n, Vec::new());
    routes.terms.resize(n, Vec::new());
    for &net in &pre.nets {
        let i = net.0 as usize;
        routes.wires[i] = pre.routes.wires.get(i).cloned().unwrap_or_default();
        routes.cell[i] = pre.routes.cell.get(i).cloned().unwrap_or_default();
        routes.gates[i] = pre.routes.gates.get(i).cloned().unwrap_or_default();
        routes.terms[i] = pre.routes.terms.get(i).cloned().unwrap_or_default();
    }
    let (hard, budget) = crate::gr::analog_tiers(&routes, reqs);
    let first = pre.report.hard_violations.iter().map(|v| Violation { rule: v.rule.clone(), margin: v.margin });
    let report = Report {
        hard_violations: first.chain(report.hard_violations).filter(|v| !v.is_batch_row()).chain(hard).collect(),
        budget_violations: budget,
        cost: pre.report.cost + report.cost,
    };
    (routes, report, stats)
}

fn bbox(a: Rect, b: Rect) -> Rect {
    let (x, y) = (a.x.min(b.x), a.y.min(b.y));
    Rect { x, y, w: (a.x + a.w).max(b.x + b.w) - x, h: (a.y + a.h).max(b.y + b.h) - y }
}

/// [`route_requests`] then [`route_rest`], each the best of `epochs`, and a
/// [`RequestRow`] per drained mirror pair and star.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn route(
    dr: &DetailedRoute,
    pins: &[(NetId, Rect, LayerId)],
    placed: &[Macro],
    reqs: &Requirements<Routes>,
    requests: &[RoutingRequest],
    metals: Metals,
    epochs: u32,
) -> (Routes, Report, Vec<RequestRow>) {
    let pre = route_requests(dr, pins, placed, reqs, requests, metals, epochs);
    let (routes, report, _) = route_rest(dr, pins, placed, reqs, requests, &pre, metals, epochs);
    let rows = rows(dr, requests, &pre, &routes, metals);
    (routes, report, rows)
}

/// Each drained mirror pair's and star's verdict on `routes`.
#[must_use]
pub fn rows(dr: &DetailedRoute, requests: &[RoutingRequest], pre: &Prerouted, routes: &Routes, metals: Metals) -> Vec<RequestRow> {
    let joins = crate::dr::joins(metals.layers, metals.cuts, dr.cfg.pin_access);
    let mut out = Vec::new();
    for (i, r) in requests.iter().enumerate() {
        let mut nets = Vec::new();
        r.nets(&mut nets);
        if r.priority() != 0 || !nets.iter().all(|n| pre.nets.contains(n)) {
            continue;
        }
        let (measured, limit, met) = match r {
            RoutingRequest::MirrorPair(p) => {
                let off = unmirrored(routes, *p) as f32;
                (off, 0.0, off == 0.0)
            }
            RoutingRequest::Star(n) => match dr.cfg.stack {
                Some(st) => {
                    let nodes = CommonNodes { nodes: vec![n.clone()], stack: st, halo_nm: dr.cfg.pitch, joins: joins.clone() };
                    let d = nodes.delta_ohm(n, routes).unwrap_or(f32::INFINITY);
                    let limit = if n.max_delta_ohm > 0.0 { n.max_delta_ohm } else { f32::NAN };
                    (d, limit, d.is_finite() && nodes.violations(routes) == 0)
                }
                None => (f32::NAN, n.max_delta_ohm, false),
            },
            RoutingRequest::Shield(_) | RoutingRequest::Keepout(_) => continue,
        };
        out.push(RequestRow { request: i as u16, kind: r.kind(), met, measured, limit });
    }
    out
}

/// Shapes of either net whose mirror about the pair's axis is no shape of
/// the other (layer and rect exact); every shape when a net is empty. The
/// axis by default is where the pins' outer edges meet: a reflection maps
/// `a`'s leftmost edge onto `b`'s rightmost.
#[must_use]
pub fn unmirrored(routes: &Routes, p: MirrorPair) -> usize {
    let (a, b) = (routes.shapes(p.a), routes.shapes(p.b));
    if a.is_empty() || b.is_empty() {
        return a.len() + b.len();
    }
    let axis_x2 = p.axis_x2.unwrap_or_else(|| {
        let ta = routes.terminals(p.a).iter().map(|t| t.at.x).min();
        let tb = routes.terminals(p.b).iter().map(|t| t.at.x + t.at.w).max();
        ta.zip(tb).map_or(0, |(l, r)| l + r)
    });
    let image = |s: &Shape| Shape { rect: Rect { x: axis_x2 - s.rect.x - s.rect.w, ..s.rect }, ..*s };
    let missing = |from: &[Shape], to: &[Shape]| from.iter().filter(|s| !to.contains(&image(s))).count();
    missing(a, b) + missing(b, a)
}

/// The report rows of the violated requests (budget tier, so a broken
/// mirror ranks an epoch worse without reading as illegal metal).
#[must_use]
pub fn violations(rows: &[RequestRow]) -> Vec<Violation> {
    rows.iter()
        .filter(|r| !r.met)
        .map(|r| {
            let over = if r.limit > 0.0 { f64::from(r.measured / r.limit - 1.0) } else { f64::from(r.measured.max(1.0)) };
            Violation::from_residual(format!("request {} {:?}", r.request, r.kind), over.max(0.001))
        })
        .collect()
}
