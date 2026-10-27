//! Hierarchical symmetric-feasible sequence pair (PLC-08; Balasa §1.3, Plantage
//! Alg. 3.3 for shadows). Every symmetry group and proximity block is a node
//! whose kids are contiguous in its parent's code, so no outsider enters its
//! bbox. [`decode`] turns a code into lattice coordinates with exact pairwise
//! gaps (PLC-07 [`Gap`]) and exact mirror equations, or says why it cannot.
//!
//! Invariant: a node's node kids have smaller indices than it (root last), so
//! index order is a post-order and reverse index order a pre-order.
//!
//! Contents:
//! - `pub use decode::{decode, to_layout, Fail, Geo, Out, Scratch}`
//! - `pub use moves::{m1, m2, m3}`
//! - `pub use verify::verify`
//! - `pub enum Kid`
//! - `pub struct Sym`
//! - `pub struct Node`
//! - `pub struct Tree`
//! - `pub enum TreeError`
//! - `Tree::build(n_cells: usize, pairs: &[(u32, u32, u16)], blocks: &[Vec<DeviceId>]) -> (Tree, Vec<TreeError>)`
//! - `Tree::same_structure(&self, o: &Tree) -> bool`
//! - `Tree::is_sf(&self, node: u16) -> bool`
//! - `Tree::make_sf(&mut self, node: u16)`
//! - `Tree::seed_from(&mut self, l: &Layout)`
//! - `Tree::seed_constructive(&mut self, w: &[i32], h: &[i32])`
//! - `pub(super) fn inverse(seq: &[u16], inv: &mut Vec<u32>)`

mod decode;
mod moves;
mod verify;

pub use decode::{decode, to_layout, Fail, Geo, Out, Scratch};
pub(crate) use moves::{m1, m2, m3};
pub use verify::verify;

use analog::base::ids::AxisId;
use analog::base::{DeviceId, Layout};

/// One kid of a tree node: a placed cell or a child node, by index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kid {
    /// A cell, by cell index.
    Cell(u16),
    /// A child node, by node index (always lower than its parent's).
    Node(u16),
}

/// A symmetry node's axis and involution: `mate[k]` = partner kid slot,
/// `mate[k] == k` = self-symmetric (centred on the axis).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sym {
    /// The axis the node is symmetric about.
    pub axis: AxisId,
    /// `mate[k]`: partner kid slot of slot `k`; an involution.
    pub mate: Vec<u16>,
}

/// `alpha`, `beta`: permutations of kid slots (the node's sequence pair).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// The node's kids, by slot; the code below permutes these slots.
    pub kids: Vec<Kid>,
    /// The positive sequence: a permutation of kid slots.
    pub alpha: Vec<u16>,
    /// The negative sequence: a permutation of kid slots.
    pub beta: Vec<u16>,
    /// `Some` for a symmetry node, whose code must stay symmetric-feasible.
    pub sym: Option<Sym>,
}

/// `home[cell] = (node, kid slot)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    /// Post-order: a node's node kids have lower indices; the root is last.
    pub nodes: Vec<Node>,
    /// Index of the root node (`nodes.len() - 1`).
    pub root: u16,
    /// `home[cell] = (node, kid slot)`.
    pub home: Vec<(u16, u16)>,
}

/// A conflict [`Tree::build`] resolved by dropping input; the tree is still valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeError {
    /// `cell` is mirrored on axis `a` and again on `b`; `b`'s group was dropped.
    CellInTwoAxes {
        /// The cell, by index.
        cell: u16,
        /// The axis the cell kept.
        a: AxisId,
        /// The axis whose group was dropped.
        b: AxisId,
    },
}

impl Tree {
    /// Symmetry nodes from `pairs` (`(a, b, axis)`, `a == b` = self) grouped by
    /// axis in first-appearance order; then one proximity node per block of
    /// `blocks` but the last (the annotator's glue) holding ≥ 2 kids: its whole
    /// symmetry nodes and its remaining cells; the root holds the rest. A cell
    /// named on a second axis drops that axis's whole group and is reported. A
    /// cell paired twice on one axis keeps its first partner. Every node starts
    /// with α = β = slot order, symmetry nodes then [`Tree::make_sf`].
    #[must_use]
    pub fn build(n_cells: usize, pairs: &[(u32, u32, u16)], blocks: &[Vec<DeviceId>]) -> (Tree, Vec<TreeError>) {
        let n = n_cells;
        let ok = |p: &&(u32, u32, u16)| (p.0 as usize) < n && (p.1 as usize) < n;
        let mut axes: Vec<u16> = Vec::new();
        for p in pairs.iter().filter(ok) {
            if !axes.contains(&p.2) {
                axes.push(p.2);
            }
        }
        let mut errs = Vec::new();
        let mut axis_of: Vec<Option<u16>> = vec![None; n];
        let mut sym_of: Vec<Option<u16>> = vec![None; n];
        let mut nodes: Vec<Node> = Vec::new();
        for &ax in &axes {
            let group: Vec<(u16, u16)> = pairs.iter().filter(ok).filter(|p| p.2 == ax).map(|p| (p.0 as u16, p.1 as u16)).collect();
            let mut clash = false;
            for c in group.iter().flat_map(|&(a, b)| [a, b]) {
                if let Some(o) = axis_of[usize::from(c)].filter(|&o| o != ax) {
                    clash = true;
                    let e = TreeError::CellInTwoAxes { cell: c, a: AxisId(o), b: AxisId(ax) };
                    if !errs.contains(&e) {
                        errs.push(e);
                    }
                }
            }
            if clash {
                continue;
            }
            let (mut cells, mut mate): (Vec<u16>, Vec<Option<u16>>) = (Vec::new(), Vec::new());
            let mut slot = |c: u16, mate: &mut Vec<Option<u16>>| {
                cells.iter().position(|&x| x == c).unwrap_or_else(|| {
                    cells.push(c);
                    mate.push(None);
                    cells.len() - 1
                })
            };
            for &(a, b) in &group {
                let (sa, sb) = (slot(a, &mut mate), slot(b, &mut mate));
                if mate[sa].is_none() && mate[sb].is_none() {
                    (mate[sa], mate[sb]) = (Some(sb as u16), Some(sa as u16));
                }
            }
            let id = nodes.len() as u16;
            for &c in &cells {
                axis_of[usize::from(c)] = Some(ax);
                sym_of[usize::from(c)] = Some(id);
            }
            let k = cells.len() as u16;
            nodes.push(Node {
                kids: cells.iter().map(|&c| Kid::Cell(c)).collect(),
                alpha: (0..k).collect(),
                beta: (0..k).collect(),
                sym: Some(Sym { axis: AxisId(ax), mate: mate.iter().enumerate().map(|(s, m)| m.unwrap_or(s as u16)).collect() }),
            });
        }
        let n_sym = nodes.len();
        let mut used = vec![false; n_sym];
        let mut homed = vec![false; n];
        let mut in_blk = vec![false; n];
        for blk in &blocks[..blocks.len().saturating_sub(1)] {
            in_blk.fill(false);
            for d in blk.iter().filter(|d| usize::from(d.0) < n) {
                in_blk[usize::from(d.0)] = true;
            }
            let mut kids = Vec::new();
            for (s, node) in nodes.iter().enumerate().take(n_sym) {
                let all = node.kids.iter().all(|k| matches!(k, Kid::Cell(c) if in_blk[usize::from(*c)]));
                if !used[s] && all {
                    kids.push(Kid::Node(s as u16));
                }
            }
            for c in (0..n).filter(|&c| in_blk[c] && sym_of[c].is_none() && !homed[c]) {
                kids.push(Kid::Cell(c as u16));
            }
            if kids.len() < 2 {
                continue;
            }
            for k in &kids {
                match *k {
                    Kid::Node(s) => used[usize::from(s)] = true,
                    Kid::Cell(c) => homed[usize::from(c)] = true,
                }
            }
            let k = kids.len() as u16;
            nodes.push(Node { kids, alpha: (0..k).collect(), beta: (0..k).collect(), sym: None });
        }
        let mut kids: Vec<Kid> = (n_sym..nodes.len()).map(|i| Kid::Node(i as u16)).collect();
        kids.extend((0..n_sym).filter(|&s| !used[s]).map(|s| Kid::Node(s as u16)));
        kids.extend((0..n).filter(|&c| sym_of[c].is_none() && !homed[c]).map(|c| Kid::Cell(c as u16)));
        let k = kids.len() as u16;
        nodes.push(Node { kids, alpha: (0..k).collect(), beta: (0..k).collect(), sym: None });
        let mut home = vec![(0, 0); n];
        for (ni, node) in nodes.iter().enumerate() {
            for (s, k) in node.kids.iter().enumerate() {
                if let Kid::Cell(c) = *k {
                    home[usize::from(c)] = (ni as u16, s as u16);
                }
            }
        }
        let root = (nodes.len() - 1) as u16;
        let mut t = Tree { nodes, root, home };
        for s in 0..n_sym {
            t.make_sf(s as u16);
        }
        (t, errs)
    }

    /// Same nodes, kids, axes and mates (codes may differ): a warm start's
    /// tree still fits the inputs (PLC-10).
    #[must_use]
    pub fn same_structure(&self, o: &Tree) -> bool {
        self.root == o.root && self.nodes.len() == o.nodes.len() && self.nodes.iter().zip(&o.nodes).all(|(a, b)| a.kids == b.kids && a.sym == b.sym)
    }

    /// Balasa eq 1.1 for distinct kids `x`, `y` of a symmetry node:
    /// `(pa[x] < pa[y]) == (pb[mate[y]] < pb[mate[x]])`. `true` off symmetry nodes.
    #[must_use]
    pub fn is_sf(&self, node: u16) -> bool {
        let nd = &self.nodes[usize::from(node)];
        let Some(s) = &nd.sym else { return true };
        let (mut pa, mut pb) = (Vec::new(), Vec::new());
        inverse(&nd.alpha, &mut pa);
        inverse(&nd.beta, &mut pb);
        let m = |k: usize| usize::from(s.mate[k]);
        let k = nd.kids.len();
        (0..k).all(|x| (0..k).all(|y| x == y || (pa[x] < pa[y]) == (pb[m(y)] < pb[m(x)])))
    }

    /// Rewrite a symmetry node's β as its α reversed through `mate`
    /// (Balasa L2389–2390); then [`Tree::is_sf`] holds. No-op elsewhere.
    pub fn make_sf(&mut self, node: u16) {
        let nd = &mut self.nodes[usize::from(node)];
        if let Some(s) = &nd.sym {
            nd.beta = nd.alpha.iter().rev().map(|&g| s.mate[usize::from(g)]).collect();
        }
    }

    /// Embed a coordinate placement: per node, α by `(x − y, slot)` and β by
    /// `(x + y, slot)` of each kid's key point (cell centre, mean centre of a
    /// node's cells), then [`Tree::make_sf`].
    pub fn seed_from(&mut self, l: &Layout) {
        let mut key: Vec<(i64, i64, i64)> = vec![(0, 0, 0); self.nodes.len()];
        for ni in 0..self.nodes.len() {
            let pts: Vec<(i64, i64, i64)> = self.nodes[ni]
                .kids
                .iter()
                .map(|k| match *k {
                    Kid::Cell(c) => (i64::from(l.x[usize::from(c)]), i64::from(l.y[usize::from(c)]), 1),
                    Kid::Node(m) => key[usize::from(m)],
                })
                .collect();
            key[ni] = pts.iter().fold((0, 0, 0), |a, p| (a.0 + p.0, a.1 + p.1, a.2 + p.2));
            let at = |s: u16| {
                let (x, y, c) = pts[usize::from(s)];
                (x / c.max(1), y / c.max(1))
            };
            let nd = &mut self.nodes[ni];
            nd.alpha.sort_by_key(|&s| (at(s).0 - at(s).1, s));
            nd.beta.sort_by_key(|&s| (at(s).0 + at(s).1, s));
            self.make_sf(ni as u16);
        }
    }

    /// Deterministic start: symmetry nodes take Balasa's initial code
    /// `α = a_1…a_p c_1…c_s b_p…b_1` (L2375–2405; `a` the lower slot of each
    /// pair), every other node one row α = β by descending kid area (`w·h`
    /// summed over a node's cells), ties by slot.
    pub fn seed_constructive(&mut self, w: &[i32], h: &[i32]) {
        let mut area: Vec<i64> = vec![0; self.nodes.len()];
        for ni in 0..self.nodes.len() {
            let ka: Vec<i64> = self.nodes[ni]
                .kids
                .iter()
                .map(|k| match *k {
                    Kid::Cell(c) => i64::from(w[usize::from(c)]) * i64::from(h[usize::from(c)]),
                    Kid::Node(m) => area[usize::from(m)],
                })
                .collect();
            area[ni] = ka.iter().sum();
            let nd = &mut self.nodes[ni];
            let k = nd.kids.len() as u16;
            if let Some(s) = &nd.sym {
                let a: Vec<u16> = (0..k).filter(|&x| s.mate[usize::from(x)] > x).collect();
                let c = (0..k).filter(|&x| s.mate[usize::from(x)] == x);
                nd.alpha = a.iter().copied().chain(c).chain(a.iter().rev().map(|&x| s.mate[usize::from(x)])).collect();
                self.make_sf(ni as u16);
            } else {
                nd.alpha = (0..k).collect();
                nd.alpha.sort_by_key(|&x| (-ka[usize::from(x)], x));
                nd.beta.clone_from(&nd.alpha);
            }
        }
    }
}

/// Inverse permutation: `inv[seq[p]] = p`.
pub(super) fn inverse(seq: &[u16], inv: &mut Vec<u32>) {
    inv.clear();
    inv.resize(seq.len(), 0);
    for (p, &k) in seq.iter().enumerate() {
        inv[usize::from(k)] = p as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::mechanics::SplitMix64;
    use crate::gp::spacing::{Face, Gap, Profile, SpacingTable};

    fn geo<'a>(w: &'a [i32], h: &'a [i32], prof: &'a [Option<&'a Profile>], t: &'a SpacingTable) -> Geo<'a> {
        Geo { w, h, prof, halo: &[], table: t, lattice: 10, axis_grid: None }
    }

    /// Balasa Example 1 (L2021–2028; dims from the Fig. 1.18 caption) as a
    /// hierarchy: the pair cells form the symmetry node, the other six are
    /// root siblings, each code restricted from the flat (EDCKAFGIHJBL,
    /// KACDEFGLHBJI) with the node at its first member's position.
    /// Oracle: Balasa §1.3 Example 1 (the book's codes and dimensions); mirror equations checked by hand
    /// arithmetic and `verify`.
    #[test]
    fn balasa_example_1_decodes_symmetric() {
        // A B C D E F G H I J K L
        let dims = [(2, 4), (3, 1), (4, 2), (5, 3), (4, 2), (2, 10), (2, 10), (2, 3), (5, 5), (4, 2), (3, 2), (3, 2)];
        let w: Vec<i32> = dims.iter().map(|d| d.0 * 1000).collect();
        let h: Vec<i32> = dims.iter().map(|d| d.1 * 1000).collect();
        let (f, g_, k, l_, c, j) = (5u32, 6, 10, 11, 2, 9);
        let (mut t, errs) = Tree::build(12, &[(f, g_, 0), (k, l_, 0), (c, j, 0)], &[]);
        assert!(errs.is_empty());
        // Sym slots: F0 G1 K2 L3 C4 J5. α = C K F G J L, β = K C F G L J.
        t.nodes[0].alpha = vec![4, 2, 0, 1, 5, 3];
        t.nodes[0].beta = vec![2, 4, 0, 1, 3, 5];
        assert!(t.is_sf(0));
        // Root slots: S0 A1 B2 D3 E4 H5 I6. α = E D S A I H B, β = S A D E H B I.
        let r = usize::from(t.root);
        t.nodes[r].alpha = vec![4, 3, 0, 1, 6, 5, 2];
        t.nodes[r].beta = vec![0, 1, 3, 4, 5, 2, 6];
        let table = SpacingTable::uniform(0, 10);
        let prof = vec![None; 12];
        let g = geo(&w, &h, &prof, &table);
        let mut out = Out::default();
        assert_eq!(decode(&t, &g, &mut Scratch::default(), &mut out), Ok(()));
        assert!(verify(&t, &g, &out));
        let cc = |i: u32| 2 * out.x0[i as usize] + w[i as usize];
        let ax2 = 2 * out.axis[0].1;
        assert_eq!((cc(f) + cc(g_), cc(k) + cc(l_), cc(c) + cc(j)), (2 * ax2, 2 * ax2, 2 * ax2));
        for a in 0..12 {
            for b in a + 1..12 {
                let apart =
                    out.x0[a] + w[a] <= out.x0[b] || out.x0[b] + w[b] <= out.x0[a] || out.y0[a] + h[a] <= out.y0[b] || out.y0[b] + h[b] <= out.y0[a];
                assert!(apart, "cells {a} and {b} overlap");
            }
        }
        // Derived by hand: in the node K is below C and L below J (2000 each);
        // the node sits at the root's origin. Balasa's flat 4000 needs A below
        // C, which a contiguous node cannot express.
        assert_eq!((out.y0[c as usize], out.y0[j as usize]), (2000, 2000));
        assert_eq!(out.axis[0].1, 6000);
    }

    /// Oracle: Balasa's condition (1.1), checked by `is_sf` on shuffled codes.
    #[test]
    fn make_sf_satisfies_condition_1_1() {
        let (mut t, _) = Tree::build(5, &[(0, 1, 0), (2, 3, 0), (4, 4, 0)], &[]);
        let mut rng = SplitMix64::new(7);
        for _ in 0..1000 {
            let a = &mut t.nodes[0].alpha;
            for i in (1..a.len()).rev() {
                a.swap(i, rng.below(i + 1));
            }
            t.make_sf(0);
            assert!(t.is_sf(0), "{:?}", t.nodes[0]);
        }
    }

    /// Two same-net n-well kids (`Gap { abut: true, min: 1270 }`) facing across
    /// x; cell 2 (below A, left of B) sets B's longest path. `wide` = 500 ⇒
    /// gap 500, banded ⇒ closed to 1270; `wide` = 0 ⇒ abutted, stays 0.
    fn band(wide: i32) -> i32 {
        let mut table = SpacingTable::uniform(0, 10);
        table.rule[0][0] = 1270; // ROLES[0] = "nwell", mergeable
        let mut p = Profile { well_net: Some(analog::base::NetId(1)), ..Profile::default() };
        for f in 0..4 {
            p.edge[f].put(0, 0);
        }
        assert_eq!(table.gap(&p, Face::R, &p), Gap { abut: true, min: 1270 });
        let (w, h) = ([1000, 1000, 1000 + wide], [1000, 2000, 500]);
        let prof = [Some(&p), Some(&p), None];
        let (mut t, _) = Tree::build(3, &[], &[]);
        // Root slots = cells A0 B1 C2: A left of B, C below A, C left of B.
        let r = usize::from(t.root);
        t.nodes[r].alpha = vec![0, 2, 1];
        t.nodes[r].beta = vec![2, 0, 1];
        let g = geo(&w, &h, &prof, &table);
        let mut out = Out::default();
        assert_eq!(decode(&t, &g, &mut Scratch::default(), &mut out), Ok(()));
        assert!(verify(&t, &g, &out));
        out.x0[1] - (out.x0[0] + w[0])
    }

    /// Oracle: the sky130 DRM: same-net wells either abut (0) or keep the
    /// 1270 nm nwell spacing, so a 500 nm gap must close to 1270.
    #[test]
    fn merge_band_is_closed() {
        assert_eq!(band(500), 1270);
        assert_eq!(band(0), 0);
    }

    /// Oracle: the input: cell 0 named on two axes by construction.
    #[test]
    fn conflicting_axes_are_reported() {
        let (t, errs) = Tree::build(4, &[(0, 1, 0), (0, 2, 1), (3, 2, 1)], &[]);
        assert_eq!(errs, vec![TreeError::CellInTwoAxes { cell: 0, a: AxisId(0), b: AxisId(1) }]);
        assert_eq!(t.nodes.iter().filter(|n| n.sym.is_some()).count(), 1);
    }
}
