//! The KLayout engine: the foundry's DRC deck over the GDS, its LVS deck
//! against the reference as SPICE element cards. Ported from 1.0's
//! `benchmarks/signoff_xcheck.py` (`drc_argv`, `klayout_ref`, `klayout_lvs`).

use std::path::PathBuf;
use std::process::Command;

use super::{deck, output, refuse, tail, threads, tool, Refused, Run, Verdict};
use crate::reference::RefKind;

/// One deck and the `-rd` variables it takes.
struct Deck {
    file: PathBuf,
    vars: Vec<(&'static str, String)>,
}

/// DRC and LVS of `run.gds` against `run.reference`.
///
/// # Errors
/// `klayout` not on `PATH`, a deck not vendored for the PDK (asap7 has no
/// open LVS deck), or a run that left no report or no verdict.
pub fn run(run: &Run) -> Result<Verdict, Refused> {
    // Both decks first: a PDK without one refuses before anything runs.
    let (d, l) = (drc_deck(run)?, lvs_deck(run)?);
    let mut v = Verdict { drc: drc_with(run, &d)?, ..Verdict::default() };
    (v.lvs_match, v.lvs_detail) = lvs_with(run, &l)?;
    Ok(v)
}

/// The DRC half alone: `(rule, markers)`.
///
/// # Errors
/// As [`run`].
pub fn drc(run: &Run) -> Result<Vec<(String, usize)>, Refused> {
    drc_with(run, &drc_deck(run)?)
}

fn drc_report(run: &Run) -> PathBuf {
    run.dir.join("drc.lyrdb")
}

fn drc_deck(run: &Run) -> Result<Deck, Refused> {
    let (input, top, rep) = (run.gds.display().to_string(), run.top.to_string(), drc_report(run).display().to_string());
    let thr = threads().to_string();
    let d = match run.name {
        // feol/beol default to false in sky130A_mr.drc: without both it checks nothing.
        "sky130" => Deck {
            file: deck("klayout", "sky130", "sky130A_mr.drc", "KLayout DRC")?,
            vars: vec![("input", input), ("top_cell", top), ("report", rep), ("feol", "1".into()), ("beol", "1".into()), ("offgrid", "1".into()), ("thr", thr)],
        },
        // Variant E is the 6-metal, 9 kÅ top stack pdks/gf180mcu.deck draws.
        // Density is a chip-level check (GPurify defers it the same way).
        "gf180mcu" => Deck {
            file: deck("klayout", "gf180mcu", "drc/gf180mcu.drc", "KLayout DRC")?,
            vars: vec![("input", input), ("topcell", top), ("report", rep), ("variant", "E".into()), ("run_mode", "deep".into()), ("threads", thr), ("decks", "all,-density".into())],
        },
        // The rule values live in the PDK's pycell tech JSON, which the deck
        // otherwise looks for at ../../python/ (not vendored as a tree).
        "ihp_sg13g2" => Deck {
            file: deck("klayout", "ihp_sg13g2", "drc/ihp-sg13g2.drc", "KLayout DRC")?,
            vars: vec![
                ("input", input),
                ("topcell", top),
                ("report", rep),
                ("run_mode", "deep".into()),
                ("threads", thr),
                ("drc_json", deck("klayout", "ihp_sg13g2", "sg13g2_tech_mod.json", "IHP DRC rule values")?.display().to_string()),
            ],
        },
        "asap7" => Deck { file: deck("klayout", "asap7", "asap7.lydrc", "KLayout DRC")?, vars: vec![("in_gds", input), ("report_file", rep)] },
        other => return Err(refuse("foundry/no-deck", format!("no KLayout DRC deck for {other}"))),
    };
    Ok(d)
}

fn drc_with(run: &Run, d: &Deck) -> Result<Vec<(String, usize)>, Refused> {
    let report = drc_report(run);
    let (log, ok) = klayout(d, &run.dir.join("drc.log"))?;
    if !ok {
        return Err(crashed(d, &log));
    }
    let xml = std::fs::read_to_string(&report).map_err(|_| refuse("foundry/run", format!("KLayout DRC wrote no report ({}):\n{}", report.display(), tail(&log, 20))))?;
    Ok(parse_lyrdb(&xml))
}

/// The LVS half alone: `(matched, the log's verdict line)`.
///
/// # Errors
/// As [`run`].
pub fn lvs(run: &Run) -> Result<(bool, String), Refused> {
    lvs_with(run, &lvs_deck(run)?)
}

fn schematic(run: &Run) -> PathBuf {
    run.dir.join(format!("{}.ref.cir", run.top))
}

fn lvs_deck(run: &Run) -> Result<Deck, Refused> {
    let schematic = schematic(run);
    let (input, rep) = (run.gds.display().to_string(), run.dir.join("lvs.lvsdb").display().to_string());
    let (sch, ext) = (schematic.display().to_string(), run.dir.join(format!("{}.ext.cir", run.top)).display().to_string());
    let thr = threads().to_string();
    let d = match run.name {
        // The deck's substrate is an empty layer joined by name: every NMOS
        // bulk is the global net `lvs_sub` (default `sky130_gnd`), so a tap
        // is never what connects it. Named after the reference's one NMOS
        // bulk net; with several (isolated wells) left at the default, so
        // LVS reports the difference rather than hiding it.
        "sky130" => {
            let mut vars = vec![("input", input), ("report", rep), ("schematic", sch), ("target_netlist", ext), ("run_mode", "deep".into()), ("schematic_simplify", "true".into()), ("thr", thr)];
            if let Some(sub) = substrate(run.reference) {
                vars.push(("lvs_sub", sub));
            }
            Deck { file: deck("klayout", "sky130", "sky130.lvs", "KLayout LVS")?, vars }
        }
        "gf180mcu" => Deck {
            file: deck("klayout", "gf180mcu", "lvs/gf180mcu.lvs", "KLayout LVS")?,
            vars: vec![("input", input), ("report", rep), ("schematic", sch), ("target_netlist", ext), ("topcell", run.top.to_string()), ("run_mode", "deep".into()), ("metal_level", "6LM".into()), ("metal_top", "9K".into()), ("mim_option", "B".into())],
        },
        // Taps extract as ntap1/ptap1 devices unless disabled; the reference has none.
        "ihp_sg13g2" => Deck {
            file: deck("klayout", "ihp_sg13g2", "lvs/sg13g2.lvs", "KLayout LVS")?,
            vars: vec![("input", input), ("report", rep), ("schematic", sch), ("target_netlist", ext), ("topcell", run.top.to_string()), ("run_mode", "deep".into()), ("disable_tap_extraction", "true".into())],
        },
        other => return Err(refuse("foundry/no-deck", format!("no open KLayout LVS deck exists for {other} (pdks/README.md)"))),
    };
    Ok(d)
}

fn lvs_with(run: &Run, d: &Deck) -> Result<(bool, String), Refused> {
    std::fs::write(schematic(run), reference(run)).map_err(|e| refuse("foundry/run", e.to_string()))?;
    // sky130.lvs exits 1 on a mismatch: the verdict line, printed only after
    // the compare finished, decides; a non-zero exit without one crashed.
    let (log, ok) = klayout(d, &run.dir.join("lvs.log"))?;
    match lvs_verdict(&log) {
        Some(v) => Ok(v),
        None if !ok => Err(crashed(d, &log)),
        None => Err(refuse("foundry/run", format!("KLayout LVS gave no verdict:\n{}", tail(&log, 20)))),
    }
}

/// One `klayout -b -r` run, its log and whether it exited 0.
fn klayout(d: &Deck, log: &std::path::Path) -> Result<(String, bool), Refused> {
    let mut cmd = Command::new(tool("klayout")?);
    cmd.arg("-b").arg("-r").arg(&d.file);
    for (k, v) in &d.vars {
        cmd.arg("-rd").arg(format!("{k}={v}"));
    }
    output(&mut cmd, log)
}

/// A non-zero exit is a crashed deck (a Ruby error, a missing include or
/// data file), refused: such a run may still leave an empty report behind,
/// which must not read as clean. Every DRC deck exits 0 on violations.
fn crashed(d: &Deck, log: &str) -> Refused {
    refuse("foundry/run", format!("klayout exited non-zero on {}:\n{}", d.file.display(), tail(log, 12)))
}

/// The reference as SPICE element cards: every deck's reader turns `M`,
/// `Q`, `R`, `C`, `D` cards into devices, while an `X` card would be a call
/// to an undefined subcircuit, compared as nothing. KLayout reads an `M`
/// card's W/L in metres, [`crate::Pdk::reference_spice`] writes µm.
#[must_use]
pub fn reference(run: &Run) -> String {
    let text = run.pdk.reference_spice(run.reference, run.top, &[]);
    let mut lines = text.lines();
    let mut out = format!("{}\n", lines.next().unwrap_or_default());
    // reference_spice writes one card line per device, in order.
    for (dev, line) in run.reference.devices.iter().zip(lines.by_ref()) {
        let letter = match dev.kind {
            RefKind::Nmos | RefKind::Pmos => "M",
            RefKind::Npn | RefKind::Pnp => "Q",
            RefKind::Resistor => "R",
            RefKind::Capacitor => "C",
            RefKind::Diode => "D",
            RefKind::Inductor => "L",
        };
        let card = match line.strip_prefix('X') {
            Some(rest) => format!("{letter}{rest}"),
            None => line.to_string(),
        };
        let card = if letter == "M" { super::suffix_lengths(&card, "u") } else { card };
        out.push_str(&card);
        out.push('\n');
    }
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// The one net every NMOS bulk terminal names, if there is exactly one.
fn substrate(reference: &crate::reference::RefInput) -> Option<String> {
    let mut bulks = reference.devices.iter().filter(|d| d.kind == RefKind::Nmos).filter_map(|d| d.terminals.get(3));
    let first = bulks.next()?;
    bulks.all(|b| b == first).then(|| first.clone())
}

/// `(category, markers)` over the `<items>` of a KLayout report database,
/// first-seen order, each item counted by its `<multiplicity>` (default 1).
/// A category is the rule name with KLayout's quotes stripped.
#[must_use]
pub fn parse_lyrdb(xml: &str) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    let Some(items) = xml.find("<items>").map(|i| &xml[i..]) else { return out };
    for item in items.split("<item>").skip(1) {
        let Some(cat) = between(item, "<category>", "</category>") else { continue };
        let rule = unescape(cat).trim_matches('\'').to_string();
        let n = between(item, "<multiplicity>", "</multiplicity>").and_then(|m| m.trim().parse().ok()).unwrap_or(1);
        match out.iter_mut().find(|(r, _)| *r == rule) {
            Some(row) => row.1 += n,
            None => out.push((rule, n)),
        }
    }
    out
}

/// `Some((true, line))` for "Netlists match", `Some((false, line))` for
/// "Netlists don't match" (sky130, gf180mcu and IHP decks all log one), else
/// `None`: a crashed run is no verdict.
#[must_use]
pub fn lvs_verdict(log: &str) -> Option<(bool, String)> {
    let line = log.lines().rev().find(|l| l.contains("Netlists match") || l.contains("Netlists don't match"))?;
    Some((line.contains("Netlists match"), line.trim().to_string()))
}

fn between<'a>(s: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let a = s.find(open)? + open.len();
    let b = s[a..].find(close)? + a;
    Some(&s[a..b])
}

fn unescape(s: &str) -> String {
    s.replace("&apos;", "'").replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A report database as KLayout 0.30.8 wrote it for sky130A_mr.drc over
    /// `pair` with one li shape narrowed (the negative control of
    /// `engine/tests/foundry.rs`): the category list trimmed to two and the
    /// generator line dropped, the items verbatim. Categories are not
    /// counted, items are, by multiplicity.
    const LYRDB: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<report-database>
 <description>SKY130 DRC runset</description>
 <original-file/>
 <top-cell>philis</top-cell>
 <tags>
 </tags>
 <categories>
  <category>
   <name>dnwell.2</name>
   <description>dnwell.2 : min. dnwell width : 3.0um</description>
   <categories>
   </categories>
  </category>
  <category>
   <name>nwell.1</name>
   <description>nwell.1 : min. nwell width : 0.84um</description>
   <categories>
   </categories>
  </category>
 </categories>
 <cells>
  <cell>
   <name>philis</name>
   <variant/>
   <layout-name/>
   <references>
   </references>
  </cell>
 </cells>
 <items>
  <item>
   <tags/>
   <category>'li.1'</category>
   <cell>philis</cell>
   <visited>false</visited>
   <multiplicity>1</multiplicity>
   <comment/>
   <image/>
   <values>
    <value>edge-pair: (0.59,0.02;0.59,0.35)|(0.735,0.35;0.735,0.02)</value>
   </values>
  </item>
  <item>
   <tags/>
   <category>'li.6'</category>
   <cell>philis</cell>
   <visited>false</visited>
   <multiplicity>1</multiplicity>
   <comment/>
   <image/>
   <values>
    <value>polygon: (0.59,0.02;0.59,0.35;0.735,0.35;0.735,0.02)</value>
   </values>
  </item>
  <item>
   <tags/>
   <category>'ct.4'</category>
   <cell>philis</cell>
   <visited>false</visited>
   <multiplicity>1</multiplicity>
   <comment/>
   <image/>
   <values>
    <value>polygon: (0.735,0.1;0.735,0.27;0.76,0.27;0.76,0.1)</value>
   </values>
  </item>
 </items>
</report-database>
"#;

    #[test]
    fn a_report_database_counts_items_by_rule() {
        let rows = |r: &[(&str, usize)]| r.iter().map(|&(a, n)| (a.to_string(), n)).collect::<Vec<_>>();
        assert_eq!(parse_lyrdb(LYRDB), rows(&[("li.1", 1), ("li.6", 1), ("ct.4", 1)]));
        let doubled = LYRDB.replacen("<multiplicity>1</multiplicity>", "<multiplicity>2</multiplicity>", 1);
        assert_eq!(parse_lyrdb(&doubled), rows(&[("li.1", 2), ("li.6", 1), ("ct.4", 1)]));
    }

    #[test]
    fn an_empty_report_database_is_clean() {
        assert!(parse_lyrdb("<report-database><categories></categories><items>\n</items></report-database>").is_empty());
    }

    #[test]
    fn the_lvs_verdict_is_read_off_the_log() {
        // Captured from sky130.lvs on pair, and on pair against a W=4 reference.
        let ok = "2026-10-25 13:17:24 -0400: Memory Usage (5254356K) : INFO : Congratulations! Netlists match.\n";
        let bad = "2026-10-25 13:17:25 -0400: Memory Usage (5254804K) : xxxxxxxx\n2026-10-25 13:17:25 -0400: Memory Usage (5254804K) : ERROR : Netlists don't match\n";
        assert!(lvs_verdict(ok).unwrap().0);
        assert!(!lvs_verdict(bad).unwrap().0);
        assert_eq!(lvs_verdict("ERROR: undefined method"), None);
    }
}
