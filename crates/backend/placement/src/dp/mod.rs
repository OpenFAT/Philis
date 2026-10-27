//! Detailed placement: bounded-move Metropolis SA over the coarse layout with
//! displacement, swap, rotate, variant-reshape and DTI-branch-flip moves.
//!
//! Every move goes through [`Sa::trial`]: a move that breaks a hard equality is
//! repaired by projecting the broken batch (a symmetry partner follows its
//! mirror), then gated lexicographically on `(violating hard batches, Σ hard
//! residual, clearance encroachment nm², Θ)`. Metropolis only votes on the PEX
//! tier (pin HPWL + priced analog cost). [`legalize::separate_overlaps`] closes
//! any residue.
//!
//! Contents:
//! - `pub use anneal::{place_sp, DpMode, PlaceInput, Start}`
//! - `pub struct Schedule`
//! - `pub struct PlaceStats`
//! - `Schedule::cold() -> Self`
//! - `Schedule::warm() -> Self`
//! - `pub fn place(coarse: &Layout, macros: &[Macro], variants: &[crate::gp::VariantSpace], reqs: &Requirements<Layout>, fixed: &[bool], locks: &locks::Locks, prices: &mut crate::gp::Prices, rules: &crate::gp::PlaceRules, net_weight: &[f32], seed: u64, schedule: Schedule) -> (Layout, Report, PlaceStats)`

pub mod anneal;
mod eval;
pub mod legalize;
pub mod locks;
mod moves;
pub mod sp;

pub use anneal::{place_sp, DpMode, PlaceInput, Start};

use analog::base::ids::BranchId;
use analog::base::{Layout, Macro, Report};
use analog::Requirements;
use moves::{sym_groups, try_branch, try_group_shift, try_move, try_pair_expand, try_pair_swap, try_reshape, try_rotate, try_swap, Sa};

use crate::gp::mechanics::{analog_phi, choose_variants, report, snap, Nets, SplitMix64};

/// The anneal's schedule. Flat path (PLC-10 step 0): initial move window `range0` (fraction of the
/// die span), `max_temps` temperature steps, `t0 = t0_scale · mean|ΔPEX|` over 128 probe moves.
/// SP path (PLC-09, [`anneal::place_sp`]): `T0 = −mean(ΔE⁺)/ln p0`, cooling `alpha`,
/// `moves_per_kid · Σ kids` moves per temperature, at most `max_temps` temperatures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Schedule {
    /// Flat path: the first move window as a fraction of the die span, shrunk by `RANGE_DECAY` per temperature.
    pub range0: f32,
    /// Both paths: the cap on temperature steps.
    pub max_temps: u32,
    /// Flat path: `T0 = t0_scale · mean|ΔPEX|` over 128 probe moves.
    pub t0_scale: f64,
    /// SP path: the acceptance probability of a mean uphill move at `T0` (Lampaert eq 4.37), in `(0, 1)`; `0` starts at `T0 = 0`.
    pub p0: f64,
    /// SP path: geometric cooling factor per temperature, in `(0, 1)`.
    pub alpha: f64,
    /// SP path: moves per temperature per tree kid.
    pub moves_per_kid: u32,
}

/// What the anneal did, for measurement only: no counter feeds a decision, so
/// the placement is the same with or without anyone reading them.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlaceStats {
    /// Temperature steps (epochs) run.
    pub temps: u32,
    /// `Sa::trial` calls: every move that reached the gate.
    pub proposals: u64,
    /// Proposals kept.
    pub accepted: u64,
    /// Codes the decoder could not realise; `None` (not measured) until
    /// PLC-08 adds a decoder.
    pub decode_fail: Option<u64>,
    /// Distinct matched pairs whose variant spaces differ, so they cannot be
    /// shape-locked ([`locks::Locks::incompatible`]).
    pub matched_incompatible: Option<u32>,
    /// [`anneal::place_sp`]: a [`anneal::Start::Warm`] tree no longer fit the
    /// inputs (cells or symmetry groups changed), so the anneal started constructive.
    pub warm_fallback: bool,
    /// [`anneal::place_sp`]: symmetry axes placed, and of them those on a
    /// routing track centreline (`axis ≡ p0/2 mod P`, PLC-28; `0` without
    /// `PlaceRules::axis_grid`).
    pub axes: u32,
    /// Of `axes`, those on a routing track centreline.
    pub axes_on_lattice: u32,
}

/// Inner moves per epoch = `MOVES_PER_CELL · n`.
const MOVES_PER_CELL: usize = 60;

const ALPHA: f64 = 0.93;

const RANGE_DECAY: f32 = 0.96;

/// Clearance-inflated area / move-region area floor: the region the SA may use
/// is grown until everything fits at this fill.
const REGION_FILL: f64 = 0.5;

/// Runaway guard for the terminal legalizer (it exits early when clean/stalled).
const LEGALIZE_SWEEPS: u32 = 64;

impl Schedule {
    /// Today's constants: refine gp, don't randomise it. SP: P0 0.6 (Lampaert eq 4.37–4.38), α 0.9.
    pub fn cold() -> Self {
        Self { range0: 0.4, max_temps: 220, t0_scale: 0.02, p0: 0.6, alpha: 0.9, moves_per_kid: 20 }
    }
    /// FLOW-08 step 3's warm start from an incumbent [policy, measure]; SP: P0 0.1.
    pub fn warm() -> Self {
        Self { range0: 0.05, max_temps: 60, t0_scale: 0.002, p0: 0.1, alpha: 0.9, moves_per_kid: 20 }
    }
}

/// Refine `coarse` into a legal placement, seed-deterministic, with the
/// anneal's [`PlaceStats`].
///
/// `net_weight[NetId]` weights each net's HPWL (see [`crate::gp::net_weights`]).
/// `macros[i]` supplies cell `i`'s pins when `variants[i]` has no alternative
/// for `coarse.variant[i]`. `fixed[i]` draws cell `i` as given (no reshape, no
/// rotation); its position is placed like any cell's.
/// `locks` turns and reshapes matched cells as one set (PLC-03).
/// `schedule` sets the anneal's window, length and starting temperature
/// ([`Schedule::cold`] is the gp-refining default).
#[allow(clippy::too_many_arguments)]
pub fn place(
    coarse: &Layout,
    macros: &[Macro],
    variants: &[crate::gp::VariantSpace],
    reqs: &Requirements<Layout>,
    fixed: &[bool],
    locks: &locks::Locks,
    prices: &mut crate::gp::Prices,
    rules: &crate::gp::PlaceRules,
    net_weight: &[f32],
    seed: u64,
    schedule: Schedule,
) -> (Layout, Report, PlaceStats) {
    let grid = rules.grid;
    let n = coarse.x.len();
    let mut rng = SplitMix64::new(seed);
    let mut l = Layout {
        x: coarse.x.clone(),
        y: coarse.y.clone(),
        hw: coarse.hw.clone(),
        hh: coarse.hh.clone(),
        variant: coarse.variant.clone(),
        axis: coarse.axis.clone(),
        branch: coarse.branch.clone(),
        groups: coarse.groups.clone(),
        orient: coarse.orient.clone(),
        power_uw: coarse.power_uw.clone(),
        temp_mc: coarse.temp_mc.clone(),
        units: coarse.units.clone(),
    };
    l.refresh_temps();
    // PLC-21: seed Mirror partners as reflections of their set's first member;
    // extents follow any member whose axes the realignment swapped.
    if l.orient.len() == n {
        let before = l.orient.clone();
        locks.align(&mut l.orient);
        for (c, o) in before.iter().enumerate() {
            if o.swaps_axes() != l.orient[c].swaps_axes() {
                (l.hw[c], l.hh[c]) = (l.hh[c], l.hw[c]);
            }
        }
    }

    let branch_ids = seed_branches(reqs, &mut l.branch);
    let sym = sym_groups(reqs, n);

    prices.bind(reqs);
    // Nets from the geometry `l.variant` names, so HPWL scores real pins.
    let nets = Nets::from_macros(&choose_variants(macros, variants, &l.variant)).weigh(net_weight);
    if n == 0 {
        let rep = report(&nets, reqs, &l, prices, rules);
        return (l, rep, PlaceStats { matched_incompatible: Some(locks.incompatible), ..Default::default() });
    }

    // Move region: the coarse footprint bbox, grown about its centre until the
    // halo-inflated cells fit at `REGION_FILL`.
    let (mut xmin, mut ymin, mut xmax, mut ymax) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    let mut need = 0.0f64;
    for i in 0..n {
        xmin = xmin.min(l.x[i] - l.hw[i]);
        ymin = ymin.min(l.y[i] - l.hh[i]);
        xmax = xmax.max(l.x[i] + l.hw[i]);
        ymax = ymax.max(l.y[i] + l.hh[i]);
        let halo = rules.halo(&l, i);
        need += f64::from(2 * l.hw[i] + halo) * f64::from(2 * l.hh[i] + halo);
    }
    let side = (need / REGION_FILL).sqrt() as i32;
    let grow = |lo: &mut i32, hi: &mut i32| {
        let short = side - (*hi - *lo);
        if short > 0 {
            *lo -= short / 2;
            *hi += short - short / 2;
        }
    };
    grow(&mut xmin, &mut xmax);
    grow(&mut ymin, &mut ymax);
    let span = (xmax - xmin).max(ymax - ymin).max(1) as f32;
    let clamp_x = |c: i32, half: i32| c.clamp(xmin + half, (xmax - half).max(xmin + half));
    let clamp_y = |c: i32, half: i32| c.clamp(ymin + half, (ymax - half).max(ymin + half));

    let mut sa = Sa::new(nets, n, reqs, prices, fixed, rules);
    sa.stats.matched_incompatible = Some(locks.incompatible);

    // t0 = t0_scale · mean |ΔPEX| over probe moves.
    let mut range = schedule.range0;
    let probe_r = (range * span) as i32 as f32;
    let pex0 = sa.pex(&l);
    let mut sum = 0.0f64;
    for _ in 0..128 {
        let c = rng.below(n);
        let (ox, oy) = (l.x[c], l.y[c]);
        l.x[c] = clamp_x(ox + rng.centered(probe_r) as i32, l.hw[c]);
        l.y[c] = clamp_y(oy + rng.centered(probe_r) as i32, l.hh[c]);
        sum += (sa.pex(&l) - pex0).abs();
        (l.x[c], l.y[c]) = (ox, oy);
    }
    // No floor in PEX units (PLC-18: PEX is O(1)); the fallback only covers a
    // layout where no probe changes PEX.
    let mean = sum / 128.0;
    let mut temp = if mean > 0.0 { mean } else { 1.0 } * schedule.t0_scale;

    let can_rotate = l.orient.len() == n;
    let can_reshape = variants.len() == n && l.variant.len() == n;
    let moves_per_epoch = MOVES_PER_CELL * n;
    let range_min = grid as f32 / span;

    // A fixed schedule: stopping at a low accept rate (the old early exit)
    // cost ota 5% C and 24% area. Lampaert 1999 pp.117–119 (T0 = −ΔC⁺/ln 0.6,
    // stop after 10 flat chains) was measured 2026-09: ota C −12% but
    // rc_filter C +7%, bjt_mirror area +22%; not adopted.
    // ponytail: ~4x dp time; revisit if runtime binds.
    for _ in 0..schedule.max_temps {
        let r = (range * span) as i32 as f32;
        for _ in 0..moves_per_epoch {
            // 70% displace, 20% swap, 2.5% branch flip, 5% reshape, else rotate.
            let roll = rng.f32();
            let c = rng.below(n);
            let _ = if !sym.is_empty() && roll < 0.08 {
                // Compound moves: every mirror equation holds before and after,
                // so no projection drags the rest of the stage.
                let g = &sym[rng.below(sym.len())];
                let (a, b) = g.pairs[rng.below(g.pairs.len())];
                match rng.below(3) {
                    0 => {
                        let (dx, dy) = (snap(rng.centered(r) as i32, grid), snap(rng.centered(r) as i32, grid));
                        try_group_shift(&mut sa, &mut l, &mut rng, temp, g, dx, dy, &clamp_x, &clamp_y)
                    }
                    1 => {
                        let d = snap(rng.centered(r) as i32 / 2, grid);
                        try_pair_expand(&mut sa, &mut l, &mut rng, temp, a, b, d, &clamp_x)
                    }
                    _ => a != b && try_pair_swap(&mut sa, &mut l, &mut rng, temp, a, b),
                }
            } else if roll < 0.70 {
                let nx = clamp_x(l.x[c] + rng.centered(r) as i32, l.hw[c]);
                let ny = clamp_y(l.y[c] + rng.centered(r) as i32, l.hh[c]);
                try_move(&mut sa, &mut l, &mut rng, temp, c, nx, ny)
            } else if roll < 0.90 {
                let o = rng.below(n);
                o != c && try_swap(&mut sa, &mut l, &mut rng, temp, c, o, &clamp_x, &clamp_y)
            } else if !branch_ids.is_empty() && roll < 0.925 {
                let bid = usize::from(branch_ids[rng.below(branch_ids.len())].0);
                try_branch(&mut sa, &mut l, &mut rng, temp, bid)
            } else if can_reshape && roll >= 0.95 {
                try_reshape(&mut sa, &mut l, &mut rng, temp, &locks.members(c, true), variants, &clamp_x, &clamp_y)
            } else {
                let set = locks.members(c, false);
                can_rotate
                    && !set.iter().any(|&m| sa.is_fixed(m))
                    && try_rotate(&mut sa, &mut l, &mut rng, temp, &set, &locks.rel, &clamp_x, &clamp_y)
            };
        }

        // One exact projection per epoch for batches violated before any move.
        project_hard(reqs, &mut l, grid);
        // Thermal field is global: refresh per epoch, never per move.
        l.refresh_temps();

        temp *= ALPHA;
        range = (range * RANGE_DECAY).max(range_min);
        sa.stats.temps += 1;
    }

    for a in &mut l.axis {
        *a = snap(*a, grid);
    }
    for i in 0..n {
        l.x[i] = snap(l.x[i], grid);
        l.y[i] = snap(l.y[i], grid);
    }
    // Grid snap can shave a clearance by a few nm; the legalizer restores it.
    legalize::separate_overlaps(&mut l, reqs, rules, LEGALIZE_SWEEPS);
    l.refresh_temps();

    let Sa { nets, stats, .. } = sa;
    let rep = report(&nets, reqs, &l, prices, rules);
    (l, rep, stats)
}

// ---- private ----

/// Disjunctive branches (DtiBand share/isolate): grow `branch` to the highest
/// id and seed each from its recognised structure. Returns the ids, sorted.
fn seed_branches(reqs: &Requirements<Layout>, branch: &mut Vec<bool>) -> Vec<BranchId> {
    let mut branch_seeds: Vec<(BranchId, bool)> = Vec::new();
    for b in &reqs.hard {
        b.branches(&mut branch_seeds);
    }
    branch_seeds.sort_unstable_by_key(|&(id, _)| id.0);
    branch_seeds.dedup();
    if let Some(&(hi, _)) = branch_seeds.last() {
        if branch.len() <= usize::from(hi.0) {
            branch.resize(usize::from(hi.0) + 1, false);
        }
    }
    for &(id, s) in &branch_seeds {
        branch[usize::from(id.0)] = s;
    }
    branch_seeds.iter().map(|&(id, _)| id).collect()
}

/// Project every violated hard batch onto its feasible set and roll the whole
/// sweep back if Φ rose. Returns whether it kept.
fn project_hard(reqs: &Requirements<Layout>, l: &mut Layout, grid: i32) -> bool {
    let before = analog_phi(reqs, l);
    if before.0 == 0 {
        return false;
    }
    let (px, py, paxis) = (l.x.clone(), l.y.clone(), l.axis.clone());
    for batch in &reqs.hard {
        // Re-checked per batch: a satisfied SymmetryGroup would still re-average its axis.
        if batch.violations(l) > 0 {
            batch.project(l, grid);
        }
    }
    if analog_phi(reqs, l) > before {
        l.x = px;
        l.y = py;
        l.axis = paxis;
        return false;
    }
    l.refresh_temps();
    true
}

#[cfg(test)]
mod tests;
