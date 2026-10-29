//! EXT-17: simulation evidence the annotator may read (operating point,
//! testbench sources, spec sensitivities), and the per-device region and role
//! derived from it. Every field is optional: [`Evidence::default`] is the
//! structure-only annotation [`crate::annotate`] runs.
//!
//! #77 (Q61): a simulator's files, read against the netlist with their names
//! resolved as the sidecar's are (Q31): an `espice-emir/1` current file
//! ([`Evidence::read_currents`]) and an operating point's saved vectors
//! ([`Evidence::read_op`]), into [`EvidenceRows`] and [`OpFacts`].

use std::collections::{HashMap, HashSet};

use analog::base::ids::{DeviceId, NetId};
use analog::base::{DeviceKind, Netlist};
use analog::intent::{DeviceFacts, DeviceRole, Diagnostic, EvidenceRows, Region, Term};
use analog::metadata::{NetClass, NetClassification};
use serde_json::Value;

/// What the flow knows beyond the netlist. Ports are `Netlist.ports` (FLOW-07).
#[derive(Clone, Debug, Default)]
pub struct Evidence {
    pub op: Option<OpFacts>,
    /// Spec sensitivities; PERF-12 fills them (M4); EXT-21 allocates set allowances
    /// from `d_vt`, EXT-25 parasitic budgets from `d_c`/`d_r`/`d_cc`.
    pub sens: Option<Sensitivities>,
    /// Nets driven by PULSE/PWL/SIN testbench sources.
    pub switching_nets: Vec<NetId>,
    /// Nets held by DC testbench sources, mV.
    pub dc_sources: Vec<(NetId, f64)>,
    /// The op point came from the synthesised probe bench (not a sign-off bias).
    pub probe_bias: bool,
    /// What evidence files state, as the readers resolved it;
    /// [`crate::annotate_with`] copies it into `Intent::evidence`.
    pub rows: EvidenceRows,
    /// The readers' findings, which [`crate::annotate_with`] appends to
    /// `Intent::diagnostics`: `evidence_unknown_name` (an instance or terminal
    /// the netlist lacks) and `evidence_unsupported` (an entry not read).
    pub diagnostics: Vec<Diagnostic>,
}

/// The netlist's devices by lowercase name, for [`device_named`].
fn device_index(nl: &Netlist) -> HashMap<String, DeviceId> {
    assert!(nl.devices.len() <= usize::from(u16::MAX), "evidence: {} devices exceed the u16 id space (AA-35)", nl.devices.len());
    nl.devices.iter().enumerate().map(|(i, d)| (d.name.to_ascii_lowercase(), DeviceId(i as u16))).collect()
}

/// `v` scaled to `scale` (µA from A is `1e6`) as an f32 row value, or `None`
/// when it is not finite there: a value past f32's range is a misread too.
fn scaled(v: f64, scale: f64) -> Option<f32> {
    Some((v * scale) as f32).filter(|x| x.is_finite())
}

/// The device an evidence name names (Q31): the netlist's flattened name,
/// with `.` or `/` between levels, in any case; else the same name past
/// ngspice's device letter (`m.x1.mn1` is `x1.mn1`). A name below a device
/// names that device, since no netlist device has a level under it: it is
/// the inner device of the device's model subcircuit (sky130 wraps every
/// model in one, so `xm1.msky130_fd_pr__nfet_01v8` is `xm1`).
fn device_named(index: &HashMap<String, DeviceId>, name: &str) -> Option<DeviceId> {
    let path = name.to_ascii_lowercase().replace('.', "/");
    let at_or_above = |p: &str| std::iter::successors(Some(p), |q| q.rsplit_once('/').map(|(up, _)| up)).find_map(|q| index.get(q).copied());
    // ngspice names a device inside a subcircuit `<letter>.<instances>.<device>`.
    let past_letter = path.split_once('/').filter(|(letter, rest)| letter.len() == 1 && rest.contains('/')).map(|(_, rest)| rest);
    at_or_above(&path).or_else(|| past_letter.and_then(at_or_above))
}

/// The op variables [`Evidence::read_op`] reads, by column: the four an
/// [`OpPoints`](analog::intent::OpPoints) row holds, then EXT-17's.
const OP_VARS: [&str; 9] = ["vds", "vgs", "id", "vdsat", "gm", "vth", "vbs", "gds", "gmbs"];
const VDS: usize = 0;
const VGS: usize = 1;
const ID: usize = 2;
const VDSAT: usize = 3;
const GM: usize = 4;
const VTH: usize = 5;
const VBS: usize = 6;
const GDS: usize = 7;
const GMBS: usize = 8;

/// The column of op variable `var` (lowercase); BSIM-CMG calls `id` `ids`.
fn op_var(var: &str) -> Option<usize> {
    OP_VARS.iter().position(|v| *v == var).or((var == "ids").then_some(ID))
}

/// A device variable's `(instance, variable)` from its vector name:
/// `@x1.mn1[vds]` (ngspice) or `x1.mn1:vds` (Spectre); `None` for any other
/// vector (a node voltage, a branch current).
fn device_var(vector: &str) -> Option<(&str, &str)> {
    match vector.strip_prefix('@') {
        Some(v) => v.strip_suffix(']')?.split_once('['),
        None => vector.split_once(':'),
    }
}

impl Evidence {
    /// `file`'s handle for its rows' `source`, appended to `rows.sources`.
    fn source(&mut self, file: &str) -> u8 {
        let n = self.rows.sources.len();
        assert!(n < usize::from(u8::MAX), "evidence: {n} files read already; a row's source is a u8");
        self.rows.sources.push(file.to_string());
        n as u8
    }

    /// Reads an `espice-emir/1` file (ESPice `--emir`, OmarSiwy/ESPice#237),
    /// named `file` for the report: one `rows.currents` row per `terminals`
    /// entry, its `instance` resolved by name (Q31) and its `terminal`
    /// against that device's own, its `avg`, `em_avg`, `rms` and `peak` in
    /// amperes as µA. The testbench's `sources` are not read.
    ///
    /// # Errors
    /// Text that is not JSON, or not an `espice-emir/1` object with a
    /// `terminals` array: nothing is read. An entry naming an instance or
    /// terminal the netlist lacks is an `evidence_unknown_name` diagnostic
    /// (one per instance), one without four finite currents or stating a
    /// terminal twice an `evidence_unsupported` one; the rest is read.
    ///
    /// # Panics
    /// When 255 evidence files were read already: a row's source is a `u8`.
    pub fn read_currents(&mut self, file: &str, json: &str, nl: &Netlist) -> Result<(), String> {
        let v: Value = serde_json::from_str(json).map_err(|e| format!("{file}: {e}"))?;
        if v.get("format").and_then(Value::as_str) != Some("espice-emir/1") {
            return Err(format!("{file}: not an espice-emir/1 file"));
        }
        let entries = v.get("terminals").and_then(Value::as_array).ok_or_else(|| format!("{file}: no terminals array"))?;
        let source = self.source(file);
        let index = device_index(nl);
        let (mut diags, mut unknown, mut seen) = (Vec::new(), HashSet::new(), HashSet::new());
        let rows = &mut self.rows.currents;
        let before = rows.device.len();
        for (i, e) in entries.iter().enumerate() {
            let text = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("");
            let (instance, pin) = (text("instance"), text("terminal"));
            let mut diag = |kind, devices, what: String| diags.push(Diagnostic { kind, devices, message: format!("{file}: terminal {i}: {what}") });
            let Some(d) = device_named(&index, instance) else {
                if unknown.insert(instance.to_ascii_lowercase()) {
                    diag("evidence_unknown_name", vec![], format!("instance {instance} is not in the netlist"));
                }
                continue;
            };
            let dev = &nl.devices[d.0 as usize];
            let Some(term) = Term::from_name(pin).filter(|_| dev.terminals.iter().any(|(t, _)| t.eq_ignore_ascii_case(pin))) else {
                diag("evidence_unknown_name", vec![d], format!("{} has no terminal {pin:?}", dev.name));
                continue;
            };
            let ua = |k: &str| e.get(k).and_then(Value::as_f64).and_then(|a| scaled(a, 1e6));
            let (Some(avg), Some(em_avg), Some(rms), Some(peak)) = (ua("avg"), ua("em_avg"), ua("rms"), ua("peak")) else {
                diag("evidence_unsupported", vec![d], format!("{} {term:?}: not four finite currents avg, em_avg, rms, peak", dev.name));
                continue;
            };
            if !seen.insert((d, term)) {
                diag("evidence_unsupported", vec![d], format!("{} {term:?} stated twice; the first kept", dev.name));
                continue;
            }
            rows.source.push(source);
            rows.device.push(d);
            rows.term.push(term);
            rows.avg_ua.push(avg);
            rows.em_avg_ua.push(em_avg);
            rows.rms_ua.push(rms);
            rows.peak_ua.push(peak);
        }
        assert!(rows.device.len() - before <= entries.len(), "at most one row per entry");
        self.diagnostics.extend(diags);
        Ok(())
    }

    /// Reads an operating point (OmarSiwy/ESPice#236), named `file` for the
    /// report: `vectors` are its `Operating Point` plot's values by vector
    /// name, as the engine decodes a raw file. A device variable is
    /// `@<instance>[<var>]` (ngspice) or `<instance>:<var>` (Spectre), its
    /// instance resolved by name (Q31); other vectors are not read. A device
    /// saving `vds`, `vgs`, `id` (BSIM-CMG: `ids`) and `vdsat` gets a
    /// `rows.op` row in mV and µA. [`Evidence::op`] becomes this operating
    /// point: each such device that saves `gm` as well, with `vth`, `vbs`,
    /// `gds` and `gmbs` where saved, so EXT-17 sets its region and role.
    ///
    /// Every problem is a diagnostic: `evidence_unknown_name` for an instance
    /// the netlist lacks (one per instance); `evidence_unsupported` for a
    /// value saved twice or out of range, a device saving some of the four
    /// but not all, devices without `gm`, or no device variable at all.
    ///
    /// # Panics
    /// When 255 evidence files were read already: a row's source is a `u8`.
    pub fn read_op(&mut self, file: &str, vectors: &[(&str, f64)], nl: &Netlist) {
        let source = self.source(file);
        let index = device_index(nl);
        let (mut diags, mut unknown, mut named) = (Vec::new(), HashSet::new(), 0usize);
        let mut vals: Vec<[Option<f64>; OP_VARS.len()]> = vec![[None; OP_VARS.len()]; nl.devices.len()];
        for &(vector, v) in vectors {
            let Some((instance, k)) = device_var(vector).and_then(|(i, var)| Some((i, op_var(&var.to_ascii_lowercase())?))) else { continue };
            named += 1;
            let mut diag = |kind, devices, what: String| diags.push(Diagnostic { kind, devices, message: format!("{file}: {vector}: {what}") });
            let Some(d) = device_named(&index, instance) else {
                if unknown.insert(instance.to_ascii_lowercase()) {
                    diag("evidence_unknown_name", vec![], format!("instance {instance} is not in the netlist"));
                }
                continue;
            };
            let slot = &mut vals[d.0 as usize][k];
            // µA from A is the largest scale a row value takes (mV from V is 1e3).
            if slot.is_some() || scaled(v, 1e6).is_none() {
                let what = if slot.is_some() { "saved twice; the first kept".to_string() } else { format!("not a finite value in range: {v}") };
                diag("evidence_unsupported", vec![d], what);
                continue;
            }
            *slot = Some(v);
        }
        let mut dev = vec![None; nl.devices.len()];
        let mut no_gm = Vec::new();
        let rows = &mut self.rows.op;
        let before = rows.device.len();
        for (i, x) in vals.iter().enumerate().filter(|(_, x)| x.iter().any(Option::is_some)) {
            let d = DeviceId(i as u16);
            let (Some(vds), Some(vgs), Some(id), Some(vdsat)) = (x[VDS], x[VGS], x[ID], x[VDSAT]) else {
                let missing: Vec<&str> = [VDS, VGS, ID, VDSAT].into_iter().filter(|&k| x[k].is_none()).map(|k| OP_VARS[k]).collect();
                let message = format!("{file}: {}: no {}; a row needs vds, vgs, id and vdsat", nl.devices[i].name, missing.join(", "));
                diags.push(Diagnostic { kind: "evidence_unsupported", devices: vec![d], message });
                continue;
            };
            rows.source.push(source);
            rows.device.push(d);
            rows.vds_mv.push((vds * 1e3) as f32);
            rows.vgs_mv.push((vgs * 1e3) as f32);
            rows.id_ua.push((id * 1e6) as f32);
            rows.vdsat_mv.push((vdsat * 1e3) as f32);
            let Some(gm) = x[GM] else {
                no_gm.push(d);
                continue;
            };
            let mv = |k: usize| x[k].map(|v| v * 1e3);
            let us = |k: usize| x[k].map(|v| v * 1e6);
            dev[i] = Some(DeviceOp {
                id_ua: id * 1e6,
                headroom_mv: (vds.abs() - vdsat.abs()) * 1e3,
                gm_us: gm * 1e6,
                power_uw: (id * vds).abs() * 1e6,
                vgs_mv: Some(vgs * 1e3),
                vbs_mv: mv(VBS),
                vth_mv: mv(VTH),
                gmb_us: us(GMBS),
                gds_us: us(GDS),
            });
        }
        assert!(rows.device.len() - before <= nl.devices.len(), "at most one row per device");
        if !no_gm.is_empty() {
            let message = format!("{file}: no gm for {} devices: their regions and roles stay structural (EXT-17)", no_gm.len());
            diags.push(Diagnostic { kind: "evidence_unsupported", devices: no_gm, message });
        }
        if named == 0 {
            let message = format!("{file}: no device variable; save each device's vds, vgs, id and vdsat (.save @m1[vds] ..., or .probe op)");
            diags.push(Diagnostic { kind: "evidence_unsupported", devices: vec![], message });
        }
        self.op = Some(OpFacts { dev, net_mv: vec![None; nl.nets.len()] });
        self.diagnostics.extend(diags);
    }
}

/// DC operating point, indexed by device and by net.
#[derive(Clone, Debug)]
pub struct OpFacts {
    pub dev: Vec<Option<DeviceOp>>,
    pub net_mv: Vec<Option<f64>>,
}

/// One device's bias. `headroom_mv` = `|V_DS| − |V_DSsat|` (`library::OpPoint`
/// folds vdsat into it); `vth`/`gmb`/`gds` are `None` until PERF-09 produces them.
#[derive(Clone, Copy, Debug)]
pub struct DeviceOp {
    pub id_ua: f64,
    pub headroom_mv: f64,
    pub gm_us: f64,
    pub power_uw: f64,
    pub vgs_mv: Option<f64>,
    pub vbs_mv: Option<f64>,
    pub vth_mv: Option<f64>,
    pub gmb_us: Option<f64>,
    pub gds_us: Option<f64>,
}

/// Per-spec sensitivities (Lampaert eqs. 2.2–2.7); PERF-12 produces them.
#[derive(Clone, Debug, Default)]
pub struct Sensitivities {
    pub specs: Vec<SpecSens>,
}

#[derive(Clone, Debug)]
pub struct SpecSens {
    pub metric: String,
    pub f0: f64,
    pub lo: Option<f64>,
    pub hi: Option<f64>,
    /// Metric range over the process corners; `None` = nominal only.
    pub proc: Option<(f64, f64)>,
    pub sigma_f: Option<f64>,
    /// Per aF of ground capacitance.
    pub d_c: Vec<(NetId, f64)>,
    /// Per Ω of series resistance.
    pub d_r: Vec<(NetId, f64)>,
    /// Per mV of threshold shift.
    pub d_vt: Vec<(DeviceId, f64)>,
    /// Per K; PERF-12 leaves it empty.
    pub d_t: Vec<(DeviceId, f64)>,
    /// Coupling sensitivity per aF between two nets (LAMP-39 ½·S_Cki).
    pub d_cc: Vec<(NetId, NetId, f64)>,
}

/// Operating region, first match wins: `|Id| < 1e-3·max_id` Off; `|Vgs| <
/// |Vth|` when both are known, else `gm/|Id| ≥ 20 /V`, Subthreshold (H12-08;
/// 20 /V is Philis policy, near the weak-inversion limit 1/(n·U_T)); negative
/// headroom Triode; otherwise Saturation.
#[must_use]
pub fn region(op: &DeviceOp, max_id_ua: f64) -> Region {
    let id = op.id_ua.abs();
    if id < 1e-3 * max_id_ua {
        Region::Off
    } else if match (op.vgs_mv, op.vth_mv) {
        (Some(vgs), Some(vth)) => vgs.abs() < vth.abs(),
        _ => op.gm_us / id >= 20.0,
    } {
        Region::Subthreshold
    } else if op.headroom_mv < 0.0 {
        Region::Triode
    } else {
        Region::Saturation
    }
}

/// Region and role per device. Roles, first match: Switch (Triode, digital
/// gate), Diode (D = G), Cascode (Saturation, non-rail S, Bias gate; before
/// CurrentSource, which it would otherwise always match), CurrentSource
/// (Saturation, Bias/Reference gate or a `shared_bias` member), Amplifier
/// (Saturation, Signal/Sensitive gate), Load (in a Load leaf). Without an op
/// only Diode, Load and Passive (R/C/L) are assigned.
#[must_use]
pub fn device_facts(
    nl: &Netlist,
    op: Option<&OpFacts>,
    classes: &[NetClassification],
    shared_bias: &[Vec<DeviceId>],
    load_leaf: &[bool],
) -> Vec<DeviceFacts> {
    let fet = |k: DeviceKind| matches!(k, DeviceKind::Nmos | DeviceKind::Pmos);
    let dev_op = |d: usize| op.and_then(|o| o.dev.get(d).copied().flatten());
    let max_id = (0..nl.devices.len()).filter(|&d| fet(nl.devices[d].kind)).filter_map(dev_op).map(|o| o.id_ua.abs()).fold(0.0, f64::max);
    let mut in_shared = vec![false; nl.devices.len()];
    shared_bias.iter().flatten().for_each(|d| in_shared[d.0 as usize] = true);
    nl.devices
        .iter()
        .enumerate()
        .map(|(d, dev)| {
            if !fet(dev.kind) {
                let role = if matches!(dev.kind, DeviceKind::Resistor | DeviceKind::Capacitor | DeviceKind::Inductor) {
                    DeviceRole::Passive
                } else {
                    DeviceRole::Unknown
                };
                return DeviceFacts { region: Region::Unknown, role };
            }
            let net = |t: &str| dev.terminals.iter().find(|(n, _)| n == t).map(|&(_, n)| n);
            let class = |t: &str| net(t).map(|n| classes[n.0 as usize].class);
            let gate = class("G");
            let region = dev_op(d).map_or(Region::Unknown, |o| region(&o, max_id));
            let sat = region == Region::Saturation;
            let gate_in = |cs: &[NetClass]| gate.is_some_and(|g| cs.contains(&g));
            let role = if region == Region::Triode && gate_in(&[NetClass::Clock, NetClass::DigitalSwitching, NetClass::DigitalStatic]) {
                DeviceRole::Switch
            } else if net("D").is_some() && net("D") == net("G") {
                DeviceRole::Diode
            } else if sat
                && gate_in(&[NetClass::Bias])
                && class("S").is_some_and(|c| !matches!(c, NetClass::Supply | NetClass::Ground | NetClass::Substrate))
            {
                DeviceRole::Cascode
            } else if sat && (gate_in(&[NetClass::Bias, NetClass::Reference]) || in_shared[d]) {
                DeviceRole::CurrentSource
            } else if sat && gate_in(&[NetClass::Signal, NetClass::Sensitive]) {
                DeviceRole::Amplifier
            } else if load_leaf[d] {
                DeviceRole::Load
            } else {
                DeviceRole::Unknown
            };
            DeviceFacts { region, role }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn op(id_ua: f64, gm_us: f64, headroom_mv: f64) -> DeviceOp {
        DeviceOp { id_ua, headroom_mv, gm_us, power_uw: 0.0, vgs_mv: None, vbs_mv: None, vth_mv: None, gmb_us: None, gds_us: None }
    }

    #[test]
    fn regions_from_headroom_and_gm() {
        assert_eq!(region(&op(0.05, 1.0, 100.0), 100.0), Region::Off);
        assert_eq!(region(&op(10.0, 250.0, 100.0), 100.0), Region::Subthreshold);
        assert_eq!(region(&op(10.0, 50.0, -20.0), 100.0), Region::Triode);
        assert_eq!(region(&op(10.0, 50.0, 100.0), 100.0), Region::Saturation);
        let vt = DeviceOp { vgs_mv: Some(300.0), vth_mv: Some(400.0), ..op(10.0, 50.0, 100.0) };
        assert_eq!(region(&vt, 100.0), Region::Subthreshold);
    }

    /// Every device biased as `o`; nets unknown.
    pub(crate) fn all_ops(nl: &Netlist, o: DeviceOp) -> Evidence {
        Evidence { op: Some(OpFacts { dev: vec![Some(o); nl.devices.len()], net_mv: vec![None; nl.nets.len()] }), ..Default::default() }
    }

    fn sc_switches() -> Netlist {
        use crate::tests::{fet, nets};
        Netlist {
            devices: vec![fet("MS1", DeviceKind::Nmos, 1, 0, 2, 3, 1_000, 150), fet("MS2", DeviceKind::Nmos, 5, 4, 6, 3, 1_000, 150)],
            nets: nets(&["a1", "p1", "b1", "VSS", "a2", "p2", "b2"]),
            ..Default::default()
        }
    }

    #[test]
    fn switch_needs_digital_gate() {
        let nl = sc_switches();
        let ev = all_ops(&nl, op(10.0, 50.0, -20.0));
        let cfg = crate::AnnotationConfig { clock_nets: vec!["p1".into(), "p2".into()], ..Default::default() };
        let p = crate::annotate_with(&nl, &cfg, &ev);
        assert!(p.intent.devices.iter().all(|f| f.role == DeviceRole::Switch), "{:?}", p.intent.devices);
        let p = crate::annotate_with(&nl, &crate::AnnotationConfig::default(), &ev);
        assert!(p.intent.devices.iter().all(|f| f.region == Region::Triode && f.role != DeviceRole::Switch), "{:?}", p.intent.devices);
    }

    /// The folded cascode of the corpus (`tests/corpus.rs`).
    pub(crate) fn folded() -> Netlist {
        use crate::tests::{fet, nets};
        use DeviceKind::{Nmos, Pmos};
        Netlist {
            devices: vec![
                fet("M1", Nmos, 1, 0, 2, 3, 10_000, 1_000),
                fet("M2", Nmos, 5, 4, 2, 3, 10_000, 1_000),
                fet("M0", Nmos, 6, 2, 3, 3, 20_000, 1_000),
                fet("M3", Pmos, 7, 0, 8, 8, 20_000, 1_000),
                fet("M4", Pmos, 7, 4, 8, 8, 20_000, 1_000),
                fet("M5", Pmos, 10, 9, 0, 8, 10_000, 1_000),
                fet("M6", Pmos, 10, 11, 4, 8, 10_000, 1_000),
                fet("M7", Nmos, 12, 9, 13, 3, 5_000, 1_000),
                fet("M8", Nmos, 12, 11, 14, 3, 5_000, 1_000),
                fet("M9", Nmos, 9, 13, 3, 3, 5_000, 1_000),
                fet("M10", Nmos, 9, 14, 3, 3, 5_000, 1_000),
            ],
            nets: nets(&["x1", "vinp", "tail", "VSS", "x2", "vinn", "vbn", "vbp1", "VDD", "o1", "vbp2", "out", "vbn2", "y1", "y2"]),
            ..Default::default()
        }
    }

    /// EXT-17/18: a Bias-gated FET with its source off the rails is a cascode,
    /// not a current source.
    #[test]
    fn cascode_before_current_source() {
        let nl = folded();
        let p = crate::annotate_with(&nl, &crate::AnnotationConfig::default(), &all_ops(&nl, op(10.0, 50.0, 100.0)));
        let role = |n: &str| p.intent.devices[nl.devices.iter().position(|d| d.name == n).unwrap()].role;
        assert_eq!((role("M5"), role("M6")), (DeviceRole::Cascode, DeviceRole::Cascode));
        assert_eq!(role("M0"), DeviceRole::CurrentSource);
    }

    /// A 1:10 NMOS mirror: `XM1` the diode reference, `XM2` ten times wider
    /// on `out`.
    fn mirror10() -> Netlist {
        use crate::tests::{fet, nets};
        Netlist {
            devices: vec![fet("XM1", DeviceKind::Nmos, 1, 1, 2, 2, 1_000, 1_000), fet("XM2", DeviceKind::Nmos, 1, 0, 2, 2, 10_000, 1_000)],
            nets: nets(&["out", "bias", "VSS"]),
            ..Default::default()
        }
    }

    /// `mirror10` at its operating point, in OmarSiwy/ESPice#237's format,
    /// written by hand: 10 µA through the reference and 100 µA through the
    /// output (at a DC point `avg = em_avg = rms = peak = |i|`, the sign kept
    /// in `avg`), then `xm9`, a device of another netlist.
    const MIRROR10_EMIR: &str = r#"{
      "format": "espice-emir/1", "deck": "mirror10_hot.sp", "analysis": "op", "temp": 125, "window": null, "em_recovery": 1,
      "terminals": [
        {"instance": "xm1", "model": "sky130_fd_pr__nfet_01v8", "terminal": "d", "net": "bias", "avg": 1.0e-05, "em_avg": 1.0e-05, "rms": 1.0e-05, "peak": 1.0e-05},
        {"instance": "xm1", "model": "sky130_fd_pr__nfet_01v8", "terminal": "s", "net": "vss", "avg": -1.0e-05, "em_avg": 1.0e-05, "rms": 1.0e-05, "peak": 1.0e-05},
        {"instance": "XM2", "model": "sky130_fd_pr__nfet_01v8", "terminal": "d", "net": "out", "avg": 1.0e-04, "em_avg": 1.0e-04, "rms": 1.0e-04, "peak": 1.0e-04},
        {"instance": "xm2", "model": "sky130_fd_pr__nfet_01v8", "terminal": "S", "net": "vss", "avg": -1.0e-04, "em_avg": 1.0e-04, "rms": 1.0e-04, "peak": 1.0e-04},
        {"instance": "xm9", "model": "sky130_fd_pr__nfet_01v8", "terminal": "d", "net": "out", "avg": 5.0e-06, "em_avg": 5.0e-06, "rms": 5.0e-06, "peak": 5.0e-06},
        {"instance": "xm9", "model": "sky130_fd_pr__nfet_01v8", "terminal": "s", "net": "vss", "avg": -5.0e-06, "em_avg": 5.0e-06, "rms": 5.0e-06, "peak": 5.0e-06}
      ],
      "sources": [{"instance": "vdd", "net": "vdd", "avg": -1.15e-04, "em_avg": 1.15e-04, "rms": 1.15e-04, "peak": 1.15e-04}]
    }"#;

    /// `(device, terminal, avg, em_avg)` per current row.
    fn current_rows(ev: &Evidence) -> Vec<(u16, Term, f32, f32)> {
        let c = &ev.rows.currents;
        (0..c.device.len()).map(|i| (c.device[i].0, c.term[i], c.avg_ua[i], c.em_avg_ua[i])).collect()
    }

    /// #77: the 1:10 mirror's currents reach `intent.evidence.currents`, the
    /// table ROUTE-EmWidth (M2) sizes by, the output branch at ten times the
    /// reference's; without evidence the table is empty and EmWidth keeps
    /// its role default. Oracle: the hand-written file.
    #[test]
    fn a_ten_to_one_mirror_carries_ten_times_the_reference_current() {
        let nl = mirror10();
        let mut ev = Evidence::default();
        ev.read_currents("mirror10.emir.json", MIRROR10_EMIR, &nl).unwrap();
        assert_eq!(current_rows(&ev), [(0, Term::D, 10.0, 10.0), (0, Term::S, -10.0, 10.0), (1, Term::D, 100.0, 100.0), (1, Term::S, -100.0, 100.0)]);
        let c = &ev.rows.currents;
        assert_eq!((c.rms_ua[2], c.peak_ua[2]), (100.0, 100.0));
        assert_eq!(ev.rows.sources, ["mirror10.emir.json"]);
        assert!((0..4).all(|i| c.origin(i) == analog::intent::Origin::Evidence { source: 0 }));
        let cfg = crate::AnnotationConfig::default();
        assert_eq!(crate::annotate_with(&nl, &cfg, &ev).intent.evidence, ev.rows);
        assert_eq!(crate::annotate(&nl, &cfg).intent.evidence, EvidenceRows::default());
    }

    /// #77 gate: a current keyed by an instance the netlist lacks is one
    /// diagnostic naming it (not one per terminal), and the rest is read.
    #[test]
    fn a_missing_instance_is_named_and_the_rest_is_read() {
        let nl = mirror10();
        let mut ev = Evidence::default();
        ev.read_currents("mirror10.emir.json", MIRROR10_EMIR, &nl).unwrap();
        let unknown: Vec<&str> = ev.diagnostics.iter().filter(|d| d.kind == "evidence_unknown_name").map(|d| d.message.as_str()).collect();
        assert_eq!(unknown, ["mirror10.emir.json: terminal 4: instance xm9 is not in the netlist"]);
        assert_eq!(ev.diagnostics.len(), 1, "{:?}", ev.diagnostics);
        assert_eq!(ev.rows.currents.device.len(), 4);
        let p = crate::annotate_with(&nl, &crate::AnnotationConfig::default(), &ev);
        assert!(p.intent.diagnostics.iter().any(|d| d.kind == "evidence_unknown_name" && d.message.contains("xm9")));
    }

    /// Entries the reader cannot use are diagnosed one by one and skipped: a
    /// terminal the device lacks, an entry missing its peak, a peak past
    /// f32's range in µA, a terminal stated twice (the first kept).
    #[test]
    fn unreadable_current_entries_are_diagnosed_and_skipped() {
        let json = r#"{"format": "espice-emir/1", "terminals": [
            {"instance": "xm1", "terminal": "x", "avg": 1e-6, "em_avg": 1e-6, "rms": 1e-6, "peak": 1e-6},
            {"instance": "xm1", "terminal": "d", "avg": 1e-6, "em_avg": 1e-6, "rms": 1e-6},
            {"instance": "xm2", "terminal": "d", "avg": 1e-6, "em_avg": 1e-6, "rms": 1e-6, "peak": 1e40},
            {"instance": "xm1", "terminal": "d", "avg": 1e-6, "em_avg": 1e-6, "rms": 1e-6, "peak": 1e-6},
            {"instance": "xm1", "terminal": "D", "avg": 2e-6, "em_avg": 2e-6, "rms": 2e-6, "peak": 2e-6}]}"#;
        let mut ev = Evidence::default();
        ev.read_currents("bad.json", json, &mirror10()).unwrap();
        let kinds: Vec<&str> = ev.diagnostics.iter().map(|d| d.kind).collect();
        assert_eq!(kinds, ["evidence_unknown_name", "evidence_unsupported", "evidence_unsupported", "evidence_unsupported"], "{:?}", ev.diagnostics);
        assert!(ev.diagnostics[0].message.contains(r#"no terminal "x""#), "{:?}", ev.diagnostics[0]);
        assert!(ev.diagnostics[2].message.contains("XM2 D: not four finite currents"), "{:?}", ev.diagnostics[2]);
        assert!(ev.diagnostics[3].message.contains("stated twice"), "{:?}", ev.diagnostics[3]);
        assert_eq!(current_rows(&ev), [(0, Term::D, 1.0, 1.0)]);
    }

    /// A file that is no `espice-emir/1` current file is refused whole, and
    /// nothing of it is read: not JSON, another format, no terminals.
    #[test]
    fn a_file_that_is_no_current_file_is_refused() {
        for bad in ["{", "[]", r#"{"format": "espice-emir/2", "terminals": []}"#, r#"{"format": "espice-emir/1"}"#] {
            let mut ev = Evidence::default();
            assert!(ev.read_currents("x.json", bad, &mirror10()).is_err(), "{bad}");
            assert_eq!(ev.rows, EvidenceRows::default(), "{bad}");
        }
    }

    /// `XM1`, an X card (sky130's model subcircuit holds its MOS); `x1/MN1`,
    /// inside subcircuit instance `x1`; `M3`, a PMOS card.
    fn three_named() -> Netlist {
        use crate::tests::{fet, nets};
        use DeviceKind::{Nmos, Pmos};
        Netlist {
            devices: vec![
                fet("XM1", Nmos, 1, 0, 2, 2, 1_000, 150),
                fet("x1/MN1", Nmos, 1, 3, 2, 2, 1_000, 150),
                fet("M3", Pmos, 1, 0, 4, 4, 1_000, 150),
            ],
            nets: nets(&["d", "g", "VSS", "x1/n", "VDD"]),
            ..Default::default()
        }
    }

    /// `(device, vds, vgs, id, vdsat)` per op row.
    fn op_rows(ev: &Evidence) -> Vec<(u16, f32, f32, f32, f32)> {
        let o = &ev.rows.op;
        (0..o.device.len()).map(|i| (o.device[i].0, o.vds_mv[i], o.vgs_mv[i], o.id_ua[i], o.vdsat_mv[i])).collect()
    }

    /// #77: every name a simulator saves a device variable under resolves to
    /// its device (Q31): ngspice's inner device of sky130's model subcircuit
    /// (`@m.xm1.msky130_fd_pr__nfet_01v8[vds]`), a device in a subcircuit
    /// (`@m.x1.mn1[ids]`, BSIM-CMG's drain current) and Spectre's `m3:vds`.
    /// The rows reach `intent.evidence.op`, the table the MOS-cascode V_DS
    /// claim (v2) scores by, and with `gm` EXT-17 sets regions. Oracle: the
    /// values in mV and µA by hand; region by EXT-17's rule (`gm/|Id|` under
    /// 20 /V, positive headroom: saturation).
    #[test]
    fn op_vectors_resolve_by_every_naming_form() {
        let nl = three_named();
        let xm1 = |v: &str| format!("@m.xm1.msky130_fd_pr__nfet_01v8[{v}]");
        let (vds, vgs, id, vdsat, gm) = (xm1("vds"), xm1("vgs"), xm1("id"), xm1("vdsat"), xm1("gm"));
        let vectors = [
            ("v(d)", 0.9),
            ("i(vdd)", -1.2e-4),
            (vds.as_str(), 0.9),
            (vgs.as_str(), 0.7),
            (id.as_str(), 1.0e-4),
            (vdsat.as_str(), 0.18),
            (gm.as_str(), 1.0e-3),
            ("@m.x1.mn1[vds]", 0.3),
            ("@m.x1.mn1[vgs]", 0.6),
            ("@m.x1.mn1[ids]", 1.0e-5),
            ("@m.x1.mn1[vdsat]", 0.12),
            ("@m.x1.mn1[gm]", 1.0e-4),
            ("m3:vds", -0.5),
            ("m3:vgs", -0.8),
            ("m3:id", -2.0e-5),
            ("m3:vdsat", -0.2),
            ("m3:gm", 3.0e-4),
        ];
        let mut ev = Evidence::default();
        ev.read_currents("hot.emir.json", r#"{"format": "espice-emir/1", "terminals": []}"#, &nl).unwrap();
        ev.read_op("hot.raw", &vectors, &nl);
        assert!(ev.diagnostics.is_empty(), "{:?}", ev.diagnostics);
        assert_eq!(op_rows(&ev), [(0, 900.0, 700.0, 100.0, 180.0), (1, 300.0, 600.0, 10.0, 120.0), (2, -500.0, -800.0, -20.0, -200.0)]);
        assert_eq!(ev.rows.sources, ["hot.emir.json", "hot.raw"]);
        assert!((0..3).all(|i| ev.rows.op.origin(i) == analog::intent::Origin::Evidence { source: 1 }));
        let xm1_op = ev.op.as_ref().unwrap().dev[0].unwrap();
        assert!((xm1_op.headroom_mv - 720.0).abs() < 1e-9 && (xm1_op.gm_us - 1_000.0).abs() < 1e-9, "{xm1_op:?}");
        assert!((xm1_op.power_uw - 90.0).abs() < 1e-9, "|Id·Vds| = 100 µA · 0.9 V: {xm1_op:?}");
        let p = crate::annotate_with(&nl, &crate::AnnotationConfig::default(), &ev);
        assert!(p.intent.devices.iter().all(|f| f.region == Region::Saturation), "{:?}", p.intent.devices);
        assert_eq!(p.intent.evidence.op, ev.rows.op);
    }

    /// Every op problem is a diagnostic and the rest is read: an instance the
    /// netlist lacks (once, however many of its vectors), a value saved
    /// twice, one that is not finite, devices saving part of the four, and a
    /// device without `gm` (its row read, EXT-17 not told).
    #[test]
    fn op_problems_are_diagnosed_and_the_rest_is_read() {
        let nl = three_named();
        let xm = |n: u8, v: &str| format!("@m.xm{n}.msky130_fd_pr__nfet_01v8[{v}]");
        let named: Vec<(String, f64)> = vec![
            (xm(9, "vds"), 0.9),
            (xm(9, "vgs"), 0.7),
            ("@m.x1.mn1[vds]".into(), 0.3),
            ("@m.x1.mn1[vds]".into(), 0.31),
            ("@m.x1.mn1[vgs]".into(), f64::NAN),
            ("@m3[vds]".into(), -0.5),
            ("@m3[vgs]".into(), -0.8),
            ("@m3[id]".into(), -2.0e-5),
            (xm(1, "vds"), 0.9),
            (xm(1, "vgs"), 0.7),
            (xm(1, "id"), 1.0e-4),
            (xm(1, "vdsat"), 0.18),
        ];
        let vectors: Vec<(&str, f64)> = named.iter().map(|(n, v)| (n.as_str(), *v)).collect();
        let mut ev = Evidence::default();
        ev.read_op("hot.raw", &vectors, &nl);
        let of = |kind: &str| ev.diagnostics.iter().filter(|d| d.kind == kind).map(|d| d.message.as_str()).collect::<Vec<_>>();
        let unknown = of("evidence_unknown_name");
        assert!(unknown.len() == 1 && unknown[0].contains("instance m.xm9.msky130_fd_pr__nfet_01v8"), "{unknown:?}");
        let unsupported = of("evidence_unsupported");
        for what in ["saved twice", "not a finite value in range: NaN", "x1/MN1: no vgs, id, vdsat", "M3: no vdsat", "no gm for 1 devices"] {
            assert!(unsupported.iter().any(|m| m.contains(what)), "{what}: {unsupported:?}");
        }
        assert_eq!(unsupported.len(), 5, "{unsupported:?}");
        assert_eq!(op_rows(&ev), [(0, 900.0, 700.0, 100.0, 180.0)]);
        assert!(ev.op.as_ref().unwrap().dev.iter().all(Option::is_none), "no device has gm");
    }

    /// A plot with no device variable (ESPice saves node voltages and branch
    /// currents unless told) says how to save them.
    #[test]
    fn an_op_without_device_variables_says_how_to_save_them() {
        let mut ev = Evidence::default();
        ev.read_op("plain.raw", &[("v(d)", 0.9), ("i(vdd)", -1.0e-4)], &three_named());
        assert_eq!(ev.diagnostics.len(), 1, "{:?}", ev.diagnostics);
        assert!(ev.diagnostics[0].message.contains(".save @m1[vds]"), "{:?}", ev.diagnostics[0]);
        assert!(ev.rows.op.device.is_empty());
    }
}
