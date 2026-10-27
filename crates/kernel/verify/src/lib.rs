#![recursion_limit = "1024"]
//! `verify` — physical verification through GPurify 2.0, run as the
//! `gpurify` binary (Q61, [`gpurify`]).
//!
//! [`signoff_checked`] runs DRC/ERC/LVS/PEX and returns a [`Signoff`]: errors
//! as a [`analog::base::Report`], deck warnings and what was not checked
//! ([`Coverage`]) apart from it; [`drc`]/[`erc`] are standalone probes.
//! [`Pdk`] (the process schema every crate reads) lives here too.

mod decks;
pub mod foundry;
pub mod gds_out;
pub mod geom;
pub mod gpurify;
pub mod netlist;
pub mod overrides;
pub mod pdk;
pub mod pdk_data;
pub mod reference;
pub mod sidecar;

use std::time::{Duration, Instant};

use analog::base::{Report, Shape, Violation};

pub use foundry::{Refused, SignoffEngine};
pub use gdsverify::ingest::deck::DeviceKind;
pub use geom::LabeledPin;
pub use gpurify::{Checks, Measurement};
pub use netlist::{device_count, extract_spice, Detail};
pub use pdk::{EmLimit, Pdk};
pub use reference::{RefDeviceIn, RefInput, RefKind};

/// Shortfall of one violation row, in one of two units. A length pair is nm:
/// `|limit − measured|`, rounded — every row is a violation, so that is a
/// minimum's shortfall and a maximum's overshoot alike (a row does not carry
/// its rule's sense; LU.2's 20 µm tap distance against 15 µm reads 5000).
/// Any other same-dimension pair (area, ratio, count, voltage, current,
/// resistance) is ‰ of the limit, `ceil(1000·|measured − limit| / |limit|)`
/// (`|limit|` floored at 1e-12), so an antenna ratio 2× over reads 1000 and
/// 1 % over reads 10; a NaN reads `i64::MAX`. Mismatched dimensions read 1.
#[must_use]
pub fn shortfall(limit: Measurement, measured: Measurement) -> i64 {
    use Measurement as M;
    // A NaN reading is no graze: it ranks as the worst miss (`as` would make it 0).
    let permille = |l: f64, m: f64| {
        let x = (1000.0 * (m - l).abs() / l.abs().max(1e-12)).ceil();
        if x.is_nan() {
            i64::MAX
        } else {
            x as i64
        }
    };
    match (limit, measured) {
        (M::Length(l), M::Length(m)) => (l - m).abs().round() as i64,
        (M::Area(l), M::Area(m))
        | (M::Ratio(l), M::Ratio(m))
        | (M::Count(l), M::Count(m))
        | (M::Voltage(l), M::Voltage(m))
        | (M::Current(l), M::Current(m))
        | (M::Resistance(l), M::Resistance(m)) => permille(l, m),
        _ => 1,
    }
}

/// Everything one signoff concluded: errors, warnings and what was not
/// checked, kept apart.
#[derive(Default)]
pub struct Signoff {
    /// Errors only: DRC/ERC/LVS rows of severity error, `engine/…`, one
    /// `lvs-coverage/…` per [`Coverage::unverified`] entry. Its `cost` is
    /// PEX's total C, fF.
    pub report: Report,
    /// Rows the deck states as warnings. Never in `report.hard_violations`.
    pub warnings: Vec<Violation>,
    /// What this run did not check: LVS-unverified devices, skipped rules.
    pub coverage: Coverage,
    /// PEX C over labelled nets, fF ([`gpurify::caps_of_spice`]); empty
    /// without PEX or when GPurify wrote no netlist.
    pub caps: CapMatrix,
    /// Wall time.
    pub elapsed: Duration,
    /// Wall time of the field-solved PEX netlist run
    /// ([`ExtractOptions::field_solve`]); zero without one.
    pub pex_field: Duration,
}

/// How PEX extracts. The default is analytical.
#[derive(Clone, Debug, Default)]
pub struct ExtractOptions {
    /// Labelled nets to field-solve (`--quasistatic`).
    pub field_solve: Vec<String>,
    /// Shapes drawn inside named cells, for the deck's `in_cells` rules (sky130
    /// x.25 allows the `pnp` marker only in the foundry's PNP cells). Ranges
    /// index the checked `shapes`; empty, no shape is in a cell.
    pub cells: Vec<analog::base::NamedCell>,
}

/// What a signoff did not check. Never a pass: a certificate requires
/// `unverified` empty, and every skipped rule is listed with its reason.
#[derive(Clone, Debug, Default)]
pub struct Coverage {
    /// Schematic devices LVS did not compare: `(kind, model hint, count)`.
    pub unverified: Vec<(RefKind, Option<String>, usize)>,
    /// Rules this run did not execute, `(rule, reason)`: deck skips, waivers, chip-level deferrals.
    pub skipped_rules: Vec<(String, String)>,
    /// `(ran, in deck)` over the EM (`EM*`) and `ir_drop` rules; ran =
    /// outcome `ran` with `examined > 0` (a rule that ran over no node
    /// checked nothing).
    pub em_ir: (u32, u32),
    /// The same `(ran, in deck)` over `ir_drop` alone: `em_ir` cannot tell
    /// a grid drop check from the EM rules an operating point arms.
    pub ir: (u32, u32),
}

/// GPurify's layout-only range checks, recorded `skipped:not_in_deck`
/// because the deck carries no limits; the reference parameter comparison
/// runs under `lvs.parameter_mismatch` for MOS W/L only, so these skips do
/// not mean LVS ignored W/L. Other kinds' values are the [`NON_MOS_VALUES`] row.
const RANGE_ONLY: [&str; 3] = ["lvs.device_count_mos", "lvs.device_count_bjt", "lvs.parametric"];

impl std::fmt::Display for Coverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (kind, model, n) in &self.unverified {
            writeln!(f, "  LVS unverified: {n} × {kind:?} {}", model.as_deref().unwrap_or("(no model)"))?;
        }
        if !self.skipped_rules.is_empty() {
            writeln!(f, "  rules not run ({}):", self.skipped_rules.len())?;
        }
        for (rule, why) in &self.skipped_rules {
            let note = if RANGE_ONLY.contains(&rule.as_str()) {
                " (range limits; MOS W/L are compared as lvs.parameter_mismatch, other values are not)"
            } else {
                ""
            };
            writeln!(f, "    {rule}: {why}{note}")?;
        }
        Ok(())
    }
}

/// Full signoff over drawn geometry, its pin labels and its schematic
/// reference: [`signoff_checked`] without intent or warnings; the
/// [`Duration`] is wall time. Coverage is kept only as the report's
/// `lvs-coverage/…` hard rows.
#[must_use]
pub fn signoff(shapes: &[Shape], pins: &[LabeledPin], reference: &RefInput, pdk: &Pdk) -> (Report, Duration) {
    let (report, t, _) = signoff_with_caps(shapes, pins, reference, pdk);
    (report, t)
}

/// Design intent signoff hands the deck's intent-gated rules (EM, IR drop):
/// supply nets and the current each carries. Default = none (those rules skip).
#[derive(Clone, Debug, Default)]
pub struct Intent {
    /// `(net name, nominal voltage mV, is ground)`.
    pub supplies: Vec<(String, f64, bool)>,
    /// `(net name, DC current µA)` from the operating point.
    pub currents: Vec<(String, f64)>,
    /// `(net name, allowed DC drop mV)`, from the op headroom
    /// (`annotator::ir`): what arms `ir_drop`.
    pub max_drop_mv: Vec<(String, f64)>,
}

/// Extracted capacitance between labelled nets, fF: `(net, None, C)` to
/// ground, `(a, Some(b), C)` coupling (see [`gpurify::caps_of_spice`]).
pub type CapMatrix = Vec<(String, Option<String>, f64)>;

/// [`signoff`], plus the extracted [`CapMatrix`] (empty when PEX did not run).
#[must_use]
pub fn signoff_with_caps(shapes: &[Shape], pins: &[LabeledPin], reference: &RefInput, pdk: &Pdk) -> (Report, Duration, CapMatrix) {
    signoff_with_intent(shapes, pins, reference, &Intent::default(), pdk)
}

/// [`signoff_checked`]'s errors, wall time and caps; warnings are dropped,
/// coverage is kept as the report's `lvs-coverage/…` hard rows.
#[must_use]
pub fn signoff_with_intent(shapes: &[Shape], pins: &[LabeledPin], reference: &RefInput, intent: &Intent, pdk: &Pdk) -> (Report, Duration, CapMatrix) {
    let s = signoff_checked(shapes, pins, reference, intent, pdk);
    (s.report, s.elapsed, s.caps)
}

/// Full signoff with design [`Intent`] (so the deck's EM/IR rules run on the
/// operating-point currents instead of skipping).
///
/// A deck error row becomes a hard [`Violation`] named
/// `{domain}/{rule}:{layer}` with its [`shortfall`] as margin (`-` for a row
/// with no layer name); a deck warning row, named the same way, goes to
/// [`Signoff::warnings`] only. A stage GPurify skipped or refused, or a
/// GPurify failure, is a hard `engine/…` violation: a check that could not
/// run never reads as clean. Rules skipped inside a stage that ran, and
/// schematic devices no deck recogniser extracts, are this run's
/// [`Coverage`]; each `(kind, model, n)` of the latter is also a hard
/// `lvs-coverage/unverified:{kind}:{model}` row of margin `n`.
#[must_use]
pub fn signoff_checked(shapes: &[Shape], pins: &[LabeledPin], reference: &RefInput, intent: &Intent, pdk: &Pdk) -> Signoff {
    signoff_extract(shapes, pins, reference, intent, &ExtractOptions::default(), pdk)
}

/// [`signoff_checked`] with extraction options. With `field_solve` set, the
/// analytical PEX netlist is followed by a timed one that field-solves those
/// nets, and its caps and `report.cost` replace the analytical ones
/// ([`Signoff::pex_field`]). A refused field solve keeps the analytical caps
/// and is a `("pex.field", why)` row of [`Coverage::skipped_rules`], never a
/// hard row: the analytical PEX ran.
#[must_use]
pub fn signoff_extract(shapes: &[Shape], pins: &[LabeledPin], reference: &RefInput, intent: &Intent, opts: &ExtractOptions, pdk: &Pdk) -> Signoff {
    let t0 = Instant::now();
    let mut s = Signoff::default();
    let fail = |s: &mut Signoff, rule: String| s.report.hard_violations.push(Violation { rule, margin: 0 });

    let (text, skipped) = match reference::spice(reference, &pdk.deck, &pdk.strings) {
        Ok(r) => r,
        Err(e) => {
            fail(&mut s, format!("engine/reference: {e}"));
            s.elapsed = t0.elapsed();
            return s;
        }
    };
    for i in skipped {
        let (kind, model) = (reference.devices[i].kind, reference.devices[i].model.clone());
        match s.coverage.unverified.iter_mut().find(|u| u.0 == kind && u.1 == model) {
            Some(u) => u.2 += 1,
            None => s.coverage.unverified.push((kind, model, 1)),
        }
    }
    // Unknown is never pass: an uncompared device is a hard row (the same
    // count every epoch, so ranking is unchanged) that keeps the run from
    // reading LVS-clean.
    for (kind, model, n) in &s.coverage.unverified {
        let rule = format!("lvs-coverage/unverified:{kind:?}:{}", model.as_deref().unwrap_or("-"));
        s.report.hard_violations.push(Violation { rule, margin: *n as i64 });
    }
    let input = gpurify::Input {
        shapes,
        pins,
        cells: &opts.cells,
        reference: Some(&text),
        ports: reference.external_ports.as_deref(),
        intent,
        field_solve: &opts.field_solve,
    };
    match gpurify::run(&input, pdk, Checks::ALL) {
        Err(e) => fail(&mut s, format!("engine/run: {e}")),
        Ok(mut out) => {
            let deferred = deferred_rules(pdk, shapes);
            out.rows.retain(|r| !deferred.iter().any(|d| d.0 == r.rule));
            out.runs.retain(|r| !deferred.iter().any(|d| d.0 == r.rule));
            harvest(&out, &mut s);
            s.coverage.skipped_rules.extend(deferred);
            if let Some(why) = out.caps_refused {
                s.coverage.skipped_rules.push(("pex.netlist".to_string(), why));
            }
            if let Some(why) = out.field_refused {
                s.coverage.skipped_rules.push(("pex.field".to_string(), why));
            }
            s.caps = out.caps;
            s.report.cost = out.total_cap_ff as f32;
            s.pex_field = out.pex_field;
        }
    }
    // Only MOS cards carry params: a compared R/D/C/BJT matches by
    // connectivity alone, its value (rc_filter's R) unchecked. Listed, so a
    // MATCH never reads as value-checked.
    let valueless = |k: RefKind| !matches!(k, RefKind::Nmos | RefKind::Pmos);
    let cards = reference.devices.iter().filter(|d| valueless(d.kind)).count();
    let uncompared: usize = s.coverage.unverified.iter().filter(|u| valueless(u.0)).map(|u| u.2).sum();
    if let n @ 1.. = cards.saturating_sub(uncompared) {
        s.coverage.skipped_rules.push((NON_MOS_VALUES.to_string(), format!("NotCompared({n} R/D/C/BJT cards matched by connectivity only)")));
    }
    s.elapsed = t0.elapsed();
    s
}

/// The [`Coverage::skipped_rules`] row for compared devices whose value LVS
/// does not compare (every kind but MOS).
pub const NON_MOS_VALUES: &str = "lvs.parameter_mismatch(non-MOS values)";

/// One located finding, for callers that want rows rather than a Report.
#[derive(Clone, Debug)]
pub struct Finding {
    /// The deck's rule id, or `engine/…` for a run that could not conclude.
    pub rule: String,
    /// Layer name, `"-"` for a finding with no layer name.
    pub layer: String,
    /// [`shortfall`], in [`Finding::unit`].
    pub margin: i64,
    /// `"nm"` for a length rule, `"permille"` (of the limit) for any other —
    /// including the placeholder 1 of mismatched dimensions and `engine/…`.
    pub unit: &'static str,
    pub x: i64,
    pub y: i64,
    /// A row the deck states as a warning: [`Signoff::warnings`] in a full
    /// signoff, so never a DRC/ERC violation count.
    pub warning: bool,
}

/// Standalone DRC: one `gpurify drc` run; a run that could not conclude is
/// a single fail-closed `engine/…` finding.
#[must_use]
pub fn drc(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk) -> Vec<Finding> {
    drc_with(shapes, pins, pdk, &ExtractOptions::default())
}

/// [`drc`] with the named cells of `opts`.
#[must_use]
pub fn drc_with(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk, opts: &ExtractOptions) -> Vec<Finding> {
    standalone(shapes, pins, pdk, Checks { drc: true, erc: false, lvs: false, pex: false }, opts)
}

/// Standalone ERC, as [`drc`].
#[must_use]
pub fn erc(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk) -> Vec<Finding> {
    erc_with(shapes, pins, pdk, &ExtractOptions::default())
}

/// [`erc`] with the named cells of `opts`, as [`drc_with`].
#[must_use]
pub fn erc_with(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk, opts: &ExtractOptions) -> Vec<Finding> {
    standalone(shapes, pins, pdk, Checks { drc: false, erc: true, lvs: false, pex: false }, opts)
}

// ---- private ----

/// Errors to the hard tier, warnings to [`Signoff::warnings`]; stage
/// denials as `engine/…`; non-`ran` rule records and the EM/IR counts to
/// the coverage.
fn harvest(out: &gpurify::Output, s: &mut Signoff) {
    for r in &out.rows {
        let v = Violation { rule: format!("{}/{}:{}", r.domain, r.rule, r.layer.as_deref().unwrap_or("-")), margin: shortfall(r.limit, r.measured) };
        if r.warning {
            s.warnings.push(v);
        } else {
            s.report.hard_violations.push(v);
        }
    }
    for (stage, why) in &out.denied {
        s.report.hard_violations.push(Violation { rule: format!("engine/{stage}: {why}"), margin: 0 });
    }
    for r in &out.runs {
        if r.outcome != "ran" {
            s.coverage.skipped_rules.push((r.rule.clone(), r.outcome.clone()));
        }
        // ponytail: labels_are_ports exempts every labelled net (Q61), until
        // GPurify reads the intent's port list.
        if r.rule == "floating_gate" {
            s.coverage.skipped_rules.push(("floating_gate".to_string(), "exempt on every labelled net: labels_are_ports, no port list".to_string()));
        }
        let ran = u32::from(r.outcome == "ran" && r.examined > 0);
        if r.rule.starts_with("EM") || r.rule == "ir_drop" {
            s.coverage.em_ir.1 += 1;
            s.coverage.em_ir.0 += ran;
        }
        if r.rule == "ir_drop" {
            s.coverage.ir.1 += 1;
            s.coverage.ir.0 += ran;
        }
    }
}

fn standalone(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk, checks: Checks, opts: &ExtractOptions) -> Vec<Finding> {
    let engine_fail = |rule: String| Finding { rule, layer: "-".into(), margin: 1, unit: "permille", x: 0, y: 0, warning: false };
    let input = gpurify::Input { shapes, pins, cells: &opts.cells, reference: None, ports: None, intent: &Intent::default(), field_solve: &[] };
    let out = match gpurify::run(&input, pdk, checks) {
        Ok(o) => o,
        Err(e) => return vec![engine_fail(format!("engine/run: {e}"))],
    };
    let deferred = deferred_rules(pdk, shapes);
    let mut findings: Vec<Finding> = out
        .rows
        .into_iter()
        .filter(|r| !deferred.iter().any(|d| d.0 == r.rule))
        .map(|r| Finding {
            unit: match (r.limit, r.measured) {
                (Measurement::Length(_), Measurement::Length(_)) => "nm",
                _ => "permille",
            },
            margin: shortfall(r.limit, r.measured),
            rule: r.rule,
            layer: r.layer.unwrap_or_else(|| "-".into()),
            x: r.x.round() as i64,
            y: r.y.round() as i64,
            warning: r.warning,
        })
        .collect();
    findings.extend(out.denied.into_iter().map(|(stage, why)| engine_fail(format!("engine/{stage}: {why}"))));
    findings
}

/// The rules this block does not answer for, `(rule, why)`: the deck's
/// waivers (`cell.waivers`), `global_density` (the whole die), and every
/// `density` (one square `window`) or `density_cmp` (`window_x` ×
/// `window_y`, sky130's `m*.density`) rule whose window is wider than the
/// shapes' bounding box — a window wider than the block measures the chip
/// around it, so that check is chip integration's. Their rows are dropped
/// and they are reported as not run, never as passed.
fn deferred_rules(pdk: &Pdk, shapes: &[Shape]) -> Vec<(String, String)> {
    use gdsverify::ingest::deck::{ParamValue, RuleSpec};
    let (st, rules) = (&pdk.strings, &pdk.deck.rules);
    let mut out = Vec::new();
    if let Some(w) = pdk.cell.get("waivers").and_then(|w| w.as_object()) {
        for (k, why) in w {
            if st.get(k).is_some_and(|id| rules.spec.iter().any(|s| s.id == id)) {
                out.push((k.clone(), format!("Waived({})", why.as_str().unwrap_or(""))));
            }
        }
    }
    let (x0, y0, x1, y1) = shapes.iter().fold((i64::MAX, i64::MAX, i64::MIN, i64::MIN), |(a, b, c, d), s| {
        let r = s.rect;
        (a.min(i64::from(r.x)), b.min(i64::from(r.y)), c.max(i64::from(r.x + r.w)), d.max(i64::from(r.y + r.h)))
    });
    let (w, h) = if shapes.is_empty() { (0, 0) } else { (x1 - x0, y1 - y0) };
    let (density, cmp, global) = (st.get("density"), st.get("density_cmp"), st.get("global_density"));
    let len = |s: &RuleSpec, k: &str| match st.get(k).and_then(|k| rules.param(s, k)) {
        Some(ParamValue::Length(d)) => Some(d.raw()),
        _ => None,
    };
    for s in &rules.spec {
        let name = st.resolve(s.id).to_string();
        if Some(s.kind) == global {
            out.push((name, "ChipLevel(global density: whole die)".to_string()));
            continue;
        }
        let window = match Some(s.kind) {
            k if k == density => len(s, "window").map(|d| (d, d)),
            k if k == cmp => len(s, "window_x").zip(len(s, "window_y")),
            _ => None,
        };
        if let Some((wx, wy)) = window.filter(|&(wx, wy)| w > 0 && h > 0 && (wx > w || wy > h)) {
            let win = if wx == wy { format!("{wx}") } else { format!("{wx}x{wy}") };
            out.push((name, format!("ChipLevel(window {win} nm > block {w}x{h} nm)")));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use analog::base::{Process, Rect};

    fn sky130() -> Pdk {
        Pdk::builtin("sky130").unwrap()
    }

    fn rect(pdk: &Pdk, layer: &str, x: i32, y: i32, w: i32, h: i32) -> Shape {
        let layer = pdk.layer(layer).unwrap_or_else(|| panic!("no layer {layer}"));
        Shape { layer, rect: Rect { x, y, w, h } }
    }

    /// sky130 x.25: a `pnp` marker is legal only inside a named foundry PNP
    /// cell. Flat, or in a cell of another name, it fires; inside the open
    /// PDK's W3p40L3p40 cell it does not. The cell path loads the same store
    /// otherwise: the met1 width finding and its location are unchanged.
    #[test]
    fn x25_reads_the_named_cell() {
        let pdk = sky130();
        let shapes = [rect(&pdk, "pnp", 0, 0, 5_000, 5_000), rect(&pdk, "met1", 10_000, 0, 100, 2_000)];
        let run = |cells: &[(&str, std::ops::Range<u32>)]| {
            let cells = cells.iter().map(|(n, r)| analog::base::NamedCell { name: (*n).into(), shapes: r.clone() }).collect();
            let mut f: Vec<(String, i64, i64)> = drc_with(&shapes, &[], &pdk, &ExtractOptions { cells, ..Default::default() })
                .into_iter()
                .filter(|f| !f.rule.ends_with("density"))
                .map(|f| (f.rule, f.x, f.y))
                .collect();
            f.sort();
            f
        };
        let flat = run(&[]);
        assert!(flat.iter().any(|f| f.0 == "x.25"), "{flat:?}");
        assert_eq!(run(&[("not_a_pnp_cell", 0..1)]), flat);
        let named = run(&[("sky130_fd_pr__pnp_05v5_W3p40L3p40", 0..1)]);
        let rest: Vec<_> = flat.into_iter().filter(|f| f.0 != "x.25").collect();
        assert_eq!(named, rest);
        assert!(rest.iter().any(|f| f.0.starts_with("m1.")), "{rest:?}");
    }

    /// Only the rows of `checks` over `shapes`, by rule id, with LVS
    /// against `reference` when it is given.
    fn rules(shapes: &[Shape], pins: &[LabeledPin], reference: Option<&RefInput>, intent: &Intent, checks: Checks, pdk: &Pdk) -> gpurify::Output {
        let text = reference.map(|r| reference::spice(r, &pdk.deck, &pdk.strings).unwrap().0);
        let input = gpurify::Input { shapes, pins, cells: &[], reference: text.as_deref(), ports: None, intent, field_solve: &[] };
        gpurify::run(&input, pdk, checks).unwrap()
    }

    fn lvs_rows(shapes: &[Shape], reference: &RefInput, pdk: &Pdk) -> Vec<String> {
        let out = rules(shapes, &[], Some(reference), &Intent::default(), Checks { drc: false, erc: false, lvs: true, pex: false }, pdk);
        assert!(out.denied.is_empty(), "{:?}", out.denied);
        out.rows.into_iter().map(|r| r.rule).filter(|r| r.starts_with("lvs.")).collect()
    }

    /// The minimal recognisable sky130 nfet: channel = poly AND diff AND
    /// nsdm, x 200..300, y 100..300 (W = 200 nm, L = 100 nm).
    fn nfet(pdk: &Pdk) -> [Shape; 4] {
        [
            rect(pdk, "poly", 200, 0, 100, 400),
            rect(pdk, "diff", 0, 100, 260, 200),
            rect(pdk, "diff", 240, 100, 260, 200),
            rect(pdk, "nsdm", 0, 0, 500, 400),
        ]
    }

    /// Metal reaching no device is floating — unless a label makes it a
    /// port, which leaves the cell (the bulk-only VSS rail case).
    ///
    /// Oracle: sky130 x.22 ([S1] unconnected conductor). Known failure:
    /// GPurify exempts no port from `unconnected_pin` until its intent file
    /// reads `ports` (Q61); the labelled rail still fires. When this
    /// assertion fails, the gap is closed: assert 0 and turn
    /// `gpurify::INTENT_PORTS` on.
    #[test]
    fn unconnected_metal_is_floating_unless_it_is_a_port() {
        let pdk = sky130();
        let shapes = [rect(&pdk, "met1", 0, 0, 1000, 1000)];
        let floating = |pins: &[LabeledPin]| erc(&shapes, pins, &pdk).iter().filter(|f| f.rule == "x.22").count();
        assert_eq!(floating(&[]), 1);
        let pin = LabeledPin { name: "VSS".into(), layer: pdk.layer("met1").unwrap().0, x: 500, y: 500 };
        assert_eq!(floating(&[pin]), 1, "known failure: no port exemption in GPurify yet (Q61)");
    }

    /// AV-04: a gate-only net reaching no other device is floating; a label
    /// exempts it (`labels_are_ports: true`, the deck's `# PHILIS:` edit), and
    /// the coverage says the exemption is blanket.
    ///
    /// Oracle: GPurify's `floating_gate` (docs/deck.md, ERC rule kinds).
    /// Known failure: an internal labelled net is exempt too, until GPurify
    /// reads the intent's port list (Q61).
    #[test]
    fn an_internal_gate_only_net_is_floating_even_when_labelled() {
        let pdk = sky130();
        let shapes = nfet(&pdk);
        let pins = [LabeledPin { name: "g".into(), layer: pdk.layer("poly").unwrap().0, x: 250, y: 350 }];
        let erc_only = Checks { drc: false, erc: true, lvs: false, pex: false };
        let floating = |pins: &[LabeledPin]| {
            rules(&shapes, pins, None, &Intent::default(), erc_only, &pdk).rows.iter().filter(|r| r.rule == "floating_gate").count()
        };
        assert_eq!(floating(&[]), 1, "an unlabelled gate-only net is floating");
        assert_eq!(floating(&pins), 0, "known failure: a labelled internal net is exempt (Q61)");
        let s = signoff_checked(&shapes, &pins, &RefInput::default(), &Intent::default(), &pdk);
        assert!(
            s.coverage.skipped_rules.iter().any(|(r, why)| r == "floating_gate" && why.contains("no port list")),
            "{:?}",
            s.coverage.skipped_rules
        );
    }

    /// The one margin arithmetic every consumer shares: nm for a length
    /// pair, either sign; 1 for a pair of different dimensions.
    ///
    /// Oracle: hand arithmetic, 170 − 100 = 70.
    #[test]
    fn shortfall_is_nm_for_lengths() {
        use Measurement::{Length, Ratio};
        assert_eq!(shortfall(Length(170.0), Length(100.0)), 70);
        assert_eq!(shortfall(Length(100.0), Length(170.0)), 70, "a maximum's overshoot is no graze");
        assert_eq!(shortfall(Length(170.0), Ratio(0.5)), 1);
    }

    /// AV-31: a non-length margin is ‰ of the limit, so an antenna ratio 2×
    /// over outranks one 1 % over.
    ///
    /// Oracle: hand arithmetic, 1000·|800 − 400|/400 = 1000.
    #[test]
    fn shortfall_reads_ratio_overshoot_in_permille() {
        use Measurement::{Count, Ratio};
        assert_eq!(shortfall(Ratio(400.0), Ratio(800.0)), 1000);
        assert_eq!(shortfall(Ratio(400.0), Ratio(404.0)), 10);
        assert_eq!(shortfall(Count(1.0), Count(0.0)), 1000);
        assert_eq!(shortfall(Ratio(400.0), Ratio(f64::NAN)), i64::MAX, "NaN is not a graze");
    }

    /// Warnings never reach the hard tier; a stage GPurify denied does.
    ///
    /// Oracle: the `Signoff` contract (errors in `report`, warnings apart).
    #[test]
    fn harvest_keeps_warnings_out_of_the_hard_tier() {
        let row = |rule: &str, warning| gpurify::Row {
            domain: "drc",
            rule: rule.into(),
            layer: Some("met1".into()),
            warning,
            x: 0.0,
            y: 0.0,
            measured: Measurement::Length(100.0),
            limit: Measurement::Length(140.0),
        };
        let out = gpurify::Output {
            rows: vec![row("a", false), row("b", true), row("c", false)],
            denied: vec![("lvs", "refused: x".into())],
            ..Default::default()
        };
        let mut s = Signoff::default();
        harvest(&out, &mut s);
        let hard: Vec<&str> = s.report.hard_violations.iter().map(|v| v.rule.as_str()).collect();
        assert_eq!(hard, ["drc/a:met1", "drc/c:met1", "engine/lvs: refused: x"]);
        assert_eq!(s.warnings.iter().map(|v| v.rule.as_str()).collect::<Vec<_>>(), ["drc/b:met1"]);
        assert_eq!(s.report.hard_violations[0].margin, 40);
    }

    /// AV-26: the skip set is returned by every run, not logged once per
    /// process.
    ///
    /// Oracle: sky130's `ir_drop` needs intent this run does not give
    /// (docs/usage.md, "Design intent").
    #[test]
    fn skipped_rules_are_reported_on_every_run() {
        let pdk = sky130();
        for w in [500, 900] {
            let s = signoff_checked(&[rect(&pdk, "met1", 0, 0, w, w)], &[], &RefInput::default(), &Intent::default(), &pdk);
            assert!(
                s.coverage.skipped_rules.iter().any(|(r, why)| r == "ir_drop" && why.contains("no_design_intent")),
                "{:?}",
                s.coverage.skipped_rules
            );
        }
    }

    /// Ground C (fF) of `net` in `caps`.
    fn ground(caps: &CapMatrix, net: &str) -> f64 {
        caps.iter().find(|(n, o, _)| n == net && o.is_none()).unwrap_or_else(|| panic!("no ground row for {net}: {caps:?}")).2
    }

    fn pin(pdk: &Pdk, name: &str, x: i32, y: i32) -> LabeledPin {
        LabeledPin { name: name.into(), layer: pdk.layer("met1").unwrap().0, x, y }
    }

    fn extract(shapes: &[Shape], pins: &[LabeledPin], opts: &ExtractOptions, pdk: &Pdk) -> Signoff {
        signoff_extract(shapes, pins, &RefInput::default(), &Intent::default(), opts, pdk)
    }

    /// PERF-16: two overlapping met1 rects extract as their union.
    ///
    /// Oracle: the deck's met1 row (pdks/gpurify/sky130.deck: 25.78 aF/µm²,
    /// 40.57 aF/µm). Union 7.5 µm², perimeter 26 µm → 1248.2 aF; the drawn
    /// rects would read 8 µm², 29 µm → 1382.8 aF.
    #[test]
    fn partial_overlaps_extract_as_their_union() {
        let pdk = sky130();
        let shapes = [rect(&pdk, "met1", 0, 0, 10_000, 500), rect(&pdk, "met1", 9000, -250, 3000, 1000)];
        let pins = [pin(&pdk, "a", 500, 250)];
        let merged = ground(&signoff_checked(&shapes, &pins, &RefInput::default(), &Intent::default(), &pdk).caps, "a");
        assert!((merged - 1.2482).abs() / 1.2482 <= 1e-3, "merged C(a) = {merged} fF, want 1.2482");
    }

    fn three_lines(pdk: &Pdk) -> ([Shape; 3], [LabeledPin; 3]) {
        let line = |y| rect(pdk, "met1", 0, y, 10_000, 500);
        ([line(0), line(1000), line(2000)], [pin(pdk, "a", 500, 250), pin(pdk, "b", 500, 1250), pin(pdk, "c", 500, 2250)])
    }

    /// Oracle: a field solve of net `a` moves C(a) and leaves C(c), which it
    /// does not solve, where the closed form put it.
    #[test]
    #[ignore = "nightly: a full field solve"]
    fn field_solve_changes_only_selected_nets() {
        let pdk = sky130();
        let (shapes, pins) = three_lines(&pdk);
        let analytic = extract(&shapes, &pins, &ExtractOptions::default(), &pdk);
        let field = extract(&shapes, &pins, &ExtractOptions { field_solve: vec!["a".into()], cells: Vec::new() }, &pdk);
        assert!(field.coverage.skipped_rules.iter().all(|(r, _)| r != "pex.field"), "{:?}", field.coverage.skipped_rules);
        let (a0, a1) = (ground(&analytic.caps, "a"), ground(&field.caps, "a"));
        assert!((a1 - a0).abs() / a0 > 1e-3, "field C(a) {a1} vs analytical {a0}");
        let (c0, c1) = (ground(&analytic.caps, "c"), ground(&field.caps, "c"));
        assert!((c1 - c0).abs() <= 1e-9, "C(c) moved: {c0} -> {c1}");
    }

    /// Oracle: a net no label names cannot be field-solved; the analytical
    /// PEX ran, so that is coverage, never an `engine/pex` row.
    #[test]
    fn a_refused_field_solve_falls_back_with_coverage() {
        let pdk = sky130();
        let (shapes, pins) = three_lines(&pdk);
        let s = extract(&shapes, &pins, &ExtractOptions { field_solve: vec!["nosuchnet".into()], cells: Vec::new() }, &pdk);
        let rules: Vec<&str> = s.report.hard_violations.iter().map(|v| v.rule.as_str()).collect();
        assert!(!rules.iter().any(|r| r.starts_with("engine/pex")), "{rules:?}");
        assert!(s.coverage.skipped_rules.iter().any(|(r, _)| r == "pex.field"), "{:?}", s.coverage.skipped_rules);
        assert!(!s.caps.is_empty());
    }

    /// AV-25: a label short is LVS's verdict, and PEX still extracts
    /// capacitance over the labels left once one per net remains.
    ///
    /// Oracle: two labels on one met1 plate are one net carrying two names
    /// (GPurify `lvs.net_seed_conflict`).
    #[test]
    fn the_label_short_fallback_still_returns_caps() {
        let pdk = sky130();
        let pins = [pin(&pdk, "A", 500, 500), pin(&pdk, "B", 1500, 500)];
        let reference = RefInput { devices: vec![], ports: vec!["A".into(), "B".into()], external_ports: None };
        let s = signoff_checked(&[rect(&pdk, "met1", 0, 0, 2000, 1000)], &pins, &reference, &Intent::default(), &pdk);
        let rules: Vec<&str> = s.report.hard_violations.iter().map(|v| v.rule.as_str()).collect();
        assert!(rules.iter().any(|r| r.starts_with("lvs/lvs.net_seed_conflict")), "{rules:?}");
        assert!(!s.caps.is_empty(), "no cap matrix: {:?}", s.coverage.skipped_rules);
    }

    /// A deliberately-too-narrow li wire trips li's minimum width with the
    /// right nm margin.
    ///
    /// Oracle: sky130 li.1, 170 nm ([S1]); 100 nm wide is 70 short. Long
    /// enough to clear li's minimum area and on the 5 nm grid.
    #[test]
    fn drc_reports_a_min_width_shortfall_in_nm() {
        let pdk = sky130();
        let f = drc(&[rect(&pdk, "li", 0, 0, 100, 1000)], &[], &pdk);
        let hit = f.iter().find(|f| f.rule == "li.1").unwrap_or_else(|| panic!("no li.1 row: {f:?}"));
        assert_eq!((hit.margin, hit.unit), (70, "nm"));
    }

    /// The deck's EM rules skip for want of design intent; with the supplies
    /// and their currents given they no longer do.
    ///
    /// Oracle: ihp's EM.Metal* rules need intent (docs/usage.md, "Design
    /// intent").
    #[test]
    fn intent_arms_the_em_rules() {
        let pdk = Pdk::builtin("ihp_sg13g2").unwrap();
        let shapes = [rect(&pdk, "met1", 0, 0, 20_000, 1_000), rect(&pdk, "met1", 0, 5_000, 20_000, 1_000)];
        let met1 = pdk.layer("met1").unwrap().0;
        let pins = [LabeledPin { name: "VDD".into(), layer: met1, x: 500, y: 500 }, LabeledPin { name: "VSS".into(), layer: met1, x: 500, y: 5_500 }];
        let erc_only = Checks { drc: false, erc: true, lvs: false, pex: false };
        let no_intent = |intent: &Intent| {
            rules(&shapes, &pins, None, intent, erc_only, &pdk)
                .runs
                .iter()
                .filter(|r| r.rule.starts_with("EM") && r.outcome == "skipped:no_design_intent")
                .count()
        };
        assert!(no_intent(&Intent::default()) > 0, "the EM rules need intent");
        let intent = Intent {
            supplies: vec![("VDD".into(), 1_800.0, false), ("VSS".into(), 1_800.0, true)],
            currents: vec![("VDD".into(), 100.0), ("VSS".into(), 100.0)],
            ..Default::default()
        };
        assert_eq!(no_intent(&intent), 0);
    }

    /// PERF-17: supplies alone leave `ir_drop` running over no node (no net
    /// states a drop limit); a supply's `max_drop_mv` makes it examine the
    /// grid.
    ///
    /// Oracle: GPurify's `ir_drop` examines the nets the intent gives a
    /// limit (docs/usage.md, "Design intent").
    #[test]
    fn max_drop_arms_ir_drop() {
        let pdk = sky130();
        let shapes = [rect(&pdk, "met1", 0, 0, 20_000, 1_000)];
        let pins = [pin(&pdk, "VDD", 500, 500)];
        let erc_only = Checks { drc: false, erc: true, lvs: false, pex: false };
        let ir = |intent: &Intent| {
            rules(&shapes, &pins, None, intent, erc_only, &pdk).runs.into_iter().find(|r| r.rule == "ir_drop").map(|r| (r.outcome, r.examined))
        };
        assert_eq!(
            ir(&Intent { supplies: vec![("VDD".into(), 1_800.0, false)], ..Default::default() }).map(|r| r.1),
            Some(0),
            "supplies alone examine nothing"
        );
        let intent = Intent { supplies: vec![("VDD".into(), 1_800.0, false)], max_drop_mv: vec![("VDD".into(), 10.0)], ..Default::default() };
        let (outcome, examined) = ir(&intent).expect("ir_drop in the deck");
        assert!(outcome == "ran" && examined > 0, "{outcome}, examined {examined}");
    }

    /// A density window wider than the block is chip-level: its rows are
    /// dropped and it is reported as not run; a block wider than the window
    /// keeps the rule.
    ///
    /// Oracle: ihp AFil.g2, activ density >= 25 % in 800 µm windows (the
    /// deck); sky130 m*.density, `density_cmp` in 700 µm windows.
    #[test]
    fn density_wider_than_the_block_is_chip_level() {
        let pdk = Pdk::builtin("ihp_sg13g2").unwrap();
        let block = |w| [rect(&pdk, "met1", 0, 0, w, w)];
        let small = deferred_rules(&pdk, &block(34_000));
        assert!(small.iter().any(|(n, why)| n == "AFil.g2" && why.starts_with("ChipLevel(window 800000 nm")), "{small:?}");
        assert!(!deferred_rules(&pdk, &block(2_000_000)).iter().any(|(n, _)| n == "AFil.g2"));
        let sky = sky130();
        let shapes = [rect(&sky, "met1", 0, 0, 10_000, 10_000)];
        let s = signoff_checked(&shapes, &[], &RefInput::default(), &Intent::default(), &sky);
        for n in ["m1.density", "m2.density", "m3.density", "m4.density"] {
            assert!(
                s.coverage.skipped_rules.iter().any(|(r, why)| r == n && why.starts_with("ChipLevel(window 700000 nm > block 10000x10000 nm)")),
                "{n} not chip-level: {:?}",
                s.coverage.skipped_rules
            );
            assert!(!s.report.hard_violations.iter().any(|v| v.rule.contains(n)), "{n} still reported");
        }
    }

    /// A fat legal rect is clean — and provably checked clean: rules ran.
    /// Guards the false-clean failure (an engine that silently checked
    /// nothing).
    ///
    /// Oracle: sky130 li rules ([S1]); a 500 × 500 nm li plate meets them.
    #[test]
    fn drc_runs_clean_on_legal_geometry_and_proves_it_ran() {
        let pdk = sky130();
        let out =
            rules(&[rect(&pdk, "li", 0, 0, 500, 500)], &[], None, &Intent::default(), Checks { drc: true, erc: false, lvs: false, pex: false }, &pdk);
        assert!(out.denied.is_empty(), "{:?}", out.denied);
        let real: Vec<_> = out.rows.iter().filter(|r| !r.rule.contains("density")).collect();
        assert!(real.is_empty(), "{real:?}");
        assert!(out.runs.iter().any(|r| r.outcome == "ran" && r.examined > 0), "zero findings must come from rules that ran");
    }

    /// The reference writer: two NMOS on the sky130 deck → two `X` cards of
    /// the deck's model, four card-order terminals each (bulk = psub for an
    /// nfet), W and L in µm with a unit; an inductor is skipped, not written.
    ///
    /// Oracle: GPurify's reference reader (docs/deck.md, `netlist_scale`: a
    /// suffixed value is absolute).
    #[test]
    fn reference_spice_writes_x_cards_of_the_deck_model() {
        let pdk = sky130();
        let dev = |terminals: [&str; 4], params: Vec<(String, f64)>| RefDeviceIn {
            kind: RefKind::Nmos,
            model: None,
            terminals: terminals.iter().map(|&t| t.into()).collect(),
            params,
        };
        let input = RefInput {
            devices: vec![
                dev(["out", "in", "vss", "vss"], vec![("w".into(), 2e-6), ("l".into(), 5e-7)]),
                RefDeviceIn { kind: RefKind::Inductor, model: None, terminals: vec!["a".into(), "b".into()], params: vec![] },
                dev(["vss", "bias", "out", "vss"], vec![("w".into(), 1e-6), ("nf".into(), 2.0)]),
            ],
            ports: vec!["in".into(), "out".into(), "vss".into()],
            external_ports: None,
        };
        let (text, skipped) = reference::spice(&input, &pdk.deck, &pdk.strings).unwrap();
        assert_eq!(skipped, [1]);
        assert_eq!(
            text,
            ".subckt top in out vss\nX0 out in vss vss sky130_fd_pr__nfet_01v8 w=2u l=0.5u\nX2 vss bias out vss sky130_fd_pr__nfet_01v8 w=1u nf=2\n.ends top\n"
        );
    }

    /// AV-01/NOTES-02: a schematic device no recogniser extracts is named in
    /// the coverage, not left on stderr.
    ///
    /// Oracle: sky130 has no VPP capacitor recogniser (the deck's devices).
    #[test]
    fn a_skipped_reference_device_is_listed_in_coverage() {
        let pdk = sky130();
        let dev =
            |kind, terminals: &[&str]| RefDeviceIn { kind, model: None, terminals: terminals.iter().map(|&t| t.into()).collect(), params: vec![] };
        let reference = RefInput {
            devices: vec![
                dev(RefKind::Nmos, &["out", "in", "vss", "vss"]),
                RefDeviceIn { model: Some("sky130_fd_pr__cap_vpp_02p4x04p6_m1m2_noshield".into()), ..dev(RefKind::Capacitor, &["a", "b"]) },
            ],
            ports: vec![],
            external_ports: None,
        };
        let s = signoff_checked(&[rect(&pdk, "li", 0, 0, 500, 500)], &[], &reference, &Intent::default(), &pdk);
        assert!(matches!(s.coverage.unverified.as_slice(), [(RefKind::Capacitor, _, 1)]), "{:?}", s.coverage.unverified);
    }

    /// A compared resistor's value is not checked (only MOS cards carry
    /// params), and coverage says so; an uncompared MOM is not counted there.
    ///
    /// Oracle: the reference's own cards, one R and one MOM.
    #[test]
    fn a_compared_resistor_value_is_listed_as_not_compared() {
        let pdk = sky130();
        let card =
            |kind, model: Option<&str>| RefDeviceIn { kind, model: model.map(Into::into), terminals: vec!["a".into(), "b".into()], params: vec![] };
        let reference = RefInput {
            devices: vec![card(RefKind::Resistor, None), card(RefKind::Capacitor, Some("cap_generic_m1m2"))],
            ports: vec![],
            external_ports: None,
        };
        let s = signoff_checked(&[], &[], &reference, &Intent::default(), &pdk);
        let row: Vec<_> = s.coverage.skipped_rules.iter().filter(|(r, _)| r == NON_MOS_VALUES).map(|(_, why)| why.as_str()).collect();
        assert_eq!(row, ["NotCompared(1 R/D/C/BJT cards matched by connectivity only)"], "{:?}", s.coverage.skipped_rules);
    }

    /// AV-01: sky130's `capm` recognises a MIM; a MOM card (no marker, no
    /// recogniser), named or model-less, is uncompared, not compared as
    /// `capm` and unpaired.
    ///
    /// Oracle: the deck's capacitor rows are MIM and varactor only.
    #[test]
    fn a_mom_card_is_unverified_not_compared_as_mim() {
        let pdk = sky130();
        for (model, rule) in
            [(Some("cap_generic_m1m2"), "lvs-coverage/unverified:Capacitor:cap_generic_m1m2"), (None, "lvs-coverage/unverified:Capacitor:-")]
        {
            let reference = RefInput {
                devices: vec![RefDeviceIn {
                    kind: RefKind::Capacitor,
                    model: model.map(Into::into),
                    terminals: vec!["top".into(), "bot".into()],
                    params: vec![],
                }],
                ports: vec![],
                external_ports: None,
            };
            // No geometry: nothing to extract, so any `lvs/` row is the card.
            let s = signoff_checked(&[], &[], &reference, &Intent::default(), &pdk);
            assert_eq!(s.coverage.unverified, [(RefKind::Capacitor, model.map(Into::into), 1)]);
            let rules: Vec<&str> = s.report.hard_violations.iter().map(|v| v.rule.as_str()).collect();
            assert!(!rules.iter().any(|r| r.starts_with("lvs/")), "{rules:?}");
            let cov: Vec<_> =
                s.report.hard_violations.iter().filter(|v| v.rule.starts_with("lvs-coverage/")).map(|v| (v.rule.as_str(), v.margin)).collect();
            assert_eq!(cov, [(rule, 1)]);
        }
    }

    /// No deck recognises an inductor: it is uncompared, and that is
    /// coverage, not an engine failure.
    ///
    /// Oracle: no `device inductor` row in any deck.
    #[test]
    fn an_inductor_is_unverified() {
        let pdk = sky130();
        let reference = RefInput {
            devices: vec![RefDeviceIn { kind: RefKind::Inductor, model: None, terminals: vec!["a".into(), "b".into()], params: vec![] }],
            ports: vec![],
            external_ports: None,
        };
        let s = signoff_checked(&[rect(&pdk, "li", 0, 0, 500, 500)], &[], &reference, &Intent::default(), &pdk);
        assert!(matches!(s.coverage.unverified.as_slice(), [(RefKind::Inductor, _, 1)]), "{:?}", s.coverage.unverified);
        let rules: Vec<&str> = s.report.hard_violations.iter().map(|v| v.rule.as_str()).collect();
        assert!(!rules.iter().any(|r| r.starts_with("engine/")), "{rules:?}");
        assert!(rules.contains(&"lvs-coverage/unverified:Inductor:-"), "{rules:?}");
    }

    /// Parametric LVS is live end to end: a wrong declared width is a
    /// reported mismatch, and the right one a clean match — the guard
    /// against the vacuous shape (both sides params-empty).
    ///
    /// Oracle: the drawn channel, W = 200 nm, L = 100 nm ([`nfet`]).
    #[test]
    fn a_wrong_reference_width_is_a_parameter_mismatch_and_the_right_one_matches() {
        let pdk = sky130();
        let reference = |w_m: f64| RefInput {
            devices: vec![RefDeviceIn {
                kind: RefKind::Nmos,
                model: None,
                terminals: vec!["d".into(), "g".into(), "s".into(), "b".into()],
                params: vec![("w".into(), w_m), ("l".into(), 1e-7)],
            }],
            ports: vec![],
            external_ports: None,
        };
        assert_eq!(lvs_rows(&nfet(&pdk), &reference(2e-7), &pdk), Vec::<String>::new(), "the declared 200 nm width matches the channel");
        let wrong = lvs_rows(&nfet(&pdk), &reference(3e-7), &pdk);
        assert!(wrong.iter().any(|r| r == "lvs.parameter_mismatch"), "300 nm declared against a 200 nm channel: {wrong:?}");
    }

    /// The body is compared: an nfet reference whose bulk is the source
    /// net, which the layout does not tie, is an LVS mismatch.
    ///
    /// Oracle: the drawn nfet's bulk is the substrate, its own net.
    #[test]
    fn a_wrong_body_tie_is_an_lvs_mismatch() {
        let pdk = sky130();
        let reference = |t: [&str; 4]| RefInput {
            devices: vec![RefDeviceIn {
                kind: RefKind::Nmos,
                model: None,
                terminals: t.iter().map(|&n| n.into()).collect(),
                params: vec![("w".into(), 2e-7), ("l".into(), 1e-7)],
            }],
            ports: vec![],
            external_ports: None,
        };
        assert_eq!(lvs_rows(&nfet(&pdk), &reference(["d", "g", "s", "b"]), &pdk), Vec::<String>::new());
        assert!(!lvs_rows(&nfet(&pdk), &reference(["d", "g", "s", "s"]), &pdk).is_empty(), "a body on the source net must mismatch");
    }

    /// Each n-well is its own net: two pfets in two wells sharing one bulk
    /// net in the reference is an LVS mismatch.
    ///
    /// Oracle: two disjoint nwell rects are two nets.
    #[test]
    fn a_pmos_in_a_foreign_well_is_an_lvs_mismatch() {
        let pdk = sky130();
        let stack = |x: i32| {
            [
                rect(&pdk, "poly", x + 200, 0, 100, 400),
                rect(&pdk, "diff", x, 100, 260, 200),
                rect(&pdk, "diff", x + 240, 100, 260, 200),
                rect(&pdk, "psdm", x, 0, 500, 400),
                rect(&pdk, "nwell", x - 200, -200, 900, 800),
            ]
        };
        let shapes: Vec<_> = stack(0).into_iter().chain(stack(5900)).collect();
        assert_eq!(device_count(&shapes, &pdk), Some(2));
        let card = |d: &str, g: &str, s: &str, b: &str| RefDeviceIn {
            kind: RefKind::Pmos,
            model: None,
            terminals: [d, g, s, b].iter().map(|&n| n.into()).collect(),
            params: vec![("w".into(), 2e-7), ("l".into(), 1e-7)],
        };
        let reference = |b1: &str, b2: &str| RefInput {
            devices: vec![card("d1", "g1", "s1", b1), card("d2", "g2", "s2", b2)],
            ports: vec![],
            external_ports: None,
        };
        assert_eq!(lvs_rows(&shapes, &reference("w1", "w2"), &pdk), Vec::<String>::new());
        assert!(!lvs_rows(&shapes, &reference("w", "w"), &pdk).is_empty(), "two wells on one bulk net must mismatch");
    }

    /// A `diom` marker over the junction with two `li` pads under it binds
    /// anode and cathode: exactly one device, and the reference writer does
    /// not skip diodes.
    ///
    /// Oracle: sky130's diode_pw2nd in miniature (the deck's diode rows).
    #[test]
    fn device_count_sees_one_diode_under_a_diom_marker() {
        let pdk = sky130();
        let shapes = [
            rect(&pdk, "diff", 0, 0, 500, 1000),
            rect(&pdk, "nsdm", -125, -125, 750, 1250),
            rect(&pdk, "diom", 0, 0, 500, 1000),
            rect(&pdk, "li", 130, 130, 240, 240),
            rect(&pdk, "li", 130, 630, 240, 240),
        ];
        assert_eq!(device_count(&shapes, &pdk), Some(1));
        let with_diode = RefInput {
            devices: vec![RefDeviceIn { kind: RefKind::Diode, model: None, terminals: vec!["a".into(), "k".into()], params: vec![] }],
            ports: vec![],
            external_ports: None,
        };
        assert_eq!(
            reference::spice(&with_diode, &pdk.deck, &pdk.strings).unwrap().1,
            Vec::<usize>::new(),
            "a deck with a diom recogniser must not skip diode cards"
        );
    }

    /// A minimal MOS-like stack derives one recognised device: the merge
    /// oracle's whole currency is this count. An empty layout is zero
    /// devices, not an abort (GPurify crashed on it before 8eadfb6).
    ///
    /// Oracle: one poly crossing one diff under nsdm is one nfet.
    #[test]
    fn device_count_sees_one_mos_in_a_minimal_stack() {
        let pdk = sky130();
        assert_eq!(device_count(&nfet(&pdk), &pdk), Some(1));
        assert_eq!(device_count(&[], &pdk), Some(0), "no geometry is zero devices, not an abort");
    }
}
