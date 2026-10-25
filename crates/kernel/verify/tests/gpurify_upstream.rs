//! GPurify (pinned 687c8bc) behaviours that contradict GPurify's own text,
//! each worked around in `src/overrides.rs`. Every test asserts the current
//! result and names the upstream-fixed one, so it fails the day GPurify is
//! fixed: then drop the override and flip the assert. Issue drafts and
//! patches: `docs/ref/research/gpurify-issues.md`.

use gdsverify::check::lvs::CompareOptions;
use gdsverify::check::report::Measurement;
use gdsverify::deck::CompiledDeck;
use gdsverify::engine::pipeline::{extract, Loaded};
use gdsverify::engine::run::{run_checks, Checks, RunOptions};
use gdsverify::geom::{Dbu, GeometryStoreBuilder, Grid};
use gdsverify::ingest::deck::parse_deck;
use gdsverify::ingest::layout::derive_layers;
use gdsverify::ingest::{Provenance, StrTable};

/// `(rule id, measured)` of every violation GPurify reports for `rects`
/// (`layer name, [x0, y0, x1, y1]` in nm) under `deck`, DRC or ERC.
fn run(deck: &str, rects: &[(&str, [i64; 4])], drc: bool) -> Vec<(String, Measurement)> {
    let grid = Grid::new(1000).expect("1 nm grid");
    let mut strings = StrTable::default();
    let deck = parse_deck(deck, grid, &mut strings).unwrap_or_else(|e| panic!("{e}"));
    let mut b = GeometryStoreBuilder::default();
    for &(name, [x0, y0, x1, y1]) in rects {
        let layer = deck.layers.id(&strings, name).unwrap_or_else(|| panic!("no layer {name}"));
        let [x0, y0, x1, y1] = [x0, y0, x1, y1].map(Dbu::new_unchecked);
        b.push(layer, &[x0, x1, x1, x0], &[y0, y0, y1, y1]);
    }
    let (mut store, _) = b.finish(deck.layers.len());
    let provenance = Provenance::default();
    derive_layers(&mut store, &deck, &provenance, &strings);
    let deck = CompiledDeck::from_deck(deck, grid, &strings).expect("deck compiles");
    let loaded = Loaded { strings, grid, deck, store, provenance, reference: None, intent: None, design: None };
    let extracted = extract(&loaded).expect("extracts");
    let checks = Checks { drc, erc: !drc, lvs: false, pex: false };
    let options = RunOptions { checks, lvs: CompareOptions::default(), quasistatic_nets: Vec::new() };
    let (out, _) = run_checks(&loaded, &extracted, &options);
    let v = &out.violations;
    (0..v.len()).map(|i| (loaded.strings.resolve(v.rule[i]).to_owned(), v.measured[i])).collect()
}

fn of<'a>(found: &'a [(String, Measurement)], rule: &str) -> Vec<&'a Measurement> {
    found.iter().filter(|f| f.0 == rule).map(|f| &f.1).collect()
}

/// Issue 1. GPurify's shipped sky130 deck says of lvtn.10 (deck lines
/// 297-299, after the foundry PVL): "an lvtn off every such well is not
/// measured". The engine measures an inner no host covers as 0, so an LVT
/// nfet's lvtn (on p-substrate, off every n-well) fails lvtn.10.
///
/// Upstream fixed: no lvtn.10 finding. Philis selects the lvtn instead
/// (`overrides.rs`, `lvtn.interacting(nwell_novar)`).
#[test]
fn sky130_lvtn10_flags_lvtn_off_every_nwell() {
    let found = run(gdsverify::pdks::SKY130, &[("lvtn", [0, 0, 2000, 2000])], true);
    // Upstream fixed: `assert!(of(&found, "lvtn.10").is_empty())`.
    assert_eq!(of(&found, "lvtn.10"), [&Measurement::Length(Dbu::new_unchecked(0))]);
}

/// Issue 1, the engine: `enclosure(..)` (`min_enclosure`) and
/// `enclosure(.., noncoincident)` both measure an inner shape with no outer
/// shape anywhere near it as 0. GPurify documents this for both kinds
/// (`Rule::MinEnclosure`, `Rule::NoncoincidentEnclosure`), but both cite
/// KLayout `enclosing`, which does not report it: "Locations, where
/// other_layer extends outside layer will not be reported as errors"
/// (KLayout DRC reference, `layer.enclosing`).
///
/// KLayout-faithful: no finding from either rule. This test flips only if
/// upstream changes the engine, not if it fixes just the sky130 deck.
#[test]
fn enclosure_measures_unhosted_inner_as_zero() {
    let deck = "grid 1nm
layer nwell = gds(64, 20)
layer lvtn = gds(125, 44)
rule enc enclosure(lvtn, nwell) >= 380nm
rule enc_nc enclosure(lvtn, nwell, noncoincident) >= 380nm
";
    let rects = [("nwell", [0, 0, 5000, 5000]), ("lvtn", [10_000, 0, 11_000, 1000])];
    let found = run(deck, &rects, true);
    let zero = Measurement::Length(Dbu::new_unchecked(0));
    // KLayout-faithful: both empty.
    assert_eq!(of(&found, "enc"), [&zero]);
    assert_eq!(of(&found, "enc_nc"), [&zero]);
}

/// Issue 2. `SoftConnectionTable`'s docs: the hard connectivity has "no soft
/// conductor or global layer". `check_soft_connection` keeps every global
/// layer that is not listed soft, so a supply strap tied to the substrate
/// (`connect global psub`) through a soft p-tap reads as two pieces, the
/// strap and the substrate, and is flagged though nothing else holds it.
///
/// Upstream fixed: no finding. Philis lists psub as soft (`overrides.rs`).
#[test]
fn soft_connection_keeps_global_substrate() {
    let found = run(SOFT_DECK, &strap(0), false);
    // Upstream fixed: `assert!(of(&found, "soft").is_empty())`.
    assert_eq!(of(&found, "soft"), [&Measurement::Count(2)]);
}

/// Issue 2's control: two straps of one supply, joined only through their
/// soft p-taps and the substrate, are a real soft connection. Flagged now and
/// after the upstream fix alike, so the fix cannot simply silence the rule.
#[test]
fn soft_connection_flags_straps_joined_only_by_substrate() {
    let mut rects = strap(0).to_vec();
    rects.extend(strap(5000));
    let found = run(SOFT_DECK, &rects, false);
    assert_eq!(of(&found, "soft"), [&Measurement::Count(2)]);
}

/// A met1 strap on a p-tap at height `y`, the tap tied to the substrate.
fn strap(y: i64) -> [(&'static str, [i64; 4]); 5] {
    [
        ("psub", [0, 0, 10_000, 10_000]),
        ("ptap", [1000, y + 1000, 9000, y + 2000]),
        ("tie", [1000, y + 1000, 9000, y + 2000]),
        ("licon", [1200, y + 1200, 1400, y + 1400]),
        ("met1", [1000, y + 1000, 9000, y + 2000]),
    ]
}

/// A supply's front end: met1 to a soft p-tap through licon, the p-tap to the
/// global substrate through a tie, as sky130's `psub_tie`.
const SOFT_DECK: &str = "grid 1nm
layer psub = gds(64, 0)
layer ptap = gds(65, 44)
layer tie = gds(65, 45)
layer licon = gds(66, 44)
layer met1 = gds(68, 20)
connect conductors [ptap, met1]
connect global psub
connect via tie [ptap, psub]
connect via licon [ptap, met1]
rule soft soft_connection(ptap)
";
