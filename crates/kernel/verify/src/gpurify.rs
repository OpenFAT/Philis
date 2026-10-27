//! GPurify 2.0 as a subprocess (Q61): drawn shapes, pin labels, a reference
//! and design intent → one `gpurify <stage> … --format json` per stage in a
//! scratch directory → violation rows, rule runs and stage verdicts; PEX's
//! `--format spice` → the capacitance matrix between labelled nets, and the
//! extracted circuit for [`netlist`].
//!
//! The JSON schema is GPurify's `src/gpurify/export/json.zig` at the rev
//! `flake.lock` pins. A stage that printed nothing failed before its report
//! (GPurify renders the whole report before the first byte). A report with a
//! `summary` states each stage's verdict, and a requested stage it does not
//! state as `ran` is denied. A report without one (GPurify before the
//! summary, Q61) that failed with no error row and every rule run is read
//! again as text for its `summary:` line.
//!
//! Contents:
//! - `pub struct Checks` (`Checks::ALL`), `pub enum Measurement`, `pub struct Row`, `pub struct RuleRun`, `pub struct Output`, `pub struct Input`
//! - `pub fn run(input: &Input, pdk: &Pdk, checks: Checks) -> Result<Output, String>`
//! - `pub fn netlist(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk) -> Result<String, String>`
//! - `pub fn dump(text: &str) -> Result<String, String>`
//! - `pub fn caps_of_spice(text: &str) -> (Vec<(String, Option<String>, f64)>, f64)`

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use analog::base::{NamedCell, Shape};
use serde_json::Value;

use crate::geom::LabeledPin;
use crate::pdk::Pdk;
use crate::Intent;

/// Which stages to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checks {
    pub drc: bool,
    pub erc: bool,
    pub lvs: bool,
    pub pex: bool,
}

/// One measured value or limit, in the JSON report's units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Measurement {
    /// nm.
    Length(f64),
    /// nm².
    Area(f64),
    Ratio(f64),
    Count(f64),
    /// mV.
    Voltage(f64),
    /// µA.
    Current(f64),
    /// Ω.
    Resistance(f64),
}

/// One violation row of a stage.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// `drc`, `erc`, `lvs` or `pex`: the stage that reported it.
    pub domain: &'static str,
    pub rule: String,
    /// The row's layer name; `None` for a row with no layer.
    pub layer: Option<String>,
    /// The deck states the row as a warning.
    pub warning: bool,
    /// A point on the offending shape, nm.
    pub x: f64,
    pub y: f64,
    pub measured: Measurement,
    pub limit: Measurement,
}

/// One rule record of a stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleRun {
    pub domain: &'static str,
    pub rule: String,
    /// `ran`, `refused`, or `skipped:<why>` as the report spells it.
    pub outcome: String,
    pub examined: u64,
}

/// What one [`run`] concluded.
#[derive(Clone, Debug, Default)]
pub struct Output {
    pub rows: Vec<Row>,
    pub runs: Vec<RuleRun>,
    /// `(stage, why)` for a requested stage that did not run to a report:
    /// skipped, refused, or GPurify's error.
    pub denied: Vec<(&'static str, String)>,
    /// PEX's capacitance over labelled nets, fF, as [`caps_of_spice`]: the
    /// field-solved run's when `field_solve` named a net and it ran.
    pub caps: Vec<(String, Option<String>, f64)>,
    /// Every C card of that netlist, labelled or not, fF.
    pub total_cap_ff: f64,
    /// Why GPurify wrote no PEX netlist: the run's violations still stand,
    /// only the caps are missing.
    pub caps_refused: Option<String>,
    /// Labels dropped from the PEX netlist run because they shorted onto a
    /// net another label already named.
    pub dropped_labels: Vec<String>,
    /// Wall time of the field-solved PEX netlist run; zero without one.
    pub pex_field: Duration,
    /// Why the field-solved run wrote no netlist (the analytical caps stand).
    pub field_refused: Option<String>,
}

/// One run's inputs.
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    pub shapes: &'a [Shape],
    pub pins: &'a [LabeledPin],
    /// Shapes drawn inside named cells (the deck's `in_cells` selections).
    pub cells: &'a [NamedCell],
    /// The LVS reference as SPICE ([`crate::reference::spice`]); `None`
    /// runs no LVS.
    pub reference: Option<&'a str>,
    /// The nets that leave the block ([`crate::RefInput::external_ports`]),
    /// for the intent file's `ports`.
    pub ports: Option<&'a [String]>,
    pub intent: &'a Intent,
    /// Nets PEX field-solves (`--quasistatic`) in a second netlist run.
    pub field_solve: &'a [String],
}

/// Whether the intent file carries the reference's external ports. GPurify
/// at the pinned rev refuses the key (Q61): on when it reads `ports`.
const INTENT_PORTS: bool = false;

/// A scratch directory removed on drop (kept under `PHILIS_KEEP_SCRATCH`).
struct Scratch(PathBuf);

/// Run `checks` over `input`. Each requested stage is one process; LVS runs
/// only with a reference.
///
/// # Errors
/// The GDS or deck cannot be written, or `gpurify` is not on `PATH`:
/// nothing ran.
pub fn run(input: &Input, pdk: &Pdk, checks: Checks) -> Result<Output, String> {
    let (gpurify, dir) = setup()?;
    let gds = dir.0.join(format!("{}.gds", crate::foundry::TOP));
    write(&gds, &crate::gds_out::gds(input.shapes, input.pins, input.cells, pdk, crate::foundry::TOP)?)?;
    let mut args = deck_args(&dir, pdk)?;
    if let Some(json) = intent_json(input.intent, input.ports) {
        let path = dir.0.join("intent.json");
        write(&path, json.as_bytes())?;
        args.extend(["--intent".into(), path.display().to_string()]);
    }
    let mut lvs_args = args.clone();
    if let Some(text) = input.reference {
        let path = dir.0.join("reference.spice");
        write(&path, text.as_bytes())?;
        lvs_args.extend(["--reference".into(), path.display().to_string()]);
    }

    let mut out = Output::default();
    let stages = [
        ("drc", checks.drc, &args),
        ("erc", checks.erc, &args),
        ("lvs", checks.lvs && input.reference.is_some(), &lvs_args),
        ("pex", checks.pex, &args),
    ];
    for (stage, on, args) in stages {
        if on {
            stage_into(&gpurify, &gds, stage, args, &mut out);
        }
    }
    if checks.pex {
        let mut pins = input.pins.to_vec();
        match spice(&gpurify, input, &mut pins, &[], pdk, &dir, &args, &mut out.dropped_labels)? {
            Ok(text) => (out.caps, out.total_cap_ff) = caps_of_spice(&text),
            Err(why) => out.caps_refused = Some(why),
        }
        // The labels the analytical run dropped name no net now.
        let field: Vec<String> = input.field_solve.iter().filter(|n| !out.dropped_labels.contains(n)).cloned().collect();
        if out.caps_refused.is_none() && !field.is_empty() {
            let t = Instant::now();
            let mut dropped = Vec::new();
            let solved = spice(&gpurify, input, &mut pins, &field, pdk, &dir, &args, &mut dropped)?;
            out.pex_field = t.elapsed();
            match solved {
                Ok(text) => (out.caps, out.total_cap_ff) = caps_of_spice(&text),
                Err(why) => out.field_refused = Some(why),
            }
        }
    }
    Ok(out)
}

/// The circuit GPurify extracts from `shapes`, as its `pex --format spice`
/// netlist without a reference: a `.subckt` named for the layout, one pin
/// per labelled net, one card per device (`M3`, `R0`, ...), then the
/// parasitic R and C cards on nodes `net#k`.
///
/// # Errors
/// `gpurify` missing, the GDS unwritable, or GPurify refusing the netlist
/// (a label short, a deck whose PEX it refuses: asap7's middle of line).
pub fn netlist(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk) -> Result<String, String> {
    let (gpurify, dir) = setup()?;
    let args = deck_args(&dir, pdk)?;
    let input = Input { shapes, pins, cells: &[], reference: None, ports: None, intent: &Intent::default(), field_solve: &[] };
    spice_once(&gpurify, &input, pins, &[], pdk, &dir, &args)?
}

/// GPurify's dump of deck `text` (`gpurify deck <deck> --format json`,
/// [`crate::deck`]).
///
/// # Errors
/// `gpurify` missing, or GPurify refusing the deck (its compile error).
pub fn dump(text: &str) -> Result<String, String> {
    let (gpurify, dir) = setup()?;
    let deck = dir.0.join("philis.deck");
    write(&deck, text.as_bytes())?;
    let (ok, stdout, stderr) = spawn(&gpurify, &["deck".into(), deck.display().to_string(), "--format".into(), "json".into()])?;
    if ok {
        Ok(stdout)
    } else {
        Err(format!("deck rejected: {stderr}"))
    }
}

/// Ground and coupling capacitance between labelled nets in a `pex --format
/// spice` netlist, fF: `(net, None, C)` to ground, `(a, Some(b), C)` with
/// `a < b`, summed per pair; and the sum of every C card. A node is its
/// net's name or `net#k`; an unlabelled net (`n<id>`, not a `.subckt` pin)
/// is dropped from the pairs, as is a coupling inside one net.
#[must_use]
pub fn caps_of_spice(text: &str) -> (Vec<(String, Option<String>, f64)>, f64) {
    let pins: Vec<&str> = text.lines().find_map(|l| l.strip_prefix(".subckt ")).map(|l| l.split_whitespace().skip(1).collect()).unwrap_or_default();
    let net = |node: &str| -> Option<String> {
        let n = node.split('#').next()?;
        pins.contains(&n).then(|| n.to_string())
    };
    let mut rows: std::collections::BTreeMap<(String, Option<String>), f64> = std::collections::BTreeMap::new();
    let mut total = 0.0;
    for line in text.lines().filter(|l| l.starts_with('C')) {
        let f: Vec<&str> = line.split_whitespace().collect();
        let [_, a, b, value] = f[..] else { continue };
        let Some(ff) = value.strip_suffix('f').and_then(|v| v.parse::<f64>().ok()) else { continue };
        total += ff;
        let key = match (net(a), b) {
            (Some(a), "0") => (a, None),
            (Some(x), _) => match net(b) {
                Some(y) if x != y => {
                    let (x, y) = if x < y { (x, y) } else { (y, x) };
                    (x, Some(y))
                }
                _ => continue,
            },
            (None, _) => continue,
        };
        *rows.entry(key).or_default() += ff;
    }
    (rows.into_iter().map(|((a, b), c)| (a, b, c)).collect(), total)
}

impl Checks {
    /// Every stage.
    pub const ALL: Self = Self { drc: true, erc: true, lvs: true, pex: true };
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // A debugging door: `PHILIS_KEEP_SCRATCH=1` leaves the GDS, deck,
        // reference and intent of every run to re-run `gpurify` by hand.
        if std::env::var_os("PHILIS_KEEP_SCRATCH").is_some() {
            eprintln!("gpurify scratch kept: {}", self.0.display());
            return;
        }
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- private ----

impl Scratch {
    fn new() -> Result<Self, String> {
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("philis-gpurify-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(Self(dir))
    }
}

/// The `gpurify` binary and a fresh scratch directory.
fn setup() -> Result<(PathBuf, Scratch), String> {
    let gpurify = crate::foundry::tool("gpurify").map_err(|r| r.to_string())?;
    Ok((gpurify, Scratch::new()?))
}

/// `--deck <file>`, the `Pdk`'s deck text written into `dir`.
fn deck_args(dir: &Scratch, pdk: &Pdk) -> Result<Vec<String>, String> {
    let deck = dir.0.join("philis.deck");
    write(&deck, pdk.source.as_bytes())?;
    Ok(vec!["--deck".into(), deck.display().to_string()])
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// `(exit 0, stdout, stderr)` of one `gpurify` process; a failed process's
/// stderr ends with its exit status (a crash prints nothing else).
fn spawn(gpurify: &Path, argv: &[String]) -> Result<(bool, String, String), String> {
    let o = Command::new(gpurify).args(argv).output().map_err(|e| format!("gpurify {}: {e}", argv.join(" ")))?;
    let mut stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
    if !o.status.success() {
        stderr = format!("{stderr} (gpurify {}: {})", argv[0], o.status).trim_start().to_string();
    }
    Ok((o.status.success(), String::from_utf8_lossy(&o.stdout).into_owned(), stderr))
}

/// One stage's rows and runs into `out`, or its denial.
fn stage_into(gpurify: &Path, gds: &Path, stage: &'static str, args: &[String], out: &mut Output) {
    let mut argv = vec![stage.to_string(), gds.display().to_string()];
    argv.extend_from_slice(args);
    argv.extend(["--format".into(), "json".into()]);
    let (passed, stdout, stderr) = match spawn(gpurify, &argv) {
        Ok(r) => r,
        Err(why) => return out.denied.push((stage, why)),
    };
    if stdout.trim().is_empty() {
        return out.denied.push((stage, stderr.trim().to_string()));
    }
    let report: Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(e) => return out.denied.push((stage, format!("unreadable JSON report: {e}"))),
    };
    let rows_before = out.rows.len();
    for v in report["violations"].as_array().into_iter().flatten() {
        match row(stage, v) {
            Some(r) => out.rows.push(r),
            None => return out.denied.push((stage, format!("unreadable violation row {v}"))),
        }
    }
    let mut all_ran = true;
    for r in report["runs"].as_array().into_iter().flatten() {
        let outcome = r["outcome"].as_str().unwrap_or("").to_string();
        all_ran &= outcome == "ran";
        out.runs.push(RuleRun {
            domain: stage,
            rule: r["rule"].as_str().unwrap_or("").to_string(),
            outcome,
            examined: r["examined"].as_u64().unwrap_or(0),
        });
    }
    if let Some(summary) = report.get("summary") {
        let status = &summary[stage];
        match status["status"].as_str() {
            Some("ran") => {}
            Some(s @ ("skipped" | "refused" | "not_selected")) => out.denied.push((stage, format!("{s}: {}", status["why"].as_str().unwrap_or("")))),
            _ => out.denied.push((stage, format!("no verdict for {stage} in the report's summary: {status}"))),
        }
        return;
    }
    // ponytail: no JSON summary at the pinned rev; a failed run with
    // nothing in the JSON to show for it is read again as text.
    let errors = out.rows[rows_before..].iter().any(|r| !r.warning);
    if !passed && !errors && all_ran {
        let status = spawn(gpurify, &argv[..argv.len() - 2]).ok().and_then(|(_, text, _)| summary_of(&text, stage));
        match status {
            Some(why) => out.denied.push((stage, why)),
            None => out.denied.push((stage, format!("failed with no row to show for it: {}", stderr.trim()))),
        }
    }
}

/// The `  <stage>: <status>` line of a text report's `summary:` block, when
/// it is not `ran`.
fn summary_of(text: &str, stage: &str) -> Option<String> {
    let tail = text.split("\nsummary:\n").nth(1)?;
    let status = tail.lines().find_map(|l| l.trim().strip_prefix(stage)?.strip_prefix(": "))?;
    (status != "ran").then(|| status.to_string())
}

fn row(domain: &'static str, v: &Value) -> Option<Row> {
    Some(Row {
        domain,
        rule: v["rule"].as_str()?.to_string(),
        // ponytail: `#<id>` until GPurify writes `layer_name` (Q61); 65535 is
        // GPurify's no-layer id.
        layer: v["layer_name"].as_str().map(str::to_string).or_else(|| v["layer"].as_u64().filter(|&l| l != 65_535).map(|l| format!("#{l}"))),
        warning: v["severity"].as_str()? != "error",
        x: v["at"]["x"].as_f64()?,
        y: v["at"]["y"].as_f64()?,
        measured: measurement(&v["measured"])?,
        limit: measurement(&v["limit"])?,
    })
}

fn measurement(v: &Value) -> Option<Measurement> {
    let x = v["value"].as_f64()?;
    Some(match v["kind"].as_str()? {
        "length" => Measurement::Length(x),
        "area" => Measurement::Area(x),
        "ratio" => Measurement::Ratio(x),
        "count" => Measurement::Count(x),
        "voltage" => Measurement::Voltage(x),
        "current" => Measurement::Current(x),
        "resistance" => Measurement::Resistance(x),
        _ => return None,
    })
}

/// PEX's netlist over `pins`, `field` nets field-solved. A label short
/// refuses the netlist naming the first label of the shorted net: that
/// label leaves `pins` for `dropped` and the netlist is written again, so
/// the net keeps one name (bounded by the pins). The inner `Err` is
/// GPurify's refusal.
#[allow(clippy::too_many_arguments)]
fn spice(
    gpurify: &Path,
    input: &Input,
    pins: &mut Vec<LabeledPin>,
    field: &[String],
    pdk: &Pdk,
    dir: &Scratch,
    args: &[String],
    dropped: &mut Vec<String>,
) -> Result<Result<String, String>, String> {
    let bound = pins.len();
    for _ in 0..=bound {
        let why = match spice_once(gpurify, input, pins, field, pdk, dir, args)? {
            Ok(text) => return Ok(Ok(text)),
            Err(why) => why,
        };
        let short = why.split("labels (").nth(1).and_then(|t| t.split(" first)").next()).map(str::to_string);
        match short {
            Some(name) if pins.iter().any(|p| p.name == name) => {
                pins.retain(|p| p.name != name);
                dropped.push(name);
            }
            _ => return Ok(Err(why)),
        }
    }
    unreachable!("each attempt drops a label, so at most every pin is dropped")
}

/// One `pex --format spice` process: `Ok(Err(stderr))` when it wrote no
/// netlist.
fn spice_once(
    gpurify: &Path,
    input: &Input,
    pins: &[LabeledPin],
    field: &[String],
    pdk: &Pdk,
    dir: &Scratch,
    args: &[String],
) -> Result<Result<String, String>, String> {
    let gds = dir.0.join("pex.gds");
    write(&gds, &crate::gds_out::gds(input.shapes, pins, input.cells, pdk, crate::foundry::TOP)?)?;
    let mut argv = vec!["pex".to_string(), gds.display().to_string()];
    argv.extend_from_slice(args);
    for net in field {
        argv.extend(["--quasistatic".into(), net.clone()]);
    }
    argv.extend(["--format".into(), "spice".into()]);
    let (_, stdout, stderr) = spawn(gpurify, &argv)?;
    Ok(if stdout.starts_with('*') { Ok(stdout) } else { Err(stderr.trim().to_string()) })
}

/// The intent file GPurify reads, `None` without supplies (the intent-gated
/// rules then skip) or ports to state. A ground joins the highest power domain: GPurify's
/// domains are voltage levels with a supply pair. One `limits` object per
/// net: GPurify refuses a net listed twice.
fn intent_json(intent: &Intent, ports: Option<&[String]>) -> Option<String> {
    let ports = ports.filter(|_| INTENT_PORTS);
    if intent.supplies.is_empty() && ports.is_none() {
        return None;
    }
    let domain = |mv: f64| format!("d{}", mv.round() as i64);
    let power_mv = intent.supplies.iter().filter(|s| !s.2).map(|s| s.1).fold(0.0, f64::max);
    let mut domains = serde_json::Map::new();
    let mut supplies = Vec::new();
    for (net, mv, ground) in &intent.supplies {
        let mv = if *ground { power_mv } else { *mv };
        domains.insert(domain(mv), serde_json::json!({ "voltage_mv": mv }));
        supplies.push(serde_json::json!({ "net": net, "domain": domain(mv), "role": if *ground { "ground" } else { "power" } }));
    }
    let mut by_net: std::collections::BTreeMap<&str, serde_json::Map<String, Value>> = std::collections::BTreeMap::new();
    for (net, ua) in intent.currents.iter().filter(|(_, ua)| *ua > 0.0) {
        by_net.entry(net).or_default().insert("budget_current_ua".into(), serde_json::json!(ua));
    }
    for (net, mv) in intent.max_drop_mv.iter().filter(|(_, mv)| *mv > 0.0 && mv.is_finite()) {
        by_net.entry(net).or_default().insert("max_drop_mv".into(), serde_json::json!(mv));
    }
    let limits: Vec<Value> = by_net
        .into_iter()
        .map(|(net, mut m)| {
            m.insert("net".into(), serde_json::json!(net));
            Value::Object(m)
        })
        .collect();
    let mut json = serde_json::json!({ "domains": domains, "supplies": supplies, "limits": limits });
    if let Some(ports) = ports {
        json["ports"] = serde_json::json!(ports);
    }
    Some(json.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: GPurify's own `pex --format spice` output for three labelled
    /// met1 lines (sky130, PEX-017 form), read by hand: C2 couples a and b.
    #[test]
    fn caps_of_spice_reads_ground_and_coupling() {
        let text = "* 0.1.0 pex\n.subckt three a b c\nC1 a 0 0.980870f\nR1 a a#1 2.5\nC2 a#1 b 0.286876f\nC3 n4 a 0.1f\nC4 b#2 b 0.5f\n.ends three\n";
        let (caps, total) = caps_of_spice(text);
        assert_eq!(caps, [("a".to_string(), None, 0.98087), ("a".to_string(), Some("b".to_string()), 0.286876)]);
        assert!((total - 1.867746).abs() < 1e-9, "{total}");
    }

    /// Oracle: the `summary:` block of GPurify's text report (docs/usage.md,
    /// "Reading the report"), copied from an asap7 run that refused PEX.
    #[test]
    fn summary_of_reads_a_refused_stage() {
        let text = "rules:\nviolations: 0\nsummary:\n  drc: ran\n  pex: refused, layer LISD: no pex row\n  1 rules skipped\n";
        assert_eq!(summary_of(text, "pex").as_deref(), Some("refused, layer LISD: no pex row"));
        assert_eq!(summary_of(text, "drc"), None);
    }

    /// Oracle: GPurify refuses a net listed twice (docs/usage.md, "Design
    /// intent"); one object per net carries both limits.
    #[test]
    fn intent_json_merges_a_nets_limits() {
        let intent =
            Intent { supplies: vec![("VDD".into(), 1_800.0, false)], currents: vec![("VDD".into(), 100.0)], max_drop_mv: vec![("VDD".into(), 5.0)] };
        let v: Value = serde_json::from_str(&intent_json(&intent, None).unwrap()).unwrap();
        assert_eq!(v["limits"].as_array().unwrap().len(), 1);
        assert_eq!(v["limits"][0]["max_drop_mv"], 5.0);
        assert_eq!(v["limits"][0]["budget_current_ua"], 100.0);
        assert!(intent_json(&Intent::default(), Some(&["in".to_string()])).is_none(), "no ports key until GPurify reads it");
    }
}
