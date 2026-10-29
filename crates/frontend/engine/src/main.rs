//! `philis`: the recipe spine from the command line. Arguments and files in;
//! the report on stdout and, with `-o DIR`, `DIR/report.txt` and
//! `DIR/evidence.json` out. The shape (flags anywhere after the subcommand,
//! exit 0 clean, 1 not clean, 2 error) is Philis 1.0's `frontend/cli`
//! (5b4fe98); the evidence flags are #77 (Q61). The help text below is the
//! CLI help `docs/API.md` points to.

use std::path::PathBuf;
use std::process::ExitCode;

const HELP: &str = "\
philis: analog layout from a SPICE netlist, by recipe (Philis 2.0)

usage: philis run <netlist.spice> <pdk> [--evidence-currents FILE] [--evidence-op FILE] [-o DIR]

  <pdk>                     a built-in deck: sky130, gf180mcu, ihp_sg13g2, asap7
  --evidence-currents FILE  an espice-emir/1 file (espice --emir): each instance
                            terminal's avg, em_avg, rms and peak current
  --evidence-op FILE        an ESPice raw file (binary or ASCII) of an .op that
                            saves each device's vds, vgs, id and vdsat
                            (.save @m1[vds] ... or .probe op); gm as well sets
                            the device's region and role
  -o, --out DIR             also write DIR/report.txt and DIR/evidence.json

Evidence is keyed by instance names, resolved against the netlist when it is
read: a name the netlist lacks is reported, and the rest is read. Without
either flag nothing changes. ROUTE-EmWidth (M2) and the MOS-cascode V_DS claim
(v2) will read these rows; until then the report lists them.

Exit status: 0 signed off with no hard violation, 1 not, 2 refused or a usage
error.";

fn main() -> ExitCode {
    match cli(std::env::args().skip(1).collect()) {
        Ok(clean) => ExitCode::from(u8::from(!clean)),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

/// `Ok(true)` when the run signed off with no hard violation.
fn cli(args: Vec<String>) -> Result<bool, String> {
    let usage = || HELP.lines().find(|l| l.starts_with("usage:")).unwrap_or(HELP).to_string();
    let mut it = args.into_iter();
    match it.next().as_deref() {
        Some("run") => {}
        Some("-h" | "--help") => {
            println!("{HELP}");
            return Ok(true);
        }
        _ => return Err(usage()),
    }
    let (mut pos, mut currents, mut op, mut out) = (Vec::new(), None, None, None);
    while let Some(a) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{a} needs a value\n{}", usage()));
        match a.as_str() {
            "--evidence-currents" => currents = Some(value()?),
            "--evidence-op" => op = Some(value()?),
            "-o" | "--out" => out = Some(PathBuf::from(value()?)),
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(true);
            }
            f if f.starts_with('-') => return Err(format!("unknown flag {f}\n{}", usage())),
            _ => pos.push(a),
        }
    }
    let [netlist, deck] = &pos[..] else { return Err(usage()) };
    let read = |p: &str| std::fs::read(p).map_err(|e| format!("read {p}: {e}"));
    let text = |p: &str| String::from_utf8(read(p)?).map_err(|e| format!("read {p}: {e}"));
    let spice = text(netlist)?;
    let pdk = verify::Pdk::builtin(deck)?;
    let currents = currents.map(|p| text(&p).map(|t| (p, t))).transpose()?;
    let op = op.map(|p| read(&p).map(|b| (p, b))).transpose()?;
    let files = engine::EvidenceFiles {
        currents: currents.as_ref().map(|(p, t)| (p.as_str(), t.as_str())),
        op: op.as_ref().map(|(p, b)| (p.as_str(), b.as_slice())),
    };
    let outcome = engine::run_with(&spice, &files, &pdk, verify::SignoffEngine::Gpurify).map_err(|r| format!("refused: {r}"))?;
    print!("{}", outcome.report);
    if let Some(dir) = out {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for (file, body) in [("report.txt", &outcome.report), ("evidence.json", &outcome.evidence_json)] {
            std::fs::write(dir.join(file), body).map_err(|e| format!("{}: {e}", dir.join(file).display()))?;
        }
    }
    Ok(outcome.signoff.report.hard_violations.is_empty())
}
