//! Placement order (placement tier): one step of an `Order` below the next.
//! In → out: a [`Layout`] → whether one target's bbox lies wholly below
//! another's, and by how much it overlaps. Cites: PLC-25 (a step reads as the
//! bbox of its devices), EXT-28 (current-flow orders, ground up), RECIPES §3
//! (FLOOR-RowStack: rows by `Order`, claim "order monotone").
//!
//! Contents:
//! - `pub struct Below`
//! - `Below::overlap_nm(self, l: &Layout) -> i32`

use crate::base::ids::Target;
use crate::base::layout::Layout;
use crate::rule::Rule;

/// `lower`'s bbox top at or under `upper`'s bbox bottom: one pair of devices
/// from consecutive steps of a vertical `Order`; an order is the batch of
/// every such pair. Touching meets it; the same cell never does.
#[derive(Clone, Copy)]
pub struct Below {
    /// A device of step `k`.
    pub lower: Target,
    /// A device of step `k + 1`.
    pub upper: Target,
}

impl Rule for Below {
    type On = Layout;
    /// Zero: an order is a legality the floorplan constructs, not a pull.
    fn cost(self, _l: &Layout) -> f32 {
        0.0
    }
    fn satisfied(self, l: &Layout) -> bool {
        self.overlap_nm(l) <= 0
    }
    fn touches(self, out: &mut Vec<u32>) {
        for t in [self.lower, self.upper] {
            if let Target::Device(d) = t {
                out.push(u32::from(d.0));
            }
        }
    }
    fn retarget(self, cell_of: &[u16]) -> Self {
        Self { lower: self.lower.retarget(cell_of), upper: self.upper.retarget(cell_of) }
    }
}

impl Below {
    /// `lower`'s top minus `upper`'s bottom, nm: ≤ 0 meets the order. One
    /// cell against itself reads its full height.
    #[must_use]
    pub fn overlap_nm(self, l: &Layout) -> i32 {
        let ((_, ly), (_, lh)) = (l.centre(self.lower), l.extent(self.lower));
        let ((_, uy), (_, uh)) = (l.centre(self.upper), l.extent(self.upper));
        (ly + lh) - (uy - uh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::ids::DeviceId;
    use crate::rule::RuleBatch;

    /// Cells 0..3 as centre y and half-height, x and width irrelevant.
    fn cells(y_hh: &[(i32, i32)]) -> Layout {
        let n = y_hh.len();
        Layout {
            x: vec![0; n],
            y: y_hh.iter().map(|c| c.0).collect(),
            hw: vec![500; n],
            hh: y_hh.iter().map(|c| c.1).collect(),
            orient: vec![crate::base::Orient::default(); n],
            variant: vec![0; n],
            axis: Vec::new(),
            branch: Vec::new(),
            groups: Vec::new(),
            power_uw: vec![0; n],
            temp_mc: vec![0; n],
            units: Default::default(),
        }
    }

    /// Oracle: spec example, the bboxes by hand: cell 0 spans y 0..1000,
    /// cell 1 1000..3000 (touching), cell 2 2500..3500 (overlaps cell 1 by 500).
    #[test]
    fn a_step_is_below_the_next_only_when_its_top_clears_the_next_bottom() {
        let l = cells(&[(500, 500), (2_000, 1_000), (3_000, 500)]);
        let d = |i| Target::Device(DeviceId(i));
        assert!(Below { lower: d(0), upper: d(1) }.satisfied(&l), "touching meets it");
        assert!(!Below { lower: d(1), upper: d(2) }.satisfied(&l), "500 nm of overlap");
        assert!(!Below { lower: d(1), upper: d(0) }.satisfied(&l), "reversed");
        assert!(!Below { lower: d(1), upper: d(1) }.satisfied(&l), "one cell is never below itself");
        assert_eq!(Below { lower: d(1), upper: d(2) }.overlap_nm(&l), 500);
        assert_eq!(Below { lower: d(0), upper: d(2) }.overlap_nm(&l), -1_500);
        // An order: steps [[0], [1, 2]] → pairs (0,1), (0,2); both clear.
        let order = vec![Below { lower: d(0), upper: d(1) }, Below { lower: d(0), upper: d(2) }];
        assert_eq!(order.violations(&l), 0);
        let swapped = vec![Below { lower: d(2), upper: d(0) }];
        assert_eq!(swapped.violations(&l), 1);
    }
}
