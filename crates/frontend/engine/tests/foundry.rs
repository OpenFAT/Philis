//! The differential oracle in process (TESTING.md): `pair` through the recipe
//! spine signs off DRC-clean and LVS-matched under every engine
//! ([`verify::SignoffEngine`]), and each foundry engine catches the two
//! negative controls (a resized schematic fails LVS, a narrowed metal fails
//! DRC). The foundry runs need klayout, magic and netgen on `PATH` (the
//! flake's dev shell, or `nix shell nixpkgs#klayout nixpkgs#magic-vlsi
//! nixpkgs#netgen-vlsi`), so they are nightly; the refusals need no tool.

use verify::SignoffEngine::{self, Gpurify, Klayout, Magic};

// GPurify's side of the differential on sky130 is `pair.rs`.

const PAIR: &str = include_str!("../../../../benchmarks/fixtures/pair.spice");

/// `pair` on `pdk`'s own thin-oxide nfet (sizes legal on all three).
fn pair(pdk: &str) -> String {
    match pdk {
        "gf180mcu" => PAIR.replace("nfet_01v8", "nfet_03v3"),
        "ihp_sg13g2" => PAIR.replace("nfet_01v8", "sg13_lv_nmos"),
        _ => PAIR.to_string(),
    }
}

/// Every hard row but GPurify's PEX refusal (not part of the gate, pair.rs).
fn findings(s: &verify::Signoff) -> Vec<String> {
    s.report.hard_violations.iter().map(|v| format!("{} ({})", v.rule, v.margin)).filter(|r| !r.starts_with("engine/pex")).collect()
}

fn signs_off(pdk: &str, engine: SignoffEngine) {
    let p = verify::Pdk::builtin(pdk).unwrap();
    let out = engine::run(&pair(pdk), &p, engine).unwrap_or_else(|r| panic!("{pdk} {engine:?}: {r}"));
    assert!(findings(&out.signoff).is_empty(), "{pdk} {engine:?}: {:?}", findings(&out.signoff));
    if engine != Gpurify {
        // What the foundry engine did not check is listed, never a pass.
        let skipped: Vec<&str> = out.signoff.coverage.skipped_rules.iter().map(|(r, _)| r.as_str()).collect();
        assert_eq!(skipped, ["erc", "pex", "em_ir"], "{pdk} {engine:?}");
    }
}

/// Rows of one check (`drc/`, `lvs/`), each with its margin, logs path cut.
fn rows(s: &verify::Signoff, check: &str) -> Vec<(String, i64)> {
    s.report.hard_violations.iter().filter(|v| v.rule.starts_with(check)).map(|v| (v.rule.split(" (logs in").next().unwrap_or_default().to_string(), v.margin)).collect()
}

/// The layout of `pair` against a schematic whose devices are twice as wide.
/// The unmodified run must match under the same engine first, so the
/// control cannot pass on a layout that already mismatched.
fn resized_fails_lvs(pdk: &str, engine: SignoffEngine) {
    let p = verify::Pdk::builtin(pdk).unwrap();
    let out = engine::run(&pair(pdk), &p, engine).unwrap_or_else(|r| panic!("{pdk} {engine:?}: {r}"));
    assert!(rows(&out.signoff, "lvs/").is_empty(), "{pdk} {engine:?}: the unmodified pair already mismatches: {:?}", findings(&out.signoff));
    let wider = engine::parse(&pair(pdk).replace("W=2u", "W=4u"), &p).unwrap();
    let s = engine::signoff(&wider, &out.placed, &out.routes, &p, engine).unwrap_or_else(|r| panic!("{pdk} {engine:?}: {r}"));
    assert!(!rows(&s, "lvs/").is_empty(), "{pdk} {engine:?}: a resized schematic still matches: {:?}", findings(&s));
}

/// The layout of `pair` with one routing-metal shape narrowed to half its
/// layer's minimum width: some DRC rule fires more often than on the
/// unmodified layout under the same engine (which may itself be dirty).
fn narrowed_fails_drc(pdk: &str, engine: SignoffEngine) {
    let p = verify::Pdk::builtin(pdk).unwrap();
    let netlist = engine::parse(&pair(pdk), &p).unwrap();
    let mut out = engine::run(&pair(pdk), &p, engine).unwrap_or_else(|r| panic!("{pdk} {engine:?}: {r}"));
    let before = rows(&out.signoff, "drc/");
    let metals = p.routing_layers();
    let shape = out.placed.iter_mut().flat_map(|m| m.shapes.iter_mut()).chain(out.routes.wires.iter_mut().flatten()).find(|s| metals.contains(&s.layer)).expect("pair draws routing metal");
    let half = p.min_width(shape.layer.0).expect("a metal has a minimum width") / 2;
    if shape.rect.w < shape.rect.h { shape.rect.w = half } else { shape.rect.h = half }
    let s = engine::signoff(&netlist, &out.placed, &out.routes, &p, engine).unwrap_or_else(|r| panic!("{pdk} {engine:?}: {r}"));
    let after = rows(&s, "drc/");
    let grew = after.iter().any(|(r, n)| before.iter().find(|(b, _)| b == r).is_none_or(|(_, m)| n > m));
    assert!(grew, "{pdk} {engine:?}: a narrowed metal adds no DRC finding: before {before:?}, after {after:?}");
}

// sky130, the proof PDK: clean and matched under both foundry engines, and
// both negative controls caught.
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn pair_signs_off_on_sky130_under_klayout() { signs_off("sky130", Klayout) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn pair_signs_off_on_sky130_under_magic() { signs_off("sky130", Magic) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_resized_schematic_fails_lvs_on_sky130_under_klayout() { resized_fails_lvs("sky130", Klayout) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_resized_schematic_fails_lvs_on_sky130_under_magic() { resized_fails_lvs("sky130", Magic) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_narrowed_metal_fails_drc_on_sky130_under_klayout() { narrowed_fails_drc("sky130", Klayout) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_narrowed_metal_fails_drc_on_sky130_under_magic() { narrowed_fails_drc("sky130", Magic) }

// gf180mcu and ihp_sg13g2: `pair` is not clean there under any engine yet
// (TESTING.md, "Foundry engines"), so only the controls that hold run: each
// engine still sees a narrowed metal, and LVS still sees a resized
// schematic wherever the unmodified pair matches.
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_resized_schematic_fails_lvs_on_gf180mcu_under_klayout() { resized_fails_lvs("gf180mcu", Klayout) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_resized_schematic_fails_lvs_on_gf180mcu_under_magic() { resized_fails_lvs("gf180mcu", Magic) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_narrowed_metal_fails_drc_on_gf180mcu_under_klayout() { narrowed_fails_drc("gf180mcu", Klayout) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_narrowed_metal_fails_drc_on_gf180mcu_under_magic() { narrowed_fails_drc("gf180mcu", Magic) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_resized_schematic_fails_lvs_on_ihp_sg13g2_under_magic() { resized_fails_lvs("ihp_sg13g2", Magic) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_narrowed_metal_fails_drc_on_ihp_sg13g2_under_klayout() { narrowed_fails_drc("ihp_sg13g2", Klayout) }
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_narrowed_metal_fails_drc_on_ihp_sg13g2_under_magic() { narrowed_fails_drc("ihp_sg13g2", Magic) }

/// asap7 has no open LVS deck and no magic tech: both foundry engines
/// refuse it by code, before any tool runs, and never fall back to GPurify.
#[test]
fn asap7_is_refused_by_the_foundry_engines() {
    let p = verify::Pdk::builtin("asap7").unwrap();
    let spice = PAIR.replace("nfet_01v8", "nmos_rvt").replace("L=0.5u", "L=20n").replace("W=2u", "W=270n");
    for engine in [Klayout, Magic] {
        let r = engine::run(&spice, &p, engine).err().unwrap_or_else(|| panic!("{engine:?} signed asap7 off"));
        assert_eq!(r.code, "foundry/no-deck", "{engine:?}: {r}");
    }
}

/// A Pdk loaded from text names no built-in, so no vendored deck: refused.
#[test]
fn an_unnamed_pdk_is_refused_by_the_foundry_engines() {
    let sky = verify::Pdk::builtin("sky130").unwrap();
    let p = verify::Pdk::load(&sky.source, &sky.cell).unwrap();
    for engine in [Klayout, Magic] {
        assert_eq!(engine::run(PAIR, &p, engine).err().map(|r| r.code), Some("foundry/no-pdk"), "{engine:?}");
    }
}

/// asap7's one foundry door: its open KLayout DRC deck (OpenROAD-flow-scripts)
/// through the DRC-only tier. `pair` is not clean there yet (ACTIVE.S.2A,
/// TESTING.md "Foundry engines"), so the control is relative: a narrowed
/// metal adds a finding.
#[test]
#[ignore = "needs klayout/magic/netgen: nightly"]
fn a_narrowed_metal_fails_drc_on_asap7_under_klayout() {
    let p = verify::Pdk::builtin("asap7").unwrap();
    let spice = PAIR.replace("nfet_01v8", "nmos_rvt").replace("L=0.5u", "L=20n").replace("W=2u", "W=270n");
    let mut out = engine::run(&spice, &p, Gpurify).unwrap();
    // DRC reads no labels.
    let before = verify::foundry::drc(Klayout, &engine::geometry(&out.placed, &out.routes), &[], &p).unwrap();
    let metals = p.routing_layers();
    let shape = out.placed.iter_mut().flat_map(|m| m.shapes.iter_mut()).find(|s| metals.contains(&s.layer)).unwrap();
    let half = p.min_width(shape.layer.0).unwrap() / 2;
    if shape.rect.w < shape.rect.h { shape.rect.w = half } else { shape.rect.h = half }
    let after = verify::foundry::drc(Klayout, &engine::geometry(&out.placed, &out.routes), &[], &p).unwrap();
    let grew = after.iter().any(|(r, n)| before.iter().find(|(b, _)| b == r).is_none_or(|(_, m)| n > m));
    assert!(grew, "asap7 KLayout DRC: a narrowed metal adds nothing: before {before:?}, after {after:?}");
}
