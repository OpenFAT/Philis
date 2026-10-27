//! Realised route geometry — the state routing rules score against.
//! In → out: per-net shapes and the cut joins → connectivity (open pieces) and
//! drawn length. Cites: Philis 1.0 `pnr_core` (Q36).
//!
//! Contents:
//! - `pub type Join`
//! - `pub struct Terminal`
//! - `pub struct GatePin`
//! - `pub struct Routes`
//! - `pub fn conductor_layers_meet(a: &Shape, b: &Shape, joins: &[Join]) -> bool`
//! - `pub fn open_components(shapes: &[Shape], joins: &[Join]) -> usize`
//! - `Routes::shapes(&self, net: NetId) -> &[Shape]`
//! - `Routes::cell_metal(&self, net: NetId) -> &[Shape]`
//! - `Routes::gate_pins(&self, net: NetId) -> &[GatePin]`
//! - `Routes::terminals(&self, net: NetId) -> &[Terminal]`
//! - `Routes::debug_check_joined(&self, ctx: &str, joins: &[Join])`
//! - `Routes::length(&self, net: NetId) -> i64`

use crate::base::geom::{LayerId, Rect, Shape};
use crate::base::ids::NetId;

/// `(cut, layer below, layer above)`: a cut and the two conductors it joins.
pub type Join = (LayerId, LayerId, LayerId);

/// A routed terminal and the DC current it draws from its net, µA
/// (+ into the device); `None` = unknown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Terminal {
    /// The pin's drawn rect, absolute nm.
    pub at: Rect,
    /// DC current, µA, positive into the device; `None` = unknown.
    pub ua: Option<f32>,
}

/// A gate pin, the device it gates and that device's gate-oxide area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatePin {
    /// The gate pin's drawn rect, absolute nm.
    pub at: Rect,
    /// Schematic device id; `u32::MAX` = unknown.
    pub dev: u32,
    /// `W·L·m` of `dev`, nm²; `0` = unknown.
    pub nm2: i64,
}

/// Every net's realised conductor, each table indexed by [`NetId`] and sized
/// by what the router realised (shorter than the netlist is legal: the
/// accessors read a missing net as empty).
#[derive(Default, Clone)]
pub struct Routes {
    /// Drawn wire/via shapes per net, by [`NetId`].
    pub wires: Vec<Vec<Shape>>,
    /// Per net, the placed cells' metal its pins reach: drawn by the cells,
    /// not the router, and scored with the wires where a rule reads the whole
    /// conductor (antenna).
    pub cell: Vec<Vec<Shape>>,
    /// Per net, its gate pins: where the conductor meets a gate, and the
    /// gate oxide behind it (a piece is charged only the gates it reaches).
    pub gates: Vec<Vec<GatePin>>,
    /// Per net, every pin `dr` routed to, absolute, with its DC current
    /// (the electromigration rule's current sources and sinks).
    pub terms: Vec<Vec<Terminal>>,
}

/// Two touching shapes conduct into each other: same layer, or one is a cut
/// joining the other's layer. Metals on non-adjacent layers that overlap in xy
/// do not.
#[must_use]
pub fn conductor_layers_meet(a: &Shape, b: &Shape, joins: &[Join]) -> bool {
    let joined = |cut: LayerId, other: LayerId| joins.iter().any(|&(c, lo, hi)| c == cut && (other == lo || other == hi));
    a.layer == b.layer || joined(a.layer, b.layer) || joined(b.layer, a.layer)
}

/// Connected components of `shapes` past the first (`0` = one conductor, or
/// none): two shapes connect iff they touch in xy ([`crate::base::Rect::touches`])
/// and [`conductor_layers_meet`]. O(k²).
#[must_use]
pub fn open_components(shapes: &[Shape], joins: &[Join]) -> usize {
    let mut seen = vec![false; shapes.len()];
    let mut parts = 0usize;
    for root in 0..shapes.len() {
        if seen[root] {
            continue;
        }
        parts += 1;
        seen[root] = true;
        let mut stack = vec![root];
        while let Some(a) = stack.pop() {
            for b in 0..shapes.len() {
                if !seen[b] && shapes[a].rect.touches(&shapes[b].rect) && conductor_layers_meet(&shapes[a], &shapes[b], joins) {
                    seen[b] = true;
                    stack.push(b);
                }
            }
        }
    }
    parts.saturating_sub(1)
}

impl Routes {
    /// Shapes of `net`; empty when unrouted or out of range (rules carry netlist
    /// ids, `wires` is sized by what the router realised).
    #[inline]
    #[must_use]
    pub fn shapes(&self, net: NetId) -> &[Shape] {
        self.wires.get(net.0 as usize).map_or(&[], Vec::as_slice)
    }

    /// [`Routes::cell`] of `net`; empty when unknown.
    #[must_use]
    pub fn cell_metal(&self, net: NetId) -> &[Shape] {
        self.cell.get(net.0 as usize).map_or(&[], Vec::as_slice)
    }

    /// [`Routes::gates`] of `net`; empty when unknown.
    #[must_use]
    pub fn gate_pins(&self, net: NetId) -> &[GatePin] {
        self.gates.get(net.0 as usize).map_or(&[], Vec::as_slice)
    }

    /// [`Routes::terms`] of `net`; empty when unknown.
    #[must_use]
    pub fn terminals(&self, net: NetId) -> &[Terminal] {
        self.terms.get(net.0 as usize).map_or(&[], Vec::as_slice)
    }

    /// Debug-only stage-boundary check: no degenerate shapes, and each net is
    /// one conductor ([`open_components`] `== 0` under `joins`). An open here is
    /// an LVS `unconnected_pin` later. O(k²) per net.
    ///
    /// # Panics
    /// In a debug build, on a degenerate shape or an open net.
    #[inline]
    pub fn debug_check_joined(&self, ctx: &str, joins: &[Join]) {
        if !cfg!(debug_assertions) {
            return;
        }
        for (net, shapes) in self.wires.iter().enumerate() {
            for (i, s) in shapes.iter().enumerate() {
                assert!(s.rect.w > 0, "{ctx}: net {net} shape {i} has no width: {:?}", s.rect);
                assert!(s.rect.h > 0, "{ctx}: net {net} shape {i} has no height: {:?}", s.rect);
            }
            let open = open_components(shapes, joins);
            assert_eq!(open, 0, "{ctx}: net {net} is open — {} pieces", open + 1);
        }
    }

    /// Drawn metal length of `net`, nm: Σ of each shape's long side.
    #[must_use]
    pub fn length(&self, net: NetId) -> i64 {
        self.shapes(net).iter().map(|s| i64::from(s.rect.w.max(s.rect.h))).sum()
    }
}
