//! M0's gate (ROADMAP): `pair` through the recipe spine is DRC 0, ERC 0 and
//! LVS MATCH, in process, on sky130 and asap7. PEX is not part of the gate:
//! asap7's front-end layers have no sourced resistance, so PEX refuses there
//! (ROADMAP, known gaps).

const PAIR: &str = include_str!("../../../../benchmarks/fixtures/pair.spice");

/// asap7 names its MOS models plainly (`nmos_rvt`) and draws 20 nm gates:
/// the same circuit at the deck's own sizes.
fn on_asap7(spice: &str) -> String {
    spice.replace("nfet_01v8", "nmos_rvt").replace("L=0.5u", "L=20n").replace("W=2u", "W=270n")
}

/// Every hard finding but PEX, and LVS ran over every device.
fn findings(s: &verify::Signoff) -> Vec<String> {
    let mut out: Vec<String> = s.report.hard_violations.iter().map(|v| v.rule.clone()).filter(|r| !r.starts_with("engine/pex")).collect();
    out.extend(s.coverage.skipped_rules.iter().filter(|(r, _)| r == "lvs").map(|(r, why)| format!("{r} skipped: {why}")));
    out.extend(s.coverage.unverified.iter().map(|(k, m, n)| format!("unverified {n} {k:?} {m:?}")));
    out
}

fn signs_off(deck: &str, spice: &str) {
    let pdk = verify::Pdk::builtin(deck).expect("a built-in deck");
    let out = engine::run(spice, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{deck}: {r}"));
    assert!(findings(&out.signoff).is_empty(), "{deck}: {:?}\n{}", findings(&out.signoff), out.report);
}

/// The negative control (Q39): the same layout against a schematic whose
/// devices are twice as wide fails LVS.
fn resized_fails(deck: &str, spice: &str, from: &str, to: &str) {
    let pdk = verify::Pdk::builtin(deck).expect("a built-in deck");
    let out = engine::run(spice, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{deck}: {r}"));
    let wider = engine::parse(&spice.replace(from, to), &pdk).expect("parses");
    let s = engine::signoff(&wider, &out.placed, &out.routes, &pdk, verify::SignoffEngine::Gpurify).expect("signs off");
    assert!(s.report.hard_violations.iter().any(|v| v.rule.starts_with("lvs/")), "{deck}: a resized schematic still matches");
}

#[test]
fn pair_signs_off_on_sky130() {
    signs_off("sky130", PAIR);
}

#[test]
fn pair_signs_off_on_asap7() {
    signs_off("asap7", &on_asap7(PAIR));
}

/// gf180mcu is nightly for conformance (TESTING.md), but `pair` is cheap and
/// in process, so it gates at merge like the proof PDKs.
#[test]
fn pair_signs_off_on_gf180mcu() {
    signs_off("gf180mcu", &PAIR.replace("nfet_01v8", "nfet_03v3"));
}

#[test]
fn a_resized_schematic_fails_lvs_on_sky130() {
    resized_fails("sky130", PAIR, "W=2u", "W=4u");
}

#[test]
fn a_resized_schematic_fails_lvs_on_asap7() {
    resized_fails("asap7", &on_asap7(PAIR), "W=270n", "W=540n");
}
