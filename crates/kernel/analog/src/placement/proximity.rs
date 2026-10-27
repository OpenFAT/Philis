//! Preferential proximity (placement tier).
//! In → out: a [`Layout`] → the edge-to-edge gap of two targets against a
//! distance cap (cost, Θ residual, usage). Cites: PLC-18 (dimensionless cost).
//!
//! Contents:
//! - `pub struct Proximity`

use crate::base::ids::Target;
use crate::base::layout::Layout;
use crate::rule::Rule;

/// Soft pull keeping related devices within `max_distance_nm` of each other,
/// edge to edge (a centre distance would make the spec depend on cell size).
/// Attracts only: a priced budget (Θ) and a cost pull, never a hard gate.
#[derive(Clone, Copy)]
pub struct Proximity {
    /// One side; a group reads as its members' bounding box.
    pub a: Target,
    /// The other side.
    pub b: Target,
    /// Largest edge-to-edge gap that costs nothing, nm; > 0.
    pub max_distance_nm: i32,
}

impl Rule for Proximity {
    type On = Layout;
    /// `(excess / max_distance)²`, excess = gap past `max_distance_nm` (PLC-18: dimensionless).
    fn cost(self, l: &Layout) -> f32 {
        let m = self.max_distance_nm.max(1) as f32;
        let e = (self.gap(l) - m).max(0.0) / m;
        e * e
    }
    fn satisfied(self, l: &Layout) -> bool {
        self.gap(l) <= self.max_distance_nm as f32
    }
    fn residual(self, l: &Layout) -> f32 {
        let m = self.max_distance_nm as f32;
        crate::rule::over(self.gap(l) - m, m)
    }
    fn usage(self, l: &Layout) -> Option<f32> {
        Some(self.gap(l) / self.max_distance_nm.max(1) as f32)
    }
    fn touches(self, out: &mut Vec<u32>) {
        for t in [self.a, self.b] {
            if let Target::Device(d) = t {
                out.push(u32::from(d.0));
            }
        }
    }
    fn retarget(self, cell_of: &[u16]) -> Self {
        Self { a: self.a.retarget(cell_of), b: self.b.retarget(cell_of), ..self }
    }
}

// ---- private ----

impl Proximity {
    /// Euclidean edge-to-edge gap, nm (`0` when touching or overlapping).
    fn gap(self, l: &Layout) -> f32 {
        let ((ax, ay), (aw, ah)) = (l.centre(self.a), l.extent(self.a));
        let ((bx, by), (bw, bh)) = (l.centre(self.b), l.extent(self.b));
        let gx = ((ax - bx).abs() - aw - bw).max(0) as f32;
        let gy = ((ay - by).abs() - ah - bh).max(0) as f32;
        gx.hypot(gy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::ids::{DeviceId, GroupId};
    use crate::rule::RuleBatch;

    /// Device ids in `(a, b)` order; a `Group` side contributes nothing.
    /// Oracle: spec example, the ids listed by hand.
    #[test]
    fn touched_yields_device_ids_only() {
        let (d, g) = (|i| Target::Device(DeviceId(i)), Target::Group(GroupId(0)));
        let batch = vec![
            Proximity { a: d(3), b: d(1), max_distance_nm: 5_000 },
            Proximity { a: g, b: d(7), max_distance_nm: 5_000 },
            Proximity { a: g, b: g, max_distance_nm: 5_000 },
        ];
        let mut ids = Vec::new();
        batch.touched(&mut ids);
        assert_eq!(ids, [3, 1, 7]);
    }
}
