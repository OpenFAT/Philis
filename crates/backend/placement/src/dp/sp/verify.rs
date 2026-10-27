//! The decoder's certificate check: a [`Tree`] code and its decoded [`Out`]
//! → whether every gap, mirror equation and lattice point holds, read from
//! the absolute coordinates alone, never from the decoder's working state.
//!
//! Contents:
//! - `pub fn verify(t: &Tree, g: &Geo, out: &Out) -> bool`

use super::decode::{merge, Geo, Kids, Out};
use super::{Kid, Tree};
use crate::gp::spacing::Profile;

/// Check a decode from scratch on absolute coordinates: in every node, every
/// two kids (node kids by their cells' bbox and merged profile) are apart on
/// some axis by a gap they accept; every mirror pair has `C_l + C_r = 2·ax2`
/// and equal centre y, every self kid `C_c = ax2`; every corner and `2·axis`
/// on the lattice.
#[must_use]
pub fn verify(t: &Tree, g: &Geo, out: &Out) -> bool {
    let lat = g.lattice.max(1);
    if out.x0.iter().chain(&out.y0).any(|v| v % lat != 0) || out.axis.iter().any(|a| (2 * a.1) % lat != 0) {
        return false;
    }
    let nn = t.nodes.len();
    let mut bbox: Vec<(i32, i32, i32, i32)> = vec![(0, 0, 0, 0); nn];
    let mut prof: Vec<Option<Profile>> = vec![None; nn];
    let mut halo: Vec<[i32; 4]> = vec![[0; 4]; nn];
    let mut sym_i = 0;
    for (ni, nd) in t.nodes.iter().enumerate() {
        let k = nd.kids.len();
        let (mut x, mut y, mut w, mut h) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut p, mut hl, mut kn) = (Vec::new(), Vec::new(), Vec::new());
        for kid in &nd.kids {
            match *kid {
                Kid::Cell(c) => {
                    let c = usize::from(c);
                    (x.push(out.x0[c]), y.push(out.y0[c]), w.push(g.w[c]), h.push(g.h[c]));
                    (p.push(g.prof.get(c).copied().flatten().copied()), hl.push(g.halo.get(c).copied().unwrap_or([0; 4])), kn.push(false));
                }
                Kid::Node(m) => {
                    let (bx, by, bw, bh) = bbox[usize::from(m)];
                    (x.push(bx), y.push(by), w.push(bw), h.push(bh));
                    (p.push(prof[usize::from(m)]), hl.push(halo[usize::from(m)]), kn.push(true));
                }
            }
        }
        let mut gc = vec![[None; 2]; k * k];
        let kd = Kids { x: &x, y: &y, w: &w, h: &h, p: &p, halo: &hl, node: &kn };
        for i in 0..k {
            for j in i + 1..k {
                if !kd.ok(g, &mut gc, i, j, 0) && !kd.ok(g, &mut gc, i, j, 1) {
                    return false;
                }
            }
        }
        if let Some(s) = &nd.sym {
            let Some(&(_, axis)) = out.axis.get(sym_i) else { return false };
            sym_i += 1;
            for a in 0..k {
                let b = usize::from(s.mate[a]);
                let ok = if a == b {
                    2 * x[a] + w[a] == 2 * axis
                } else {
                    2 * x[a] + w[a] + 2 * x[b] + w[b] == 4 * axis && 2 * y[a] + h[a] == 2 * y[b] + h[b]
                };
                if !ok {
                    return false;
                }
            }
        }
        let x0 = x.iter().copied().min().unwrap_or(0);
        let y0 = y.iter().copied().min().unwrap_or(0);
        let x1 = (0..k).map(|i| x[i] + w[i]).max().unwrap_or(0);
        let y1 = (0..k).map(|i| y[i] + h[i]).max().unwrap_or(0);
        let rel_x: Vec<i32> = x.iter().map(|v| v - x0).collect();
        let rel_y: Vec<i32> = y.iter().map(|v| v - y0).collect();
        let kd = Kids { x: &rel_x, y: &rel_y, w: &w, h: &h, p: &p, halo: &hl, node: &kn };
        (prof[ni], halo[ni]) = merge(&kd, x1 - x0, y1 - y0);
        bbox[ni] = (x0, y0, x1 - x0, y1 - y0);
    }
    true
}
