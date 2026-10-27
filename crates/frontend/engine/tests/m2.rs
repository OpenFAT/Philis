//! M2's gate (ROADMAP): `ota`, `tt_ota`, `ota_constrained` through the recipe
//! spine (symmetric row stack, glue row, constrained routing first) are DRC
//! 0, ERC 0 and LVS MATCH on both proof PDKs. Release-speed fixtures: run
//! with `cargo test --release`.

const OTA: &str = include_str!("../../../../benchmarks/fixtures/ota.spice");

/// Every hard finding but PEX (signoff-only, Q29), and LVS over every device.
fn findings(s: &verify::Signoff) -> Vec<String> {
    let mut out: Vec<String> = s.report.hard_violations.iter().map(|v| v.rule.clone()).filter(|r| !r.starts_with("engine/pex")).collect();
    out.extend(s.coverage.skipped_rules.iter().filter(|(r, _)| r == "lvs").map(|(r, why)| format!("{r} skipped: {why}")));
    out.extend(s.coverage.unverified.iter().map(|(k, m, n)| format!("unverified {n} {k:?} {m:?}")));
    out
}

/// The 5T OTA: a Cc1d input pair and load on one axis, the tail below, every
/// finger folded within the deck's tap reach (LU.2/LU.3).
#[test]
fn ota_signs_off_on_sky130() {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let out = engine::run(OTA, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{r}"));
    assert!(findings(&out.signoff).is_empty(), "{:?}\n{}", findings(&out.signoff), out.report);
}

/// The same OTA at asap7's names and sizes: 20 nm gates, widths a multiple of
/// the 54 nm fin pitch. Routed M2 meets V1.M2.AUX.2 and the M2 tip tables.
#[test]
fn ota_signs_off_on_asap7() {
    let spice = OTA
        .replace("nfet_01v8", "nmos_rvt")
        .replace("pfet_01v8", "pmos_rvt")
        .replace("L=1u", "L=20n")
        .replace("L=2u", "L=20n")
        .replace("W=10u", "W=540n")
        .replace("W=20u", "W=1080n")
        .replace("W=40u", "W=2160n");
    let pdk = verify::Pdk::builtin("asap7").expect("a built-in deck");
    let out = engine::run(&spice, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{r}"));
    assert!(findings(&out.signoff).is_empty(), "{:?}\n{}", findings(&out.signoff), out.report);
}
