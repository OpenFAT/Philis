//! The sidecar `Reference` (UW-ASIC/Philis#78) through the recipe spine:
//! the issue's gate as far as M0 reaches. `dac4` waits for M3 (it refuses
//! at parse on its capacitors), so `ota`, the one multi-tile MOS fixture,
//! stands in for it on sky130; asap7 takes one tile in M0 (fin rows need
//! M2's glue row), so `pair` covers it there. The engine names instances as
//! SCIR does, without SPICE's element letter: `M5` is the netlist's `XM5`.

use analog::base::{DeviceId, Rect};

const OTA: &str = include_str!("../../../../benchmarks/fixtures/ota.spice");
const PAIR: &str = include_str!("../../../../benchmarks/fixtures/pair.spice");
const DAC4: &str = include_str!("../../../../benchmarks/fixtures/dac4.spice");
const DAC4_REFERENCE: &str = include_str!("../../../../benchmarks/fixtures/dac4.reference.json");

/// `ota` as Philis 1.0 drew it (progress snapshot 8f84de4,
/// `docs/progress/layouts/ota.svg`, 41.60 × 52.31 µm): the tail across the
/// bottom, the ringed input pair in the middle, the load across the top;
/// each block's front-end box over the layout's.
const OTA_FROM_1_0: &str = r#"[{"constraint": "Reference", "source": "Philis 1.0 ota at 8f84de4, 41.60 x 52.31 um", "aspect": 0.795,
    "rows": [["M5"], ["M1"], ["M3"]],
    "extents": {"M5": {"x": 0.000, "y": 0.000, "w": 1.000, "h": 0.222},
                "M1": {"x": 0.280, "y": 0.249, "w": 0.441, "h": 0.443},
                "M3": {"x": 0.034, "y": 0.746, "w": 0.932, "h": 0.254}}}]"#;

fn pdk(deck: &str) -> verify::Pdk {
    verify::Pdk::builtin(deck).expect("a built-in deck")
}

/// Every hard finding but PEX (signoff-only, Q29), and LVS over every device.
fn findings(s: &verify::Signoff) -> Vec<String> {
    let mut out: Vec<String> = s.report.hard_violations.iter().map(|v| v.rule.clone()).filter(|r| !r.starts_with("engine/pex")).collect();
    out.extend(s.coverage.skipped_rules.iter().filter(|(r, _)| r == "lvs").map(|(r, why)| format!("{r} skipped: {why}")));
    out.extend(s.coverage.unverified.iter().map(|(k, m, n)| format!("unverified {n} {k:?} {m:?}")));
    out.sort();
    out
}

fn run(spice: &str, sidecar: Option<&str>, deck: &verify::Pdk) -> engine::Outcome {
    engine::run_with(spice, sidecar, deck, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{r}"))
}

/// The drawn tile holding the device named `name`.
fn tile_of(out: &engine::Outcome, netlist: &engine::Netlist, name: &str) -> Rect {
    let d = netlist.flat.devices.iter().position(|d| d.name == name).unwrap_or_else(|| panic!("no device {name}"));
    let t = (0..out.calls.len()).find(|&t| out.calls.members_of(t).contains(&DeviceId(d as u16))).expect("every device has a call");
    out.placed[t].bbox
}

/// The GDS a run writes, labels aside: every placed and routed shape.
fn gds(out: &engine::Outcome, deck: &verify::Pdk) -> Vec<u8> {
    verify::gds_out::gds(&engine::geometry(&out.placed, &out.routes), &[], deck, "top").expect("writes")
}

fn union(boxes: impl Iterator<Item = Rect>) -> Rect {
    boxes
        .reduce(|a, b| {
            let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
            Rect { x: x0, y: y0, w: (a.x + a.w).max(b.x + b.w) - x0, h: (a.y + a.h).max(b.y + b.h) - y0 }
        })
        .expect("a tile")
}

/// The gate's first check as M0 has it: `ota` under the reference its
/// Philis 1.0 layout gives keeps the rows (oracle: the drawn tiles, bottom
/// to top), the outline takes the aspect (within 1 %), signoff finds
/// nothing M0's own row does not (oracle: the same spine without the
/// reference), and the report quotes the reference and every block's
/// deviation. M0's tiles are not resized (M1's bounded pick), so the drawn
/// aspect is reported, not met: its tail tile is 14 µm wide and 42 µm tall,
/// 1.0's was a 42 µm row.
#[test]
fn ota_keeps_the_rows_of_its_philis_1_0_layout() {
    let sky = pdk("sky130");
    let netlist = engine::parse(OTA, &sky).expect("parses");
    let today = run(OTA, None, &sky);
    let out = run(OTA, Some(OTA_FROM_1_0), &sky);
    let (tail, pair, load) = (tile_of(&out, &netlist, "M5"), tile_of(&out, &netlist, "M1"), tile_of(&out, &netlist, "M3"));
    assert!(tail.y + tail.h < pair.y && pair.y + pair.h < load.y, "rows out of order: tail {tail:?}, pair {pair:?}, load {load:?}");
    let (_, fit) = out.reference.as_ref().expect("the reference floorplan ran");
    let aspect = f64::from(fit.outline.0) / f64::from(fit.outline.1);
    assert!((aspect / 0.795 - 1.0).abs() < 0.01, "outline aspect {aspect}");
    assert_eq!(findings(&out.signoff), findings(&today.signoff), "the reference floorplan adds or loses a finding\n{}", out.report);
    for part in ["reference: sidecar entry 0, \"Philis 1.0 ota at 8f84de4", "aspect: target 0.795, outline 0.79", ", drawn ", "M5 (row 0): target x 0.000 y 0.000 w 1.000 h 0.222; calls", "M1 (row 1)", "M3 (row 2)"] {
        assert!(out.report.contains(part), "{part:?} missing:\n{}", out.report);
    }
}

/// The gate's first check where M0's tiles match their extents: a reference
/// written from a layout brings that layout back. The prior design is M0's
/// own `ota` row; its reference names each tile's box over the row's. Under
/// it, and under its mirror image (the row reversed), the tiles keep the
/// reference's order and the drawn tiles' aspect is the reference's within
/// 5 % (oracle: the prior layout's measured boxes, not the floorplan's).
#[test]
fn a_reference_written_from_a_layout_brings_back_its_rows_and_aspect() {
    let sky = pdk("sky130");
    let netlist = engine::parse(OTA, &sky).expect("parses");
    let prior = run(OTA, None, &sky);
    let whole = union(prior.placed.iter().map(|m| m.bbox));
    let aspect = f64::from(whole.w) / f64::from(whole.h);
    let mut order: Vec<usize> = (0..prior.placed.len()).collect();
    order.sort_by_key(|&t| prior.placed[t].bbox.x);
    let name = |t: usize| netlist.flat.devices[usize::from(prior.calls.members_of(t)[0].0)].name.clone();
    for mirrored in [false, true] {
        let extent = |t: usize| {
            let b = prior.placed[t].bbox;
            let (x, w) = (f64::from(b.x - whole.x) / f64::from(whole.w), f64::from(b.w) / f64::from(whole.w));
            let x = if mirrored { (1.0 - x - w).max(0.0) } else { x };
            format!(r#""{}": {{"x": {x}, "y": {}, "w": {w}, "h": {}}}"#, name(t), f64::from(b.y - whole.y) / f64::from(whole.h), f64::from(b.h) / f64::from(whole.h))
        };
        let row: Vec<usize> = if mirrored { order.iter().rev().copied().collect() } else { order.clone() };
        let names: Vec<String> = row.iter().map(|&t| format!("{:?}", name(t))).collect();
        let extents: Vec<String> = row.iter().map(|&t| extent(t)).collect();
        let json = format!(r#"[{{"constraint": "Reference", "aspect": {aspect}, "rows": [[{}]], "extents": {{{}}}}}]"#, names.join(", "), extents.join(", "));
        let out = run(OTA, Some(&json), &sky);
        let drawn: Vec<Rect> = row.iter().map(|&t| tile_of(&out, &netlist, &name(t))).collect();
        assert!(drawn.windows(2).all(|p| p[0].x + p[0].w < p[1].x), "mirrored {mirrored}: not left to right as the row lists them: {drawn:?}");
        let got = union(out.placed.iter().map(|m| m.bbox));
        let got = f64::from(got.w) / f64::from(got.h);
        assert!((got / aspect - 1.0).abs() <= 0.05, "mirrored {mirrored}: aspect {got:.3} for {aspect:.3}\n{}", out.report);
    }
}

/// The gate's second check: the fixture without the entry is byte-identical
/// to today's output. "Today" is the spine with no sidecar, the only door M0
/// had; the sidecar with the `Reference` entry removed is `[]`. Negative
/// control: with the entry the GDS moves.
#[test]
fn without_the_entry_the_output_is_byte_identical() {
    let sky = pdk("sky130");
    for spice in [OTA, PAIR] {
        let (today, without) = (run(spice, None, &sky), run(spice, Some("[]"), &sky));
        assert_eq!(gds(&today, &sky), gds(&without, &sky));
        assert_eq!(today.report, without.report);
        assert!(without.reference.is_none());
    }
    assert_ne!(gds(&run(OTA, None, &sky), &sky), gds(&run(OTA, Some(OTA_FROM_1_0), &sky), &sky), "the reference moved nothing");
}

/// The gate's third check: a reference that puts the input pair's two
/// members (a common-centroid set on sky130, Moderate by role) in two rows
/// is refused, naming the set and the entry. Negative control: the pair in
/// one block plans.
#[test]
fn a_reference_splitting_the_input_pair_is_refused() {
    let sky = pdk("sky130");
    let split = r#"[{"constraint": "Reference", "aspect": 1, "rows": [["M1"], ["M2"]],
        "extents": {"M1": {"x": 0, "y": 0, "w": 1, "h": 0.5}, "M2": {"x": 0, "y": 0.5, "w": 1, "h": 0.5}}}]"#;
    let r = engine::run_with(OTA, Some(split), &sky, verify::SignoffEngine::Gpurify).err().expect("refused");
    assert_eq!(r.code, "floorplan/reference", "{r}");
    for part in ["common-centroid matched set", "[M1, M2]", "sidecar entry 0", "M1 (row 0) and M2 (row 1)"] {
        assert!(r.message.contains(part), "{part:?} missing: {r}");
    }
    let whole = r#"[{"constraint": "Reference", "aspect": 1, "rows": [["M1"]], "extents": {"M1": {"x": 0, "y": 0, "w": 1, "h": 1}}}]"#;
    assert!(engine::run_with(OTA, Some(whole), &sky, verify::SignoffEngine::Gpurify).is_ok());
}

/// Both proof PDKs: `pair` (one tile, so asap7 takes it in M0) under a
/// one-block reference signs off DRC, ERC and LVS clean, and the report
/// carries the reference section.
#[test]
fn pair_takes_a_reference_on_both_proof_pdks() {
    let one = r#"[{"constraint": "Reference", "source": "one block", "aspect": 1.5, "rows": [["M1"]], "extents": {"M1": {"x": 0, "y": 0, "w": 1, "h": 1}}}]"#;
    let asap7 = PAIR.replace("nfet_01v8", "nmos_rvt").replace("L=0.5u", "L=20n").replace("W=2u", "W=270n");
    for (deck, spice) in [("sky130", PAIR.to_string()), ("asap7", asap7)] {
        let out = run(&spice, Some(one), &pdk(deck));
        assert!(findings(&out.signoff).is_empty(), "{deck}: {:?}\n{}", findings(&out.signoff), out.report);
        assert!(out.report.contains("reference: sidecar entry 0, \"one block\""), "{deck}:\n{}", out.report);
    }
}

/// Where the gate's first check stops: `dac4` refuses at parse (M0 reads
/// MOS only; capacitor arrays and the array floorplan are M3), with its
/// reference exactly as without it, so the reference changes nothing
/// before the floorplan. When M3 parses `dac4`, this test fails and the
/// gate's first check takes its place.
#[test]
fn dac4_refuses_at_parse_until_m3() {
    let sky = pdk("sky130");
    let today = engine::run(DAC4, &sky, verify::SignoffEngine::Gpurify).err().expect("M0 refuses dac4");
    let with = engine::run_with(DAC4, Some(DAC4_REFERENCE), &sky, verify::SignoffEngine::Gpurify).err().expect("refused");
    assert_eq!(with, today);
    assert_eq!(today.code, "parse", "{today}");
    assert!(today.message.contains("C0") && today.message.contains("cap_generic_m1m2"), "{today}");
}

/// A sidecar that is not a JSON array refuses with code `sidecar`.
#[test]
fn a_sidecar_that_is_not_an_array_is_refused() {
    let r = engine::run_with(PAIR, Some("{}"), &pdk("sky130"), verify::SignoffEngine::Gpurify).err().expect("refused");
    assert_eq!(r.code, "sidecar", "{r}");
}
