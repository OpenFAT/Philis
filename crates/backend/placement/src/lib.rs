//! `placement`: floorplan recipes (Q21) and the free-form placer moved from
//! Philis 1.0 (Q16): [`reference`], the floorplan a sidecar `Reference`
//! sets (Q61); [`gp`] global placement and [`dp`] detailed placement, which
//! reuses `gp`'s mechanics. The row stack and array families are M2 and M3.

pub mod dp;
pub mod gp;
pub mod reference;
