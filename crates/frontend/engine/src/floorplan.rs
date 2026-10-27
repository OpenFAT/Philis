//! The floorplan choice for a block (`arrange`'s caller): the symmetric row
//! stack when the annotator states mirror pairs on a planar deck, else the
//! glue row ([`crate::arrange`]). Rows follow the annotator's vertical
//! `Order`s (EXT-28 current chains, sidecar orders; RECIPES §3).
//!
//! Contents:
//! - `pub fn stack_of(calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat) -> Option<Stack>`
//! - `pub fn targets(calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat, bounds: (i32, i32), pdk: &Pdk) -> placement::floorplan::Targets`
//! - `pub fn arrange_block(tiles: &mut Vec<Macro>, calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat, pdk: &Pdk) -> Result<Layout, Refusal>`

use analog::base::netlist::Netlist as Flat;
use analog::base::{DeviceId, DeviceKind, Layout, Macro};
use analog::intent::AxisDir;
use placement::floorplan::{Side, Stack};
use verify::Pdk;

use crate::{refuse, RecipeCalls, Refusal};

/// The stack for `calls`, or `None` when no stated pair spans the block
/// (nothing to mirror: the glue row places it).
///
/// A pair inside one call makes that tile self-symmetric (`Axis`); a pair
/// across two calls of equal size makes them a `Left`/`Right` pair in one
/// row. Every other tile sits on the axis. Rows follow the annotator's
/// vertical `Order`s (RECIPES §3): a call ranks at its step in the first
/// such order naming one of its devices, `steps[0]` at the bottom; calls no
/// order names come last. Ties go NMOS first, then PMOS (one well region),
/// then call order; each axis tile is its own row, a pair shares the row of
/// whichever half comes first. Whether the rows keep each order is scored
/// after placement ([`crate::claims::order_verdicts`]), never assumed here.
/// ponytail: every order's step `k` is rank `k`, one scale for all chains,
/// and reversible orders keep the sense they were emitted in.
#[must_use]
pub fn stack_of(calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat) -> Option<Stack> {
    let call_of = |d: u32| (0..calls.len()).find(|&i| calls.members_of(i).contains(&DeviceId(d as u16)));
    let mut side = vec![Side::Axis; calls.len()];
    let mut partner: Vec<Option<usize>> = vec![None; calls.len()];
    let mut any = false;
    for (a, b) in mirror_pairs(problem) {
        let (Some(ca), Some(cb)) = (call_of(a), call_of(b)) else { continue };
        any = true;
        if ca != cb && partner[ca].is_none() && partner[cb].is_none() && calls.member_count[ca] == calls.member_count[cb] {
            let (l, r) = (ca.min(cb), ca.max(cb));
            (side[l], side[r]) = (Side::Left, Side::Right);
            (partner[l], partner[r]) = (Some(r), Some(l));
        }
    }
    if !any {
        return None;
    }
    // u8::MAX: no order names the call. ponytail: a step past 254 ranks 254.
    let mut rank = vec![u8::MAX; calls.len()];
    for o in problem.intent.order.iter().filter(|o| o.dir == AxisDir::V) {
        for (k, step) in o.steps.iter().enumerate() {
            for c in step.iter().filter_map(|d| call_of(u32::from(d.0))) {
                if rank[c] == u8::MAX {
                    rank[c] = k.min(254) as u8;
                }
            }
        }
    }
    let pmos = |i: usize| flat.devices[calls.members_of(i)[0].0 as usize].kind == DeviceKind::Pmos;
    let mut order: Vec<usize> = (0..calls.len()).collect();
    order.sort_by_key(|&i| (rank[i], pmos(i), i));
    let mut row: Vec<Option<u8>> = vec![None; calls.len()];
    let mut next = 0u8;
    for &i in &order {
        if let Some(r) = partner[i].and_then(|p| row[p]) {
            row[i] = Some(r);
            continue;
        }
        row[i] = Some(next);
        next = next.saturating_add(1);
    }
    let row: Vec<u8> = row.into_iter().map(|r| r.expect("every call gets a row")).collect();
    assert!((0..calls.len()).all(|i| partner[i].is_none_or(|p| row[p] == row[i])), "a pair's halves share a row");
    Some(Stack { row, side })
}

/// Each call's top-down bounds inside a `bounds` block
/// ([`placement::floorplan::targets`]), before any tile is drawn: over
/// [`stack_of`]'s rows, or one row of every call where it finds none (the
/// glue row). Tiles keep the widest spacing any two layer roles need apart.
#[must_use]
pub fn targets(calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat, bounds: (i32, i32), pdk: &Pdk) -> placement::floorplan::Targets {
    let stack = stack_of(calls, problem, flat).unwrap_or_else(|| Stack { row: vec![0; calls.len()], side: vec![Side::Axis; calls.len()] });
    let roles: Vec<&str> = pdk.roles.iter().map(|(r, _)| r.as_str()).collect();
    let gap = placement::floorplan::Spacing::of(pdk, &roles).nm.into_iter().max().unwrap_or(0);
    placement::floorplan::targets(&stack, bounds, gap)
}

/// The floorplan: the symmetric row stack where [`stack_of`] finds one on a
/// planar deck, else the glue row ([`crate::arrange`]).
///
/// # Errors
/// `floorplan`: the row stack or the glue row refuses, or the placed rows
/// break a user `Order` (sidecar, not reversible; RECIPES §3). An extracted
/// order they break is a violated claim instead ([`crate::claims::score_claims`]).
pub fn arrange_block(tiles: &mut Vec<Macro>, calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat, pdk: &Pdk) -> Result<Layout, Refusal> {
    let fin = analog::base::Process::layer(pdk, "fin").is_some();
    let layout = match stack_of(calls, problem, flat).filter(|_| !fin && tiles.len() > 1) {
        Some(stack) => {
            let roles: Vec<&str> = pdk.roles.iter().map(|(r, _)| r.as_str()).collect();
            let spacing = placement::floorplan::Spacing::of(pdk, &roles);
            let lattice = substrate3::cells::builder::cut_lattice(pdk).max(1);
            let f = placement::floorplan::row_stack(tiles, &stack, &spacing, lattice).map_err(|e| refuse("floorplan", format!("{e:?}")))?;
            f.layout()
        }
        None => crate::arrange(tiles, pdk)?,
    };
    let broken = crate::claims::order_verdicts(calls, problem, &layout)
        .into_iter()
        .find(|&(k, status, _)| status == crate::claims::Status::Violated && !problem.intent.order[usize::from(k)].reversible);
    if let Some((k, _, overlap_nm)) = broken {
        return Err(refuse("floorplan", format!("order {k}: a step overlaps the next by {overlap_nm} nm; the rows cannot keep this user order")));
    }
    Ok(layout)
}

// ---- private ----

/// Every mirror pair `(device a, device b)` the annotator's placement
/// requirements state.
fn mirror_pairs(problem: &annotator::Problem) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let p = &problem.placement;
    for batch in p.hard.iter().chain(&p.budget).chain(&p.cost) {
        batch.mirror_pairs(&mut out);
    }
    out.into_iter().map(|(a, b, _)| (a, b)).collect()
}
