//! MOS-Ratioed, the Moderate Current rung (`RECIPES.md` §2.1, Q19): matched
//! MOS sets the annotator left without a unitization row → Ratioed calls, and
//! a call's members → the one-unit unitization it draws (1.0
//! `sets::unitize`). The members (one kind, model and L) are drawn from one
//! unit finger in Cc1d's diffusion-legal centroid row with end dummies (1.0
//! `diffusion_cc_row`); the row is a palindrome, so every member's centroid
//! sits on its centre. Counts with no such row (an odd member at the gcd unit
//! whose half the deck cannot draw, or no diffusion-legal order) draw nothing
//! and the ladder steps down.
//!
//! Contents:
//! - `pub(crate) fn plan(problem: &annotator::Problem, flat: &Flat, pdk: &Pdk, calls: &mut RecipeCalls, taken: &mut [bool])`
//! - `pub(crate) fn unitize(flat: &Flat, members: &[DeviceId], base: Option<&analog::cell::Unitization>, pdk: &Pdk, process: &dyn analog::base::Process) -> Option<analog::Constraints>`

use analog::base::netlist::{Device, Netlist as Flat};
use analog::base::{DeviceId, DeviceKind};
use verify::Pdk;

use crate::{RecipeCalls, RecipeKind};

/// Appends one Ratioed call per matched MOS set of two or more members that
/// has no annotator row (its members' fingers differ), whose ladder holds
/// Ratioed, and that Ratioed draws on this deck; marks its members `taken`.
/// Only Ratioed draws members of unequal fingers, so no other rung is tried:
/// a set it cannot draw stays for the later passes.
pub(crate) fn plan(problem: &annotator::Problem, flat: &Flat, pdk: &Pdk, calls: &mut RecipeCalls, taken: &mut [bool]) {
    for s in &problem.intent.sets {
        let members: Vec<DeviceId> = s.members.iter().map(|m| m.device).collect();
        let all_mos = members.iter().all(|d| matches!(flat.devices[d.0 as usize].kind, DeviceKind::Nmos | DeviceKind::Pmos));
        let has_row =
            problem.constraints.unitization.iter().any(|u| u.devices.len() == members.len() && u.devices.iter().all(|d| members.contains(d)));
        if !all_mos || has_row || members.len() < 2 || members.iter().any(|d| taken[d.0 as usize]) {
            continue;
        }
        let Some(step) = crate::ladder(Some(s.class), Some(s.kind)).iter().position(|&k| k == RecipeKind::MosRatioed) else { continue };
        let mos = crate::Mos::new(flat, &members, pdk);
        let process = mos.get();
        let draws = unitize(flat, &members, None, pdk, process)
            .is_some_and(|c| !crate::mos_variants(RecipeKind::MosRatioed, flat, &members, &c, process).is_empty());
        if !draws {
            continue;
        }
        members.iter().for_each(|d| taken[d.0 as usize] = true);
        calls.push(RecipeKind::MosRatioed, &members, None);
        *calls.ladder_step.last_mut().expect("just pushed") = step as u8;
    }
}

/// MOS-Ratioed's unitization: every member drawn from one unit finger, the
/// gcd of the members' fingers folded to the tap reach, member `k` as that
/// unit's integer count of its own total W, with end dummies. A member with
/// an odd count would put a device boundary on a drain, so then the unit
/// halves, every count even. `base` (the annotator's row) lends its class and
/// route matching.
///
/// `None`, never a rounded or resized unit: members of another kind, model or
/// L, or a unit (or the half) off the grid or below what the deck draws
/// ([`crate::drawable_size`]).
///
/// # Panics
/// `members` is empty.
pub(crate) fn unitize(
    flat: &Flat,
    members: &[DeviceId],
    base: Option<&analog::cell::Unitization>,
    pdk: &Pdk,
    process: &dyn analog::base::Process,
) -> Option<analog::Constraints> {
    assert!(!members.is_empty(), "a call has members");
    let devs: Vec<&Device> = members.iter().map(|d| &flat.devices[d.0 as usize]).collect();
    let sizes: Vec<analog::base::MosSize> = devs.iter().map(|d| d.mos_size()).collect::<Option<_>>()?;
    let (d0, l) = (devs[0], sizes[0].l_nm);
    if devs.iter().zip(&sizes).any(|(d, s)| d.kind != d0.kind || d.model != d0.model || s.l_nm != l) {
        return None;
    }
    let total: Vec<i64> = sizes.iter().map(|s| s.w_finger_nm() * i64::from(s.fingers())).collect();
    let g = sizes.iter().map(|s| s.w_finger_nm() / i64::from(crate::fold(s.w_finger_nm(), l, process))).fold(0, gcd);
    let odd = total.iter().any(|t| (t / g.max(1)) % 2 == 1);
    let unit = if odd { g / 2 } else { g };
    if (odd && g % 2 != 0) || unit <= 0 || unit % i64::from(process.grid().max(1)) != 0 || crate::drawable_finger(unit, l, d0, pdk).is_err() {
        return None;
    }
    let dev_nf: Vec<u16> = total.iter().map(|t| u16::try_from(t / unit).ok()).collect::<Option<_>>()?;
    assert!(total.iter().zip(&dev_nf).all(|(t, &n)| i64::from(n) * unit == *t), "every member is a whole number of units");
    let r = dev_nf.iter().fold(0, |a, &n| gcd(a, i64::from(n))) as u16;
    Some(analog::Constraints {
        unitization: vec![analog::cell::Unitization {
            devices: members.to_vec(),
            device_type: d0.kind,
            target_ratio: dev_nf.iter().map(|&n| n / r).collect(),
            dev_nf,
            unit_w: i32::try_from(unit).ok()?,
            unit_l: i32::try_from(l).ok()?,
            series_parallel: analog::cell::SeriesParallel::Parallel,
            dummy_required: true,
            route_matching_required: base.is_some_and(|u| u.route_matching_required),
            class: base.and_then(|u| u.class),
            kind: base.and_then(|u| u.kind),
            series: Vec::new(),
            style: base.and_then(|u| u.style),
        }],
        ..analog::Constraints::default()
    })
}

// ---- private ----

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}
