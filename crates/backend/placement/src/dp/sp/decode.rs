//! Sequence-pair decoding (PLC-07, PLC-08; Balasa §1.3, Plantage Alg. 3.3
//! for shadows): a [`Tree`] code plus cell sizes → lattice coordinates with
//! exact pairwise gaps and exact mirror equations, or the [`Fail`] that says
//! why the code has none; [`verify`] re-checks a decoded [`Out`] from its
//! coordinates alone.
//!
//! Contents:
//! - `pub struct Geo<'a>`
//! - `pub struct Out`
//! - `pub enum Fail`
//! - `pub struct Scratch`
//! - `pub(super) struct Kids<'s>`
//! - `Scratch::touch(&mut self, t: &Tree, node: u16)`
//! - `Scratch::touch_cell(&mut self, t: &Tree, c: usize)`
//! - `Scratch::invalidate(&mut self)`
//! - `pub fn decode(t: &Tree, g: &Geo, s: &mut Scratch, out: &mut Out) -> Result<(), Fail>`
//! - `pub fn to_layout(out: &Out, w: &[i32], h: &[i32], l: &mut Layout)`
//! - `Kids::gap(&self, g: &Geo, gc: &mut [[Option<Gap>; 2]], i: usize, j: usize, ax: usize) -> (Gap, i32)`
//! - `Kids::ok(&self, g: &Geo, gc: &mut [[Option<Gap>; 2]], i: usize, j: usize, ax: usize) -> bool`
//! - `pub(super) fn merge(k: &Kids, wd: i32, ht: i32) -> (Option<Profile>, [i32; 4])`

use analog::base::ids::AxisId;
use analog::base::Layout;

use super::{inverse, Kid, Tree};
use crate::gp::spacing::{Face, Gap, Profile, SpacingTable};

/// Per cell, for its current (variant, orient): extents (even, nm), oriented
/// profile (`None` = spaced at `table.fallback`), routing halo per [`Face`]
/// (empty or short = zero; PLC-15). `axis_grid = Some((p0, P))` puts every axis
/// on a routing track centreline (PLC-28).
pub struct Geo<'a> {
    /// Cell width, nm, even, after orient.
    pub w: &'a [i32],
    /// Cell height, nm, even, after orient.
    pub h: &'a [i32],
    /// Cell face profile after orient; `None` spaces the cell at `table.fallback`.
    pub prof: &'a [Option<&'a Profile>],
    /// Routing halo per cell by [`Face`], nm; empty or short = zero.
    pub halo: &'a [[i32; 4]],
    /// The role × role spacing the gaps come from.
    pub table: &'a SpacingTable,
    /// Every decoded corner and `2·axis` is a multiple of it, nm (> 0).
    pub lattice: i32,
    /// `(p0, P)`: every axis at `p0/2 (mod P)`, a track centreline (PLC-28); `None` = lattice only.
    pub axis_grid: Option<(i32, i32)>,
}

/// Decoded cell lower-left corners, one `(axis, x)` per symmetry node in node
/// order, and how many `fix_monotone` passes the decode needed.
#[derive(Default, Debug)]
pub struct Out {
    /// Cell lower-left x, nm, by cell index.
    pub x0: Vec<i32>,
    /// Cell lower-left y, nm, by cell index.
    pub y0: Vec<i32>,
    /// One `(axis, x)` per symmetry node, in node order, nm.
    pub axis: Vec<(AxisId, i32)>,
    /// `fix_monotone` passes the decode needed (0 when the first estimate held).
    pub fixes: u32,
}

/// Why a code has no realisation; payload: node index.
#[derive(Debug, PartialEq, Eq)]
pub enum Fail {
    /// A mirror pair's centre heights did not meet within `pairs + 2` raising passes; payload: node index.
    SymY(u16),
    /// No axis on the lattice (or track grid) fits: self kids' widths disagree mod `2·lattice`, or the axis did not settle within `2·pairs + 2` passes; payload: node index.
    SymX(u16),
    /// Closing the merge bands did not settle within `2·kids + 2` passes; payload: node index.
    Band(u16),
}

/// Reused buffers so a decode allocates only on growth, and each node's last
/// result: [`decode`] re-decodes only nodes [`Scratch::touch`] marked (PLC-10).
#[derive(Default)]
pub struct Scratch {
    /// Per node: needs a re-decode; parent node and slot there (empty = nothing cached yet).
    dirty: Vec<bool>,
    parent: Vec<(u16, u16)>,
    /// Per node: it is, or contains, a symmetry node (PLC-28 origin rounding).
    has_sym: Vec<bool>,
    /// Per node: its kid-pair gap cache `[R, T]` by `i·k + j`, and per slot
    /// whether that kid's profile changed since (its row and column are stale).
    gcn: Vec<Vec<[Option<Gap>; 2]>>,
    stale: Vec<Vec<bool>>,
    pa: Vec<u32>,
    pb: Vec<u32>,
    /// Per node, per kid slot: position relative to the node's origin.
    rel_x: Vec<Vec<i32>>,
    rel_y: Vec<Vec<i32>>,
    lb: Vec<i32>,
    lby: Vec<i32>,
    /// Per node: merged profile (`None` when a member has none), halo, `(W, H)`, `2·axis` relative, origin.
    node_prof: Vec<Option<Profile>>,
    node_halo: Vec<[i32; 4]>,
    dim: Vec<(i32, i32)>,
    ax2: Vec<i32>,
    org: Vec<(i32, i32)>,
    /// Current node's kids: extents, profile, halo, is-a-node.
    kw: Vec<i32>,
    kh: Vec<i32>,
    kp: Vec<Option<Profile>>,
    khalo: Vec<[i32; 4]>,
    knode: Vec<bool>,
}

/// Kid boxes of one node (relative or absolute), with the geometry the gaps read.
pub(super) struct Kids<'s> {
    pub(super) x: &'s [i32],
    pub(super) y: &'s [i32],
    pub(super) w: &'s [i32],
    pub(super) h: &'s [i32],
    pub(super) p: &'s [Option<Profile>],
    pub(super) halo: &'s [[i32; 4]],
    pub(super) node: &'s [bool],
}

impl Scratch {
    /// Mark `node` and its ancestors for re-decode: its code, or a kid's
    /// extents or profile, changed (or changed back).
    pub fn touch(&mut self, t: &Tree, node: u16) {
        if self.parent.len() != t.nodes.len() {
            return;
        }
        let mut n = usize::from(node);
        loop {
            self.dirty[n] = true;
            if n == usize::from(t.root) {
                break;
            }
            n = usize::from(self.parent[n].0);
        }
    }

    /// Cell `c`'s extents or profile changed: re-decode its node, recompute its gaps.
    pub fn touch_cell(&mut self, t: &Tree, c: usize) {
        if self.parent.len() != t.nodes.len() {
            return;
        }
        let (node, slot) = t.home[c];
        if let Some(st) = self.stale[usize::from(node)].get_mut(usize::from(slot)) {
            *st = true;
        }
        self.touch(t, node);
    }

    /// Drop every cached node result (a new tree or a wholesale code change).
    pub fn invalidate(&mut self) {
        self.parent.clear();
    }
}

/// Decode `t` (post-order; only nodes marked by [`Scratch::touch`] once
/// `s` holds a decode of `t`), then assemble absolute corners (pre-order) into
/// `out`. `Err` names the first node with no realisation; `out` is then stale.
pub fn decode(t: &Tree, g: &Geo, s: &mut Scratch, out: &mut Out) -> Result<(), Fail> {
    let nn = t.nodes.len();
    s.rel_x.resize_with(nn, Vec::new);
    s.rel_y.resize_with(nn, Vec::new);
    s.node_prof.resize(nn, None);
    s.node_halo.resize(nn, [0; 4]);
    s.dim.resize(nn, (0, 0));
    s.ax2.resize(nn, 0);
    s.org.resize(nn, (0, 0));
    if s.parent.len() != nn {
        s.parent.clear();
        s.parent.resize(nn, (t.root, 0));
        for (ni, nd) in t.nodes.iter().enumerate() {
            for (slot, k) in nd.kids.iter().enumerate() {
                if let Kid::Node(m) = *k {
                    s.parent[usize::from(m)] = (ni as u16, slot as u16);
                }
            }
        }
        s.dirty.clear();
        s.dirty.resize(nn, true);
        s.gcn.clear();
        s.gcn.resize(nn, Vec::new());
        s.stale.clear();
        s.stale.resize(nn, Vec::new());
        s.has_sym.clear();
        for nd in &t.nodes {
            let deep = nd.kids.iter().any(|k| matches!(*k, Kid::Node(m) if s.has_sym[usize::from(m)]));
            s.has_sym.push(nd.sym.is_some() || deep);
        }
    }
    let mut fixes = 0;
    for ni in 0..nn {
        if s.dirty[ni] {
            decode_node(t, g, s, ni, &mut fixes)?;
            s.dirty[ni] = false;
            // Its merged profile may have changed: the parent's gaps to it are stale.
            if ni != usize::from(t.root) {
                let (p, slot) = s.parent[ni];
                if let Some(st) = s.stale[usize::from(p)].get_mut(usize::from(slot)) {
                    *st = true;
                }
            }
        }
    }
    out.fixes = fixes;
    out.x0.clear();
    out.x0.resize(t.home.len(), 0);
    out.y0.clear();
    out.y0.resize(t.home.len(), 0);
    s.org[usize::from(t.root)] = (0, 0);
    for ni in (0..nn).rev() {
        let (ox, oy) = s.org[ni];
        for (slot, kid) in t.nodes[ni].kids.iter().enumerate() {
            let at = (ox + s.rel_x[ni][slot], oy + s.rel_y[ni][slot]);
            match *kid {
                Kid::Cell(c) => (out.x0[usize::from(c)], out.y0[usize::from(c)]) = at,
                Kid::Node(m) => s.org[usize::from(m)] = at,
            }
        }
    }
    out.axis.clear();
    for (ni, nd) in t.nodes.iter().enumerate() {
        if let Some(sym) = &nd.sym {
            out.axis.push((sym.axis, (2 * s.org[ni].0 + s.ax2[ni]) / 2));
        }
    }
    Ok(())
}

/// Assembly step 8: `x = x0 + w/2`, `hw = w/2` (same for y), and each
/// symmetry node's axis into `l.axis` (grown when short).
pub fn to_layout(out: &Out, w: &[i32], h: &[i32], l: &mut Layout) {
    for c in 0..out.x0.len() {
        (l.hw[c], l.hh[c]) = (w[c] / 2, h[c] / 2);
        (l.x[c], l.y[c]) = (out.x0[c] + w[c] / 2, out.y0[c] + h[c] / 2);
    }
    for &(a, v) in &out.axis {
        let a = usize::from(a.0);
        if l.axis.len() <= a {
            l.axis.resize(a + 1, v);
        }
        l.axis[a] = v;
    }
}

impl Kids<'_> {
    /// Gap kid `i` owes `j` across `i`'s face `R` (`ax = 0`) or `T` (`ax = 1`),
    /// cached. A node kid never abuts: its merged profile hides deeper members.
    pub(super) fn gap(&self, g: &Geo, gc: &mut [[Option<Gap>; 2]], i: usize, j: usize, ax: usize) -> (Gap, i32) {
        let k = self.x.len();
        let gp = *gc[i * k + j][ax].get_or_insert_with(|| {
            let mut gp = gap(g, self.p[i].as_ref(), [Face::R, Face::T][ax], self.p[j].as_ref());
            gp.abut &= !self.node[i] && !self.node[j];
            gp
        });
        let halo = if ax == 0 { self.halo[i][2] + self.halo[j][0] } else { self.halo[i][3] + self.halo[j][1] };
        (gp, halo)
    }

    /// `i` and `j` are apart on axis `ax` by a separation their gap accepts.
    pub(super) fn ok(&self, g: &Geo, gc: &mut [[Option<Gap>; 2]], i: usize, j: usize, ax: usize) -> bool {
        let (p, s) = if ax == 0 { (self.x, self.w) } else { (self.y, self.h) };
        let (lo, hi) = if p[i] + s[i] <= p[j] {
            (i, j)
        } else if p[j] + s[j] <= p[i] {
            (j, i)
        } else {
            return false;
        };
        let (gp, halo) = self.gap(g, gc, lo, hi, ax);
        sep_ok(p[hi] - p[lo] - s[lo], gp, halo)
    }
}

/// Merged node profile and halo: per face and role, the least member inset
/// plus the member's distance to that face; `present` ORed, `matched` the max
/// class, `set`/`well_net` when common. `None` when any member has no profile.
pub(super) fn merge(k: &Kids, wd: i32, ht: i32) -> (Option<Profile>, [i32; 4]) {
    let mut halo = [0; 4];
    let mut out = Profile::default();
    let (mut net, mut set) = (None, None);
    let mut all = true;
    for i in 0..k.x.len() {
        let d = [k.x[i], k.y[i], wd - k.x[i] - k.w[i], ht - k.y[i] - k.h[i]];
        for f in 0..4 {
            halo[f] = halo[f].max(k.halo[i][f] - d[f]);
        }
        let Some(p) = &k.p[i] else {
            all = false;
            continue;
        };
        for (f, &df) in d.iter().enumerate() {
            let e = &p.edge[f];
            let mut bits = e.present;
            while bits != 0 {
                let r = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                out.edge[f].put(r, e.inset[r].saturating_add(df));
            }
        }
        net = Some(if net.is_none_or(|n| n == p.well_net) { p.well_net } else { None });
        set = Some(if set.is_none_or(|s| s == p.set) { p.set } else { None });
        out.matched = out.matched.max(p.matched);
    }
    (out.well_net, out.set) = (net.flatten(), set.flatten());
    (all.then_some(out), halo)
}

// ---- private ----

/// `table.gap` when both profiles exist, else `fallback` (as `PlaceRules::gaps`).
fn gap(g: &Geo, pi: Option<&Profile>, f: Face, pj: Option<&Profile>) -> Gap {
    match (pi, pj) {
        (Some(a), Some(b)) => g.table.gap(a, f, b),
        _ => Gap { abut: false, min: g.table.fallback },
    }
}

/// A separation `s` satisfies `gp` plus `halo`: abutment (`s == 0`, no halo) or `s ≥ min + halo`.
fn sep_ok(s: i32, gp: Gap, halo: i32) -> bool {
    (gp.abut && halo == 0 && s == 0) || s >= gp.min + halo
}

/// The longest-path edge weight: `0` when abutment is allowed, else `min + halo`.
fn need(gp: Gap, halo: i32) -> i32 {
    if gp.abut && halo == 0 {
        0
    } else {
        gp.min + halo
    }
}

fn round_up(v: i32, q: i32) -> i32 {
    let q = q.max(1);
    v.div_euclid(q) * q + if v.rem_euclid(q) == 0 { 0 } else { q }
}

/// Decode one node into `s.rel_x/rel_y[ni]`, `s.dim[ni]`, `s.node_prof[ni]`,
/// `s.ax2[ni]` (steps 1–6 of plan-04 PLC-08). Kid nodes must be decoded.
fn decode_node(t: &Tree, g: &Geo, s: &mut Scratch, ni: usize, fixes: &mut u32) -> Result<(), Fail> {
    let nd = &t.nodes[ni];
    let k = nd.kids.len();
    let fail_at = ni as u16;
    s.kw.clear();
    s.kh.clear();
    s.kp.clear();
    s.khalo.clear();
    s.knode.clear();
    for kid in &nd.kids {
        let (w, h, p, halo, is_node) = match *kid {
            Kid::Cell(c) => {
                let c = usize::from(c);
                (g.w[c], g.h[c], g.prof.get(c).copied().flatten().copied(), g.halo.get(c).copied().unwrap_or([0; 4]), false)
            }
            Kid::Node(m) => {
                let m = usize::from(m);
                (s.dim[m].0, s.dim[m].1, s.node_prof[m], s.node_halo[m], true)
            }
        };
        s.kw.push(w);
        s.kh.push(h);
        s.kp.push(p);
        s.khalo.push(halo);
        s.knode.push(is_node);
    }
    inverse(&nd.alpha, &mut s.pa);
    inverse(&nd.beta, &mut s.pb);
    if s.gcn[ni].len() != k * k {
        s.gcn[ni].clear();
        s.gcn[ni].resize(k * k, [None; 2]);
        s.stale[ni].clear();
        s.stale[ni].resize(k, false);
    }
    for j in 0..k {
        if std::mem::take(&mut s.stale[ni][j]) {
            for i in 0..k {
                (s.gcn[ni][i * k + j], s.gcn[ni][j * k + i]) = ([None; 2], [None; 2]);
            }
        }
    }
    s.lb.clear();
    s.lb.resize(k, 0);
    s.lby.clear();
    s.lby.resize(k, 0);
    let mut x = std::mem::take(&mut s.rel_x[ni]);
    let mut y = std::mem::take(&mut s.rel_y[ni]);
    x.clear();
    x.resize(k, 0);
    y.clear();
    y.resize(k, 0);

    let Scratch { pa, pb, lb, lby, kw, kh, kp, khalo, knode, gcn, has_sym, .. } = s;
    // PLC-28 step 3: a kid that is or holds a symmetry node starts on a multiple of `P`.
    let on_grid: Vec<bool> = nd.kids.iter().map(|k| matches!(*k, Kid::Node(m) if has_sym[usize::from(m)])).collect();
    let period = g.axis_grid.map(|(_, p)| p);
    let gc = &mut gcn[ni][..];
    let (pa, beta) = (&*pa, &nd.beta);
    let _ = pb;
    let (kw, kh, kp, khalo, knode) = (&kw[..], &kh[..], &kp[..], &khalo[..], &knode[..]);
    let mate = nd.sym.as_ref().map(|s| &s.mate);
    let pairs: Vec<(usize, usize)> = mate.map_or(Vec::new(), |m| (0..k).filter(|&a| usize::from(m[a]) > a).map(|a| (a, usize::from(m[a]))).collect());
    // A self kid sits at `(ax2 - w)/2`, on the lattice only when `ax2 ≡ w (mod 2·lattice)`:
    // every self kid's width must agree on that residue.
    let l2 = 2 * g.lattice.max(1);
    let mut self_w = None;
    for a in (0..k).filter(|&a| mate.is_some_and(|m| usize::from(m[a]) == a)) {
        if *self_w.get_or_insert(kw[a].rem_euclid(l2)) != kw[a].rem_euclid(l2) {
            return Err(Fail::SymX(fail_at));
        }
    }
    let p = pairs.len();
    let mut ax2 = 0;
    let band_cap = 2 * k + 2;
    let mut pass = 0;
    loop {
        // Step 2: Y longest path in β order; symmetry pairs equalise centres.
        for ypass in 1..=p + 2 {
            for bj in 0..k {
                let j = usize::from(beta[bj]);
                let mut v = lby[j];
                for &bi in &beta[..bj] {
                    let i = usize::from(bi);
                    if pa[i] > pa[j] {
                        let kd = Kids { x: &x, y: &y, w: kw, h: kh, p: kp, halo: khalo, node: knode };
                        let (gp, halo) = kd.gap(g, gc, i, j, 1);
                        v = v.max(y[i] + kh[i] + need(gp, halo));
                    }
                }
                y[j] = v;
            }
            let mut raised = false;
            for &(a, b) in &pairs {
                let (ca, cb) = (2 * y[a] + kh[a], 2 * y[b] + kh[b]);
                let (lo, want) = match ca.cmp(&cb) {
                    std::cmp::Ordering::Less => (a, cb),
                    std::cmp::Ordering::Greater => (b, ca),
                    std::cmp::Ordering::Equal => continue,
                };
                lby[lo] = lby[lo].max(round_up((want - kh[lo]) / 2, g.lattice));
                raised = true;
            }
            if !raised {
                break;
            }
            if ypass == p + 2 {
                return Err(Fail::SymY(fail_at));
            }
        }
        // Steps 3–4: X longest path with shadows; symmetry nodes raise right
        // members and self kids onto `ax2` and repeat until exact (fix_monotone).
        let step_x = |x: &mut [i32], ax2: Option<i32>, gc: &mut [[Option<Gap>; 2]]| {
            for bj in 0..k {
                let j = usize::from(beta[bj]);
                let mut v = lb[j];
                for &bi in &beta[..bj] {
                    let i = usize::from(bi);
                    if pa[i] < pa[j] {
                        let kd = Kids { x: &*x, y: &y, w: kw, h: kh, p: kp, halo: khalo, node: knode };
                        let gx = if kd.ok(g, gc, i, j, 1) {
                            0
                        } else {
                            let (gp, halo) = kd.gap(g, gc, i, j, 0);
                            need(gp, halo)
                        };
                        v = v.max(x[i] + kw[i] + gx);
                    }
                }
                if let (Some(ax2), Some(m)) = (ax2, mate) {
                    let l = usize::from(m[j]);
                    if l == j {
                        v = v.max((ax2 - kw[j]) / 2);
                    } else if pa[l] < pa[j] {
                        v = v.max((2 * ax2 - (2 * x[l] + kw[l]) - kw[j]) / 2);
                    }
                }
                if let (Some(p), true) = (period, on_grid[j]) {
                    v = round_up(v, p);
                }
                x[j] = v;
            }
        };
        step_x(&mut x, None, gc);
        if mate.is_some() {
            let bound = |x: &[i32]| {
                let pv = pairs.iter().map(|&(a, b)| (2 * x[a] + kw[a] + 2 * x[b] + kw[b] + 1).div_euclid(2));
                let sv = (0..k).filter(|&a| mate.is_some_and(|m| usize::from(m[a]) == a)).map(|a| 2 * x[a] + kw[a]);
                pv.chain(sv).max().unwrap_or(0)
            };
            let snap = |v: i32| axis_snap(v, g.lattice, self_w, g.axis_grid);
            ax2 = snap(bound(&x)).ok_or(Fail::SymX(fail_at))?;
            let mut it = 0;
            loop {
                step_x(&mut x, Some(ax2), gc);
                let b = bound(&x);
                if b == ax2 {
                    break;
                }
                it += 1;
                if it > 2 * p + 2 {
                    return Err(Fail::SymX(fail_at));
                }
                *fixes += 1;
                ax2 = snap(b.max(ax2)).ok_or(Fail::SymX(fail_at))?;
            }
        }
        // Step 6: close merge bands (0 < gap < min with `abut`) on every related
        // pair not separated acceptably on the other axis.
        let mut raised = false;
        for bj in 0..k {
            let j = usize::from(beta[bj]);
            for &bi in &beta[..bj] {
                let i = usize::from(bi);
                let kd = Kids { x: &x, y: &y, w: kw, h: kh, p: kp, halo: khalo, node: knode };
                if kd.ok(g, gc, i, j, 0) || kd.ok(g, gc, i, j, 1) {
                    continue;
                }
                let ax = usize::from(pa[i] > pa[j]);
                let (gp, halo) = kd.gap(g, gc, i, j, ax);
                if ax == 0 {
                    lb[j] = lb[j].max(x[i] + kw[i] + gp.min + halo);
                } else {
                    lby[j] = lby[j].max(y[i] + kh[i] + gp.min + halo);
                }
                raised = true;
            }
        }
        if !raised {
            break;
        }
        pass += 1;
        if pass > band_cap {
            return Err(Fail::Band(fail_at));
        }
    }
    let wd = (0..k).map(|i| x[i] + kw[i]).max().unwrap_or(0);
    let ht = (0..k).map(|i| y[i] + kh[i]).max().unwrap_or(0);
    let kd = Kids { x: &x, y: &y, w: kw, h: kh, p: kp, halo: khalo, node: knode };
    let (prof, halo) = merge(&kd, wd, ht);
    s.node_prof[ni] = prof;
    s.node_halo[ni] = halo;
    s.dim[ni] = (wd, ht);
    s.ax2[ni] = ax2;
    s.rel_x[ni] = x;
    s.rel_y[ni] = y;
    Ok(())
}

/// Step 2x′: the least admissible `2·axis` ≥ `v`: `≡ self_w (mod 2·lattice)`
/// with self-symmetric kids (`self_w`: their common width residue, so each
/// corner `(2·axis − w)/2` lands on the lattice), else a multiple of `lattice`.
/// With `grid = Some((p0, P))` (PLC-28) the least `≥ v` with
/// `2·axis ≡ p0 (mod 2P)`, so the node-relative axis is `≡ p0/2 (mod P)`
/// (a track centreline once the node's origin is a multiple of `P`); `p0` and
/// `2P` are multiples of `2·lattice`, so `None` when `self_w ≠ 0`: a self kid
/// an odd number of lattice steps wide cannot be centred on a track.
fn axis_snap(v: i32, lattice: i32, self_w: Option<i32>, grid: Option<(i32, i32)>) -> Option<i32> {
    let l2 = 2 * lattice.max(1);
    match (grid, self_w) {
        (Some((p0, _)), Some(r)) if (p0 - r).rem_euclid(l2) != 0 => None,
        (Some((p0, p)), _) => Some(v + (p0 - v).rem_euclid(2 * p)),
        (None, Some(r)) => Some(v + (r - v).rem_euclid(l2)),
        (None, None) => Some(round_up(v, lattice)),
    }
}
