//! The applied rule-set an algorithm consumes.
//! In → out: none; the annotator assembles [`Requirements`], the placer and
//! router read it. Cites: the `lex-min(V, Θ, PEX)` tiers of `base::report`.
//!
//! Contents:
//! - `pub struct Requirements<On>`

use crate::rule::RuleBatch;

/// Every rule batch for one scored state `On`, partitioned by tier: which `Vec`
/// a batch sits in *is* its enforcement mode.
///
/// - `hard` (V): [`RuleBatch::violations`]. Legality and exact equalities
///   (repaired by [`RuleBatch::project`], never by weight).
/// - `budget` (Θ): [`RuleBatch::residual`], priced by `gp::Prices`.
/// - `cost` (PEX): [`RuleBatch::cost`].
///
/// A kind may sit in `hard` or `budget` **and** in `cost` (feasible set +
/// gradient toward it), never in both `hard` and `budget`.
///
/// **Order within an arm is a contract for untagged batches:** `gp::Prices`
/// re-finds a tagged batch by its `BatchMeta::id`, so order only matters for
/// untagged batches, where it re-finds by `(kind(), ordinal among untagged
/// same kind)` and reordering silently moves prices. In-loop DRC batches are
/// appended to `hard` and truncated off, keeping the prefix.
pub struct Requirements<On> {
    /// Tier V: legality and exact equalities.
    pub hard: Vec<Box<dyn RuleBatch<On>>>,
    /// Tier Θ: priced budgets.
    pub budget: Vec<Box<dyn RuleBatch<On>>>,
    /// Tier PEX: the objective's cost terms.
    pub cost: Vec<Box<dyn RuleBatch<On>>>,
}

impl<On> Default for Requirements<On> {
    fn default() -> Self {
        Self { hard: Vec::new(), budget: Vec::new(), cost: Vec::new() }
    }
}
