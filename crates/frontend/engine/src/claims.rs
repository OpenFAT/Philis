//! The `claims` table (`docs/TRAITS.md` Claims, `docs/ARCHITECTURE.md`): what
//! each recipe call promises about its constraint, and whether the drawn
//! geometry keeps it. Recipes declare ([`claims_of`]); [`score_claims`] fills
//! the status with the `kernel/analog` rules free-form placement is scored
//! by, read from the tiles' `Unit` rows, never from the recipe (Q2). The
//! row stack's own claim, every vertical `Order` kept (RECIPES §3), rides
//! under the [`FLOORPLAN`] call, scored by [`analog::placement::Below`].
//!
//! Six questions. In: the call table, its tiles and their floorplan; out: one
//! row per (call, promise). How many: at most five per call, calls ≤ 65 535,
//! plus one per vertical order. Widths: call `u16`, constraint `u32`, kind
//! and status `u8`, promise four bytes, measured `f32`. Access: written once
//! in call order, walked by the report and the conformance checks.
//! Lifetime: one run. Parallel: per call, not needed at M1's sizes.
//!
//! Contents:
//! - `pub const NO_INTENT: ConstraintId`
//! - `pub const FLOORPLAN: u16`
//! - `pub enum Promise`
//! - `pub enum Status`
//! - `pub struct Claims`
//! - `Claims::len(&self) -> usize`
//! - `Claims::is_empty(&self) -> bool`
//! - `Promise::rule(self) -> RuleKind`
//! - `pub fn claims_of(kind: RecipeKind) -> &'static [Promise]`
//! - `pub fn score_claims(calls: &RecipeCalls, tiles: &[Macro], layout: &Layout, problem: &annotator::Problem, pdk: &Pdk) -> Claims`
//! - `pub fn order_verdicts(calls: &RecipeCalls, problem: &annotator::Problem, layout: &Layout) -> Vec<(u16, Status, i32)>`

use std::sync::Arc;

use analog::base::Target;
use analog::base::{DeviceId, Layout, Macro, Process, Rect, UnitLib};
use analog::intent::ConstraintId;
use analog::matching::moments::{cancelled_order, Pt};
use analog::placement::{Below, EnvGeo, LiveEnvironment, MatchedSet, OrientCheck, OrientationSet};
use analog::{Rule, RuleBatch, RuleKind};
use verify::Pdk;

use crate::{RecipeCalls, RecipeKind};

/// The constraint column of a call whose set the annotator unitized without
/// inferring a `MatchSpec` (parallel devices: `pair`, `quad`).
pub const NO_INTENT: ConstraintId = ConstraintId(u32::MAX);

/// The call column of a claim the floorplan makes rather than a set call.
pub const FLOORPLAN: u16 = u16::MAX;

/// What a recipe promises about one constraint (`RECIPES.md` §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Promise {
    /// Every member's channels on one axis ([`OrientCheck::Axis`]).
    ParallelChannels,
    /// Every member drawn from one unit: equal unit weights `W·L`.
    UnitIdentity,
    /// The pairs' centroid offset, reported and not promised (Adjacent).
    CentroidOffset,
    /// Every pair `(0, i)`'s unit centroids within the coincidence tolerance.
    Coincidence,
    /// The members' unit moments equal through this order.
    MomentsCancelled(u8),
    /// Equal mean signed current, Φ_H and Φ_V ([`OrientCheck::Phi`]).
    PhiEqual,
    /// Equal WPE and OSE distances for every pair `(0, i)`.
    EqualEnvironment,
    /// Every step of `intent.order[k]` (a vertical order) placed wholly
    /// below the next; measured is the worst overlap, nm.
    OrderMonotone(u16),
}

/// A claim's verdict after scoring (`TRAITS.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Met,
    Violated,
    /// The rule's inputs are missing (no units, no deck range): reported,
    /// never certified.
    Unknown,
    /// A measurement with no promise behind it ([`Promise::CentroidOffset`]).
    Reported,
}

/// The claims table, SoA, rows in call order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Claims {
    pub call: Vec<u16>,
    pub constraint: Vec<ConstraintId>,
    pub rule: Vec<RuleKind>,
    pub promised: Vec<Promise>,
    pub status: Vec<Status>,
    /// The scorer's number: Δ/tolerance for coincidence, the worst residual
    /// at the promised order, Φ residual, environment usage, centroid offset
    /// in nm, unit-weight spread. `NaN` when unknown.
    pub measured: Vec<f32>,
}

impl Claims {
    #[must_use]
    pub fn len(&self) -> usize {
        self.call.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.call.is_empty()
    }
    fn push(&mut self, call: u16, constraint: ConstraintId, promised: Promise, (status, measured): (Status, f32)) {
        self.call.push(call);
        self.constraint.push(constraint);
        self.rule.push(promised.rule());
        self.promised.push(promised);
        self.status.push(status);
        self.measured.push(measured);
    }
}

impl std::fmt::Display for Claims {
    /// One line per row: `claim call 0 c3: CentroidGroup MomentsCancelled(1) Met (0)`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.len() {
            let c = self.constraint[i];
            let id = if c == NO_INTENT { "-".to_string() } else { c.0.to_string() };
            let call = if self.call[i] == FLOORPLAN { "floorplan".to_string() } else { format!("call {}", self.call[i]) };
            writeln!(f, "claim {call} c{id}: {:?} {:?} {:?} ({})", self.rule[i], self.promised[i], self.status[i], self.measured[i])?;
        }
        Ok(())
    }
}

impl Promise {
    /// The rule that scores it.
    #[must_use]
    pub fn rule(self) -> RuleKind {
        match self {
            Promise::ParallelChannels | Promise::PhiEqual => RuleKind::OrientationSet,
            Promise::UnitIdentity | Promise::CentroidOffset | Promise::Coincidence => RuleKind::MatchedSet,
            Promise::MomentsCancelled(_) => RuleKind::CentroidGroup,
            Promise::EqualEnvironment => RuleKind::Environment,
            Promise::OrderMonotone(_) => RuleKind::Order,
        }
    }
}

/// What a call of `kind` promises about its set (`RECIPES.md` §2.1).
#[must_use]
pub fn claims_of(kind: RecipeKind) -> &'static [Promise] {
    match kind {
        RecipeKind::MosAdjacent => &[Promise::ParallelChannels, Promise::UnitIdentity, Promise::CentroidOffset],
        RecipeKind::MosCc1d => &[Promise::Coincidence, Promise::MomentsCancelled(1), Promise::PhiEqual, Promise::EqualEnvironment],
        RecipeKind::MosCc2d => &[Promise::MomentsCancelled(2), Promise::PhiEqual],
        // RECIPES.md: LOD equality (the environment), Φ equality; not
        // common-centroid, so the offset is reported. ponytail: R_s equality
        // has no scorer yet; it lands with the path-R claim (Q29).
        RecipeKind::MosInterdigitated => &[Promise::PhiEqual, Promise::EqualEnvironment, Promise::CentroidOffset],
        // RECIPES.md: coincidence per member, equal unit; Φ equality as the
        // MOD Current row's OrientationSet. ponytail: "no diffusion break"
        // has no kernel/analog rule to score it; it lands with one.
        RecipeKind::MosRatioed => &[Promise::Coincidence, Promise::UnitIdentity, Promise::PhiEqual, Promise::EqualEnvironment],
    }
}

/// Declare every multi-member call's claims and score each on `tiles` placed
/// by `layout` (`arrange`'s, one cell per call). A single device claims
/// nothing. `tiles` are in their own frames, as `build_tiles` returns them.
/// A row stack (`layout` has an axis; the glue row has none) adds one
/// [`Promise::OrderMonotone`] row per vertical order ([`order_verdicts`]).
///
/// # Panics
/// `tiles` or `layout` not one per call.
#[must_use]
pub fn score_claims(calls: &RecipeCalls, tiles: &[Macro], layout: &Layout, problem: &annotator::Problem, pdk: &Pdk) -> Claims {
    assert_eq!(tiles.len(), calls.len(), "one tile per call");
    assert_eq!(layout.x.len(), calls.len(), "one placed cell per call");
    assert!(calls.len() <= usize::from(u16::MAX), "calls are u16 handles");
    let l = with_units(calls, tiles, layout);
    let geo = env_geo(tiles, pdk);
    let tol_nm = substrate3::cells::builder::cut_lattice(pdk).max(1) as f32 / 2.0;
    let mut out = Claims::default();
    for (i, tile) in tiles.iter().enumerate() {
        let members = calls.members_of(i);
        if members.len() < 2 {
            continue;
        }
        let class =
            calls.unitization[i].and_then(|u| problem.constraints.unitization[usize::from(u)].class).unwrap_or(analog::intent::MatchClass::Moderate);
        let constraint = intent_of(problem, members);
        for &p in claims_of(calls.kind[i]) {
            let scored = match p {
                Promise::ParallelChannels => orientation(&l, members, OrientCheck::Axis),
                Promise::PhiEqual => orientation(&l, members, OrientCheck::Phi),
                Promise::UnitIdentity => unit_identity(tile, members.len()),
                Promise::CentroidOffset | Promise::Coincidence => matched(&l, members, tol_nm, p),
                Promise::MomentsCancelled(n) => moments(&l, members, n),
                Promise::EqualEnvironment => environment(&l, members, i as u16, geo.clone(), class),
                Promise::OrderMonotone(_) => unreachable!("the floorplan's promise, no set recipe's"),
            };
            out.push(i as u16, constraint, p, scored);
        }
    }
    if !layout.axis.is_empty() {
        for (k, status, overlap_nm) in order_verdicts(calls, problem, layout) {
            out.push(FLOORPLAN, NO_INTENT, Promise::OrderMonotone(k), (status, overlap_nm as f32));
        }
    }
    assert!(out.status.iter().zip(&out.promised).all(|(&s, &p)| (s == Status::Reported) == (p == Promise::CentroidOffset)));
    out
}

/// Each vertical `Order` of two or more steps, scored on `layout` (cell `i`
/// is call `i`) by [`Below`] over every device pair of consecutive steps:
/// its index in `intent.order`, the verdict, and the worst overlap in nm (≤ 0
/// kept). A device in no call is not scored.
///
/// # Panics
/// More than 65 535 orders, or `layout` with fewer cells than `calls`.
#[must_use]
pub fn order_verdicts(calls: &RecipeCalls, problem: &annotator::Problem, layout: &Layout) -> Vec<(u16, Status, i32)> {
    assert!(problem.intent.order.len() <= usize::from(u16::MAX), "orders are u16-indexed");
    assert!(layout.x.len() >= calls.len(), "one placed cell per call");
    let n_devices = calls.members.iter().map(|d| usize::from(d.0) + 1).max().unwrap_or(0);
    let mut cell_of = vec![u16::MAX; n_devices];
    for c in 0..calls.len() {
        calls.members_of(c).iter().for_each(|d| cell_of[usize::from(d.0)] = c as u16);
    }
    let in_call = |d: &&DeviceId| cell_of.get(usize::from(d.0)).is_some_and(|&c| c != u16::MAX);
    let mut out = Vec::new();
    for (k, o) in problem.intent.order.iter().enumerate().filter(|(_, o)| o.dir == analog::intent::AxisDir::V && o.steps.len() >= 2) {
        let batch: Vec<Below> = o
            .steps
            .windows(2)
            .flat_map(|w| {
                let upper = &w[1];
                w[0].iter().filter(in_call).flat_map(move |&a| upper.iter().filter(in_call).map(move |&b| (a, b)))
            })
            .map(|(a, b)| Below { lower: Target::Device(a), upper: Target::Device(b) }.retarget(&cell_of))
            .collect();
        let worst = batch.iter().map(|b| b.overlap_nm(layout)).max().unwrap_or(i32::MIN);
        let (status, _) = verdict(&batch, layout, worst as f32);
        out.push((k as u16, status, worst));
    }
    out
}

// ---- private ----

/// The intent set drawing exactly `members`, else [`NO_INTENT`].
fn intent_of(problem: &annotator::Problem, members: &[DeviceId]) -> ConstraintId {
    let mut want: Vec<u16> = members.iter().map(|d| d.0).collect();
    want.sort_unstable();
    let same = |s: &&analog::intent::MatchSpec| {
        let mut have: Vec<u16> = s.members.iter().map(|m| m.device.0).collect();
        have.sort_unstable();
        have == want
    };
    problem.intent.sets.iter().find(same).map_or(NO_INTENT, |s| s.id)
}

/// `layout` with every tile's units: cell `i` is call `i`, one variant each.
fn with_units(calls: &RecipeCalls, tiles: &[Macro], layout: &Layout) -> Layout {
    let n_devices = calls.members.iter().map(|d| usize::from(d.0) + 1).max().unwrap_or(0);
    let mut cell_of = vec![u16::MAX; n_devices];
    let members: Vec<Vec<DeviceId>> = (0..calls.len()).map(|i| calls.members_of(i).to_vec()).collect();
    for (c, m) in members.iter().enumerate() {
        m.iter().for_each(|d| cell_of[usize::from(d.0)] = c as u16);
    }
    let alts: Vec<[(Rect, &[analog::base::Unit]); 1]> = tiles.iter().map(|m| [(m.bbox, &m.units[..])]).collect();
    let units = UnitLib::build(cell_of, &members, alts.iter().map(|a| &a[..]));
    Layout {
        x: layout.x.clone(),
        y: layout.y.clone(),
        hw: layout.hw.clone(),
        hh: layout.hh.clone(),
        orient: layout.orient.clone(),
        variant: layout.variant.clone(),
        axis: layout.axis.clone(),
        branch: layout.branch.clone(),
        groups: layout.groups.clone(),
        power_uw: layout.power_uw.clone(),
        temp_mc: layout.temp_mc.clone(),
        units: Arc::new(units),
    }
}

/// Every tile's n-well and diffusion rects in its own frame. `None` when the
/// deck names neither layer: environment then reads unknown.
fn env_geo(tiles: &[Macro], pdk: &Pdk) -> Option<Arc<EnvGeo>> {
    let (nwell, diff) = (pdk.layer("nwell")?, pdk.layer("diff")?);
    let on = |m: &Macro, layer| vec![m.shapes.iter().filter(|s| s.layer == layer).map(|s| s.rect).collect::<Vec<_>>()];
    Some(Arc::new(EnvGeo {
        bbox: tiles.iter().map(|m| vec![m.bbox]).collect(),
        wells: tiles.iter().map(|m| on(m, nwell)).collect(),
        diffs: tiles.iter().map(|m| on(m, diff)).collect(),
    }))
}

/// A batch's verdict: violated over unknown over met.
fn verdict(b: &dyn RuleBatch<Layout>, l: &Layout, measured: f32) -> (Status, f32) {
    if b.violations(l) > 0 {
        (Status::Violated, measured)
    } else if b.unknown(l) > 0 {
        (Status::Unknown, f32::NAN)
    } else {
        (Status::Met, measured)
    }
}

fn orientation(l: &Layout, members: &[DeviceId], check: OrientCheck) -> (Status, f32) {
    let set = OrientationSet { members: members.to_vec(), check, cell_of: Vec::new() };
    verdict(&set, l, set.residual(l) as f32)
}

/// Equal weights over every member's units; unknown when the tile drew none.
fn unit_identity(tile: &Macro, members: usize) -> (Status, f32) {
    let w = || tile.units.iter().filter(|u| usize::from(u.owner) < members).map(|u| u.weight);
    let (Some(lo), Some(hi)) = (w().min(), w().max()) else { return (Status::Unknown, f32::NAN) };
    let spread = if hi > 0 { (hi - lo) as f32 / hi as f32 } else { 0.0 };
    (if lo == hi { Status::Met } else { Status::Violated }, spread)
}

/// [`MatchedSet`]'s ledger per pair `(0, i)`: the worst coincidence (Δ over
/// the tolerance), or the worst centroid offset in nm for the reported row.
/// The coefficients do not enter either number.
fn matched(l: &Layout, members: &[DeviceId], tol_nm: f32, p: Promise) -> (Status, f32) {
    use analog::matching::{class::Family, mismatch};
    let set = MatchedSet::for_family(
        members.to_vec(),
        Family::Mos,
        mismatch::MatchKind::Voltage,
        analog::matching::class::MatchClass::Moderate,
        mismatch::Coeffs::default(),
        mismatch::Budget::Eta(mismatch::GRADIENT_SHARE),
        vec![0.0; members.len()],
        tol_nm,
    );
    let ledgers: Vec<_> = (1..members.len()).map(|i| set.ledger(l, i)).collect();
    if p == Promise::CentroidOffset {
        return (Status::Reported, ledgers.iter().map(|g| g.delta_m_nm).fold(0.0, f32::max));
    }
    let Some(worst) = ledgers.iter().map(|g| g.coincidence).collect::<Option<Vec<f32>>>().map(|c| c.into_iter().fold(0.0, f32::max)) else {
        return (Status::Unknown, f32::NAN);
    };
    (if worst <= 1.0 { Status::Met } else { Status::Violated }, worst)
}

/// [`cancelled_order`] over the members' placed units, relative tolerance
/// 1e-3 (as `MatchedSet`'s ledger rows).
fn moments(l: &Layout, members: &[DeviceId], n: u8) -> (Status, f32) {
    assert!((1..=4).contains(&n), "cancelled_order reads orders 1..=4");
    let pts: Vec<Vec<Pt>> = members.iter().map(|&d| l.units.of_device(l, d).map(Pt::from).collect()).collect();
    if pts.iter().any(Vec::is_empty) {
        return (Status::Unknown, f32::NAN);
    }
    let sets: Vec<&[Pt]> = pts.iter().map(Vec::as_slice).collect();
    let (order, r) = cancelled_order(&sets, n, 1e-3);
    (if order >= n { Status::Met } else { Status::Violated }, r[usize::from(n)] as f32)
}

/// [`LiveEnvironment`] over pairs `(0, i)` of one call (cell `cell`), at the
/// class's WPE clearance and LOD moat (`matching::class::mos_env`).
fn environment(l: &Layout, members: &[DeviceId], cell: u16, geo: Option<Arc<EnvGeo>>, class: analog::intent::MatchClass) -> (Status, f32) {
    let Some(geo) = geo else { return (Status::Unknown, f32::NAN) };
    let env = analog::matching::class::mos_env(class);
    let live = LiveEnvironment {
        pairs: members[1..].iter().map(|&m| (members[0], m, cell, cell)).collect(),
        geo,
        wpe_min_nm: env.wpe_nm as f32,
        ose_range_nm: env.moat_nm as f32,
    };
    verdict(&live, l, live.worst_usage(l).unwrap_or(f32::NAN))
}
