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

/// Negative control: a die too narrow for any variant of a call refuses,
/// naming the call and the widths, never drawing a wider tile.
#[test]
fn a_die_narrower_than_every_variant_refuses() {
    let io = OTA_CONSTRAINED_IO.replacen("30000", "3000", 1);
    let r = in_harness(OTA_CONSTRAINED, &io).err().expect("refuses");
    assert!(r.code == "interface" && r.message.contains("call ") && r.message.contains("narrowest variant"), "{r}");
}

/// Negative control: a die wide enough but shorter than the block refuses.
#[test]
fn a_die_shorter_than_the_block_refuses() {
    let io = OTA_CONSTRAINED_IO.replacen("70000", "20000", 1);
    let r = in_harness(OTA_CONSTRAINED, &io).err().expect("refuses");
    assert!(r.code == "interface" && r.message.contains("do not fit"), "{r}");
}

/// The width target is the harness's alone: without an interface the tail
/// (M5, 160 µm of channel) still draws as its one wide row, wider than the
/// 30 µm die; inside the harness it folds to fit.
#[test]
fn the_tail_folds_only_under_a_die() {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let tail = |out: &engine::Outcome| {
        let i = (0..out.calls.len()).find(|&i| out.calls.members_of(i).len() == 1).expect("the tail's own call");
        out.placed[i].bbox
    };
    let free = engine::run(OTA_CONSTRAINED, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{r}"));
    let held = in_harness(OTA_CONSTRAINED, OTA_CONSTRAINED_IO).unwrap_or_else(|r| panic!("{r}"));
    let (f, h) = (tail(&free), tail(&held));
    assert!(f.w > 30_000 && h.w < 30_000 && h.h > f.h, "free {f:?}, in the die {h:?}");
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
