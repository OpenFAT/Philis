//! The sign-off engine seam: the caller picks which engine signs a layout
//! off, and the answer is the same [`Signoff`] table whichever ran.
//!
//! - [`SignoffEngine::Gpurify`] is [`crate::signoff_checked`], in process.
//! - [`SignoffEngine::Klayout`] writes the GDS ([`crate::gds_out`]) and runs
//!   the foundry's KLayout DRC and LVS decks (`pdks/klayout/<pdk>/`).
//! - [`SignoffEngine::Magic`] runs magic's DRC and extraction and netgen's
//!   LVS on the foundry's tech and setup files (`pdks/magic/<pdk>/`).
//!
//! Never a fallback: a missing tool, a PDK with no vendored deck for the
//! engine, or a tool run that produced no verdict is a coded [`Refused`],
//! not a GPurify run and not a silent pass. What an engine does not check
//! (ERC, PEX) is a [`crate::Coverage::skipped_rules`] row.
//!
//! Findings are rows of `report.hard_violations`: `drc/<rule>` with margin =
//! the number of markers of that rule (the tools report no shortfall), and
//! `lvs/mismatch` (margin 1) when the LVS verdict is not a match.

pub mod klayout;
pub mod magic;

use std::path::{Path, PathBuf};
use std::time::Instant;

use analog::base::{Shape, Violation};

use crate::geom::LabeledPin;
use crate::pdk::Pdk;
use crate::reference::RefInput;
use crate::{Intent, Signoff};

/// Which engine signs off. Chosen by the caller, never by fallback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SignoffEngine {
    /// GPurify in process: DRC, ERC, LVS, PEX.
    #[default]
    Gpurify,
    /// The foundry's KLayout DRC and LVS decks.
    Klayout,
    /// magic DRC and extraction, netgen LVS.
    Magic,
}

impl SignoffEngine {
    /// Every engine, for a caller that runs them all (the differential test).
    pub const ALL: [Self; 3] = [Self::Gpurify, Self::Klayout, Self::Magic];

    /// The name the CLI and the report use.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Gpurify => "gpurify",
            Self::Klayout => "klayout",
            Self::Magic => "magic",
        }
    }
}

/// Why an engine could not sign off: a coded diagnostic, never a pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    /// `foundry/no-pdk` (the [`Pdk`] is not a built-in, so no deck is
    /// vendored for it), `foundry/no-deck` (none vendored for this engine on
    /// this PDK), `foundry/no-tool` (the binary is not on `PATH`),
    /// `foundry/gds` (the geometry cannot be written), `foundry/run` (the
    /// tool ran but left no verdict).
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Refused {}

pub(crate) fn refuse(code: &'static str, message: impl Into<String>) -> Refused {
    Refused { code, message: message.into() }
}

/// Sign `shapes` off against `reference` with `engine`. `intent` arms
/// GPurify's EM/IR rules and is unused by the foundry engines (their decks
/// have none; recorded as coverage).
///
/// # Errors
/// A [`Refused`] from a foundry engine; [`SignoffEngine::Gpurify`] never
/// refuses (its failures are `engine/…` rows, as [`crate::signoff_checked`]).
pub fn signoff(
    engine: SignoffEngine,
    shapes: &[Shape],
    pins: &[LabeledPin],
    reference: &RefInput,
    intent: &Intent,
    pdk: &Pdk,
) -> Result<Signoff, Refused> {
    match engine {
        SignoffEngine::Gpurify => Ok(crate::signoff_checked(shapes, pins, reference, intent, pdk)),
        SignoffEngine::Klayout => foundry(engine, shapes, pins, reference, pdk, klayout::run),
        SignoffEngine::Magic => foundry(engine, shapes, pins, reference, pdk, magic::run),
    }
}

/// What one foundry run concluded, before it becomes a [`Signoff`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    /// `(rule, markers)`, in the tool's order.
    pub drc: Vec<(String, usize)>,
    /// LVS matched.
    pub lvs_match: bool,
    /// The tool's own words on a mismatch, for the report.
    pub lvs_detail: String,
}

/// The files one run reads, in its scratch directory.
pub struct Run<'a> {
    pub dir: &'a Path,
    pub gds: PathBuf,
    pub top: &'a str,
    pub reference: &'a RefInput,
    pub pdk: &'a Pdk,
    pub name: &'static str,
}

/// The cell name every foundry run gives the layout and the reference.
/// Upper case: KLayout's SPICE reader upper-cases subcircuit names (IHP's
/// deck then pairs `PHILIS` with no layout cell `philis`).
pub const TOP: &str = "PHILIS";

fn foundry(
    engine: SignoffEngine,
    shapes: &[Shape],
    pins: &[LabeledPin],
    reference: &RefInput,
    pdk: &Pdk,
    run: fn(&Run) -> Result<Verdict, Refused>,
) -> Result<Signoff, Refused> {
    let t0 = Instant::now();
    let (mut v, kept) = in_scratch(engine, shapes, pins, reference, pdk, run, |v| v.drc.is_empty() && v.lvs_match)?;
    if let Some(dir) = kept {
        v.lvs_detail = format!("{} (logs in {})", v.lvs_detail, dir.display());
    }
    let mut s = Signoff::default();
    for (rule, n) in v.drc {
        s.report.hard_violations.push(Violation { rule: format!("drc/{rule}"), margin: n as i64 });
    }
    if !v.lvs_match {
        s.report.hard_violations.push(Violation { rule: format!("lvs/mismatch: {}", v.lvs_detail), margin: 1 });
    }
    let by = engine.name();
    for (check, what) in [("erc", "ERC"), ("pex", "parasitic extraction"), ("em_ir", "EM and IR-drop rules")] {
        s.coverage.skipped_rules.push((check.to_string(), format!("NotRun(the {by} engine runs no {what})")));
    }
    s.elapsed = t0.elapsed();
    Ok(s)
}

/// DRC alone with `engine`: `(rule, markers)`, warnings excluded. The fine
/// tier under [`signoff`], and the only foundry door on asap7, whose open
/// KLayout DRC deck has no LVS beside it. GPurify's rows are
/// `{rule}:{layer}`, as in [`crate::Signoff`].
///
/// # Errors
/// As [`signoff`].
pub fn drc(engine: SignoffEngine, shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk) -> Result<Vec<(String, usize)>, Refused> {
    let mut rows: Vec<(String, usize)> = Vec::new();
    match engine {
        SignoffEngine::Gpurify => {
            for f in crate::drc(shapes, pins, pdk).into_iter().filter(|f| !f.warning) {
                let rule = format!("{}:{}", f.rule, f.layer);
                match rows.iter_mut().find(|(r, _)| *r == rule) {
                    Some(row) => row.1 += 1,
                    None => rows.push((rule, 1)),
                }
            }
        }
        SignoffEngine::Klayout => rows = in_scratch(engine, shapes, pins, &RefInput::default(), pdk, klayout::drc, Vec::is_empty)?.0,
        SignoffEngine::Magic => rows = in_scratch(engine, shapes, pins, &RefInput::default(), pdk, magic::drc, Vec::is_empty)?.0,
    }
    Ok(rows)
}

/// Write the GDS into a fresh scratch directory and call `f` there. The
/// directory is removed unless the run had findings (`clean` is false) or a
/// tool ran without a verdict (`foundry/run`); then its path is returned
/// (or named in the refusal) so the logs can be read.
fn in_scratch<T>(
    engine: SignoffEngine,
    shapes: &[Shape],
    pins: &[LabeledPin],
    reference: &RefInput,
    pdk: &Pdk,
    f: fn(&Run) -> Result<T, Refused>,
    clean: fn(&T) -> bool,
) -> Result<(T, Option<PathBuf>), Refused> {
    let name = pdk.name.ok_or_else(|| {
        refuse("foundry/no-pdk", format!("the {} engine finds its decks by built-in PDK name; this Pdk was loaded from text", engine.name()))
    })?;
    let bytes = crate::gds_out::gds(shapes, pins, &[], pdk, TOP).map_err(|e| refuse("foundry/gds", e))?;
    let dir = scratch(engine)?;
    let gds = dir.join(format!("{TOP}.gds"));
    std::fs::write(&gds, bytes).map_err(|e| refuse("foundry/run", format!("{}: {e}", gds.display())))?;
    match f(&Run { dir: &dir, gds, top: TOP, reference, pdk, name }) {
        Ok(v) if clean(&v) => {
            let _ = std::fs::remove_dir_all(&dir);
            Ok((v, None))
        }
        Ok(v) => Ok((v, Some(dir))),
        Err(r) if r.code == "foundry/run" => Err(refuse(r.code, format!("{} (logs in {})", r.message, dir.display()))),
        Err(r) => {
            let _ = std::fs::remove_dir_all(&dir);
            Err(r)
        }
    }
}

/// A fresh scratch directory for one run.
fn scratch(engine: SignoffEngine) -> Result<PathBuf, Refused> {
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("philis-{}-{}-{n}", engine.name(), std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| refuse("foundry/run", format!("{}: {e}", dir.display())))?;
    Ok(dir)
}

/// The vendored deck tree (`pdks/` at the repository root).
#[must_use]
pub fn decks_root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../pdks"))
}

/// `pdks/<engine>/<pdk>/<file>`, which must exist.
pub(crate) fn deck(engine: &str, pdk: &str, file: &str, what: &str) -> Result<PathBuf, Refused> {
    let p = decks_root().join(engine).join(pdk).join(file);
    if p.is_file() {
        Ok(p)
    } else {
        Err(refuse("foundry/no-deck", format!("no {what} deck for {pdk}: {} is not vendored (pdks/README.md)", p.display())))
    }
}

/// `name` on `PATH`.
pub(crate) fn tool(name: &str) -> Result<PathBuf, Refused> {
    std::env::var_os("PATH").and_then(|path| std::env::split_paths(&path).map(|d| d.join(name)).find(|p| p.is_file())).ok_or_else(|| {
        refuse(
            "foundry/no-tool",
            format!("{name} is not on PATH (nix shell nixpkgs#klayout nixpkgs#magic-vlsi nixpkgs#netgen-vlsi, or the flake's dev shell)"),
        )
    })
}

/// Run `cmd`, returning stdout + stderr (also written to `log`) and whether
/// it exited 0; a spawn failure is `foundry/run`.
pub(crate) fn output(cmd: &mut std::process::Command, log: &Path) -> Result<(String, bool), Refused> {
    let out = cmd.output().map_err(|e| refuse("foundry/run", format!("{cmd:?}: {e}")))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    let _ = std::fs::write(log, &text);
    Ok((text, out.status.success()))
}

/// The last `n` lines of a log, for a refusal's message.
pub(crate) fn tail(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// `card` with `suffix` appended to its `w=`/`l=` values (µm as
/// [`crate::Pdk::reference_spice`] writes them, unitless).
pub(crate) fn suffix_lengths(card: &str, suffix: &str) -> String {
    card.split(' ')
        .map(|t| if t.starts_with("w=") || t.starts_with("l=") { format!("{t}{suffix}") } else { t.to_string() })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The thread count handed to the tools.
pub(crate) fn threads() -> usize {
    std::thread::available_parallelism().map_or(4, std::num::NonZero::get)
}
