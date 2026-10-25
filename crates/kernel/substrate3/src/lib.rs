//! # `substrate3` — the Substrate2 fork, plus Philis (Q35)
//!
//! The crate root re-exports [Substrate2](https://github.com/ucb-substrate/substrate2)
//! (`block`, `types`, `schematic`, `layout`, `simulation`, `context`, …) and
//! its IRs ([`layir`], [`scir`], [`spice`]). Write generators the Substrate
//! way; this crate adds what Philis brings on top of it:
//!
//! - [`pdk`]: typed PDKs for every Philis deck (gf180mcu, ihp_sg13g2, asap7
//!   generated from their decks; sky130 is upstream's), the [`pdk::Mos`]
//!   primitive on any of them, and the [`pdk::MosTile`] layout on the
//!   generated ones.
//! - [`signoff`]: in-process DRC/ERC/LVS/PEX of a Substrate block through
//!   GPurify, for any PDK `verify` has a deck for.
//!
//! Substrate's derives (`#[derive(Io, Block)]`) name the crate `substrate`
//! in the code they expand to, so a crate that uses them also depends on
//! `substrate` directly (`substrate = { workspace = true }`).
//!
//! - [`cells`]: the device blocks, Philis 1.0's `cells` generators (Q8); the
//!   typed tiles in [`pdk`] draw through them.
//!
//! The fork's source is `upstream/` (Substrate2 at `01d9600`, BSD-3; see
//! `README.md`), its own workspace; this crate depends on its crates by path.

pub use substrate::*;

pub use layir;
pub use scir;
pub use spice;

mod adapter;
pub mod cells;
pub mod pdk;
pub mod signoff;
