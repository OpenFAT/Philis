//! `analog` — what good analog layout requires, independent of any algorithm.
//!
//! - [`base`]: the tables everything is scored against (ids, netlist, `Layout`,
//!   `Routes`, `Macro`, reports); Philis 1.0's `pnr_core`, folded in (Q36).
//! - [`cell`]: structural directives `cells` reads cold ([`Constraints`]).
//! - [`placement`]: [`Rule`]s scored against `crate::base::Layout` (gp, dp).
//! - [`routing`]: [`Rule`]s scored against `crate::base::Routes` (gr, dr).
//! - [`matching`]: unit moments, CC patterns and the mismatch ledger of matched sets.
//! - [`metadata`]: net classification the annotator turns into routing rules.
//!
//! Rules are `Copy` values in per-kind `Vec`s scored through a monomorphised
//! loop; the only `dyn` is [`RuleBatch`], once per kind. The annotator
//! assembles them into [`Requirements`], whose arm (hard / budget / cost) is
//! the rule's enforcement tier.

pub mod base;
pub mod cell;
pub mod constraints;
pub mod intent;
pub mod matching;
pub mod metadata;
pub mod placement;
pub mod requirements;
pub mod routing;
pub mod rule;

pub use constraints::Constraints;
pub use requirements::Requirements;
pub use rule::{RepairKind, Rule, RuleBatch, RuleKind};
