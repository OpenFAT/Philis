//! The magic engine: magic's full DRC style and its LVS extraction over the
//! GDS, then netgen's LVS of that extraction against
//! [`crate::Pdk::reference_spice`] with the foundry's setup file.

use std::process::{Command, Stdio};

use super::{deck, output, refuse, tail, tool, Refused, Run, Verdict};

/// `(tech file, netgen setup, length suffix)` under `pdks/magic/<pdk>/`.
/// The suffix is the unit `ext2spice lvs` writes W and L in with that tech
/// (sky130's are bare µm, gf180mcu's and IHP's carry `u`), so the reference
/// states them the same way: netgen compares the numbers.
fn files(pdk: &str) -> Result<(&'static str, &'static str, &'static str), Refused> {
    Ok(match pdk {
        "sky130" => ("sky130A.tech", "sky130A_setup.tcl", ""),
        "gf180mcu" => ("gf180mcuD.tech", "gf180mcuD_setup.tcl", "u"),
        "ihp_sg13g2" => ("ihp-sg13g2.tech", "ihp-sg13g2_setup.tcl", "u"),
        other => return Err(refuse("foundry/no-deck", format!("no open magic tech file or netgen setup exists for {other} (pdks/README.md)"))),
    })
}

/// DRC, extraction and LVS.
///
/// # Errors
/// `magic` or `netgen` not on `PATH`, no tech file for the PDK, or a run
/// that left no DRC count, no extracted netlist, or no netgen verdict.
pub fn run(run: &Run) -> Result<Verdict, Refused> {
    let (tech, setup, unit) = files(run.name)?;
    let tech = deck("magic", run.name, tech, "magic tech")?;
    let setup = deck("magic", run.name, setup, "netgen setup")?;
    let (magic, netgen) = (tool("magic")?, tool("netgen")?);
    let top = run.top;
    // Labels become ports, so the extracted subcircuit has the pins netgen
    // pairs with the reference's (GDS text reads as plain labels).
    let extract = format!("port makeall\nextract all\next2spice lvs\next2spice -o {top}.ext.spice\n");
    let drc = session(run, &magic, &tech, &extract)?;
    let layout = run.dir.join(format!("{top}.ext.spice"));
    if !layout.is_file() {
        return Err(refuse("foundry/run", "magic extracted no netlist (magic.log)"));
    }
    let reference = run.dir.join(format!("{top}.ref.spice"));
    let text: String = run.pdk.reference_spice(run.reference, top, &[]).lines().map(|l| super::suffix_lengths(l, unit) + "\n").collect();
    std::fs::write(&reference, text).map_err(|e| refuse("foundry/run", e.to_string()))?;
    let comp = run.dir.join("comp.out");
    let mut cmd = Command::new(netgen);
    cmd.arg("-batch")
        .arg("lvs")
        .arg(format!("{} {top}", layout.display()))
        .arg(format!("{} {top}", reference.display()))
        .arg(&setup)
        .arg(&comp)
        .current_dir(run.dir);
    // netgen's exit status is not its verdict; comp.out's last line is.
    let (log, _) = output(&mut cmd, &run.dir.join("netgen.log"))?;
    let report = std::fs::read_to_string(&comp).unwrap_or_default();
    let (lvs_match, lvs_detail) =
        netgen_verdict(&report).ok_or_else(|| refuse("foundry/run", format!("netgen gave no verdict:\n{}", tail(&log, 20))))?;
    Ok(Verdict { drc, lvs_match, lvs_detail })
}

/// DRC alone, one magic session.
///
/// # Errors
/// As [`run`], less netgen.
pub fn drc(run: &Run) -> Result<Vec<(String, usize)>, Refused> {
    let (tech, _, _) = files(run.name)?;
    let tech = deck("magic", run.name, tech, "magic tech")?;
    session(run, &tool("magic")?, &tech, "")
}

/// One batch session: read the GDS, full-style DRC listed by reason, then
/// `more` (the extraction), then quit. Its DRC rows.
fn session(run: &Run, magic: &std::path::Path, tech: &std::path::Path, more: &str) -> Result<Vec<(String, usize)>, Refused> {
    let top = run.top;
    let script = format!(
        "crashbackups stop\ndrc off\ngds read {top}.gds\nload {top}\nselect top cell\ndrc style drc(full)\ndrc check\ndrc catchup\n\
         foreach {{why boxes}} [drc listall why] {{ puts \"PHILIS_DRC\\t$why\\t[llength $boxes]\" }}\n\
         drc count total\n{more}quit -noprompt\n"
    );
    let mut cmd = Command::new(magic);
    cmd.args(["-dnull", "-noconsole", "-T"]).arg(tech).current_dir(run.dir).stdin(Stdio::piped());
    let log = with_stdin(&mut cmd, &script, &run.dir.join("magic.log"))?;
    parse_drc(&log).ok_or_else(|| refuse("foundry/run", format!("magic printed no DRC count:\n{}", tail(&log, 20))))
}

fn with_stdin(cmd: &mut Command, input: &str, log: &std::path::Path) -> Result<String, Refused> {
    use std::io::Write;
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| refuse("foundry/run", format!("{cmd:?}: {e}")))?;
    child.stdin.take().expect("piped").write_all(input.as_bytes()).map_err(|e| refuse("foundry/run", e.to_string()))?;
    let out = child.wait_with_output().map_err(|e| refuse("foundry/run", e.to_string()))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::write(log, &text);
    Ok(text)
}

/// `(why, boxes)` per `PHILIS_DRC` line of the script's listing; `None`
/// without magic's `Total DRC errors found:` line (the run never got there).
/// A total above the listed boxes (errors in subcells `listall` missed) is
/// one `magic.unlisted` row, so the count never drops.
#[must_use]
pub fn parse_drc(log: &str) -> Option<Vec<(String, usize)>> {
    let total: usize = log.lines().find_map(|l| l.trim().strip_prefix("Total DRC errors found:"))?.trim().parse().ok()?;
    let mut rows: Vec<(String, usize)> = Vec::new();
    for l in log.lines().filter_map(|l| l.strip_prefix("PHILIS_DRC\t")) {
        let mut f = l.rsplitn(2, '\t');
        let (Some(n), Some(why)) = (f.next(), f.next()) else { continue };
        let n: usize = n.trim().parse().unwrap_or(1);
        match rows.iter_mut().find(|(r, _)| r == why) {
            Some(row) => row.1 += n,
            None => rows.push((why.to_string(), n)),
        }
    }
    let listed: usize = rows.iter().map(|r| r.1).sum();
    if total > listed {
        rows.push(("magic.unlisted".to_string(), total - listed));
    }
    Some(rows)
}

/// netgen's `Final result:` line: a unique match is a match; anything else
/// (no match, port errors, symmetry ties) is not, and neither is a unique
/// match netgen reports `Property errors were found.` beside (a W or L that
/// differs: its final line still says "match uniquely"). `None` without the
/// line.
#[must_use]
pub fn netgen_verdict(report: &str) -> Option<(bool, String)> {
    let line = report.lines().rev().find(|l| l.starts_with("Final result:"))?.trim().to_string();
    let property = report.contains("Property errors were found.");
    let ok = line.contains("match uniquely") && !line.contains("port errors") && !property;
    Some((ok, if property { format!("{line} Property errors were found.") } else { line }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_drc_rows_are_listed_by_reason() {
        // Captured from magic 8.3.629 with sky130A.tech over pair with one li shape narrowed.
        let log = "PHILIS_DRC\tLocal interconnect width < 0.17um (li.1)\t3\nPHILIS_DRC\tpoly contact width        < 0.17um (licon.1)\t1\nPHILIS_DRC\tLocal interconnect minimum area < 0.0561um^2 (li.6)\t6\nTotal DRC errors found: 3\n";
        let rows = parse_drc(log).unwrap();
        assert_eq!(rows.iter().map(|r| r.1).collect::<Vec<_>>(), [3, 1, 6]);
        assert_eq!(rows[0].0, "Local interconnect width < 0.17um (li.1)");
        assert_eq!(parse_drc("Total DRC errors found: 0\n").unwrap(), []);
        assert_eq!(parse_drc("Total DRC errors found: 4\n").unwrap(), [("magic.unlisted".to_string(), 4)]);
        assert_eq!(parse_drc("magic crashed"), None);
    }

    #[test]
    fn the_netgen_verdict_is_its_final_result() {
        assert!(netgen_verdict("Circuit 1 contains 2 devices\nFinal result: Circuits match uniquely.\n").unwrap().0);
        assert!(!netgen_verdict("Final result: Netlists do not match.\n").unwrap().0);
        assert!(!netgen_verdict("Final result: Circuits match uniquely with port errors.\n").unwrap().0);
        // Captured from netgen 1.5.320 with the sky130 setup, pair against a W=4 reference.
        let property = "Netlists match uniquely with property errors.\nsky130_fd_pr__nfet_01v8:0 vs. sky130_fd_pr__nfet_01v8:0:\n w circuit1: 2   circuit2: 4   (delta=66.7%, cutoff=1%)\n\nFinal result: Circuits match uniquely.\nProperty errors were found.\n";
        assert!(!netgen_verdict(property).unwrap().0);
        assert_eq!(netgen_verdict(""), None);
    }
}
