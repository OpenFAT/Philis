//! Recognition corpus (EXT-01, plan-01 T2/T3/T4/T6/T7): 18 circuits plus 3
//! negative-only ones, each with the leaves and [`Canon`] the annotator gives **today**. The
//! expectations characterise, they do not endorse: each later EXT item edits the
//! rows it changes in the same commit, so its diff shows exactly what moved.
//!
//! Contents: none public.

#[path = "../common/mod.rs"]
mod common;
mod data;
mod emits;

use std::collections::{BTreeMap, BTreeSet};

use annotator::{annotate, AnnotationConfig, BlockKind};

use common::{canon, canon_leaves, id_pairs, net, permute, strongarm_cfg, Canon, STRONGARM};
use data::{CIRCUITS, COMPOUNDS, EXPECTED, NEGATIVE};

// ---- private ----

fn cfg(name: &str) -> AnnotationConfig {
    if name == "strongarm" {
        strongarm_cfg()
    } else {
        common::cfg()
    }
}

fn all() -> impl Iterator<Item = (&'static str, &'static str)> {
    CIRCUITS.into_iter().chain(NEGATIVE)
}

/// Oracle: golden: the EXPECTED rows characterise today's output (see the file header). No independent
/// oracle.
#[test]
fn corpus_expectations() {
    assert_eq!(CIRCUITS.map(|c| c.0), EXPECTED.map(|r| r.0), "one row per circuit, same order");
    let owned = |v: &[(&str, &str)]| v.iter().map(|(a, b)| ((*a).to_string(), (*b).to_string())).collect();
    let mut bad = Vec::new();
    for ((name, src), (_, leaves, pairs, selfs, net_pairs, axes, sets)) in CIRCUITS.into_iter().zip(EXPECTED) {
        let nl = net(src);
        let p = annotate(&nl, &cfg(name));
        let got = (canon(&p, &nl), canon_leaves(&p, &nl));
        let exp = (
            Canon {
                sets: sets
                    .iter()
                    .map(|(m, k, c)| (m.iter().map(|&(n, p, s)| (n.to_string(), p, s)).collect(), (*k).to_string(), (*c).to_string()))
                    .collect(),
                pairs: owned(pairs),
                selfs: selfs.iter().map(|s| (*s).to_string()).collect(),
                net_pairs: owned(net_pairs),
                axes,
            },
            leaves.iter().map(|(k, m)| ((*k).to_string(), m.iter().map(|s| (*s).to_string()).collect())).collect::<BTreeSet<(String, Vec<String>)>>(),
        );
        if got != exp {
            bad.push(format!("{name}:\n  got  {got:?}\n  want {exp:?}"));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// T3 at M0: no symmetry pair, no matched set, and no matched leaf (DiffPair,
/// CurrentMirror, Load: today's matched sets, before EXT-12's `MatchSpec`).
fn assert_unmatched(name: &str, src: &str) {
    let nl = net(src);
    let p = annotate(&nl, &cfg(name));
    let c = canon(&p, &nl);
    assert!(c.pairs.is_empty() && c.sets.is_empty(), "{name}: {c:?}");
    assert!(p.intent.compounds.is_empty(), "{name}: compounds {:?}", common::canon_intent(&p, &nl));
    let matched: Vec<_> = annotator::block::leaves(&p.blocks)
        .into_iter()
        .filter(|b| matches!(b.kind, BlockKind::DiffPair | BlockKind::CurrentMirror | BlockKind::Load))
        .map(|b| (b.kind, b.devices.clone()))
        .collect();
    assert!(matched.is_empty(), "{name}: matched leaves {matched:?}");
}

/// The netlist named `name` in [`CIRCUITS`] or [`NEGATIVE`].
fn src(name: &str) -> &'static str {
    all().find(|c| c.0 == name).unwrap_or_else(|| panic!("no circuit {name}")).1
}

/// Oracle: the inputs: a BJT mirror and an inverter chain hold no FET pattern by construction.
#[test]
fn negative_corpus() {
    for name in ["bjt_mirror", "inv_chain"] {
        assert_unmatched(name, src(name));
    }
}

/// Oracle: the inputs: switches and unrelated equal FETs hold no pattern by construction.
#[test]
fn negative_corpus_sc_switches_and_equal_fets() {
    for name in ["sc_switches", "equal_fets"] {
        assert_unmatched(name, src(name));
    }
}

/// Oracle: metamorphic: 20 seeded permutations of each netlist give the same canonical output.
#[test]
fn permutation_invariance() {
    for (name, src) in all() {
        let nl = net(src);
        let p = annotate(&nl, &cfg(name));
        let base = (canon(&p, &nl), canon_leaves(&p, &nl), common::canon_intent(&p, &nl));
        for seed in 1..=20 {
            let q = permute(&nl, seed);
            let pq = annotate(&q, &cfg(name));
            assert_eq!((canon(&pq, &q), canon_leaves(&pq, &q), common::canon_intent(&pq, &q)), base, "{name} seed {seed}");
        }
    }
}

/// T4 on the HSMPG tree (EXT-13): rendered by name, it is the same under permutation.
/// Oracle: metamorphic: the HSMPG tree rendered by name is the same under permutation.
#[test]
fn tree_is_permutation_invariant() {
    let tree = |nl: &analog::base::Netlist, name: &str| {
        let names: Vec<&str> = nl.devices.iter().map(|d| d.name.as_str()).collect();
        annotator::graph::render(&annotate(nl, &cfg(name)).intent.tree, &names)
    };
    for (name, src) in CIRCUITS {
        let nl = net(src);
        let base = tree(&nl, name);
        assert!(base.starts_with("Root{"), "{name}: {base}");
        for seed in 1..=3 {
            assert_eq!(tree(&permute(&nl, seed), name), base, "{name} seed {seed}");
        }
    }
}

/// T6 over the emitted placement arms of `name`: the unordered device pairs
/// carrying both a `Proximity` and an `Isolation`, and the devices in more
/// than one hard `Symmetry` entry (two pairs, or two axes). Both empty is T6.
fn conflicts(name: &str, src: &str) -> (Vec<(String, String)>, Vec<String>) {
    let nl = net(src);
    let p = annotate(&nl, &cfg(name));
    let arms = || p.placement.hard.iter().chain(&p.placement.budget).chain(&p.placement.cost);
    let pairs = |kind: &str| -> BTreeSet<(u32, u32)> {
        arms().filter(|b| b.kind().ends_with(kind)).flat_map(|b| id_pairs(b.as_ref())).map(|(a, b)| (a.min(b), a.max(b))).collect()
    };
    let dev = |d: u32| nl.devices[d as usize].name.clone();
    let both = pairs("::Proximity").intersection(&pairs("::Isolation")).map(|&(a, b)| (dev(a), dev(b))).collect();
    let mut mirror = Vec::new();
    p.placement.hard.iter().for_each(|b| b.mirror_pairs(&mut mirror));
    let mut seen = BTreeMap::new();
    for &(a, b, _) in &mirror {
        for d in if a == b { vec![a] } else { vec![a, b] } {
            *seen.entry(d).or_insert(0) += 1;
        }
    }
    let twice = seen.into_iter().filter(|&(_, n)| n > 1).map(|(d, _)| dev(d)).collect();
    (both, twice)
}

fn assert_no_conflicts(name: &str, src: &str) {
    let (both, twice) = conflicts(name, src);
    assert!(both.is_empty(), "{name}: Proximity and Isolation on {both:?}");
    assert!(twice.is_empty(), "{name}: devices in two symmetry entries: {twice:?}");
}

/// T6 on every corpus circuit but strongarm, which has its own test below.
/// Oracle: certificate: the conflict check over the emitted batches finds none.
#[test]
fn no_emitted_conflicts() {
    for (name, src) in all().filter(|c| c.0 != "strongarm") {
        assert_no_conflicts(name, src);
    }
}

/// GAP-09: no rule clash on any corpus circuit (master §6 "EXT M3").
/// Oracle: certificate: no `conflict` diagnostic on any corpus circuit.
#[test]
fn clean_corpus_has_no_conflicts() {
    for (name, src) in all() {
        let p = annotate(&net(src), &cfg(name));
        let c: Vec<_> = p.intent.diagnostics.iter().filter(|d| d.kind == "conflict").collect();
        assert!(c.is_empty(), "{name}: {c:?}");
    }
}

/// AA-13: the clocked tail `mn0` sits in the input pair's stage, so it gets
/// Proximity ≤ 5 µm to `mn1`/`mn2`; Isolation
/// from them would contradict it. GAP-04 exempts same-block pairs.
/// Oracle: certificate: the conflict check on strongarm finds none (GAP-04 exempts same-block pairs).
#[test]
fn no_emitted_conflicts_strongarm() {
    assert_no_conflicts("strongarm", STRONGARM);
}

/// EXT-09/EXT-24: `Differential` comes only from recognised structure (a
/// compound's mirrored nets), never a positional/O(N²) device-pair scan, and
/// lands in `routing.budget`, not `.hard`.
/// Oracle: the inputs: dac4 has no pair, so no Differential; elsewhere only budget rows.
#[test]
fn differential_comes_from_recognized_pairs() {
    use analog::RuleBatch;
    let count = |arm: &[Box<dyn RuleBatch<analog::base::Routes>>]| -> usize {
        arm.iter().filter(|b| b.kind().ends_with("::Differential")).map(|b| b.count()).sum()
    };
    // dac4 has no recognised pair (it is a DAC capacitor bank): 0 Differential
    // anywhere (today's positional scan finds 10 on its NMOS/PMOS switch pairs).
    let nl = net(&src("dac4").replace("VSS", "0"));
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(count(&p.routing.hard) + count(&p.routing.budget) + count(&p.routing.cost), 0, "dac4");
    assert!(xtalk(&nl, &p).is_empty(), "dac4 crosstalk");

    // ota5t's one compound mirrors (vinm, vinp) and (vout1, vout2): exactly 2
    // Differential, in budget only, and a CrosstalkExclusion from each input
    // gate to each output drain (the Voltage set's G×D).
    let nl = net(src("ota5t"));
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(count(&p.routing.hard), 0, "ota5t hard");
    assert_eq!(count(&p.routing.budget), 2, "ota5t budget");
    let want: BTreeSet<_> =
        [("vinm", "vout1"), ("vinm", "vout2"), ("vinp", "vout1"), ("vinp", "vout2")].map(|(a, b)| (a.to_string(), b.to_string())).into();
    let got = xtalk(&nl, &p);
    assert_eq!((got.len(), got.into_iter().collect::<BTreeSet<_>>()), (4, want), "ota5t crosstalk");

    // latch: its two DiffPair leaves share drains (q, qb); one rule per net pair.
    let nl = net(src("latch"));
    let p = annotate(&nl, &AnnotationConfig::default());
    assert_eq!(count(&p.routing.budget), 1, "latch Differential");
    assert_eq!(xtalk(&nl, &p), [("q".to_string(), "qb".to_string())], "latch crosstalk");
}

/// `CrosstalkExclusion` net-name pairs (each sorted), in emission order.
fn xtalk(nl: &analog::base::Netlist, p: &annotator::Problem) -> Vec<(String, String)> {
    let mut v = Vec::new();
    for b in p.routing.budget.iter().filter(|b| b.kind().ends_with("::CrosstalkExclusion")) {
        let mut nets = Vec::new();
        b.touched(&mut nets);
        for c in nets.chunks(2) {
            let (a, z) = (nl.nets[c[0] as usize].name.clone(), nl.nets[c[1] as usize].name.clone());
            v.push(if a < z { (a, z) } else { (z, a) });
        }
    }
    v
}

/// T7: every device is in a requirement or reported `Unconstrained(reason)`.
/// Oracle: partition: every device once, constrained or with a reason.
#[test]
fn coverage_is_total() {
    use annotator::Coverage::{Constrained, Grouped, Unconstrained};
    for (name, src) in all() {
        let nl = net(src);
        let p = annotate(&nl, &cfg(name));
        let ids: Vec<u16> = p.coverage.iter().map(|(d, _)| d.0).collect();
        assert_eq!(ids, (0..nl.devices.len() as u16).collect::<Vec<_>>(), "{name}: one entry per device, in id order");
        for (d, c) in &p.coverage {
            if let Unconstrained(why) = c {
                assert!(["do_not_identify", "unknown size", "no pattern"].contains(why), "{name} {d:?}: {why}");
            }
        }
        match name {
            "ota5t" => assert!(p.coverage.iter().all(|c| c.1 == Constrained), "{name}: {:?}", p.coverage),
            // A role-less composite: recognised, nothing emitted on it.
            "chain4" => assert!(p.coverage.iter().all(|c| c.1 == Grouped("series_stack_4")), "{name}: {:?}", p.coverage),
            // EXT-20: its FeedbackRatio set emits a MatchedSet.
            "rdiv" => assert!(p.coverage.iter().all(|c| c.1 == Constrained), "{name}: {:?}", p.coverage),
            _ => {}
        }
    }
}

/// EXT-10: a batch's id survives a netlist permutation (ids follow emission
/// order, which follows EXT-06's canonical block order).
/// Oracle: metamorphic: a batch's id is the same after permuting the netlist.
#[test]
fn ids_survive_permutation() {
    let id_of = |nl: &analog::base::Netlist| {
        let p = annotate(nl, &AnnotationConfig::default());
        let want: BTreeSet<&str> = ["XM1", "XM2"].into();
        let b = p.placement.cost.iter().find(|b| {
            let mut ids = Vec::new();
            b.touched(&mut ids);
            b.kind() == "MatchedSet" && ids.iter().map(|&d| nl.devices[d as usize].name.as_str()).collect::<BTreeSet<_>>() == want
        });
        b.expect("XM1/XM2 MatchedSet cost batch").meta().expect("tagged").id
    };
    let nl = net(src("ota5t"));
    let base = id_of(&nl);
    for s in 1..=5 {
        assert_eq!(id_of(&permute(&nl, s)), base, "seed {s}");
    }
}

/// Oracle: golden: the COMPOUNDS rows characterise today's output. No independent oracle.
#[test]
fn compound_expectations() {
    let owned = |v: &[(&str, &str)]| v.iter().map(|(a, b)| ((*a).to_string(), (*b).to_string())).collect();
    let mut bad = Vec::new();
    for (name, src) in all() {
        let nl = net(src);
        let p = annotate(&nl, &cfg(name));
        let got = common::canon_intent(&p, &nl);
        let exp = COMPOUNDS.iter().find(|r| r.0 == name).map_or_else(Canon::default, |&(_, pairs, selfs, np, axes)| Canon {
            pairs: owned(pairs),
            selfs: selfs.iter().map(|s| (*s).to_string()).collect(),
            net_pairs: owned(np),
            axes,
            ..Canon::default()
        });
        if got != exp {
            bad.push(format!("{name}:\n  got  {got:?}\n  want {exp:?}"));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
    // Gilbert's LO nets and tail sit on the axis (AA-02 trace §2.13-7c).
    let nl = net(src("gilbert"));
    let p = annotate(&nl, &AnnotationConfig::default());
    let self_nets: BTreeSet<&str> = p.intent.compounds[0].self_nets.iter().map(|n| nl.nets[n.0 as usize].name.as_str()).collect();
    assert!(["tail", "lop", "lon"].iter().all(|n| self_nets.contains(n)), "{self_nets:?}");
    // three_stage: the second and third stages stay out of every pair.
    let c = common::canon_intent(&annotate(&net(src("three_stage")), &AnnotationConfig::default()), &net(src("three_stage")));
    assert!(c.pairs.iter().all(|(a, b)| !["M6", "M7", "M8", "M9"].contains(&a.as_str()) && !["M6", "M7", "M8", "M9"].contains(&b.as_str())));
}

/// Every table `annotate` returns that prints, one string per circuit.
fn printed(name: &str, src: &str) -> String {
    let nl = net(src);
    let p = annotate(&nl, &cfg(name));
    let leaves: Vec<_> = annotator::block::leaves(&p.blocks).into_iter().map(|b| (b.kind, b.template, b.devices.clone())).collect();
    format!("{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{leaves:?}", p.intent, p.net_classes, p.groups, p.abutment, p.missing, p.coverage)
}

/// AGENTS.md determinism: one job and N jobs give the same bytes. Each
/// thread hashes with its own random seeds, so a table filled in hash order
/// prints differently across them.
/// Oracle: the run itself, repeated: four threads against this one, per circuit.
#[test]
fn annotate_prints_the_same_bytes_on_every_thread() {
    let one: Vec<String> = all().map(|(name, src)| printed(name, src)).collect();
    std::thread::scope(|s| {
        let jobs: Vec<_> = (0..4).map(|_| s.spawn(|| all().map(|(name, src)| printed(name, src)).collect::<Vec<String>>())).collect();
        for j in jobs {
            let got = j.join().expect("annotates");
            for ((name, _), (a, b)) in all().zip(one.iter().zip(&got)) {
                assert!(a == b, "{name} differs across threads:\n{a}\n---\n{b}");
            }
        }
    });
}
