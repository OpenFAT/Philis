//! M0's gate (ROADMAP): `pair` through the recipe spine is DRC 0, ERC 0 and
//! LVS MATCH, in process, on sky130 and asap7.

const PAIR: &str = include_str!("../../../../benchmarks/fixtures/pair.spice");

fn signs_off(deck: &str, spice: &str) {
    let pdk = verify::Pdk::builtin(deck).expect("a built-in deck");
    let out = engine::run(spice, &pdk).unwrap_or_else(|r| panic!("{deck}: {r}"));
    let hard: Vec<&str> = out.signoff.report.hard_violations.iter().map(|v| v.rule.as_str()).filter(|r| !r.starts_with("engine/pex")).collect();
    assert!(hard.is_empty(), "{deck}:\n{}", out.report);
}

#[test]
fn pair_signs_off_on_sky130() {
    signs_off("sky130", PAIR);
}

/// asap7 names its MOS models plainly (`nmos_rvt`), and has no `L=0.5u`
/// device: the same circuit at the deck's own gate length.
#[test]
fn pair_signs_off_on_asap7() {
    let spice = PAIR.replace("nfet_01v8", "nmos_rvt").replace("L=0.5u", "L=20n").replace("W=2u", "W=270n");
    signs_off("asap7", &spice);
}
