//! REL-07: guard rings by role. Rings go on injectors and aggressors, and on
//! victims only when they have a quiet return of their own (Hastings §14.2,
//! §12.2.9; Charbon §8.6.1 / App. D.3: a ring on a shared return can be
//! worse than none). Anything else gets no ring: the cells carry their own
//! taps (CELL-13).
//!
//! Contents:
//! - `pub enum Carrier`
//! - `pub struct RingInputs<'a>`
//! - `pub fn plan(i: &RingInputs) -> (Vec<GuardRingRequirement>, Vec<(&'static str, &'static str)>)`
//! - `pub(crate) fn plan_tagged(netlist: &Netlist, net_classes: &[NetClassification], intent: &Intent, aggressor: &[bool], victim: &[bool], cfg: &crate::AnnotationConfig) -> (Vec<GuardRingRequirement>, Vec<(&'static str, &'static str)>)`

use analog::base::ids::{DeviceId, NetId};
use analog::base::{DeviceKind, Netlist, SubstrateKind};
use analog::cell::{GuardRingRequirement, GuardRingType, RingRole};
use analog::intent::Intent;
use analog::metadata::NetClassification;

/// Which minority carrier a device injects into the substrate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Carrier {
    /// Electrons (an NMOS diffusion or NPN emitter in p-substrate).
    Electrons,
    /// Holes (a PMOS diffusion in an n-well).
    Holes,
}

/// Plain per-device data, so the policy is table-testable without ports or a PDK.
pub struct RingInputs<'a> {
    /// The circuit, for device kinds and bulk nets.
    pub netlist: &'a Netlist,
    /// EXT-23 Switching/Capacitive/ImpactIonization; fallback: touches a Clock-class net.
    pub aggressor: &'a [bool],
    /// EXT-23 victims (Moderate+ set members, Reference-gated devices).
    pub victim: &'a [bool],
    /// EXT-23 MinorityElectron/MinorityHole; all `None` without ports.
    pub injector: &'a [Option<Carrier>],
    /// The deck's substrate: on epi over low-resistivity substrate rings do not help.
    pub substrate: SubstrateKind,
    /// The victims' own ring return.
    pub quiet_ring_net: Option<NetId>,
    /// The supply an electron-collecting ring ties to.
    pub highest_supply: Option<NetId>,
    /// The Ground-class net of lowest `NetId`.
    pub ground: Option<NetId>,
    /// Deck `min_guard_ring_width`, nm.
    pub min_ring_width_nm: i32,
    /// Sidecar `ecgr_min_width_nm` (CELL-17); `None` on every shipped deck.
    pub ecgr_min_width_nm: Option<i32>,
    /// `cells::post_cell::drawable(Ecgr, ..)`.
    pub ecgr_drawable: bool,
    /// `cells::post_cell::drawable(Hcgr, ..)`.
    pub hcgr_drawable: bool,
    /// Sidecar `IsolatedTub` (GAP-14): members and tie net.
    pub tubs: &'a [(Vec<DeviceId>, NetId)],
    /// `cells::post_cell::drawable(Tub, ..)`.
    pub tub_drawable: bool,
}

/// At most one ring per device, the first matching row winning (plan-06
/// REL-07): 0 a member of user tub `k` (GAP-14) → a shareable `Tub { id: k }`
/// victim ring on its tie, when the deck draws a deep n-well (else
/// `("IsolatedTub", ..)` and the rows below); 1 electron injector → supply-tied `Ecgr`; 2 hole injector →
/// ground-tied `Hcgr`; 3/4 an injector whose collector cannot be drawn → a
/// majority tap on its bulk; 5 an aggressor (not a victim) → a shareable tap
/// on its bulk; 6 a victim, once some device got a row 1–5 ring, → a
/// shareable tap on `quiet_ring_net`; 7 anything else → none. Rows 5–6 are
/// skipped on `EpiOnLowRes` (rings give only 7–10 dB there). A device
/// without a `B` terminal gets no ring. Returns the rings and the
/// `("GuardRing", why)` notes for inputs that limited them.
#[must_use]
pub fn plan(i: &RingInputs) -> (Vec<GuardRingRequirement>, Vec<(&'static str, &'static str)>) {
    let (mut rings, mut missing) = (Vec::new(), Vec::new());
    let note = |m: &mut Vec<(&'static str, &'static str)>, why: &'static str| {
        if !m.contains(&("GuardRing", why)) {
            m.push(("GuardRing", why));
        }
    };
    let ring = |d: usize, ring_type, role, connection_net, min_width_nm, shareable| GuardRingRequirement {
        device: DeviceId(d as u16),
        ring_type,
        shareable,
        min_width_nm,
        max_ring_resistance_mohm: 100_000,
        connection_net,
        role,
    };
    let epi = i.substrate == SubstrateKind::EpiOnLowRes;
    let w = i.min_ring_width_nm;
    let mut victims = Vec::new();
    for (d, dev) in i.netlist.devices.iter().enumerate() {
        let Some(&(_, bulk)) = dev.terminals.iter().find(|(t, _)| t == "B") else { continue };
        let pmos = dev.kind == DeviceKind::Pmos;
        if let Some((k, (_, tie))) = i.tubs.iter().enumerate().find(|(_, (m, _))| m.contains(&DeviceId(d as u16))) {
            if i.tub_drawable {
                rings.push(ring(d, GuardRingType::Tub { id: k as u16 }, RingRole::Victim, *tie, w, true));
                continue;
            }
            let why = ("IsolatedTub", "deck has no deep n-well: tub drawn as an ordinary ring");
            if !missing.contains(&why) {
                missing.push(why);
            }
        }
        let r = match i.injector.get(d).copied().flatten() {
            Some(Carrier::Electrons) => match i.highest_supply.filter(|_| i.ecgr_drawable) {
                Some(vdd) => {
                    if i.ecgr_min_width_nm.is_none() {
                        note(&mut missing, "ECGR width rule not given: collection efficiency unknown");
                    }
                    Some(ring(d, GuardRingType::Ecgr, RingRole::Injector, vdd, w.max(i.ecgr_min_width_nm.unwrap_or(0)), false))
                }
                None => {
                    if i.ecgr_drawable {
                        note(&mut missing, "injector ring: no supply/ground net");
                    }
                    note(&mut missing, "ECGR not drawable: electron injector has a majority ring only");
                    Some(ring(d, GuardRingType::Tap { in_well: false }, RingRole::Injector, bulk, w, false))
                }
            },
            Some(Carrier::Holes) => match i.ground.filter(|_| i.hcgr_drawable) {
                Some(gnd) => Some(ring(d, GuardRingType::Hcgr, RingRole::Injector, gnd, w, false)),
                None => {
                    if i.hcgr_drawable {
                        note(&mut missing, "injector ring: no supply/ground net");
                    }
                    Some(ring(d, GuardRingType::Tap { in_well: true }, RingRole::Injector, bulk, w, false))
                }
            },
            None if epi => None,
            None if i.victim.get(d).copied().unwrap_or(false) => {
                victims.push((d, pmos));
                None
            }
            None if i.aggressor.get(d).copied().unwrap_or(false) => {
                Some(ring(d, GuardRingType::Tap { in_well: pmos }, RingRole::Aggressor, bulk, w, true))
            }
            None => None,
        };
        rings.extend(r);
    }
    // Row 6: victims ring only against a device that got a row 1–5 ring.
    if !rings.is_empty() && !victims.is_empty() {
        match i.quiet_ring_net {
            Some(q) => rings.extend(victims.into_iter().map(|(d, pmos)| ring(d, GuardRingType::Tap { in_well: pmos }, RingRole::Victim, q, w, true))),
            None => note(&mut missing, "victim rings: no quiet ring return (SUB-30)"),
        }
    }
    (rings, missing)
}

/// The guard rings by role from the EXT-23 substrate tags (REL-07): injector
/// carriers from the aggressors, the quiet ring net (the sidecar's, else an
/// analog ground no aggressor touches), the highest supply by DC level.
/// Returns the rings and the missing-rule notes, an `IsolatedTub` note first
/// when a tub's tie is no supply or ground net (GAP-14).
pub(crate) fn plan_tagged(
    netlist: &Netlist,
    net_classes: &[NetClassification],
    intent: &Intent,
    aggressor: &[bool],
    victim: &[bool],
    cfg: &crate::AnnotationConfig,
) -> (Vec<GuardRingRequirement>, Vec<(&'static str, &'static str)>) {
    use analog::metadata::NetClass;
    let of_class = |c: NetClass| net_classes.iter().filter(move |k| k.class == c).map(|k| k.net);
    let mut injector = vec![None; netlist.devices.len()];
    for a in &intent.aggressors {
        match a.inject {
            analog::intent::Inject::MinorityElectron => injector[a.device.0 as usize] = Some(Carrier::Electrons),
            analog::intent::Inject::MinorityHole => injector[a.device.0 as usize] = Some(Carrier::Holes),
            _ => {}
        }
    }
    let touched_by_aggressor = |n: NetId| netlist.devices.iter().zip(aggressor).any(|(d, &a)| a && d.terminals.iter().any(|t| t.1 == n));
    let name = |n: NetId| netlist.nets[n.0 as usize].name.to_lowercase();
    let quiet_ring_net = match &cfg.quiet_ring_net {
        Some(q) => (0..netlist.nets.len()).map(|n| NetId(n as u16)).find(|&n| name(n) == q.to_lowercase()),
        None => {
            of_class(NetClass::Ground).filter(|&n| ["avss", "vssa", "agnd"].iter().any(|k| name(n).contains(k))).find(|&n| !touched_by_aggressor(n))
        }
    };
    let p = &cfg.process;
    let mut tub_notes = Vec::new();
    // GAP-14: the tie is a quiet supply; any other net is drawn as asked and reported.
    if cfg.tubs.iter().any(|(_, t)| !of_class(NetClass::Supply).chain(of_class(NetClass::Ground)).any(|n| n == *t)) {
        tub_notes.push(("IsolatedTub", "tie is not a supply net"));
    }
    let (rings, notes) = plan(&RingInputs {
        netlist,
        aggressor,
        victim,
        injector: &injector,
        substrate: p.substrate,
        quiet_ring_net,
        // The Supply net at the highest known DC level; the lowest id
        // when the op point gives none.
        highest_supply: of_class(NetClass::Supply).max_by_key(|n| (intent.nets[n.0 as usize].dc_mv.map(|v| v.0), std::cmp::Reverse(n.0))),
        ground: of_class(NetClass::Ground).min_by_key(|n| n.0),
        min_ring_width_nm: p.min_ring_width_nm,
        ecgr_min_width_nm: p.ecgr_min_width_nm,
        ecgr_drawable: p.ecgr_drawable,
        hcgr_drawable: p.hcgr_drawable,
        tubs: &cfg.tubs,
        tub_drawable: p.tub_drawable,
    });
    tub_notes.extend(notes);
    (rings, tub_notes)
}
