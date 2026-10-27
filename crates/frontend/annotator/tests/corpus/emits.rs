//! What the corpus emits beyond recognition: one group per overlapping
//! requirement, matched sets and their classes and units, net classes,
//! substrate tags, shields, star nodes, Kelvin taps and the audit.
//!
//! Contents: none public.

use super::*;

// ---- private ----

/// AA-01 (EXT-13, closed by EXT-15's shared-bias groups): `mirror6` is one 6-device
/// `Matching` node with MR the set's reference; three_stage's tail sits in the
/// compound's Symmetry node and in the bias set with M7, M9.
/// Oracle: AA-01 by hand: mirror6 is one six-device Matching node.
#[test]
fn overlapping_requirements_share_one_group() {
    let nl = net(src("mirror6"));
    let p = annotate(&nl, &cfg("mirror6"));
    let names: Vec<&str> = nl.devices.iter().map(|d| d.name.as_str()).collect();
    let t = annotator::graph::render(&p.intent.tree, &names);
    assert!(t.contains("Matching{MO1,MO2,MO3,MO4,MO5,MR}"), "{t}");
    let s = &p.intent.sets[0];
    assert_eq!(s.reference.map(|r| names[s.members[r].device.0 as usize]), Some("MR"));
    let nl = net(src("three_stage"));
    let p = annotate(&nl, &cfg("three_stage"));
    let names: Vec<&str> = nl.devices.iter().map(|d| d.name.as_str()).collect();
    let t = annotator::graph::render(&p.intent.tree, &names);
    assert_eq!(t.matches("Symmetry{").count(), 1, "{t}");
    assert!(t.contains("Matching{M3,M7,M9}"), "{t}");
}

/// `benchmarks/fixtures/mirror_ratio.spice` (M1 Status): the 1:2:4 mirror is one
/// matched set and one Unitization, not split per size class.
/// Oracle: the fixture's 1:2:4 mirror is one matched set, by construction.
#[test]
fn mirror_ratio_is_one_set() {
    let nl = net("XM1 d1 d1 VSS VSS nfet_01v8 W=2u L=1u nf=2 | XM2 d2 d1 VSS VSS nfet_01v8 W=4u L=1u nf=4
                  XM3 d3 d1 VSS VSS nfet_01v8 W=8u L=1u nf=8");
    let p = annotate(&nl, &common::cfg());
    assert_eq!(p.intent.sets.len(), 1);
    assert_eq!(p.intent.sets[0].members.len(), 3);
    let u: Vec<_> = p.constraints.unitization.iter().filter(|u| u.devices.len() > 1).collect();
    assert_eq!(u.len(), 1);
    assert_eq!(u[0].devices.len(), 3);
    assert_eq!(p.constraints.unitization.len(), 1, "no device left to a second unitization");
}

/// T10 (EXT-16): with no spec every class is a Role default; a 1σ offset spec makes
/// the Voltage sets Spec-sourced (6·0.5 = 3 mV → Moderate) and leaves Current sets on Role (card D-i).
/// Oracle: metamorphic: with no spec every class is Role; a 0.5 mV spec makes Voltage sets Spec (6·0.5 = 3
/// mV).
#[test]
fn class_sources() {
    for (name, src) in all() {
        let p = annotate(&net(src), &cfg(name));
        assert!(p.intent.sets.iter().all(|s| s.class_source == analog::intent::ClassSource::Role), "{name}");
    }
    let mut c = common::cfg();
    c.offset_sigma_mv = Some(0.5);
    let p = annotate(&net(src("ota5t")), &c);
    for s in &p.intent.sets {
        let want = if s.kind == analog::intent::MatchKind::Voltage { analog::intent::ClassSource::Spec } else { analog::intent::ClassSource::Role };
        assert_eq!((s.class_source, s.class), (want, analog::intent::MatchClass::Moderate), "{s:?}");
    }
}

/// EXT-14 step 9 (card D-d): σ = 0.3 mV puts 6σ = 1.8 mV under 3 mV, so the
/// DiffPair set is Exceptional Voltage and its compound Perfect; 0.5 mV keeps Mirror.
/// Oracle: hand arithmetic: 6·0.3 = 1.8 mV < 3 mV is Exceptional, 3 mV is not.
#[test]
fn exceptional_voltage_compound_is_perfect() {
    use analog::intent::{MatchClass, MatchKind, SymKind};
    for (sigma, kind) in [(0.3, SymKind::Perfect), (0.5, SymKind::Mirror)] {
        let mut c = common::cfg();
        c.offset_sigma_mv = Some(sigma);
        let p = annotate(&net(src("ota5t")), &c);
        let dp = p.intent.sets.iter().find(|s| s.kind == MatchKind::Voltage).unwrap();
        let want = if sigma < 0.5 { MatchClass::Exceptional } else { MatchClass::Moderate };
        assert_eq!((dp.class, p.intent.compounds[0].kind), (want, kind), "σ = {sigma}");
    }
}

/// EXT-15/16 per-set Unitization flags: folded's Minimal bias set {M7, M8} (one
/// finger W/L) needs no dummies and is Adjacent; dac4's Ratio set, downgraded
/// below Moderate by hand, drops dummies and route matching (Ratio < Moderate).
/// Oracle: the class table: Minimal needs no dummies and is Adjacent.
#[test]
fn per_set_unitization_follows_class() {
    use analog::intent::{ArrayStyle, MatchClass};
    let nl = net(src("folded"));
    let p = annotate(&nl, &cfg("folded"));
    let id = |nl: &analog::base::Netlist, n: &str| nl.devices.iter().position(|d| d.name == n).unwrap() as u16;
    let u = p.constraints.unitization.iter().find(|u| u.devices.iter().any(|d| d.0 == id(&nl, "M7"))).expect("M7 covered");
    let mut names: Vec<&str> = u.devices.iter().map(|d| nl.devices[d.0 as usize].name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(
        (names.as_slice(), u.dummy_required, u.route_matching_required, u.class, u.style),
        (&["M7", "M8"][..], false, true, Some(MatchClass::Minimal), Some(ArrayStyle::Adjacent))
    );
    let nl = net(src("dac4"));
    let p = annotate(&nl, &cfg("dac4"));
    let mut models = Vec::new();
    let drawn: Vec<_> = nl.devices.iter().map(|d| annotator::size::drawn(d, &mut models)).collect();
    let mut sets = p.intent.sets.clone();
    sets[0].class = MatchClass::Minimal;
    let c = annotator::constraints::assemble(&nl, &drawn, &p.blocks, &sets);
    let u = c.unitization.iter().find(|u| u.devices.iter().any(|d| d.0 == id(&nl, "XC0"))).expect("XC0 covered");
    assert_eq!((u.dummy_required, u.route_matching_required, u.class), (false, false, Some(MatchClass::Minimal)));
}

/// EXT-19: the split DAC's bridge is checked against `(C_T^LSB/C_T^MSB)·C_u`
/// (4/3 of the 9 µm² unit = 3×4 µm): exact gives no diagnostic, `l=5u` gives one.
/// The set is one Unitization although the bridge is off the unit (CELL-21).
/// Oracle: hand arithmetic: (C_T^LSB/C_T^MSB)·C_u = 4/3 of 9 µm²; `l=5u` misses it.
#[test]
fn split_dac_bridge_value() {
    let bridge = |p: &annotator::Problem| p.intent.diagnostics.iter().any(|d| d.kind == "bridge_cap_value");
    let p = annotate(&net(src("splitdac")), &cfg("splitdac"));
    assert!(!bridge(&p), "{:?}", p.intent.diagnostics);
    assert!(matches!(p.intent.sets[0].origin, analog::intent::Origin::PassiveSet { rule: "split_dac" }));
    // CELL-21: the bridge's own W/L does not split the set; one Unitization holds all six.
    assert!(
        p.constraints.unitization.iter().any(|u| u.devices.len() == 6),
        "{:?}",
        p.constraints.unitization.iter().map(|u| &u.devices).collect::<Vec<_>>()
    );
    let p = annotate(&net(&src("splitdac").replace("CA tl tm cap w=3u l=4u", "CA tl tm cap w=3u l=5u")), &cfg("splitdac"));
    assert!(bridge(&p), "{:?}", p.intent.diagnostics);
}

/// EXT-19 step 4 gate: every device cellgen's deleted `dac_banks`, `bjt_groups`
/// and `parallel_groups` grouped on the bench fixtures is in one annotator
/// Unitization with cellgen's counts and flags.
/// Oracle: differential: cellgen's deleted grouping (counts and flags) on the same fixtures.
#[test]
fn annotator_covers_cellgen_groups() {
    let cases: [(&str, &[&str], &[u16], bool, bool); 4] = [
        ("dac4", &["XC0", "XC1", "XC2", "XC3", "XC4"], &[1, 1, 2, 4, 8], true, true),
        ("bgr_core", &["XQ1", "XQ2"], &[1, 8], false, true),
        ("pair", &["XM1", "XM2"], &[1, 1], false, false),
        ("quad", &["XM1", "XM2", "XM3", "XM4"], &[1, 1, 1, 1], false, false),
    ];
    for (name, devs, nf, dummy, route) in cases {
        let nl = net(src(name));
        let p = annotate(&nl, &cfg(name));
        let id = |n: &str| nl.devices.iter().position(|d| d.name == n).unwrap() as u16;
        let u =
            p.constraints.unitization.iter().find(|u| u.devices.iter().any(|d| d.0 == id(devs[0]))).unwrap_or_else(|| panic!("{name}: uncovered"));
        let names: Vec<&str> = u.devices.iter().map(|d| nl.devices[d.0 as usize].name.as_str()).collect();
        assert_eq!((names.as_slice(), u.dev_nf.as_slice(), u.dummy_required, u.route_matching_required), (devs, nf, dummy, route), "{name}");
    }
}

/// Ids touched by each `kind` batch of `arm`, in batch order.
fn touched_of(arm: &[Box<dyn analog::RuleBatch<analog::base::Layout>>], kind: &str) -> Vec<Vec<u32>> {
    (arm.iter().filter(|b| b.kind() == kind || b.kind().ends_with(&format!("::{kind}"))))
        .map(|b| {
            let mut t = Vec::new();
            b.touched(&mut t);
            t
        })
        .collect()
}

/// T5 (EXT-20): one `MatchedSet` batch per intent set, in set order, on exactly its devices.
/// Oracle: the intent's sets: one MatchedSet batch each, on exactly its devices.
#[test]
fn one_rule_per_set() {
    for (name, src) in all() {
        let nl = net(src);
        let p = annotate(&nl, &cfg(name));
        let got = touched_of(&p.placement.budget, "MatchedSet");
        assert_eq!(got.len(), p.intent.sets.len(), "{name}");
        for (t, s) in got.iter().zip(&p.intent.sets) {
            let mut t = t.clone();
            t.sort_unstable();
            let mut want: Vec<u32> = s.members.iter().map(|m| u32::from(m.device.0)).collect();
            want.sort_unstable();
            assert_eq!(t, want, "{name}");
            assert!(t.windows(2).all(|w| w[0] != w[1]), "{name}: a device twice in one set");
        }
    }
    let nl = net(src("ota5t"));
    let p = annotate(&nl, &cfg("ota5t"));
    let names: BTreeSet<Vec<&str>> = touched_of(&p.placement.budget, "MatchedSet")
        .into_iter()
        .map(|t| {
            let mut n: Vec<&str> = t.iter().map(|&d| nl.devices[d as usize].name.as_str()).collect();
            n.sort_unstable();
            n
        })
        .collect();
    assert_eq!(names, BTreeSet::from([vec!["XM1", "XM2"], vec!["XM3", "XM4"]]));
}

/// AA-24 (EXT-20): `mirror6` is one ratioed set, MR in slot 0, matched and
/// pulled to MR (Minimal) but not mirrored; made Moderate it loses the pull.
/// Oracle: AA-24: a ratioed set is matched, not mirrored; metamorphic with the class raised.
#[test]
fn ratioed_mirror_not_mirrored() {
    let count = |arm: &[Box<dyn analog::RuleBatch<analog::base::Layout>>], k: &str| {
        arm.iter().filter(|b| b.kind() == k || b.kind().ends_with(&format!("::{k}"))).map(|b| b.count()).sum::<usize>()
    };
    let nl = net(src("mirror6"));
    let c = cfg("mirror6");
    let p = annotate(&nl, &c);
    let mr = nl.devices.iter().position(|d| d.name == "MR").unwrap() as u32;
    assert_eq!(count(&p.placement.hard, "Symmetry"), 0);
    let sets = touched_of(&p.placement.budget, "MatchedSet");
    assert_eq!(sets.len(), 1);
    assert_eq!((sets[0].len(), sets[0][0]), (6, mr), "{sets:?}");
    assert_eq!(count(&p.placement.budget, "Proximity"), 5);
    assert!(touched_of(&p.placement.budget, "Proximity").concat().chunks(2).all(|ab| ab.contains(&mr)));

    let mut intent = p.intent.clone();
    (intent.sets[0].class, intent.sets[0].class_source) = (analog::intent::MatchClass::Moderate, analog::intent::ClassSource::User);
    let mut models = Vec::new();
    let drawn: Vec<_> = nl.devices.iter().map(|d| annotator::size::drawn(d, &mut models)).collect();
    let r = annotator::emit::placement(&intent, &p.blocks, &[], &nl, &drawn, &c.process, c.offset_sigma_mv, &c.policy);
    assert_eq!((count(&r.hard, "Symmetry"), count(&r.budget, "Proximity")), (0, 0));
}

/// T5/AA-24 (EXT-20): only an equal couple mirrors. Unequal units (unit
/// `Some`) or, without a unit, unequal drawn geometry drop the couple from the
/// hard Symmetry; its MatchedSet and DtiBand stay.
/// Oracle: T5/AA-24: an unequal couple drops from Symmetry; its MatchedSet and DtiBand stay.
#[test]
fn unequal_couple_matched_not_mirrored() {
    let nl = net(src("ota5t"));
    let mut c = cfg("ota5t");
    c.process.dti = Some((5_000, 300));
    let p = annotate(&nl, &c);
    let id = |n: &str| nl.devices.iter().position(|d| d.name == n).unwrap() as u32;
    let (m1, m2, m3, m4) = (id("XM1"), id("XM2"), id("XM3"), id("XM4"));
    let mut models = Vec::new();
    let drawn: Vec<_> = nl.devices.iter().map(|d| annotator::size::drawn(d, &mut models)).collect();
    let i = p.intent.sets.iter().position(|s| s.members.iter().any(|m| u32::from(m.device.0) == m2)).unwrap();
    let has = |v: &[Vec<u32>], a: u32, b: u32| v.concat().chunks(2).any(|ab| ab == [a, b] || ab == [b, a]);
    let check = |intent: &analog::intent::Intent, drawn: &[annotator::size::Drawn], mirrored: bool| {
        let r = annotator::emit::placement(intent, &p.blocks, &[], &nl, drawn, &c.process, c.offset_sigma_mv, &c.policy);
        let sym = touched_of(&r.hard, "Symmetry");
        assert_eq!(has(&sym, m1, m2), mirrored, "{sym:?}");
        assert!(has(&sym, m3, m4), "{sym:?}");
        assert!(touched_of(&r.budget, "MatchedSet").iter().any(|t| t.contains(&m1) && t.contains(&m2)));
        assert!(has(&touched_of(&r.hard, "DtiBand"), m1, m2));
    };
    check(&p.intent, &drawn, true);

    let mut intent = p.intent.clone();
    assert!(intent.sets[i].unit.is_some());
    let j = intent.sets[i].members.iter().position(|m| u32::from(m.device.0) == m2).unwrap();
    intent.sets[i].members[j].parallel += 1;
    check(&intent, &drawn, false);

    let mut intent = p.intent.clone();
    intent.sets[i].unit = None;
    check(&intent, &drawn, true);
    let mut wide = drawn.clone();
    wide[m2 as usize].w_finger_nm = wide[m2 as usize].w_finger_nm.map(|w| w * 2);
    check(&intent, &wide, false);
}

/// EXT-20: a bipolar set's MatchedSet carries the deck's `bjt_ka_pct_um`;
/// without it the random term is unknown. `bgr_core`'s PNPs are fixed-geometry
/// (no W/L, area 0 to `emit`), so the test writes a 3.4×3.4 µm emitter.
/// Oracle: the deck key: with `bjt_ka_pct_um` the random term is known; without it, unknown.
#[test]
fn bipolar_set_reads_bjt_ka() {
    let nl = net(src("bgr_core"));
    let mut models = Vec::new();
    let mut drawn: Vec<_> = nl.devices.iter().map(|d| annotator::size::drawn(d, &mut models)).collect();
    for d in &mut drawn {
        (d.w_finger_nm, d.l_nm) = (Some(3_400), Some(3_400));
    }
    let rows = |ka: Option<f32>| {
        let mut c = cfg("bgr_core");
        c.process.bjt_ka_pct_um = ka;
        let p = annotate(&nl, &c);
        let i = p.intent.sets.iter().position(|s| s.family == analog::intent::Family::Bipolar).expect("the bipolar set");
        let r = annotator::emit::placement(&p.intent, &p.blocks, &[], &nl, &drawn, &c.process, c.offset_sigma_mv, &c.policy);
        let set = r.budget.iter().filter(|b| b.kind() == "MatchedSet").nth(i).expect("set i's batch");
        let n = nl.devices.len();
        let l = analog::base::Layout {
            x: (0..n as i32).map(|k| k * 100_000).collect(),
            y: vec![0; n],
            hw: vec![0; n],
            hh: vec![0; n],
            axis: vec![0; 8],
            groups: vec![],
            orient: vec![analog::base::Orient::default(); n],
            variant: vec![0; n],
            branch: Vec::new(),
            power_uw: vec![0; n],
            temp_mc: vec![0; n],
            units: Default::default(),
        };
        let mut rows = Vec::new();
        set.ledger_rows(&l, &mut rows);
        rows
    };
    let r = rows(Some(1.0));
    assert!(!r.is_empty() && r.iter().all(|r| r.sigma_rand > 0.0), "{r:?}");
    let r = rows(None);
    assert!(!r.is_empty() && r.iter().all(|r| r.sigma_rand == 0.0 && !r.known), "{r:?}");
}

/// EXT-17: without evidence every device has facts, and no region is guessed.
/// Oracle: the input: no evidence, so every region is Unknown.
#[test]
fn no_evidence_no_regions() {
    for (name, src) in all() {
        let nl = net(src);
        let p = annotate(&nl, &cfg(name));
        assert_eq!(p.intent.devices.len(), nl.devices.len(), "{name}");
        assert!(p.intent.devices.iter().all(|f| f.region == analog::intent::Region::Unknown), "{name}");
    }
}

/// EXT-18: refined net classes on the corpus.
/// Oracle: hand-labelled classes per corpus net (EXT-18), written from each circuit's function.
#[test]
fn net_classes() {
    use analog::metadata::NetClass::{self, *};
    let check = |name: &str, want: &[(&[&str], NetClass)]| {
        let nl = net(src(name));
        let p = annotate(&nl, &cfg(name));
        assert_eq!(p.intent.nets.len(), nl.nets.len(), "{name}");
        for &(nets, class) in want {
            for n in nets {
                let i = nl.nets.iter().position(|x| x.name == *n).unwrap();
                assert_eq!(p.net_classes[i].class, class, "{name}: {n}");
            }
        }
        p
    };
    check(
        "dac4",
        &[(&["d0", "d1", "d2", "d3", "b0", "b1", "b2", "b3"], DigitalStatic), (&["top"], Sensitive), (&["VDD"], Supply), (&["VSS"], Ground)],
    );
    check("three_stage", &[(&["vbias"], Bias), (&["vin_p", "vin_n"], Sensitive), (&["n3"], Signal)]);
    check("folded", &[(&["vbp1", "vbp2", "vbn", "vbn2"], Bias)]);
    check("bjt_mirror", &[(&["in", "outn", "outp"], Signal)]);
    let p = check("ota5t", &[(&["vbn", "vbias"], Bias), (&["vinp", "vinm"], Sensitive)]);
    let vbn = net(src("ota5t")).nets.iter().position(|x| x.name == "vbn").unwrap();
    assert_eq!(p.intent.nets[vbn].evidence, analog::intent::EvidenceLevel::Structure);
}

fn names(nl: &analog::base::Netlist, ds: impl Iterator<Item = analog::base::ids::DeviceId>) -> Vec<String> {
    let mut v: Vec<String> = ds.map(|d| nl.devices[d.0 as usize].name.clone()).collect();
    v.sort();
    v
}

/// EXT-23: the clocked tail and precharge devices aggress; the Moderate sets
/// are victims minus aggressors; the Minimal output inverter is not.
/// Oracle: the StrongARM's clocked devices (mn0, mp7..mp10) by name, from the circuit's own clocking.
#[test]
fn strongarm_tags() {
    let nl = net(STRONGARM);
    let p = annotate(&nl, &strongarm_cfg());
    assert!(p.intent.aggressors.iter().all(|a| a.inject == analog::intent::Inject::Switching));
    assert_eq!(names(&nl, p.intent.aggressors.iter().map(|a| a.device)), ["mn0", "mp10", "mp7", "mp8", "mp9"]);
    assert_eq!(names(&nl, p.intent.victims.iter().map(|v| v.device)), ["mn1", "mn2", "mn3", "mn4", "mp11", "mp12", "mp5", "mp6"]);
}

/// EXT-23, C19/T8: the 5T OTA has no aggressor, so no guard ring.
/// Oracle: the textbook 5T OTA has no clocked device, so no aggressor.
#[test]
fn ota5t_tags() {
    let nl = net(src("ota5t"));
    let p = annotate(&nl, &cfg("ota5t"));
    assert!(p.intent.aggressors.is_empty(), "{:?}", p.intent.aggressors);
    assert_eq!(names(&nl, p.intent.victims.iter().map(|v| v.device)), ["XM1", "XM2", "XM3", "XM4", "XM5"]);
    let xm5 = p.intent.victims.iter().find(|v| nl.devices[v.device.0 as usize].name == "XM5").unwrap();
    assert_eq!(xm5.reason, "bias/reference gate");
    assert_eq!(p.constraints.guard_rings.len(), 0, "C19: no aggressor, no ring");
    // GAP-03 / REL T8: with only gates and rails as pins (the corpus netlist has
    // no port list), no diffusion reaches a pin: no minority injector, no ring.
    let mut nl = nl;
    nl.ports = ["vinp", "vinm", "vbias", "vbn", "VDD", "VSS"]
        .map(|n| analog::base::ids::NetId(nl.nets.iter().position(|x| x.name == n).unwrap() as u16))
        .to_vec();
    let p = annotate(&nl, &cfg("ota5t"));
    assert!(p.intent.aggressors.is_empty(), "{:?}", p.intent.aggressors);
    assert_eq!(p.constraints.guard_rings.len(), 0);
}

/// EXT-23: DAC4's switch logic is digital, so no inverter device is a victim.
/// Oracle: the input: DAC4's switch inverters are digital.
#[test]
fn dac4_logic_is_no_victim() {
    let nl = net(src("dac4"));
    let p = annotate(&nl, &cfg("dac4"));
    let v = names(&nl, p.intent.victims.iter().map(|v| v.device));
    assert!(!v.iter().any(|n| n.starts_with("XMN") || n.starts_with("XMP")), "{v:?}");
}

fn net_id(nl: &analog::base::Netlist, n: &str) -> analog::base::ids::NetId {
    analog::base::ids::NetId(nl.nets.iter().position(|x| x.name == n).unwrap_or_else(|| panic!("{n}")) as u16)
}

/// EXT-24: folded's Differential pairs are its compound's mirrored nets.
/// Oracle: the folded cascode's mirrored nets, by name.
#[test]
fn folded_differential_pairs() {
    let nl = net(src("folded"));
    let p = annotate(&nl, &cfg("folded"));
    let want: BTreeSet<(String, String)> =
        [("vinn", "vinp"), ("x1", "x2"), ("o1", "out"), ("y1", "y2")].iter().map(|&(a, b)| common::sorted(a, b)).collect();
    assert_eq!(canon(&p, &nl).net_pairs, want);
}

/// EXT-24: strongarm shields its four analog nets to `vss`, whose return the
/// clocked tail shares (SUB-30 diagnostic). With no aggressor net (the clock
/// renamed and unconfigured: `clk` is a clock by name alone), no shield.
/// Oracle: metamorphic: with the clock renamed and unconfigured, no shield.
#[test]
fn strongarm_shields() {
    let nl = net(STRONGARM);
    let p = annotate(&nl, &strongarm_cfg());
    let shielded: Vec<(String, String)> = p
        .intent
        .nets
        .iter()
        .enumerate()
        .filter_map(|(i, f)| Some((nl.nets[i].name.clone(), nl.nets[f.shield_ref?.0 as usize].name.clone())))
        .collect();
    let mut victims: Vec<&str> = shielded.iter().map(|s| s.0.as_str()).collect();
    victims.sort_unstable();
    assert_eq!(victims, ["vin", "vin_o", "vip", "vip_o"]);
    assert!(shielded.iter().all(|s| s.1 == "vss"), "{shielded:?}");
    assert!(p.intent.diagnostics.iter().any(|d| d.kind == "shared_shield_return"));
    let quiet = net(&STRONGARM.replace("clk", "en"));
    let c = AnnotationConfig { clock_nets: Vec::new(), ..strongarm_cfg() };
    assert!(annotate(&quiet, &c).intent.nets.iter().all(|f| f.shield_ref.is_none()));
}

fn names_of(nl: &analog::base::Netlist, ds: &[analog::base::ids::DeviceId]) -> Vec<String> {
    names(nl, ds.iter().copied())
}

/// EXT-24: ota5t's input pair shares `vtail`, its load `VDD`; the tail is the
/// single feed, so no star.
/// Oracle: the textbook 5T OTA: the pair shares `vtail`, the load `VDD`; one feed, so no star.
#[test]
fn ota5t_common_nodes() {
    let nl = net(src("ota5t"));
    let p = annotate(&nl, &cfg("ota5t"));
    let got: BTreeSet<(String, Vec<String>, Vec<String>)> =
        p.intent.common_nodes.iter().map(|c| (nl.nets[c.net.0 as usize].name.clone(), names_of(&nl, &c.a), names_of(&nl, &c.b))).collect();
    let want: BTreeSet<_> =
        [("vtail", "XM1", "XM2"), ("VDD", "XM3", "XM4")].iter().map(|&(n, a, b)| (n.to_string(), vec![a.to_string()], vec![b.to_string()])).collect();
    assert_eq!((p.intent.common_nodes.len(), got), (2, want));
    assert!(p.intent.stars.is_empty());
}

/// EXT-24: mirror6's reference MR against each output on `VSS`; an unrelated
/// device returning through `VSS` makes it a star of the six members.
/// Oracle: the input: MR against each of five outputs on `VSS`.
#[test]
fn mirror6_common_nodes_and_star() {
    let nl = net(src("mirror6"));
    let p = annotate(&nl, &cfg("mirror6"));
    assert_eq!(p.intent.common_nodes.len(), 5);
    assert!(p.intent.common_nodes.iter().all(|c| c.net == net_id(&nl, "VSS") && names_of(&nl, &c.a) == ["MR"]));
    assert!(p.intent.stars.is_empty());
    let nl = net(&format!("{} | MX z g VSS VSS nfet w=2u l=1u", src("mirror6")));
    let p = annotate(&nl, &cfg("mirror6"));
    assert_eq!(p.intent.stars.len(), 1);
    assert_eq!((p.intent.stars[0].net, p.intent.stars[0].branches.len()), (net_id(&nl, "VSS"), 6));
}

/// EXT-24 step 4 with an op point: `MX` on mirror6's `VSS` is a star only when
/// its current exceeds 1% of the six members' (60 µA here); a port is the feed.
/// Oracle: hand arithmetic: 1 % of the six members' 60 µA is the threshold.
#[test]
fn star_needs_nonmember_current_with_an_op_point() {
    use annotator::evidence::DeviceOp;
    let nl = net(&format!("{} | MX z g VSS VSS nfet w=2u l=1u", src("mirror6")));
    let mx = nl.devices.iter().position(|d| d.name == "MX").unwrap();
    let stars = |mx_ua: f64, ports: Vec<analog::base::ids::NetId>| {
        let op = DeviceOp {
            id_ua: 10.0,
            headroom_mv: 200.0,
            gm_us: 100.0,
            power_uw: 0.0,
            vgs_mv: None,
            vbs_mv: None,
            vth_mv: None,
            gmb_us: None,
            gds_us: None,
        };
        let mut dev = vec![Some(op); nl.devices.len()];
        dev[mx] = Some(DeviceOp { id_ua: mx_ua, ..op });
        let ev = annotator::Evidence { op: Some(annotator::OpFacts { dev, net_mv: vec![None; nl.nets.len()] }), ..Default::default() };
        let nl = analog::base::Netlist { ports, ..nl.clone() };
        annotator::annotate_with(&nl, &cfg("mirror6"), &ev).intent.stars
    };
    let s = stars(1.0, vec![]);
    assert_eq!((s.len(), s[0].net, s[0].branches.len()), (1, net_id(&nl, "VSS"), 6));
    assert!(stars(-0.5, vec![]).is_empty(), "0.5 µA ≤ 0.6 µA: no star");
    assert_eq!(stars(1.0, vec![net_id(&nl, "VSS")]).len(), 1);
    // ota5t's tail feeds `vtail` (no star); when `vtail` is a port, the port is
    // the feed and the tail drain is non-member current.
    let mut nl = net(src("ota5t"));
    nl.ports = vec![net_id(&nl, "vtail")];
    let p = annotate(&nl, &cfg("ota5t"));
    assert_eq!(p.intent.stars.iter().map(|s| (s.net, s.root.is_none())).collect::<Vec<_>>(), [(net_id(&nl, "vtail"), true)]);
}

/// EXT-26 step 4: a sidecar `NetClass` beats the 'CurrentSource gate is Bias'
/// pass. mirror6's outputs are CurrentSources (shared bias) gated by `ref`.
/// Oracle: EXT-26 step 4: a sidecar class has User evidence and wins.
#[test]
fn user_net_class_survives_the_current_source_gate_pass() {
    use annotator::evidence::DeviceOp;
    let nl = net(src("mirror6"));
    let op = DeviceOp {
        id_ua: 10.0,
        headroom_mv: 200.0,
        gm_us: 100.0,
        power_uw: 0.0,
        vgs_mv: None,
        vbs_mv: None,
        vth_mv: None,
        gmb_us: None,
        gds_us: None,
    };
    let ev = annotator::Evidence {
        op: Some(annotator::OpFacts { dev: vec![Some(op); nl.devices.len()], net_mv: vec![None; nl.nets.len()] }),
        ..Default::default()
    };
    let r = net_id(&nl, "ref");
    let user = AnnotationConfig { net_classes: vec![(r, analog::metadata::NetClass::Signal)], ..cfg("mirror6") };
    let p = annotator::annotate_with(&nl, &user, &ev);
    assert!(p.intent.devices.iter().any(|f| f.role == analog::intent::DeviceRole::CurrentSource));
    assert_eq!(
        (p.net_classes[r.0 as usize].class, p.intent.nets[r.0 as usize].evidence),
        (analog::metadata::NetClass::Signal, analog::intent::EvidenceLevel::User)
    );
}

/// EXT-24: a sense resistor across the input pair's gates is Kelvin-sensed at
/// each end by the gate on that net (one request per terminal).
/// Oracle: the input: RS across the pair's gates, so one Kelvin request per terminal.
#[test]
fn kelvin_from_sense_resistor() {
    let nl = net(&format!("{} | RS vinp vinm rpoly w=2u l=10u", src("ota5t")));
    let p = annotate(&nl, &cfg("ota5t"));
    let rs = analog::base::ids::DeviceId(nl.devices.iter().position(|d| d.name == "RS").unwrap() as u16);
    let got: Vec<(analog::intent::Term, Vec<String>)> = p
        .intent
        .kelvins
        .iter()
        .filter(|k| k.device == rs)
        .map(|k| (k.term, k.sense.iter().map(|s| nl.devices[s.0 .0 as usize].name.clone()).collect()))
        .collect();
    use analog::intent::Term::{N, P};
    assert_eq!(got, [(P, vec!["XM1".to_string()]), (N, vec!["XM2".to_string()])]);
}

/// EXT-24: the DAC's shared top plate is capacitive, its driven plates resistive.
/// Oracle: the DAC's structure: the shared top plate is capacitive, the driven plates resistive.
#[test]
fn dac4_plate_rc() {
    let nl = net(src("dac4"));
    let p = annotate(&nl, &cfg("dac4"));
    let rc = |n: &str| p.intent.nets[net_id(&nl, n).0 as usize].rc;
    assert_eq!(rc("top"), analog::intent::RcClass::C);
    for b in ["b0", "b1", "b2", "b3"] {
        assert_eq!(rc(b), analog::intent::RcClass::R, "{b}");
    }
}

/// EXT-29: the audit runs on every circuit with an op point (Id 10 µA everywhere). gm 100 µS gives
/// 2·Id/gm = 200 mV of overdrive: three_stage has no `vgst_low`; gm 400 µS (50 mV) flags its
/// Current Moderate load set {M4, M5}.
/// Oracle: hand arithmetic: 2·Id/gm = 200 mV at 100 µS, 50 mV at 400 µS.
#[test]
fn audit_on_corpus() {
    use annotator::evidence::DeviceOp;
    let run = |src: &str, name: &str, gm_us: f64| {
        let nl = net(src);
        let op =
            DeviceOp { id_ua: 10.0, headroom_mv: 200.0, gm_us, power_uw: 0.0, vgs_mv: None, vbs_mv: None, vth_mv: None, gmb_us: None, gds_us: None };
        let ev = annotator::Evidence {
            op: Some(annotator::OpFacts { dev: vec![Some(op); nl.devices.len()], net_mv: vec![None; nl.nets.len()] }),
            ..Default::default()
        };
        let p = annotator::annotate_with(&nl, &cfg(name), &ev);
        let low: Vec<Vec<String>> =
            p.intent.diagnostics.iter().filter(|d| d.kind == "vgst_low").map(|d| names(&nl, d.devices.iter().copied())).collect();
        low
    };
    for (name, src) in all() {
        run(src, name, 100.0);
    }
    assert!(run(src("three_stage"), "three_stage", 100.0).is_empty());
    assert_eq!(run(src("three_stage"), "three_stage", 400.0), [vec!["M4", "M5"]]);
}
