//! Recognition regression net — ported from `frontend/annotator`'s tests onto
//! `analog::base::Netlist`. Proves the DSL catalog recognises the canonical analog
//! primitives after the port, and that the `do_not_identify` override suppresses a
//! block.
//!
//! Contents:
//! - `pub(crate) fn fet(name: &str, kind: DeviceKind, g: u16, d: u16, s: u16, b: u16, w: i64, l: i64) -> Device`
//! - `pub(crate) fn nets(names: &[&str]) -> Vec<Net>`
//! - `pub(crate) fn ota() -> Netlist`
//! - `pub(crate) fn three_stage() -> Netlist`

mod budgets;
mod rings;

use analog::base::ids::{DeviceId, NetId};
use analog::base::netlist::{Device, DeviceKind, Net, Netlist};
use analog::RuleBatch;

use crate::{annotate, AnnotationConfig, Block, BlockKind};

/// A FET with G,D,S,B terminals on the given net ids, W/L in nm.
pub(crate) fn fet(name: &str, kind: DeviceKind, g: u16, d: u16, s: u16, b: u16, w: i64, l: i64) -> Device {
    Device {
        name: name.into(),
        kind,
        model: String::new(),
        terminals: vec![("G".into(), NetId(g)), ("D".into(), NetId(d)), ("S".into(), NetId(s)), ("B".into(), NetId(b))],
        params: vec![("w".into(), w), ("l".into(), l)],
    }
}

pub(crate) fn nets(names: &[&str]) -> Vec<Net> {
    names.iter().map(|n| Net { name: (*n).into() }).collect()
}

/// 5-transistor OTA. Nets: 0=vout1 1=vinp 2=vtail 3=VSS 4=vout2 5=vinm 6=vbias
/// 7=VDD 8=vbn.
pub(crate) fn ota() -> Netlist {
    Netlist {
        devices: vec![
            fet("XM1", DeviceKind::Nmos, 1, 0, 2, 3, 10_000, 1_000),
            fet("XM2", DeviceKind::Nmos, 5, 4, 2, 3, 10_000, 1_000),
            fet("XM3", DeviceKind::Pmos, 6, 0, 7, 7, 20_000, 1_000),
            fet("XM4", DeviceKind::Pmos, 6, 4, 7, 7, 20_000, 1_000),
            fet("XM5", DeviceKind::Nmos, 8, 2, 3, 3, 40_000, 2_000),
        ],
        nets: nets(&["vout1", "vinp", "vtail", "VSS", "vout2", "vinm", "vbias", "VDD", "vbn"]),
        ..Default::default()
    }
}

/// The three-stage op-amp (`examples/three_stage_opamp`, corpus `three_stage`).
pub(crate) fn three_stage() -> Netlist {
    // Nets: 0=n1 1=vin_p 2=tail 3=vss 4=n2 5=vin_n 6=vbias 7=vdd 8=n3 9=vout.
    let (n, p) = (DeviceKind::Nmos, DeviceKind::Pmos);
    Netlist {
        devices: vec![
            fet("M1", n, 1, 0, 2, 3, 4_000, 500),
            fet("M2", n, 5, 4, 2, 3, 4_000, 500),
            fet("M3", n, 6, 2, 3, 3, 8_000, 500),
            fet("M4", p, 0, 0, 7, 7, 6_000, 500),
            fet("M5", p, 0, 4, 7, 7, 6_000, 500),
            fet("M6", p, 4, 8, 7, 7, 12_000, 500),
            fet("M7", n, 6, 8, 3, 3, 6_000, 500),
            fet("M8", p, 8, 9, 7, 7, 40_000, 500),
            fet("M9", n, 6, 9, 3, 3, 20_000, 500),
        ],
        nets: nets(&["n1", "vin_p", "tail", "vss", "n2", "vin_n", "vbias", "vdd", "n3", "vout"]),
        ..Default::default()
    }
}

// ---- private ----

/// Block (non-glue) containing device id `d`, if any.
fn block_of(blocks: &[Block], d: u16) -> Option<&Block> {
    blocks.iter().find(|b| !matches!(b.kind, BlockKind::Glue) && b.devices.contains(&DeviceId(d)))
}

#[test]
fn diff_pair_halves_share_a_block() {
    let nl = ota();
    let p = annotate(&nl, &AnnotationConfig::default());
    // XM1 (0) and XM2 (1) must land in the same recognised block — whether the
    // winner is a bare diff_pair or a composite (five_transistor_ota / DP+load).
    let b1 = block_of(&p.blocks, 0).expect("XM1 recognised");
    let b2 = block_of(&p.blocks, 1).expect("XM2 recognised");
    assert!(std::ptr::eq(b1, b2), "diff-pair halves split across blocks");
}

#[test]
fn current_mirror_recognised() {
    // XM1 diode-connected (D=G=vref net 0), XM2 output (D=iout net 1, G=vref).
    let nl = Netlist {
        devices: vec![fet("XM1", DeviceKind::Pmos, 0, 0, 2, 2, 5_000, 1_000), fet("XM2", DeviceKind::Pmos, 0, 1, 2, 2, 5_000, 1_000)],
        nets: nets(&["vref", "iout", "VDD"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    let b = block_of(&p.blocks, 0).expect("mirror recognised");
    assert_eq!(b.kind, BlockKind::CurrentMirror);
    assert_eq!(b.devices, [DeviceId(0), DeviceId(1)], "slot 0 is the diode reference");
}

#[test]
fn cross_coupled_recognised() {
    // XM1: G=outn(1) D=outp(0); XM2: G=outp(0) D=outn(1).
    let nl = Netlist {
        devices: vec![fet("XM1", DeviceKind::Pmos, 1, 0, 2, 2, 2_000, 500), fet("XM2", DeviceKind::Pmos, 0, 1, 2, 2, 2_000, 500)],
        nets: nets(&["outp", "outn", "VDD", "VSS"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    let b = block_of(&p.blocks, 0).expect("cross-coupled recognised");
    assert_eq!(b.kind, BlockKind::DiffPair, "cross-coupled pairs are symmetric matched pairs");
}

#[test]
fn do_not_identify_suppresses_block() {
    let nl = ota();
    let cfg = AnnotationConfig { do_not_identify: [0u32].into_iter().collect(), ..Default::default() };
    let p = annotate(&nl, &cfg);
    // XM1 (device 0) is blocked → it cannot appear in any recognised block; it
    // falls through to glue.
    assert!(block_of(&p.blocks, 0).is_none(), "XM1 should be suppressed from recognition");
    let glue = p.blocks.iter().find(|b| matches!(b.kind, BlockKind::Glue)).expect("glue");
    assert!(glue.devices.contains(&DeviceId(0)));
}

/// Find a leaf (top block or nested sub_block) of `kind` covering exactly the
/// given device ids, searching the hierarchy.
fn has_leaf(blocks: &[Block], kind: BlockKind, ids: &[u16]) -> bool {
    fn walk(b: &Block, kind: BlockKind, want: &[u16]) -> bool {
        let here = b.kind == kind && {
            let mut got: Vec<u16> = b.devices.iter().map(|d| d.0).collect();
            got.sort_unstable();
            let mut w = want.to_vec();
            w.sort_unstable();
            got == w
        };
        here || b.sub_blocks.iter().any(|c| walk(c, kind, want))
    }
    blocks.iter().any(|b| walk(b, kind, ids))
}

/// Total individual rules across a hard/cost batch list.
fn count(batches: &[Box<dyn RuleBatch<analog::base::Layout>>]) -> usize {
    batches.iter().map(|b| b.count()).sum()
}

#[test]
fn diff_pair_lives_in_the_hierarchy() {
    // Whether the OTA matches as a composite (diff pair swallowed → recovered as a
    // sub_block) or as a standalone diff_pair, a DiffPair leaf covering {XM1,XM2}
    // must exist somewhere in the hierarchy.
    let nl = ota();
    let p = annotate(&nl, &AnnotationConfig::default());
    assert!(has_leaf(&p.blocks, BlockKind::DiffPair, &[0, 1]), "no DiffPair leaf for XM1/XM2");
}

#[test]
fn diff_pair_emits_its_constraints() {
    // A recognised diff pair emits Symmetry (hard) and MatchedSet (budget +
    // cost: one pair's gradient, thermal and LOD ledger, PLAN §4d) — the
    // BlockKind mapping, driven by recognition. MatchedSet is never Hard: its
    // allowance follows device AREA (Pelgrom `σ²_u = A²/(W·L)`), which no
    // placement move can change, so gating SA on it is inert. See
    // `backend/TODO.md` §2.
    let nl = ota();
    let p = annotate(&nl, &AnnotationConfig::default());
    assert!(count(&p.placement.hard) >= 1, "expected >=1 hard placement rule (sym), got {}", count(&p.placement.hard));
    assert!(count(&p.placement.budget) >= 1, "expected >=1 budget placement rule (thermal), got {}", count(&p.placement.budget));
    assert!(count(&p.placement.cost) >= 2, "expected >=2 cost placement rules (matched set/prox), got {}", count(&p.placement.cost));

    // The partition itself is the contract: nothing in `hard` may be a rule the
    // placer cannot act on.
    let hard_kinds: Vec<&str> = p.placement.hard.iter().map(|b| b.kind()).collect();
    assert!(!hard_kinds.iter().any(|k| k.contains("MatchedSet")), "MatchedSet must not gate placement moves: {hard_kinds:?}");
}

#[test]
fn budget_rules_land_in_exactly_one_partition() {
    // D15's invariant, and the one this partition exists to establish: the two-arm
    // code simulated the missing middle tier by registering a batch in *both* `hard`
    // and `cost` via `clone`, and a batch in two arms is counted twice by
    // `Report::lex`, priced twice by `gp::Prices`, and sits both above and below the
    // feasibility frontier. Double-registration is a `clone` away, so it is asserted.
    let nl = ota();
    let p = annotate(&nl, &AnnotationConfig::default());
    let arms: [(&str, &Vec<Box<dyn RuleBatch<analog::base::Routes>>>); 3] =
        [("hard", &p.routing.hard), ("budget", &p.routing.budget), ("cost", &p.routing.cost)];
    for kind in ["Differential", "CrosstalkExclusion", "ParasiticBudget", "CouplingBudget"] {
        let hits: Vec<&str> = arms.iter().filter(|(_, a)| a.iter().any(|b| b.kind().ends_with(kind))).map(|(name, _)| *name).collect();
        assert_eq!(hits, ["budget"], "{kind} must be registered once, in `budget`");
    }

    // The trap, from the other side: `SymmetryGroup` matches the double-registration
    // shape that *locates* a budget and is emphatically not one — an exact equality is
    // not tradeable at any price, and pricing it is PLAN §4a's park-one-grid-unit-off
    // failure reported as convergence.
    // (`SymmetryGroup` reports itself as `"Symmetry"` — the rule it enforces, not the
    // container.)
    let sym = |a: &Vec<Box<dyn RuleBatch<analog::base::Layout>>>| a.iter().any(|b| b.kind() == "Symmetry");
    assert!(sym(&p.placement.hard), "SymmetryGroup must stay Hard");
    assert!(!sym(&p.placement.budget), "SymmetryGroup must never be priced as a budget");

    // And the placement-tier budget, from the other other side: `MatchedSet`
    // carries an allowance on derived fields (gradient, live thermal, LOD), is
    // tradeable while converging, and PLAN §4d calls it a budget outright. Its
    // cost copy is a gradient, not a classification; the classified copy must
    // be priced, never gated.
    let set = |a: &Vec<Box<dyn RuleBatch<analog::base::Layout>>>| a.iter().any(|b| b.kind().ends_with("MatchedSet"));
    assert!(set(&p.placement.budget), "MatchedSet must be a priced budget");
    assert!(!set(&p.placement.hard), "MatchedSet must never gate legality as Hard");
}

#[test]
fn matched_pairs_get_orientation_rules() {
    // MAT-05: channel axes are a hard placement rule (a quarter-turned partner is
    // illegal); Φ is budget only (a discrete flip, no gradient for a cost copy).
    let p = annotate(&ota(), &AnnotationConfig::default());
    let has = |a: &Vec<Box<dyn RuleBatch<analog::base::Layout>>>| a.iter().any(|b| b.kind() == "Orientation");
    assert!(has(&p.placement.hard), "Orientation axis must be Hard");
    assert!(has(&p.placement.budget), "Orientation Φ must be a budget");
    assert!(!has(&p.placement.cost), "Orientation has no cost copy");
}

#[test]
fn parasitic_and_coupling_report_an_overshoot_not_a_count() {
    // D17: Θ sums residuals, and a residual is only summable normalised by its own
    // budget. These two batches carry a spec *and* a measured quantity, so a budget
    // blown many times over must read as many times worse — the property a count cannot
    // express, and the one that lets the search sit on a huge violation forever.
    use analog::base::geom::{LayerId, Rect, Shape};
    let wire = |y: i32| Shape { layer: LayerId(0), rect: Rect { x: 0, y, w: 100_000_000, h: 1 } };
    // 100 mm of metal per net — past every class's length budget by ~10×; the two runs
    // are parallel 1 nm apart, so the coupling sum is past its budget too.
    let routes = analog::base::routes::Routes { wires: vec![vec![wire(0)], vec![wire(2)]], ..Default::default() };

    // Budgets exist only when the deck gives Cox and wire C (sky130-like numbers).
    let mut cfg = AnnotationConfig::default();
    cfg.process.gate_af_per_um2 = Some(8_325.0);
    cfg.process.wire_af_per_um = Some(50.0);
    let p = annotate(&ota(), &cfg);
    for kind in ["ParasiticBudget", "CouplingBudget"] {
        let b = p.routing.budget.iter().find(|b| b.kind().ends_with(kind)).expect("budget batch");
        let (v, res) = (f64::from(b.violations(&routes)), b.residual(&routes));
        assert!(v > 0.0, "{kind}: fixture must actually violate");
        assert!(res > v, "{kind}: residual {res} degraded to a count of {v}");
    }
}

#[test]
fn dti_bands_are_one_hard_batch_with_dense_ids_seeded_share() {
    // The producer half of PLAN §4b: every matched pair in the OTA (diff pair,
    // mirror legs, load) gets a `DtiBand`, all in ONE batch registered hard+cost —
    // the hard copy is legality (the full disjunction), the cost copy is what makes
    // `dp`'s branch flip priceable, and the budget arm never sees it (a disjunction
    // is not tradeable, and `Prices::bind` asserts hard ∩ budget = ∅).
    // Bands exist only on a trench process: the deck supplies the rule.
    let mut cfg = AnnotationConfig::default();
    cfg.process.dti = Some((2_000, 1_000));
    let p = annotate(&ota(), &cfg);
    let is_dti = |b: &Box<dyn RuleBatch<analog::base::Layout>>| b.kind().ends_with("DtiBand");
    assert_eq!(p.placement.hard.iter().filter(|b| is_dti(b)).count(), 1, "one DtiBand batch, registered once at the end of placement()");
    assert!(p.placement.cost.iter().any(is_dti), "the cost copy prices the flip");
    assert!(!p.placement.budget.iter().any(is_dti), "a disjunction is never a budget");

    // Dense, distinct, allocation order = emission order — the id contract that
    // keeps one pair's commitment from silently transferring to another.
    let mut seeds = Vec::new();
    let dti = p.placement.hard.iter().find(|b| is_dti(b)).unwrap();
    dti.branches(&mut seeds);
    assert!(seeds.len() >= 2, "the OTA has more than one matched pair: {seeds:?}");
    for (i, &(id, isolate)) in seeds.iter().enumerate() {
        assert_eq!(usize::from(id.0), i, "ids must be dense in emission order");
        assert!(!isolate, "matched structures seed `share` — one trench by design");
    }
}

#[test]
fn abutment_excludes_mixed_polarity_groups() {
    // D16: `groups` is the recognition table and deliberately contains composites
    // spanning both polarities; `abutment` is diffusion-sharing permission. Granting
    // abutment on a mixed group lets an NMOS and a PMOS merge implants — DRC-clean,
    // LVS-fatal, and unrepairable downstream because nothing is illegal.
    let nl = ota(); // XM1/XM2/XM5 Nmos, XM3/XM4 Pmos
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(p.abutment.len(), p.groups.len(), "abutment is indexed by GroupId too");
    for (gi, (grp, ab)) in p.groups.iter().zip(&p.abutment).enumerate() {
        let mut kinds = grp.iter().filter_map(|d| nl.devices.get(d.0 as usize)).map(|d| d.kind);
        let first = kinds.next();
        let same = kinds.all(|k| Some(k) == first);
        // A mixed group is truncated to one member (which grants no abutment, since
        // `group_index` ignores groups below two members) rather than emptied — an
        // empty group panics `Layout::bbox`.
        assert_eq!(ab.len(), if same { grp.len() } else { 1 }, "group {gi}: {grp:?} -> {ab:?}");
    }
}

#[test]
fn glue_only_netlist_emits_no_placement() {
    // Two unrelated resistors → nothing recognised → no placement constraints.
    let nl = Netlist {
        devices: vec![
            Device {
                name: "R1".into(),
                kind: DeviceKind::Resistor,
                model: String::new(),
                terminals: vec![("A".into(), NetId(0)), ("B".into(), NetId(1))],
                params: vec![],
            },
            Device {
                name: "R2".into(),
                kind: DeviceKind::Resistor,
                model: String::new(),
                terminals: vec![("A".into(), NetId(2)), ("B".into(), NetId(3))],
                params: vec![],
            },
        ],
        nets: nets(&["a", "b", "c", "d"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(count(&p.placement.hard), 0);
    assert_eq!(count(&p.placement.cost), 0);
}

#[test]
fn missing_is_relevant() {
    // Two resistors: no matched leaf and no gate, so no MatchedSet or Antenna entry.
    let nl = Netlist {
        devices: vec![
            Device {
                name: "R1".into(),
                kind: DeviceKind::Resistor,
                model: String::new(),
                terminals: vec![("A".into(), NetId(0)), ("B".into(), NetId(1))],
                params: vec![],
            },
            Device {
                name: "R2".into(),
                kind: DeviceKind::Resistor,
                model: String::new(),
                terminals: vec![("A".into(), NetId(2)), ("B".into(), NetId(3))],
                params: vec![],
            },
        ],
        nets: nets(&["a", "b", "c", "d"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    // ... and no gate area, so no net budget either.
    assert!(!p.missing.iter().any(|m| ["MatchedSet", "Antenna", "ParasiticBudget", "CouplingBudget"].contains(&m.0)), "{:?}", p.missing);
    let p = annotate(&ota(), &AnnotationConfig::default());
    assert!(p.missing.contains(&("MatchedSet", "deck svt_uv_per_um — distance term unknown")), "{:?}", p.missing);
    assert!(p.missing.contains(&("ParasiticBudget", "deck gate_cap_af_um2")), "{:?}", p.missing);
}

#[test]
fn every_batch_is_tagged() {
    let mut clocked = ota();
    clocked.nets.push(Net { name: "clk".into() });
    clocked.nets.push(Net { name: "sw".into() });
    let (clk, sw) = (clocked.nets.len() as u16 - 2, clocked.nets.len() as u16 - 1);
    clocked.devices.push(fet("XS", DeviceKind::Nmos, clk, sw, 3, 3, 1_000, 150));
    for nl in [ota(), clocked] {
        let p = annotate(&nl, &AnnotationConfig::default());
        let (pl, ro) = (&p.placement, &p.routing);
        let mut ids: Vec<u32> = (pl.hard.iter().chain(&pl.budget).chain(&pl.cost))
            .map(|b| b.meta().expect("placement batch tagged").id.0)
            .chain(ro.hard.iter().chain(&ro.budget).chain(&ro.cost).map(|b| b.meta().expect("routing batch tagged").id.0))
            .collect();
        let total = ids.len() as u32;
        assert!(total > 0);
        assert_eq!(ids, (0..total).collect::<Vec<_>>(), "dense, in arm order");
        ids.dedup();
        assert_eq!(ids.len() as u32, total);

        // Origin: the XM1/XM2 pair came from its pattern; Isolation and routing from net classes.
        let net_class = Some(analog::intent::Origin::NetClass);
        let origin = |b: &dyn analog::RuleBatch<analog::base::Layout>| b.meta().map(|m| m.origin);
        let pair = (pl.cost.iter()).find(|b| {
            let mut t = Vec::new();
            b.touched(&mut t);
            t.sort_unstable();
            b.kind() == "MatchedSet" && t == [0, 1]
        });
        assert_eq!(origin(pair.expect("XM1/XM2 MatchedSet").as_ref()), Some(analog::intent::Origin::Pattern { template: "five_transistor_ota" }));
        let iso = (pl.hard.iter().chain(&pl.budget).chain(&pl.cost)).filter(|b| b.kind().ends_with("::Isolation"));
        assert!(iso.map(|b| origin(b.as_ref())).all(|o| o == net_class));
        assert!(ro.hard.iter().chain(&ro.budget).chain(&ro.cost).all(|b| b.meta().map(|m| m.origin) == net_class));
    }
}

#[test]
fn every_device_accounted_for() {
    // No device is lost: recognised ∪ glue == all devices, disjoint.
    let nl = ota();
    let p = annotate(&nl, &AnnotationConfig::default());
    let mut seen = vec![false; nl.devices.len()];
    for b in &p.blocks {
        for d in &b.devices {
            assert!(!seen[d.0 as usize], "device {} in two blocks", d.0);
            seen[d.0 as usize] = true;
        }
    }
    assert!(seen.into_iter().all(|s| s), "some device unaccounted for");
}

#[test]
fn a_unitization_never_mixes_kinds_or_sizes() {
    // `unit_w`/`unit_l`/`device_type` are one-per-group scalars, so every member
    // must genuinely carry them. When they did not (the block-level W read off
    // `devices[0]`), `cells::builder::sizing` drew the odd member at the wrong
    // size and LVS reported `lvs.parameter_mismatch`. The OTA mixes N/P and
    // three widths inside its recognised blocks, so it is the case that bites.
    let nl = ota();
    let p = annotate(&nl, &AnnotationConfig::default());
    let param = |d: DeviceId, k: &str| nl.devices[d.0 as usize].params.iter().find(|(n, _)| n == k).map(|&(_, v)| v);
    for u in &p.constraints.unitization {
        for &d in &u.devices {
            assert_eq!(nl.devices[d.0 as usize].kind, u.device_type, "unitization {:?} mixes kinds", u.devices);
            let w_finger = nl.devices[d.0 as usize].mos_size().map(|s| s.w_finger_nm());
            assert_eq!(w_finger, Some(i64::from(u.unit_w)), "unitization {:?} mixes W", u.devices);
            assert_eq!(param(d, "l"), Some(i64::from(u.unit_l)), "unitization {:?} mixes L", u.devices);
        }
    }
}

#[test]
fn guard_rings_tie_to_the_guarded_device_s_bulk() {
    // A ring is a substrate/well tap: it must land on the bulk rail, never on
    // whatever net the netlist numbered first. `connection_net` used to be a
    // hardcoded `NetId(0)` — here `vout1` — and `dr` folds ring pins in as real
    // routing terminals, so the router wired every guard ring in the design to
    // that signal net.
    // REL-07: only an aggressor gets a ring, so M1 is clocked; its bulk
    // (net 3) is not net 0.
    let nl = Netlist {
        devices: vec![fet("M1", DeviceKind::Nmos, 1, 2, 3, 3, 1_000, 150), fet("M2", DeviceKind::Pmos, 1, 2, 4, 4, 1_000, 150)],
        nets: nets(&["out", "clk", "x", "vss", "vdd"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig { clock_nets: vec!["clk".into()], ..AnnotationConfig::default() });
    assert!(!p.constraints.guard_rings.is_empty(), "clocked FETs get rings");
    for r in &p.constraints.guard_rings {
        let dev = &nl.devices[r.device.0 as usize];
        let bulk = dev.terminals.iter().find(|(t, _)| t == "B").expect("a FET states a bulk").1;
        assert_eq!(r.connection_net, bulk, "{}'s ring ties to net {} instead of its bulk {}", dev.name, r.connection_net.0, bulk.0);
    }
}

#[test]
fn a_differential_stage_is_symmetric_about_one_axis() {
    // 5T OTA with a biased (non-diode) load: one five_transistor_ota stage. The
    // diff pair and the load mirror about the stage axis; the tail sits on it.
    let p = annotate(&ota(), &AnnotationConfig::default());
    assert_eq!(p.blocks[0].devices.len(), 5, "tail and biased load belong to the OTA stage");
    let sym: usize = p.placement.hard.iter().filter(|b| b.kind() == "Symmetry").map(|b| b.count()).sum();
    assert_eq!(sym, 3, "diff pair + load pair + self-symmetric tail");
    let tail = p.constraints.unitization.iter().find(|u| u.devices == [DeviceId(4)]);
    assert!(tail.is_some(), "the tail gets a sizing directive");
    assert!(p.constraints.guard_rings.is_empty(), "no aggressor: no ring (T8)");
}

/// AA-23: only declared selfs go on the axis. `telescopic_ota_full` declares
/// the tail (slot 6) and no role for the load bias (slot 7, a diode PMOS on the
/// loads' source). Nets: 0=inp 1=x1 2=tail 3=VSS 4=inn 5=x2 6=vbn2 7=o1 8=o2
/// 9=vbp 10=VDD 11=vbn.
#[test]
fn telescopic_slot7_is_not_self_symmetric() {
    let (n, p) = (DeviceKind::Nmos, DeviceKind::Pmos);
    let nl = Netlist {
        devices: vec![
            fet("M0", n, 0, 1, 2, 3, 10_000, 1_000),
            fet("M1", n, 4, 5, 2, 3, 10_000, 1_000),
            fet("M2", n, 6, 7, 1, 3, 10_000, 1_000),
            fet("M3", n, 6, 8, 5, 3, 10_000, 1_000),
            fet("M4", p, 9, 7, 10, 10, 20_000, 1_000),
            fet("M5", p, 9, 8, 10, 10, 20_000, 1_000),
            fet("M6", n, 11, 2, 3, 3, 40_000, 1_000),
            fet("M7", p, 9, 9, 10, 10, 5_000, 1_000),
        ],
        nets: nets(&["inp", "x1", "tail", "VSS", "inn", "x2", "vbn2", "o1", "o2", "vbp", "VDD", "vbn"]),
        ..Default::default()
    };
    let pr = annotate(&nl, &AnnotationConfig::default());
    assert_eq!((pr.blocks[0].template, pr.blocks[0].devices.len()), ("telescopic_ota_full", 8));
    let mut m = Vec::new();
    pr.placement.hard.iter().for_each(|b| b.mirror_pairs(&mut m));
    let selfs: Vec<u32> = m.iter().filter(|p| p.0 == p.1).map(|p| p.0).collect();
    assert_eq!(selfs, [6], "the tail only, not slot 7: {m:?}");
}

#[test]
fn a_shared_gate_chain_is_a_series_stack_not_a_cascode() {
    // chain4: four same-size nfets in series on one gate net.
    let nl = Netlist {
        devices: (0..4u16).map(|i| fet(&format!("M{i}"), DeviceKind::Nmos, 0, i + 1, i + 2, 6, 2_000, 500)).collect(),
        nets: nets(&["g", "a", "b", "c", "d", "e", "VSS"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(p.blocks[0].devices.len(), 4, "one series_stack_4 group");
    assert!(p.blocks[0].sub_blocks.iter().all(|b| b.kind == BlockKind::Stack), "stack pairs, not a cascode");
    // The shared gate net (gates only) is a bias line (EXT-18: was `Sensitive`).
    assert_eq!(p.net_classes[0].class, analog::metadata::NetClass::Bias);
}

#[test]
fn a_cascode_stack_is_adjacent_not_matched() {
    // M1 (bottom) drain feeds M2 (top) source: a stack, not a matched pair.
    let nl = Netlist {
        devices: vec![fet("M1", DeviceKind::Nmos, 0, 1, 2, 2, 4_000, 500), fet("M2", DeviceKind::Nmos, 3, 4, 1, 2, 8_000, 500)],
        nets: nets(&["vin", "x", "VSS", "vcas", "out"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(p.blocks[0].kind, BlockKind::Stack);
    let kinds: Vec<&str> = p.placement.cost.iter().map(|b| b.kind()).collect();
    assert!(kinds.iter().any(|k| k.ends_with("Proximity")), "{kinds:?}");
    assert!(!kinds.iter().any(|k| k.ends_with("MatchedSet")), "{kinds:?}");
    assert!(p.placement.hard.is_empty());
    // `vcas` (gates M2 only, no DC path) is a bias line (EXT-18: was `Sensitive`).
    assert_eq!(p.net_classes[3].class, analog::metadata::NetClass::Bias);
}

/// EXT-07: a `Stack` is adjacent/symmetric, not a gate reference, so it is no
/// longer an isolation victim. A clocked switch elsewhere used to be an
/// aggressor that forced isolation onto the stack's two devices. Since EXT-23
/// a Bias-gated FET is a victim on its own, so `R1`/`R2` give both gates a DC
/// path (Signal, not Bias): only the stack membership is under test.
#[test]
fn a_cascode_stack_is_not_an_isolation_victim() {
    let r = |name: &str, p: u16| Device {
        name: name.into(),
        kind: DeviceKind::Resistor,
        model: String::new(),
        terminals: vec![("P".into(), analog::base::NetId(p)), ("N".into(), analog::base::NetId(2))],
        params: vec![],
    };
    let nl = Netlist {
        devices: vec![
            fet("M1", DeviceKind::Nmos, 0, 1, 2, 2, 4_000, 500),
            fet("M2", DeviceKind::Nmos, 3, 4, 1, 2, 8_000, 500),
            fet("XS", DeviceKind::Nmos, 5, 6, 2, 2, 1_000, 150),
            r("R1", 0),
            r("R2", 3),
        ],
        nets: nets(&["vin", "x", "VSS", "vcas", "out", "clk", "sw"]),
        ..Default::default()
    };
    let p = annotate(&nl, &AnnotationConfig::default());
    let mut touched = Vec::new();
    for b in &p.placement.cost {
        if b.kind().ends_with("Isolation") {
            b.touched(&mut touched);
        }
    }
    assert!(!touched.contains(&0) && !touched.contains(&1), "stack devices are no longer isolation victims: {touched:?}");
}
