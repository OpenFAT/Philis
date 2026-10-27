//! `placement`: floorplan recipes (to come, Q21) and the free-form placer
//! moved from Philis 1.0 (Q16): [`gp`] global placement and [`dp`] detailed
//! placement, which reuses `gp`'s mechanics.

pub mod dp;
pub mod floorplan;
pub mod gp;
