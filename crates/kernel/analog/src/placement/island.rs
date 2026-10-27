//! Symmetry-island connectivity (PLC-12; Balasa Def. 2.1): the members of one
//! symmetry group should form one connected island, each touching another
//! across a shared edge, not merely sit near the same axis.
//!
//! In → out: a [`Layout`] and an island's members → its connected components
//! (islands past the first are the violation). Cites: Balasa Def. 2.1; PLC-12.
//!
//! Contents:
//! - `pub struct SymmetryIsland`
//! - `pub fn components(l: &Layout, members: &[Target], touch_nm: i32) -> u32`

use crate::base::ids::Target;
use crate::base::layout::Layout;
use crate::base::UnionFind;

use crate::rule::RuleBatch;

/// One island: `members` (cells after retarget) and the largest gap that
/// still counts as touching, nm.
#[derive(Clone, Debug)]
pub struct SymmetryIsland {
    /// The symmetry group's cells.
    pub members: Vec<Target>,
    /// Largest edge distance that still counts as touching, nm; ≥ 0.
    pub touch_nm: i32,
}

/// Connected components of `members` under [`adjacent`]; `0` with no members.
/// O(n²) in the member count.
#[must_use]
pub fn components(l: &Layout, members: &[Target], touch_nm: i32) -> u32 {
    let n = members.len();
    let mut uf = UnionFind::new(n);
    for i in 0..n {
        for j in i + 1..n {
            if adjacent(l, members[i], members[j], touch_nm) {
                uf.union(i as u32, j as u32);
            }
        }
    }
    let mut roots: Vec<u32> = (0..n as u32).map(|i| uf.find(i)).collect();
    roots.sort_unstable();
    roots.dedup();
    roots.len() as u32
}

impl RuleBatch<Layout> for SymmetryIsland {
    /// Islands past the first.
    fn cost(&self, l: &Layout) -> f32 {
        self.extra(l) as f32
    }
    fn violations(&self, l: &Layout) -> u32 {
        u32::from(self.extra(l) > 0)
    }
    fn residual(&self, l: &Layout) -> f64 {
        f64::from(self.extra(l))
    }
    fn kind(&self) -> &'static str {
        "SymmetryIsland"
    }
    fn count(&self) -> usize {
        1
    }
    fn touched(&self, out: &mut Vec<u32>) {
        for t in &self.members {
            if let Target::Device(d) = t {
                out.push(u32::from(d.0));
            }
        }
    }
    fn retarget(&mut self, cell_of: &[u16]) {
        for t in &mut self.members {
            *t = t.retarget(cell_of);
        }
    }
}

// ---- private ----

/// `a` and `b` share an edge: strictly overlapping projections on one axis and
/// an edge distance ≤ `touch_nm` on the other (a corner contact is not
/// adjacency, so this is not `Layout::edge_gap`, which is 0 there).
fn adjacent(l: &Layout, a: Target, b: Target, touch_nm: i32) -> bool {
    let (ax, ay, aw, ah) = l.bbox(a);
    let (bx, by, bw, bh) = l.bbox(b);
    let (dx, dy) = ((ax - bx).abs(), (ay - by).abs());
    (dx < aw + bw && dy - (ah + bh) <= touch_nm) || (dy < ah + bh && dx - (aw + bw) <= touch_nm)
}

impl SymmetryIsland {
    fn extra(&self, l: &Layout) -> u32 {
        components(l, &self.members, self.touch_nm).saturating_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::ids::DeviceId;

    /// 1000 nm squares at the given centres.
    fn squares(c: &[(i32, i32)]) -> Layout {
        let n = c.len();
        Layout {
            x: c.iter().map(|p| p.0).collect(),
            y: c.iter().map(|p| p.1).collect(),
            hw: vec![500; n],
            hh: vec![500; n],
            axis: vec![0],
            groups: vec![],
            orient: vec![crate::base::Orient::default(); n],
            variant: vec![0; n],
            branch: Vec::new(),
            power_uw: vec![0; n],
            temp_mc: vec![0; n],
            units: Default::default(),
        }
    }

    fn island(n: u16, touch_nm: i32) -> SymmetryIsland {
        SymmetryIsland { members: (0..n).map(|i| Target::Device(DeviceId(i))).collect(), touch_nm }
    }

    /// Oracle: spec example, two abutting squares by hand.
    #[test]
    fn abutting_pair_is_one_island() {
        let l = squares(&[(0, 0), (1_000, 0)]);
        assert_eq!((island(2, 280).residual(&l), island(2, 280).violations(&l)), (0.0, 0));
    }

    /// Oracle: spec example, the touch threshold by hand (280 nm joins, 290 nm splits).
    #[test]
    fn a_gap_splits_the_island() {
        let ok = squares(&[(0, 0), (1_280, 0)]);
        assert_eq!(island(2, 280).residual(&ok), 0.0);
        let split = squares(&[(0, 0), (1_000 + 280 + 10, 0)]);
        assert_eq!((island(2, 280).residual(&split), island(2, 280).violations(&split)), (1.0, 1));
    }

    /// Oracle: spec example, Balasa Def. 2.1 (a corner is not an edge).
    #[test]
    fn diagonal_corner_touch_is_not_adjacent() {
        let l = squares(&[(0, 0), (1_000, 1_000)]);
        assert_eq!(island(2, 280).residual(&l), 1.0);
    }

    /// Oracle: spec example, the ids listed by hand.
    #[test]
    fn touched_names_every_member() {
        let mut ids = Vec::new();
        let mut i = island(3, 0);
        i.members.push(Target::Group(crate::base::GroupId(0)));
        i.touched(&mut ids);
        assert_eq!(ids, [0, 1, 2]);
    }
}
