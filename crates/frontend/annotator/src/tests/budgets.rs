//! Net-class and budget tests: hand-built netlists → `annotate`'s net classes,
//! parasitic, coupling, antenna and offset budgets, and the missing-rule list
//! (EXT-21, EXT-25, AA-25).
//!
//! Contents: none public.

use super::*;

// ---- private ----

/// EXT-07 variant: once `vcas` also touches a channel (not gates only), its
/// class is ordinary `Signal`, not `Sensitive`: with `Stack` no longer a gate
/// reference in `is_sensitive`, a `vcas` that touches a channel is `Signal`.
#[test]
fn a_cascode_gate_net_with_a_channel_use_is_signal() {
    let nl = Netlist {
        devices: vec![
            fet("M1", DeviceKind::Nmos, 0, 1, 2, 2, 4_000, 500),
            fet("M2", DeviceKind::Nmos, 3, 4, 1, 2, 8_000, 500),
            fet("MD", DeviceKind::Nmos, 0, 3, 2, 2, 4_000, 500),
        ],
        nets: nets(&["vin", "x", "VSS", "vcas", "out"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(p.net_classes[3].class, analog::metadata::NetClass::Signal);
}

#[test]
fn antenna_gate_area_lands_on_the_gate_net_not_the_drain() {
    // G ≠ D on purpose: a diode-connected device would hide an index slip.
    let nl = Netlist { devices: vec![fet("XM1", DeviceKind::Nmos, 0, 1, 2, 2, 1_000, 1_000)], nets: nets(&["g", "d", "VSS"]), ..Default::default() };
    let mut cfg = AnnotationConfig::default();
    cfg.process.antenna_max_ratio = Some(400.0);
    let p = annotate(&nl, &cfg);
    let ant = p.routing.hard.iter().find(|b| b.kind().ends_with("Antenna")).expect("antenna batch");
    let mut nets_hit = Vec::new();
    ant.touched(&mut nets_hit);
    assert_eq!(nets_hit, vec![0], "gate area charged to net {nets_hit:?}, want the gate (0)");
}

#[test]
fn shields_are_requested_only_against_a_clock() {
    let is_shield = |b: &Box<dyn RuleBatch<analog::base::Routes>>| b.kind().ends_with("Shield");
    let quiet = annotate(&ota(), &AnnotationConfig::default());
    assert!(!quiet.routing.budget.iter().any(is_shield), "no aggressor, no shield");

    // The same OTA plus a clocked switch on its own nets.
    let mut nl = ota();
    nl.nets.push(Net { name: "clk".into() });
    nl.nets.push(Net { name: "sw".into() });
    let (clk, sw) = (nl.nets.len() as u16 - 2, nl.nets.len() as u16 - 1);
    nl.devices.push(fet("XS", DeviceKind::Nmos, clk, sw, 3, 3, 1_000, 150));
    nl.devices.push(fet("XC", DeviceKind::Nmos, clk, 0, 3, 3, 1_000, 150));
    let clocked = annotate(&nl, &AnnotationConfig::default());
    let b = clocked.routing.budget.iter().find(|b| is_shield(b)).expect("shield batch");
    let mut pairs = Vec::new();
    b.shield_pairs(&mut pairs);
    assert!(!pairs.is_empty());
    let vss = 3u32;
    assert!(pairs.iter().all(|&(_, r, _)| r == vss), "shielded by ground: {pairs:?}");
}

#[test]
fn a_shielded_victim_books_no_coupling_to_its_shield() {
    // RTE-02: the ground shield the clocked OTA asks for is not an aggressor.
    use analog::base::geom::{LayerId, Rect, Shape};
    let mut nl = ota();
    nl.nets.push(Net { name: "clk".into() });
    nl.nets.push(Net { name: "sw".into() });
    let (clk, sw) = (nl.nets.len() as u16 - 2, nl.nets.len() as u16 - 1);
    nl.devices.push(fet("XS", DeviceKind::Nmos, clk, sw, 3, 3, 1_000, 150));
    nl.devices.push(fet("XC", DeviceKind::Nmos, clk, 0, 3, 3, 1_000, 150));
    let mut cfg = AnnotationConfig::default();
    cfg.process.gate_af_per_um2 = Some(8_325.0);
    cfg.process.wire_af_per_um = Some(50.0);
    let p = annotate(&nl, &cfg);
    let mut pairs = Vec::new();
    p.routing.budget.iter().find(|b| b.kind().ends_with("Shield")).expect("shield batch").shield_pairs(&mut pairs);
    let (victim, vss, _) = pairs[0];
    let coup = p.routing.budget.iter().find(|b| b.kind().ends_with("CouplingBudget")).expect("coupling batch");
    // 100 mm of victim between its shield tracks 1 nm away: far past any budget.
    let wire = |y: i32| Shape { layer: LayerId(0), rect: Rect { x: 0, y, w: 100_000_000, h: 1 } };
    let mut wires = vec![Vec::new(); nl.nets.len()];
    wires[victim as usize] = vec![wire(0)];
    wires[vss as usize] = vec![wire(-2), wire(2)];
    let shielded = analog::base::routes::Routes { wires: wires.clone(), ..Default::default() };
    assert_eq!(coup.violations(&shielded), 0, "its own shield is not an aggressor");
    // The same track held by the clock is.
    wires[vss as usize].clear();
    wires[clk as usize] = vec![wire(2)];
    let clocked = analog::base::routes::Routes { wires, ..Default::default() };
    assert!(coup.violations(&clocked) > 0, "a clock beside the victim still counts");
}

/// EXT-20 (AA-23): a mirror with no compound is matched, not mirrored: one
/// `MatchedSet` on both devices and its Axis Orientation, no Symmetry.
#[test]
fn a_lone_mirror_is_matched_not_mirrored() {
    let nl = Netlist {
        devices: vec![fet("XM1", DeviceKind::Pmos, 0, 0, 2, 2, 5_000, 1_000), fet("XM2", DeviceKind::Pmos, 0, 1, 2, 2, 5_000, 1_000)],
        nets: nets(&["vref", "iout", "VDD"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    let count = |a: &Vec<Box<dyn RuleBatch<analog::base::Layout>>>, k: &str| a.iter().filter(|b| b.kind() == k).map(|b| b.count()).sum::<usize>();
    assert_eq!(count(&p.placement.hard, "Symmetry") + count(&p.placement.cost, "Symmetry"), 0);
    let sets: Vec<Vec<u32>> = (p.placement.budget.iter().filter(|b| b.kind() == "MatchedSet"))
        .map(|b| {
            let mut t = Vec::new();
            b.touched(&mut t);
            t.sort_unstable();
            t
        })
        .collect();
    assert_eq!(sets, [vec![0, 1]]);
    assert_eq!(p.placement.hard.iter().filter(|b| b.kind() == "Orientation").count(), 1);
}

#[test]
fn matched_sets_are_budgeted_and_missing_deck_terms_are_listed() {
    let is_set = |b: &Box<dyn RuleBatch<analog::base::Layout>>| b.kind() == "MatchedSet";
    let bare = annotate(&ota(), &AnnotationConfig::default());
    assert!(bare.placement.budget.iter().any(is_set), "coincidence needs no deck data");
    assert!(bare.missing.iter().any(|m| m.0 == "MatchedSet"), "the deck terms are listed unknown");

    let mut cfg = AnnotationConfig::default();
    cfg.process.avt_mv_um = [Some(5.0), Some(6.0)];
    cfg.process.svt_uv_per_um = Some(4.0);
    let full = annotate(&ota(), &cfg);
    assert!(full.placement.budget.iter().any(is_set));
    assert!(!full.missing.iter().any(|m| m.0 == "MatchedSet"));
}

/// MAT-16: with the deck's S(L) fit the XM1/XM2 pair's gradient term reads
/// S at L = 0.48 µm (0.580 µV/µm), 0.356 of the worst-case 1.63.
#[test]
fn svt_fit_sets_s_of_l() {
    let mut nl = ota();
    for d in &mut nl.devices[..2] {
        d.params.iter_mut().filter(|(k, _)| k == "l").for_each(|(_, v)| *v = 480);
    }
    // Two point cells 1 mm apart, no units.
    let l = analog::base::Layout {
        x: vec![0, 1_000_000],
        y: vec![0; 2],
        hw: vec![0; 2],
        hh: vec![0; 2],
        axis: vec![0; 8],
        groups: vec![],
        orient: vec![analog::base::Orient::default(); 2],
        variant: vec![0; 2],
        branch: Vec::new(),
        power_uw: vec![0; 2],
        temp_mc: vec![0; 2],
        units: Default::default(),
    };
    let ratio = |fit| {
        let mut cfg = AnnotationConfig::default();
        cfg.process.avt_mv_um = [Some(5.0), Some(6.0)];
        cfg.process.svt_fit = fit;
        cfg.process.svt_uv_per_um = Some(1.63);
        let p = annotate(&nl, &cfg);
        let set = (p.placement.budget.iter())
            .find(|b| {
                let mut t = Vec::new();
                b.touched(&mut t);
                t.sort_unstable();
                b.kind() == "MatchedSet" && t == [0, 1]
            })
            .expect("XM1/XM2 MatchedSet");
        let mut rows = Vec::new();
        set.ledger_rows(&l, &mut rows);
        rows[0].sigma_layout / 1.63
    };
    let r = ratio(Some((0.1835, 0.03533)));
    assert!((r - 0.356).abs() < 0.005, "{r}");
    let r = ratio(None);
    assert!((r - 1.0).abs() < 1e-4, "{r}");
}

#[test]
fn clocked_devices_are_kept_away_from_matched_ones() {
    let is_iso = |b: &Box<dyn RuleBatch<analog::base::Layout>>| b.kind().ends_with("Isolation");
    let quiet = annotate(&ota(), &AnnotationConfig::default());
    assert!(!quiet.placement.cost.iter().any(is_iso), "no aggressor, no isolation");

    let mut nl = ota();
    nl.nets.push(Net { name: "clk".into() });
    nl.nets.push(Net { name: "sw".into() });
    let (clk, sw) = (nl.nets.len() as u16 - 2, nl.nets.len() as u16 - 1);
    nl.devices.push(fet("XS", DeviceKind::Nmos, clk, sw, 3, 3, 1_000, 150));
    let p = annotate(&nl, &AnnotationConfig::default());
    let b = p.placement.cost.iter().find(|b| is_iso(b)).expect("isolation pull");
    assert_eq!(b.count(), 5, "XS against the four matched devices and the bias-gated tail (EXT-23)");
    assert!(!p.placement.budget.iter().any(is_iso), "uncalibrated: a pull, not a budget");
    assert!(p.missing.iter().any(|m| m.0 == "Isolation"), "and reported unknown");

    let mut cfg = AnnotationConfig::default();
    cfg.process.substrate = analog::base::SubstrateKind::EpiOnLowRes;
    cfg.process.epi_nm = Some(3_000);
    let p = annotate(&nl, &cfg);
    assert!(p.placement.budget.iter().any(is_iso));
    assert!(!p.missing.iter().any(|m| m.0 == "Isolation"));
}

#[test]
fn proximity_is_a_priced_budget_not_just_a_pull() {
    // MAT-07: the tail's distance to the input pair is an allowance the search
    // must pay for exceeding, not a report nobody enforces.
    let p = annotate(&ota(), &AnnotationConfig::default());
    let prox =
        |a: &Vec<Box<dyn RuleBatch<analog::base::Layout>>>| a.iter().filter(|b| b.kind().ends_with("Proximity")).map(|b| b.count()).sum::<usize>();
    assert!(prox(&p.placement.budget) > 0);
    assert_eq!(prox(&p.placement.budget), prox(&p.placement.cost), "every budget keeps its pull");
    assert!(!p.placement.hard.iter().any(|b| b.kind().ends_with("Proximity")));
}

#[test]
fn capacitor_plate_nets_get_no_invented_budget() {
    // An inverter drives a DAC bit plate; the top plate feeds a comparator
    // gate. A gate-load budget on either would be invented: the limits are
    // array specs (ARR-03/05), so both read unknown and say so.
    let cap = |name: &str, p: u16, n: u16| Device {
        name: name.into(),
        kind: DeviceKind::Capacitor,
        model: String::new(),
        terminals: vec![("P".into(), NetId(p)), ("N".into(), NetId(n))],
        params: vec![],
    };
    // nets: 0 top, 1 b0, 2 d0, 3 VDD, 4 VSS, 5 cmp
    let nl = Netlist {
        devices: vec![
            cap("XC1", 0, 1),
            fet("XMP", DeviceKind::Pmos, 2, 1, 3, 3, 1_000, 150),
            fet("XMN", DeviceKind::Nmos, 2, 1, 4, 4, 500, 150),
            fet("XMC", DeviceKind::Nmos, 0, 5, 4, 4, 1_000, 150),
        ],
        nets: nets(&["top", "b0", "d0", "VDD", "VSS", "cmp"]),
        ..Default::default()
    };
    let mut cfg = AnnotationConfig::default();
    cfg.process.gate_af_per_um2 = Some(8_325.0);
    let p = annotate(&nl, &cfg);
    assert_eq!(p.net_classes[0].c_budget_af, None, "top plate");
    assert_eq!(p.net_classes[1].c_budget_af, None, "bit plate");
    assert!(p.net_classes[2].c_budget_af.is_some(), "the inverter input keeps its gate-load budget");
    assert!(p.missing.iter().any(|m| m.0 == "ParasiticBudget" && m.1.contains("ARR-05")));
}

#[test]
#[should_panic(expected = "annotator: 65536 devices/nets exceed the u16 id space (65535)")]
fn more_nets_than_u16_ids_is_refused_not_wrapped() {
    // `NetId(n as u16)` would alias net 65536 onto net 0 (AA-35).
    let nl = Netlist { devices: Vec::new(), nets: vec![Net { name: "n".into() }; 65_536], ..Default::default() };
    let _ = annotate(&nl, &AnnotationConfig::default());
}

#[test]
fn intent_axes_per_compound() {
    let p = annotate(&ota(), &AnnotationConfig::default());
    // EXT-14 fills compounds (one here), EXT-15 sets (DP and load; without a unit
    // deck they have no unit); EXT-20: one axis per compound.
    assert!(p.intent.sets.len() == 2 && p.intent.compounds.len() == 1);
    assert_eq!(p.axis_count, 1);
    assert!(p.blocks.len() > 1, "not per block");
}

/// EXT-21: offset f0 0, hi 5 (M 5) touches the DP (S 1) and the load (S 0.2);
/// K 2 gives the DP 5/(2·1) = 2.5 mV (its cap 3·5/√10 = 4.74 does not bind)
/// and the load min(12.5, 3·5/√20 = 3.35).
#[test]
fn ext21_ota_dp_allowance_below_load() {
    use crate::evidence::{Evidence, Sensitivities, SpecSens};
    let mut cfg = AnnotationConfig::default();
    cfg.process.avt_mv_um = [Some(5.0), Some(5.0)];
    let d_vt = vec![(DeviceId(0), 1.0), (DeviceId(1), -1.0), (DeviceId(2), 0.2), (DeviceId(3), -0.2)];
    let s = SpecSens {
        metric: "offset".into(),
        f0: 0.0,
        lo: None,
        hi: Some(5.0),
        proc: None,
        sigma_f: None,
        d_c: vec![],
        d_r: vec![],
        d_vt,
        d_t: vec![],
        d_cc: vec![],
    };
    let ev = Evidence { sens: Some(Sensitivities { specs: vec![s] }), ..Evidence::default() };
    let p = crate::annotate_with(&ota(), &cfg, &ev);
    let allowance = |d: u16| p.intent.sets.iter().find(|s| s.members.iter().any(|m| m.device == DeviceId(d))).and_then(|s| s.allowance);
    let (dp, load) = (allowance(0).expect("DP allowance"), allowance(2).expect("load allowance"));
    assert!((dp - 2.5).abs() < 1e-4, "{dp}");
    assert!(dp < load, "{dp} vs {load}");
    let bare = annotate(&ota(), &cfg);
    assert!(bare.intent.sets.iter().all(|s| s.allowance.is_none()));
}

/// EXT-21 class rules: σ_f = 4 mV puts the load (σ ≈ 1.58 mV, S = 0.2) at weight
/// ≈ 0.006 < `minor_weight` and the DP (σ ≈ 2.24 mV, S = 1) at ≈ 0.31, so D5 makes
/// only the load (Minimal, Spec); the ceiling 2 + β·σ_f leaves a 2 mV margin, so
/// the DP's allowance is 1 mV and its 6 mV target makes it (Moderate, Spec), apart
/// from both the role default (Moderate, Role) and D5. (At the 2.5 mV of
/// `ext21_ota_dp_allowance_below_load` the 15 mV target is itself Minimal.)
#[test]
fn ext21_minor_weight_and_allowance_set_class() {
    use crate::evidence::{Evidence, Sensitivities, SpecSens};
    use analog::intent::{ClassSource, MatchClass};
    let mut cfg = AnnotationConfig::default();
    cfg.process.avt_mv_um = [Some(5.0), Some(5.0)];
    let d_vt = vec![(DeviceId(0), 1.0), (DeviceId(1), -1.0), (DeviceId(2), 0.2), (DeviceId(3), -0.2)];
    let sf = 4.0;
    let s = SpecSens {
        metric: "offset".into(),
        f0: 0.0,
        lo: None,
        hi: Some(2.0 + cfg.policy.beta_target * sf),
        proc: None,
        sigma_f: Some(sf),
        d_c: vec![],
        d_r: vec![],
        d_vt,
        d_t: vec![],
        d_cc: vec![],
    };
    let ev = Evidence { sens: Some(Sensitivities { specs: vec![s] }), ..Evidence::default() };
    let p = crate::annotate_with(&ota(), &cfg, &ev);
    let set = |d: u16| p.intent.sets.iter().find(|s| s.members.iter().any(|m| m.device == DeviceId(d))).expect("set");
    let (dp, load) = (set(0), set(2));
    assert!(load.weight.is_some_and(|w| w < cfg.policy.minor_weight), "{:?}", load.weight);
    assert!(dp.weight.is_some_and(|w| w > cfg.policy.minor_weight), "{:?}", dp.weight);
    assert_eq!((load.class, load.class_source), (MatchClass::Minimal, ClassSource::Spec));
    assert_ne!((dp.class, dp.class_source), (MatchClass::Minimal, ClassSource::Spec));
    assert_eq!(dp.kind, analog::intent::MatchKind::Voltage);
    assert!((dp.allowance.expect("DP allowance") - 1.0).abs() < 1e-4, "{:?}", dp.allowance);
    let mut diags = Vec::new();
    let mut ctx = crate::class::ClassCtx { user: None, spec_6sigma: Some(6.0 * 1.0), role: crate::class::SetRole::InputPair, diags: &mut diags };
    assert_eq!((dp.class, dp.class_source), crate::class::class_of(dp, &mut ctx));
    assert_eq!((dp.class, dp.class_source), (MatchClass::Moderate, ClassSource::Spec));
    // Without σ_f there is no weight, so the load keeps its role class.
    let bare = annotate(&ota(), &cfg);
    let load = bare.intent.sets.iter().find(|s| s.members.iter().any(|m| m.device == DeviceId(2))).expect("set");
    assert_ne!(load.class, MatchClass::Minimal);
}

/// EXT-25 (AA-25): a drain-only net's load is off-netlist, so it is unbudgeted
/// and listed missing until a sidecar `Load` states it; a gate-driving net
/// (vbias) keeps its gate-load budget.
#[test]
fn drain_only_net_without_load_is_unknown() {
    let mut cfg = AnnotationConfig::default();
    cfg.process.gate_af_per_um2 = Some(8325.0);
    cfg.process.wire_af_per_um = Some(50.0);
    let budget = |p: &crate::Problem, n: u16| p.net_classes[n as usize].c_budget_af;
    let missing = |p: &crate::Problem| p.missing.iter().any(|m| m.0 == "ParasiticBudget" && m.1.contains("AA-25"));
    let p = annotate(&ota(), &cfg);
    assert_eq!((budget(&p, 4), budget(&p, 2)), (None, None), "vout2, vtail");
    assert!(missing(&p));
    assert!(budget(&p, 6).is_some(), "vbias drives gates");
    cfg.loads = vec![(NetId(4), 1e6)];
    let p = annotate(&ota(), &cfg);
    assert!(budget(&p, 4).is_some());
    assert!(budget(&p, 6).is_some());
}
