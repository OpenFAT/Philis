//! `placement`: floorplan recipes (Q20 to Q22: [`floorplan`]) and the free-form placer
//! moved from Philis 1.0 (Q16): [`gp`] global placement and [`dp`] detailed
//! placement, which reuses `gp`'s mechanics.
//!
//! Contents: none public.

#![warn(missing_docs)]

pub mod dp;
pub mod floorplan;
pub mod gp;
