//! Per-circuit structural intent (cold; read by `cells`, not scored).
//! In → out: none; the annotator's [`Constraints`] rows. Cites: nothing.
//!
//! Contents:
//! - `pub struct Constraints`

use crate::cell::{GuardRingRequirement, Unitization};

/// `Clone` because `library::run` retargets `guard_rings` into collapsed cell
/// ids on its own copy.
#[derive(Default, Clone)]
pub struct Constraints {
    /// One row per matched set the cells draw as units.
    pub unitization: Vec<Unitization>,
    /// One row per guard ring to draw.
    pub guard_rings: Vec<GuardRingRequirement>,
}
