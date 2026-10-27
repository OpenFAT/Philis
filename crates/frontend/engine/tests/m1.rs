//! M1's gate (ROADMAP): `mirror_ratio` and `quad` through the recipe spine
//! are DRC 0, ERC 0 and LVS MATCH on sky130 and asap7, each set drawn by the
//! first rung of its ladder the deck can draw (`engine::ladder`).

const MIRROR_RATIO: &str = include_str!("../../../../benchmarks/fixtures/mirror_ratio.spice");
const QUAD: &str = include_str!("../../../../benchmarks/fixtures/quad.spice");

/// The circuit at asap7's names and sizes: `nmos_rvt`, 20 nm gates, widths
/// as fin multiples (270 nm = 10 fins per µm-class finger).
fn on_asap7(spice: &str) -> String {
    spice
        .replace("nfet_01v8", "nmos_rvt")
        .replace("L=1u", "L=20n")
        .replace("L=0.5u", "L=20n")
        .replace("W=2u", "W=270n")
        .replace("W=4u", "W=540n")
        .replace("W=8u", "W=1080n")
}

/// Every hard finding but PEX (signoff-only, Q29), and LVS over every device.
fn findings(s: &verify::Signoff) -> Vec<String> {
    let mut out: Vec<String> = s.report.hard_violations.iter().map(|v| v.rule.clone()).filter(|r| !r.starts_with("engine/pex")).collect();
    out.extend(s.coverage.skipped_rules.iter().filter(|(r, _)| r == "lvs").map(|(r, why)| format!("{r} skipped: {why}")));
    out.extend(s.coverage.unverified.iter().map(|(k, m, n)| format!("unverified {n} {k:?} {m:?}")));
    out
}

fn signs_off(deck: &str, spice: &str) -> engine::Outcome {
    let pdk = verify::Pdk::builtin(deck).expect("a built-in deck");
    let out = engine::run(spice, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{deck}: {r}"));
    println!("{deck}:\n{}", out.report);
    assert!(findings(&out.signoff).is_empty(), "{deck}: {:?}\n{}", findings(&out.signoff), out.report);
    out
}

#[test]
fn mirror_ratio_signs_off_on_sky130() {
    signs_off("sky130", MIRROR_RATIO);
}

#[test]
fn mirror_ratio_signs_off_on_asap7() {
    signs_off("asap7", &on_asap7(MIRROR_RATIO));
}

#[test]
fn quad_signs_off_on_sky130() {
    signs_off("sky130", QUAD);
}

#[test]
fn quad_signs_off_on_asap7() {
    signs_off("asap7", &on_asap7(QUAD));
}
