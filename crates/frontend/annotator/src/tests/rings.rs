//! REL-07 guard-ring tests: hand-built netlists with tagged injectors and
//! victims → `rings::plan`'s rings and missing-rule notes.
//!
//! Contents: none public.

use analog::base::SubstrateKind;
use analog::cell::{GuardRingType, RingRole};

use super::*;
use crate::rings::{plan, Carrier, RingInputs};

const SUB30: (&str, &str) = ("GuardRing", "victim rings: no quiet ring return (SUB-30)");

// ---- private ----

/// M1 (NMOS, gate `clk`), M2/M3 (NMOS victims), M4 (PMOS). Nets: 0=clk
/// 1=x 2=vss 3=vssq 4=vdd.
fn nl() -> Netlist {
    Netlist {
        devices: vec![
            fet("M1", DeviceKind::Nmos, 0, 1, 2, 2, 1_000, 150),
            fet("M2", DeviceKind::Nmos, 1, 1, 2, 2, 1_000, 150),
            fet("M3", DeviceKind::Nmos, 1, 1, 2, 2, 1_000, 150),
            fet("M4", DeviceKind::Pmos, 1, 1, 4, 4, 1_000, 150),
        ],
        nets: nets(&["clk", "x", "vss", "vssq", "vdd"]),
        ..Default::default()
    }
}

fn inputs<'a>(nl: &'a Netlist, aggressor: &'a [bool], victim: &'a [bool], injector: &'a [Option<Carrier>]) -> RingInputs<'a> {
    RingInputs {
        netlist: nl,
        aggressor,
        victim,
        injector,
        substrate: SubstrateKind::Bulk,
        quiet_ring_net: None,
        highest_supply: Some(NetId(4)),
        ground: Some(NetId(2)),
        min_ring_width_nm: 420,
        ecgr_min_width_nm: None,
        ecgr_drawable: false,
        hcgr_drawable: false,
        tubs: &[],
        tub_drawable: false,
    }
}

/// GAP-14: tub members get the tub's ring on its tie; the rest keep their rows.
/// Oracle: GAP-14: the input's tub members, tied to net 4, by construction.
#[test]
fn tub_members_get_a_tub_ring() {
    let nl = nl();
    let tubs = [(vec![DeviceId(1), DeviceId(2)], NetId(4))];
    let i = RingInputs { tubs: &tubs, tub_drawable: true, ..inputs(&nl, &[true, false, false, false], &[false; 4], &[None; 4]) };
    let (rings, missing) = plan(&i);
    let tub: Vec<_> =
        rings.iter().filter(|r| r.ring_type == GuardRingType::Tub { id: 0 }).map(|r| (r.device, r.connection_net, r.shareable)).collect();
    assert_eq!(tub, [(DeviceId(1), NetId(4), true), (DeviceId(2), NetId(4), true)]);
    let other: Vec<_> = rings.iter().filter(|r| !matches!(r.ring_type, GuardRingType::Tub { .. })).map(|r| (r.device, r.ring_type)).collect();
    assert_eq!(other, [(DeviceId(0), GuardRingType::Tap { in_well: false })]);
    assert!(!missing.iter().any(|m| m.0 == "IsolatedTub"));
}

/// Oracle: the input: a deck with no deep n-well cannot draw a tub, so the note is owed.
#[test]
fn no_dnwell_falls_back_and_says_so() {
    let nl = nl();
    let tubs = [(vec![DeviceId(1), DeviceId(2)], NetId(4))];
    let (rings, missing) = plan(&RingInputs { tubs: &tubs, ..inputs(&nl, &[true, false, false, false], &[false; 4], &[None; 4]) });
    assert!(rings.iter().all(|r| !matches!(r.ring_type, GuardRingType::Tub { .. })));
    assert!(missing.contains(&("IsolatedTub", "deck has no deep n-well: tub drawn as an ordinary ring")), "{missing:?}");
}

/// Oracle: REL-07: no aggressor and no injector in the textbook 5T OTA, so no ring.
#[test]
fn an_ota_without_clocks_or_ports_gets_no_rings() {
    assert!(annotate(&ota(), &AnnotationConfig::default()).constraints.guard_rings.is_empty());
}

/// Oracle: REL-07's role table: an aggressor NMOS gets a tap ring on its bulk.
#[test]
fn clocked_devices_get_aggressor_tap_rings() {
    let nl = nl();
    let (rings, missing) = plan(&inputs(&nl, &[true, false, false, false], &[false, true, false, false], &[None; 4]));
    assert_eq!(rings.len(), 1);
    let r = &rings[0];
    assert_eq!(
        (r.device, r.ring_type, r.role, r.connection_net),
        (DeviceId(0), GuardRingType::Tap { in_well: false }, RingRole::Aggressor, NetId(2))
    );
    assert!(missing.contains(&SUB30), "{missing:?}");
}

/// Oracle: REL-07: a quiet ring net is given, so both victims get shareable rings on it.
#[test]
fn a_quiet_net_turns_on_victim_rings() {
    let nl = nl();
    let i = RingInputs { quiet_ring_net: Some(NetId(3)), ..inputs(&nl, &[true, false, false, false], &[false, true, true, false], &[None; 4]) };
    let (rings, missing) = plan(&i);
    let v: Vec<_> = rings.iter().filter(|r| r.role == RingRole::Victim).collect();
    assert_eq!(v.len(), 2);
    assert!(v.iter().all(|r| r.connection_net == NetId(3) && r.shareable));
    assert!(!missing.contains(&SUB30));
}

/// Oracle: REL-07: an electron injector gets an ECGR tied to the highest supply, the deck's 420 nm width.
#[test]
fn an_electron_injector_gets_a_supply_tied_ecgr() {
    let nl = nl();
    let inj = [Some(Carrier::Electrons), None, None, None];
    let (rings, missing) = plan(&RingInputs { ecgr_drawable: true, ..inputs(&nl, &[false; 4], &[false; 4], &inj) });
    assert_eq!(rings.len(), 1);
    let r = &rings[0];
    assert_eq!((r.ring_type, r.role, r.connection_net, r.min_width_nm, r.shareable), (GuardRingType::Ecgr, RingRole::Injector, NetId(4), 420, false));
    assert!(missing.contains(&("GuardRing", "ECGR width rule not given: collection efficiency unknown")), "{missing:?}");
}

/// Oracle: REL-07: no HCGR drawable, so a hole injector gets a well tap.
#[test]
fn a_hole_injector_falls_back_to_a_well_tap() {
    let nl = nl();
    let inj = [None, None, None, Some(Carrier::Holes)];
    let (rings, _) = plan(&inputs(&nl, &[false; 4], &[false; 4], &inj));
    assert_eq!(rings.len(), 1);
    let r = &rings[0];
    assert_eq!((r.device, r.ring_type, r.role, r.connection_net), (DeviceId(3), GuardRingType::Tap { in_well: true }, RingRole::Injector, NetId(4)));
}

/// Oracle: REL-07: on epi over low-resistivity substrate, rings do not help, so none.
#[test]
fn epi_substrate_draws_no_aggressor_or_victim_rings() {
    let nl = nl();
    let i = RingInputs {
        substrate: SubstrateKind::EpiOnLowRes,
        quiet_ring_net: Some(NetId(3)),
        ..inputs(&nl, &[true, false, false, false], &[false, true, true, false], &[None; 4])
    };
    assert!(plan(&i).0.is_empty());
}
