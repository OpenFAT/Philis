//! #77: simulation evidence through the recipe spine. Files in
//! (`engine::run_with`, `philis run`), rows out: in the report, with their
//! instance and source file, and in the Outcome's `evidence.json`. Names
//! resolve when the files are read (Q31), a name the netlist lacks is a
//! diagnostic and the run goes on, and without evidence nothing changes.
//! Oracle: `benchmarks/fixtures/pair.emir.json` and `pair.op.raw`, written by
//! hand from OmarSiwy/ESPice#237 and #236 (the formats ESPice will write),
//! and the `pair` run without them.

use serde_json::{json, Value};

const PAIR: &str = include_str!("../../../../benchmarks/fixtures/pair.spice");
const EMIR: &str = include_str!("../../../../benchmarks/fixtures/pair.emir.json");
const OP_RAW: &[u8] = include_bytes!("../../../../benchmarks/fixtures/pair.op.raw");
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../benchmarks/fixtures/");

/// The fixture with `xm3`, a device of another netlist, as its first entry.
fn with_a_foreign_instance() -> String {
    let xm3 = r#"{"instance": "xm3", "model": "sky130_fd_pr__nfet_01v8", "terminal": "d", "net": "d", "avg": 1.0e-05, "em_avg": 1.0e-05, "rms": 1.0e-05, "peak": 1.0e-05},"#;
    let out = EMIR.replacen("\"terminals\": [", &format!("\"terminals\": [\n    {xm3}"), 1);
    assert_ne!(out, EMIR, "the entry went in");
    out
}

/// Every hard finding but PEX (signoff-only, Q29).
fn findings(s: &verify::Signoff) -> Vec<String> {
    s.report.hard_violations.iter().map(|v| v.rule.clone()).filter(|r| !r.starts_with("engine/pex")).collect()
}

/// The report without its evidence section: the lines from `evidence:` to
/// the next unindented one.
fn without_evidence_section(report: &str) -> String {
    let mut inside = false;
    report
        .lines()
        .filter(|l| {
            inside = l.starts_with("evidence:") || (inside && l.starts_with("  "));
            !inside
        })
        .map(|l| format!("{l}\n"))
        .collect()
}

/// The gate's missing-instance case, end to end on sky130: `xm3` is one
/// diagnostic naming it and the run signs off as without evidence; every
/// row is in the report with its instance and source file and in
/// `evidence.json`; the report differs from the run without evidence only
/// by that section, and that run's JSON table is empty.
#[test]
fn evidence_reaches_the_report_and_the_json_table_and_the_run_goes_on() {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let emir = with_a_foreign_instance();
    let files = engine::EvidenceFiles { currents: Some(("pair.emir.json", &emir)), op: Some(("pair.op.raw", OP_RAW)) };
    let out = engine::run_with(PAIR, &files, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{r}"));
    assert!(findings(&out.signoff).is_empty(), "{:?}\n{}", findings(&out.signoff), out.report);
    for line in [
        "evidence: 8 terminal currents, 2 operating points",
        "  XM1 D: avg 50 µA, em_avg 50 µA, rms 50 µA, peak 50 µA (pair.emir.json)",
        "  XM2 S: avg -50 µA, em_avg 50 µA, rms 50 µA, peak 50 µA (pair.emir.json)",
        "  XM1: vds 900 mV, vgs 800 mV, id 50 µA, vdsat 180 mV (pair.op.raw)",
        "  XM2: vds 900 mV, vgs 800 mV, id 50 µA, vdsat 180 mV (pair.op.raw)",
        "  evidence_unknown_name: pair.emir.json: terminal 0: instance xm3 is not in the netlist",
    ] {
        assert!(out.report.lines().any(|l| l == line), "missing {line:?} in\n{}", out.report);
    }
    let table: Value = serde_json::from_str(&out.evidence_json).expect("evidence.json is JSON");
    assert_eq!(table["currents"].as_array().map(Vec::len), Some(8), "{table:#}");
    assert_eq!(
        table["currents"][0],
        json!({"instance": "XM1", "terminal": "D", "source": "pair.emir.json", "avg_ua": 50.0, "em_avg_ua": 50.0, "rms_ua": 50.0, "peak_ua": 50.0})
    );
    assert_eq!(
        table["op"][1],
        json!({"instance": "XM2", "source": "pair.op.raw", "vds_mv": 900.0, "vgs_mv": 800.0, "id_ua": 50.0, "vdsat_mv": 180.0})
    );

    let plain = engine::run(PAIR, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{r}"));
    assert!(!plain.report.contains("evidence"), "{}", plain.report);
    assert_eq!(serde_json::from_str::<Value>(&plain.evidence_json).expect("JSON"), json!({"currents": [], "op": []}));
    assert_eq!(without_evidence_section(&out.report), plain.report, "evidence changed more than its section");
}

/// Flat device names are the netlist's own, as evidence keys them (Q31): the
/// fork's SCIR drops each SPICE letter (`XM2` is instance `M2`) and `parse`
/// puts it back, so ESPice's `x1.mn1`, a MOS card inside subcircuit instance
/// `X1`, and `xm2`, an X card at the top, name `X1/MN1` and `XM2`.
#[test]
fn flat_names_are_the_netlists_own() {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let spice = ".subckt inv a y vss\nMN1 y a vss vss nfet_01v8 W=1u L=0.15u\n.ends inv\n\
                 .subckt top a y vss\nX1 a y vss inv\nXM2 y a vss vss nfet_01v8 W=1u L=0.15u\n.ends top\n";
    let netlist = engine::parse(spice, &pdk).unwrap_or_else(|r| panic!("{r}"));
    let names: Vec<&str> = netlist.flat.devices.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, ["X1/MN1", "XM2"]);
    let json = r#"{"format": "espice-emir/1", "terminals": [
        {"instance": "x1.mn1", "terminal": "d", "avg": 1e-6, "em_avg": 1e-6, "rms": 1e-6, "peak": 1e-6},
        {"instance": "xm2", "terminal": "d", "avg": 2e-6, "em_avg": 2e-6, "rms": 2e-6, "peak": 2e-6}]}"#;
    let ev = engine::evidence(&engine::EvidenceFiles { currents: Some(("top.emir.json", json)), op: None }, &netlist.flat).expect("reads");
    assert!(ev.diagnostics.is_empty(), "{:?}", ev.diagnostics);
    assert_eq!(ev.rows.currents.device, [analog::base::DeviceId(0), analog::base::DeviceId(1)]);
}

/// A file that is not what its field says is refused (`evidence`) before
/// anything is drawn: a raw file given as currents, a current file as the op.
#[test]
fn a_file_that_is_not_what_its_field_says_is_refused() {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let raw_text = std::str::from_utf8(OP_RAW).expect("an ASCII raw");
    for files in [
        engine::EvidenceFiles { currents: Some(("pair.op.raw", raw_text)), op: None },
        engine::EvidenceFiles { currents: None, op: Some(("pair.emir.json", EMIR.as_bytes())) },
    ] {
        let refusal = engine::run_with(PAIR, &files, &pdk, verify::SignoffEngine::Gpurify).err().expect("refused");
        assert_eq!(refusal.code, "evidence", "{refusal}");
    }
}

/// ESPice writes binary raws by default: the same plot as native-endian f64
/// samples reads exactly as the ASCII fixture. A raw with no operating
/// point, or two, is refused. Oracle: the ASCII reading (differential) and
/// the fixture's hand-written values.
#[test]
fn a_binary_raw_reads_as_its_ascii_twin() {
    let ascii = engine::op_vectors(OP_RAW).expect("the fixture reads");
    assert_eq!(ascii.len(), 13);
    assert_eq!(ascii[5], ("@m.xm1.msky130_fd_pr__nfet_01v8[id]".to_string(), 5e-5));
    let text = std::str::from_utf8(OP_RAW).expect("an ASCII raw");
    let (head, _) = text.split_once("Values:\n").expect("an ASCII raw has Values:");
    let mut binary = format!("{head}Binary:\n").into_bytes();
    for (_, v) in &ascii {
        binary.extend(v.to_ne_bytes());
    }
    assert_eq!(engine::op_vectors(&binary).expect("the binary twin reads"), ascii);
    let tran = text.replace("Plotname: Operating Point", "Plotname: Transient Analysis");
    assert!(engine::op_vectors(tran.as_bytes()).is_err(), "no operating point");
    assert!(engine::op_vectors(format!("{text}{text}").as_bytes()).is_err(), "two operating points");
}

/// `philis run` with both flags writes the report and the evidence table and
/// exits 0 on a clean signoff; a usage error exits 2.
#[test]
fn philis_run_writes_the_report_and_the_evidence_table() {
    let dir = std::env::temp_dir().join(format!("philis-run-evidence-{}", std::process::id()));
    let fixture = |f: &str| format!("{FIXTURES}{f}");
    let currents = fixture("pair.emir.json");
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_philis"))
        .args(["run", &fixture("pair.spice"), "sky130", "--evidence-currents", &currents, "--evidence-op", &fixture("pair.op.raw"), "-o"])
        .arg(&dir)
        .output()
        .expect("philis runs");
    assert_eq!(run.status.code(), Some(0), "{}\n{}", String::from_utf8_lossy(&run.stdout), String::from_utf8_lossy(&run.stderr));
    let report = std::fs::read_to_string(dir.join("report.txt")).expect("report.txt");
    assert_eq!(String::from_utf8_lossy(&run.stdout), report, "stdout is the report");
    assert!(report.contains("evidence: 8 terminal currents, 2 operating points"), "{report}");
    assert!(report.contains(&format!("  XM1 D: avg 50 µA, em_avg 50 µA, rms 50 µA, peak 50 µA ({currents})")), "{report}");
    let table: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("evidence.json")).expect("evidence.json")).expect("JSON");
    assert_eq!((table["currents"].as_array().map(Vec::len), table["op"].as_array().map(Vec::len)), (Some(8), Some(2)));
    std::fs::remove_dir_all(&dir).expect("the scratch directory goes");
    let usage = std::process::Command::new(env!("CARGO_BIN_EXE_philis")).args(["run", "only.spice"]).output().expect("philis runs");
    assert_eq!(usage.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&usage.stderr).starts_with("usage: philis run"), "{}", String::from_utf8_lossy(&usage.stderr));
}
