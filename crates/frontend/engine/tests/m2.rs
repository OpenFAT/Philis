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

const TT_OTA: &str = include_str!("../../../../benchmarks/fixtures/tt_ota.spice");
const OTA_CONSTRAINED: &str = include_str!("../../../../benchmarks/fixtures/ota_constrained.spice");
const TT_OTA_IO: &str = include_str!("../../../../benchmarks/fixtures/tt_ota.interface.json");
const OTA_CONSTRAINED_IO: &str = include_str!("../../../../benchmarks/fixtures/ota_constrained.interface.json");

fn in_harness(spice: &str, io: &str) -> Result<engine::Outcome, engine::Refusal> {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let io = engine::interface::Interface::from_json(io).unwrap_or_else(|r| panic!("{r}"));
    engine::run_in(spice, Some(&io), &pdk, verify::SignoffEngine::Gpurify)
}

fn signs_off_in_harness(spice: &str, io: &str) {
    let out = in_harness(spice, io).unwrap_or_else(|r| panic!("{r}"));
    println!("{}", out.report);
    assert!(findings(&out.signoff).is_empty(), "{:?}\n{}", findings(&out.signoff), out.report);
}

/// The 5T OTA in a 30 × 70 µm die, six met3 pins on all four edges.
#[test]
fn ota_constrained_signs_off_on_sky130() {
    signs_off_in_harness(OTA_CONSTRAINED, OTA_CONSTRAINED_IO);
}

/// The 5T OTA in a TinyTapeout tile: power on met5, signals on met4.
#[test]
fn tt_ota_signs_off_on_sky130() {
    signs_off_in_harness(TT_OTA, TT_OTA_IO);
}

/// Negative control: a pin on an internal net (`vtail`) is a coded refusal.
#[test]
fn a_pin_on_a_non_port_net_refuses() {
    let io = OTA_CONSTRAINED_IO.replacen("\"vinp\"", "\"vtail\"", 1);
    let r = in_harness(OTA_CONSTRAINED, &io).err().expect("refuses");
    assert!(r.code == "interface" && r.message.contains("vtail"), "{r}");
}

/// Negative control: a die smaller than the block is a coded refusal.
#[test]
fn a_die_smaller_than_the_block_refuses() {
    let io = OTA_CONSTRAINED_IO.replacen("30000", "3000", 1);
    let r = in_harness(OTA_CONSTRAINED, &io).err().expect("refuses");
    assert!(r.code == "interface" && r.message.contains("do not fit"), "{r}");
}
