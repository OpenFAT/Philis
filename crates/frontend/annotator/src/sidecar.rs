//! EXT-26: the user constraint sidecar, an ALIGN-style JSON array of
//! `{"constraint": <kind>, ...}` entries, parsed into [`AnnotationConfig`]
//! fields. Names resolve case-insensitively; an unknown name skips its entry
//! with a `sidecar_unknown_name` diagnostic. Only kinds with a reader today
//! are applied; anything else (`Align`, `HorizontalDistance`, …) is
//! `sidecar_unsupported`.
//!
//! | kind                                     | fields                                  |
//! |------------------------------------------|-----------------------------------------|
//! | `PowerPorts`, `GroundPorts`, `ClockPorts` | `ports: [net]`                          |
//! | `DoNotIdentify`                          | `instances`                             |
//! | `GroupBlocks`                            | `instances`, `instance_name` (an alias) |
//! | `SymmetricBlocks`                        | `direction` (`V`/`H`), `pairs: [[name]]`|
//! | `SymmetricNets`                          | `net1`, `net2`                          |
//! | `Match`                                  | `instances`, `class`, optional `kind`   |
//! | `NetClass`                               | `nets`, `class`                         |
//! | `OffsetBudget`                           | `instances`, `sigma_mv`                 |
//! | `Kelvin`                                 | `pin: "R/P"`, `sense: ["M/G"]`          |
//! | `Load`                                   | `net`, `ff` (external load, fF)         |
//! | `Order`                                  | `instances` (names or aliases), `direction` (`bottom_to_top`, `top_to_bottom`, `left_to_right`, `right_to_left`) |
//! | `IsolatedTub`                            | `instances` (NMOS, one bulk net), `tie` |
//! | `Reference`                              | `aspect`, `rows: [[name]]`, `extents: {name: {x, y, w, h}}`; optional `source`, `unit_area: {name: ratio}`, `area_ratio` |
//!
//! ## `Reference` (UW-ASIC/Philis#78, Q61)
//!
//! A prior design's floorplan, which the floorplan takes as its targets in
//! place of the stage order (Q21, Q22):
//!
//! ```json
//! {"constraint": "Reference", "source": "sar10 v1, gf180mcu, 2025 shuttle", "aspect": 1.6,
//!  "rows": [["cdac"], ["comparator", "bias"], ["sar_logic"]],
//!  "extents": {"cdac":       {"x": 0.00, "y": 0.00, "w": 1.00, "h": 0.55},
//!              "comparator": {"x": 0.00, "y": 0.55, "w": 0.60, "h": 0.20},
//!              "bias":       {"x": 0.60, "y": 0.55, "w": 0.40, "h": 0.20},
//!              "sar_logic":  {"x": 0.00, "y": 0.75, "w": 1.00, "h": 0.25}},
//!  "unit_area": {"cdac": 1.0}, "area_ratio": 1.0}
//! ```
//!
//! - A name is a `GroupBlocks` alias from an earlier entry or an instance,
//!   resolved as `Order` resolves them (Q31). A name the netlist lacks is a
//!   `sidecar_unknown_name` diagnostic and drops that block; the rest stand.
//! - `aspect` is the outline's width over its height. Extents are fractions
//!   of the outline from its lower-left corner; `rows[0]` is the bottom row,
//!   names left to right. Every listed name has an extent inside the
//!   outline, extents in a row do not overlap, and each row sits on or above
//!   the one below. A shape that breaks any of this, a device in two blocks,
//!   or a second `Reference` is `sidecar_unsupported`, and the entry is
//!   dropped (the first `Reference` stands).
//! - `unit_area` (a block's unit over the reference's) and `area_ratio` (the
//!   block area over the reference's, after scaling by the ratio of total
//!   device area) are checked (> 0, names in a row) and carried; M0 applies
//!   neither (`placement::reference`).
//! - The reader only parses: the conflicts with the circuit's constraints,
//!   the targets and the report belong to the floorplan
//!   (`placement::reference`, `engine::arrange_reference`).

use std::collections::HashMap;

use analog::base::ids::{DeviceId, NetId};
use analog::base::Netlist;
use analog::intent::{AxisDir, ConstraintId, Diagnostic, KelvinReq, MatchClass, MatchKind, Order, RefBlock, Reference, Term};
use analog::metadata::NetClass;
use serde_json::Value;

use crate::symmetry::Seed;
use crate::AnnotationConfig;

/// Parse `json` against `nl`. `Err` only for text that is not JSON or not a
/// top-level array; every entry-level problem is a diagnostic.
pub fn parse(json: &str, nl: &Netlist) -> Result<(AnnotationConfig, Vec<Diagnostic>), String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("constraints: {e}"))?;
    let Value::Array(entries) = v else { return Err("constraints: top level is not an array".into()) };
    let mut cfg = AnnotationConfig::default();
    let mut diags = Vec::new();
    let mut alias: HashMap<String, Vec<DeviceId>> = HashMap::new();
    let device = |n: &str| nl.devices.iter().position(|d| d.name.eq_ignore_ascii_case(n)).map(|i| DeviceId(i as u16));
    let net = |n: &str| nl.nets.iter().position(|x| x.name.eq_ignore_ascii_case(n)).map(|i| NetId(i as u16));
    for (i, e) in entries.iter().enumerate() {
        let id = ConstraintId(u32::MAX - i as u32);
        let kind = e.get("constraint").and_then(Value::as_str).unwrap_or("");
        let strs = |k: &str| -> Vec<String> {
            e.get(k).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(String::from).collect()
        };
        macro_rules! unknown {
            ($what:expr) => {
                diags.push(Diagnostic { kind: "sidecar_unknown_name", devices: vec![], message: format!("entry {i} ({kind}): unknown {}", $what) })
            };
        }
        // All named devices, or `Err` naming the first unknown one.
        let devices =
            |names: &[String]| -> Result<Vec<DeviceId>, String> { names.iter().map(|n| device(n).ok_or_else(|| format!("instance {n}"))).collect() };
        match kind {
            "PowerPorts" | "GroundPorts" | "ClockPorts" => {
                let ports = strs("ports");
                if let Some(bad) = ports.iter().find(|p| net(p).is_none()) {
                    unknown!(format!("net {bad}"));
                    continue;
                }
                let list = match kind {
                    "PowerPorts" => &mut cfg.supply_nets,
                    "GroundPorts" => &mut cfg.ground_nets,
                    _ => &mut cfg.clock_nets,
                };
                list.extend(ports);
            }
            "DoNotIdentify" => match devices(&strs("instances")) {
                Ok(ds) => cfg.do_not_identify.extend(ds.iter().map(|d| u32::from(d.0))),
                Err(n) => unknown!(n),
            },
            "GroupBlocks" => match devices(&strs("instances")) {
                Ok(ds) => {
                    if let Some(name) = e.get("instance_name").and_then(Value::as_str) {
                        alias.insert(name.to_ascii_lowercase(), ds.clone());
                    }
                    cfg.groups.push((i as u32, ds));
                }
                Err(n) => unknown!(n),
            },
            "SymmetricBlocks" => {
                cfg.symmetry_dir = Some(if e.get("direction").and_then(Value::as_str) == Some("H") {
                    analog::intent::AxisDir::H
                } else {
                    analog::intent::AxisDir::V
                });
                for pair in e.get("pairs").and_then(Value::as_array).into_iter().flatten() {
                    let names: Vec<&str> = pair.as_array().into_iter().flatten().filter_map(Value::as_str).collect();
                    let resolve = |n: &str| alias.get(&n.to_ascii_lowercase()).cloned().or_else(|| device(n).map(|d| vec![d]));
                    let got: Option<Vec<Vec<DeviceId>>> = names.iter().map(|n| resolve(n)).collect();
                    let Some(got) = got else {
                        unknown!(format!("instance in {names:?}"));
                        continue;
                    };
                    match got.as_slice() {
                        [one] if one.len() == 1 => cfg.seeds.push(Seed::SelfDevice(one[0], id)),
                        [one] if one.len() == 2 => cfg.seeds.push(Seed::Devices(one[0], one[1], id)),
                        [a, b] if a.len() == b.len() => cfg.seeds.extend(a.iter().zip(b).map(|(&x, &y)| Seed::Devices(x, y, id))),
                        _ => diags.push(Diagnostic {
                            kind: "sidecar_unsupported",
                            devices: got.concat(),
                            message: format!("entry {i}: symmetric group {names:?}"),
                        }),
                    }
                }
            }
            "SymmetricNets" => {
                let (a, b) = (e.get("net1").and_then(Value::as_str).unwrap_or(""), e.get("net2").and_then(Value::as_str).unwrap_or(""));
                match (net(a), net(b)) {
                    (Some(x), Some(y)) => cfg.seeds.push(Seed::Nets(x, y, id)),
                    _ => unknown!(format!("net {a}/{b}")),
                }
            }
            "Match" => {
                let class = match e.get("class").and_then(Value::as_str).map(str::to_ascii_lowercase).as_deref() {
                    Some("minimal") => MatchClass::Minimal,
                    Some("moderate") => MatchClass::Moderate,
                    Some("exceptional") => MatchClass::Exceptional,
                    _ => {
                        diags.push(Diagnostic { kind: "sidecar_unsupported", devices: vec![], message: format!("entry {i}: Match class") });
                        continue;
                    }
                };
                let mk = match e.get("kind").and_then(Value::as_str).map(str::to_ascii_lowercase).as_deref() {
                    Some("voltage") => Some(MatchKind::Voltage),
                    Some("current") => Some(MatchKind::Current),
                    Some("ratio") => Some(MatchKind::Ratio),
                    _ => None,
                };
                match devices(&strs("instances")) {
                    Ok(ds) => {
                        // GAP-09 (c): kind, model and L must agree (W may differ: a ratioed set).
                        let mut models = Vec::new();
                        let key = |d: DeviceId, models: &mut Vec<String>| {
                            let dev = &nl.devices[d.0 as usize];
                            let s = crate::size::drawn(dev, models);
                            (dev.kind, s.model, s.l_nm)
                        };
                        let first = ds.first().map(|&d| key(d, &mut models));
                        if ds.iter().any(|&d| Some(key(d, &mut models)) != first) {
                            diags.push(crate::conflict::diag(&[id], ds, "Match on unequal kind/model/L; entry dropped"));
                            continue;
                        }
                        cfg.classes.push((ds, class, mk));
                    }
                    Err(n) => unknown!(n),
                }
            }
            "NetClass" => {
                let class = match e.get("class").and_then(Value::as_str).map(str::to_ascii_lowercase).as_deref() {
                    Some("signal") => NetClass::Signal,
                    Some("clock") => NetClass::Clock,
                    Some("supply") => NetClass::Supply,
                    Some("ground") => NetClass::Ground,
                    Some("sensitive") => NetClass::Sensitive,
                    Some("substrate") => NetClass::Substrate,
                    Some("bias") => NetClass::Bias,
                    Some("reference") => NetClass::Reference,
                    // Conservative: undeclared logic may toggle, so it is an aggressor.
                    Some("digital") => NetClass::DigitalSwitching,
                    Some("digital_static") => NetClass::DigitalStatic,
                    Some("noisy") => NetClass::Noisy,
                    _ => {
                        diags.push(Diagnostic { kind: "sidecar_unsupported", devices: vec![], message: format!("entry {i}: NetClass class") });
                        continue;
                    }
                };
                for n in strs("nets") {
                    match net(&n) {
                        Some(x) => cfg.net_classes.push((x, class)),
                        None => unknown!(format!("net {n}")),
                    }
                }
            }
            "OffsetBudget" => match (devices(&strs("instances")), e.get("sigma_mv").and_then(Value::as_f64)) {
                (Ok(ds), Some(s)) => cfg.offset_budgets.push((ds, s as f32)),
                (Err(n), _) => unknown!(n),
                (Ok(_), None) => diags.push(Diagnostic {
                    kind: "sidecar_unsupported",
                    devices: vec![],
                    message: format!("entry {i}: OffsetBudget without sigma_mv"),
                }),
            },
            "Kelvin" => {
                let pin = |s: &str| -> Option<(DeviceId, Term)> {
                    let (d, t) = s.split_once('/')?;
                    let t = match t.to_ascii_uppercase().as_str() {
                        "G" => Term::G,
                        "D" => Term::D,
                        "S" => Term::S,
                        "B" => Term::B,
                        "C" => Term::C,
                        "E" => Term::E,
                        "P" => Term::P,
                        "N" => Term::N,
                        _ => return None,
                    };
                    Some((device(d)?, t))
                };
                let at = e.get("pin").and_then(Value::as_str).unwrap_or("");
                let sense: Option<Vec<(DeviceId, Term)>> = strs("sense").iter().map(|s| pin(s)).collect();
                match (pin(at), sense) {
                    (Some((device, term)), Some(sense)) => cfg.kelvins.push(KelvinReq { device, term, sense }),
                    _ => unknown!(format!("pin in {at}")),
                }
            }
            "Load" => {
                let name = e.get("net").and_then(Value::as_str).unwrap_or("");
                match (net(name), e.get("ff").and_then(Value::as_f64)) {
                    (Some(n), Some(ff)) => cfg.loads.push((n, (ff * 1000.0) as f32)),
                    (None, _) => unknown!(format!("net {name}")),
                    (Some(_), None) => {
                        diags.push(Diagnostic { kind: "sidecar_unsupported", devices: vec![], message: format!("entry {i}: Load without ff") })
                    }
                }
            }
            "Order" => {
                let (dir, flip) = match e.get("direction").and_then(Value::as_str) {
                    Some("bottom_to_top") => (AxisDir::V, false),
                    Some("top_to_bottom") => (AxisDir::V, true),
                    Some("left_to_right") => (AxisDir::H, false),
                    Some("right_to_left") => (AxisDir::H, true),
                    _ => {
                        diags.push(Diagnostic { kind: "sidecar_unsupported", devices: vec![], message: format!("entry {i}: Order direction") });
                        continue;
                    }
                };
                let names = strs("instances");
                let steps: Option<Vec<Vec<DeviceId>>> =
                    names.iter().map(|n| alias.get(&n.to_ascii_lowercase()).cloned().or_else(|| device(n).map(|d| vec![d]))).collect();
                match steps {
                    Some(mut steps) => {
                        if flip {
                            steps.reverse();
                        }
                        cfg.order.push(Order { steps, dir, reversible: false, weight: 1.0 });
                    }
                    None => unknown!(format!("instance in {names:?}")),
                }
            }
            "IsolatedTub" => {
                let ds = match devices(&strs("instances")) {
                    Ok(ds) => ds,
                    Err(n) => {
                        unknown!(n);
                        continue;
                    }
                };
                let tie_name = e.get("tie").and_then(Value::as_str).unwrap_or("");
                let Some(tie) = net(tie_name) else {
                    unknown!(format!("net {tie_name}"));
                    continue;
                };
                let bulk = |d: &DeviceId| {
                    let dev = &nl.devices[d.0 as usize];
                    (dev.kind == analog::base::DeviceKind::Nmos).then(|| dev.terminals.iter().find(|(t, _)| t == "B").map(|t| t.1)).flatten()
                };
                if ds.is_empty() || ds.iter().any(|d| bulk(d).is_none() || bulk(d) != bulk(&ds[0])) {
                    diags.push(Diagnostic {
                        kind: "sidecar_unsupported",
                        devices: ds,
                        message: format!("entry {i}: IsolatedTub members must be NMOS on one bulk net"),
                    });
                    continue;
                }
                if ds.iter().any(|d| cfg.tubs.iter().any(|(m, _)| m.contains(d))) {
                    diags.push(Diagnostic {
                        kind: "sidecar_unsupported",
                        devices: ds,
                        message: format!("entry {i}: device already in an IsolatedTub"),
                    });
                    continue;
                }
                // One dnwell needs its members side by side: a user group (GroupBlocks proximity) keeps them so.
                cfg.groups.push((i as u32, ds.clone()));
                cfg.tubs.push((ds, tie));
            }
            "Reference" => {
                if cfg.reference.is_some() {
                    diags.push(Diagnostic {
                        kind: "sidecar_unsupported",
                        devices: vec![],
                        message: format!("entry {i} (Reference): a second Reference; the first stands"),
                    });
                    continue;
                }
                let resolve = |n: &str| alias.get(&n.to_ascii_lowercase()).cloned().or_else(|| device(n).map(|d| vec![d]));
                cfg.reference = reference(e, i as u32, &resolve, nl, &mut diags);
            }
            _ => diags.push(Diagnostic { kind: "sidecar_unsupported", devices: vec![], message: format!("entry {i} ({kind})") }),
        }
    }
    Ok((cfg, diags))
}

/// Slack on the sums a reference's fractions make (`0.55 + 0.20 + 0.25`).
const FRACTION_EPS: f64 = 1e-9;

/// The `Reference` entry `e`, index `i`, its names resolved by `resolve`.
/// `None` with one `sidecar_unsupported` diagnostic when its shape is wrong
/// ([`reference_shape`]) or a device lands in two blocks. A name `resolve`
/// does not know is a `sidecar_unknown_name` diagnostic and drops that
/// block only (`None` when no block is left).
fn reference(e: &Value, i: u32, resolve: &dyn Fn(&str) -> Option<Vec<DeviceId>>, nl: &Netlist, diags: &mut Vec<Diagnostic>) -> Option<Reference> {
    let unsupported = |why: String| Diagnostic { kind: "sidecar_unsupported", devices: vec![], message: format!("entry {i} (Reference): {why}") };
    let shape = match reference_shape(e) {
        Ok(r) => r,
        Err(why) => {
            diags.push(unsupported(why));
            return None;
        }
    };
    let mut blocks = Vec::with_capacity(shape.blocks.len());
    for mut b in shape.blocks {
        match resolve(&b.name) {
            Some(devices) => {
                b.devices = devices;
                blocks.push(b);
            }
            None => diags.push(Diagnostic {
                kind: "sidecar_unknown_name",
                devices: vec![],
                message: format!("entry {i} (Reference): unknown name {}", b.name),
            }),
        }
    }
    let mut owner: HashMap<DeviceId, usize> = HashMap::new();
    for (k, b) in blocks.iter().enumerate() {
        for &d in &b.devices {
            if let Some(j) = owner.insert(d, k).filter(|&j| j != k) {
                let name = &nl.devices[d.0 as usize].name;
                diags.push(Diagnostic { devices: vec![d], ..unsupported(format!("{name} is in blocks {} and {}", blocks[j].name, b.name)) });
                return None;
            }
        }
    }
    (!blocks.is_empty()).then_some(Reference { entry: i, blocks, ..shape })
}

/// A `Reference` entry checked for what the floorplan needs, its blocks
/// with no devices yet; `Err` names the first field that is wrong. Rows
/// list names once each, bottom row first; every listed name has an extent
/// inside the outline and nothing else has one; within a row extents run
/// left to right without overlap, and each row sits on or above the one
/// below it.
fn reference_shape(e: &Value) -> Result<Reference, String> {
    let positive = |k: &str| -> Result<Option<f32>, String> {
        e.get(k).map(|v| v.as_f64().filter(|x| x.is_finite() && *x > 0.0).map(|x| x as f32).ok_or(format!("{k} must be a number > 0"))).transpose()
    };
    let aspect = positive("aspect")?.ok_or("aspect must be a number > 0")?;
    let area_ratio = positive("area_ratio")?;
    let rows = e.get("rows").and_then(Value::as_array).filter(|r| !r.is_empty()).ok_or("rows must list rows of names, bottom first")?;
    let extents = e.get("extents").and_then(Value::as_object).ok_or("extents must map every name to {x, y, w, h}")?;
    let units = e.get("unit_area").map(|u| u.as_object().ok_or("unit_area must map names to numbers")).transpose()?;
    let mut blocks: Vec<RefBlock> = Vec::new();
    // Each block's extent as written, for the agreement checks below.
    let mut exact: Vec<[f64; 4]> = Vec::new();
    for (r, row) in rows.iter().enumerate() {
        let row_id = u16::try_from(r).map_err(|_| "more than 65535 rows".to_string())?;
        let names = row.as_array().filter(|n| !n.is_empty()).ok_or(format!("row {r} must list names"))?;
        for n in names {
            let name = n.as_str().ok_or(format!("row {r}: a name must be a string"))?;
            if blocks.iter().any(|b| b.name == name) {
                return Err(format!("{name} is listed twice"));
            }
            let x = extents.get(name).ok_or(format!("{name} has no extent"))?;
            let f = |k: &str| x.get(k).and_then(Value::as_f64).filter(|v| v.is_finite());
            let (Some(ex), Some(ey), Some(ew), Some(eh)) = (f("x"), f("y"), f("w"), f("h")) else {
                return Err(format!("{name}: an extent is numbers x, y, w, h"));
            };
            if ex < 0.0 || ey < 0.0 || ew <= 0.0 || eh <= 0.0 || ex + ew > 1.0 + FRACTION_EPS || ey + eh > 1.0 + FRACTION_EPS {
                return Err(format!("{name}: extent ({ex}, {ey}, {ew}, {eh}) is not inside the outline"));
            }
            let unit_area = units
                .and_then(|u| u.get(name))
                .map(|v| v.as_f64().filter(|v| v.is_finite() && *v > 0.0).map(|v| v as f32).ok_or(format!("unit_area of {name} must be a number > 0")))
                .transpose()?;
            let extent = analog::intent::Extent { x: ex as f32, y: ey as f32, w: ew as f32, h: eh as f32 };
            blocks.push(RefBlock { name: name.to_string(), devices: Vec::new(), row: row_id, extent, unit_area });
            exact.push([ex, ey, ew, eh]);
        }
    }
    if let Some(k) = extents.keys().chain(units.into_iter().flat_map(|u| u.keys())).find(|k| !blocks.iter().any(|b| &b.name == *k)) {
        return Err(format!("{k} has an extent or a unit_area but no row"));
    }
    for (w, x) in blocks.windows(2).zip(exact.windows(2)) {
        if w[0].row == w[1].row && x[1][0] < x[0][0] + x[0][2] - FRACTION_EPS {
            return Err(format!("row {}: {} must sit right of {}", w[0].row, w[1].name, w[0].name));
        }
    }
    let rows_of = |r: u16| blocks.iter().zip(&exact).filter(move |(b, _)| b.row == r).map(|(_, x)| x);
    for r in 1..=blocks.last().map_or(0, |b| b.row) {
        let top = rows_of(r - 1).map(|x| x[1] + x[3]).fold(f64::MIN, f64::max);
        if rows_of(r).any(|x| x[1] < top - FRACTION_EPS) {
            return Err(format!("row {r} must sit on or above row {}", r - 1));
        }
    }
    let source = e.get("source").and_then(Value::as_str).unwrap_or_default().to_string();
    Ok(Reference { entry: 0, source, aspect, blocks, area_ratio })
}

impl AnnotationConfig {
    /// [`parse`]: the sidecar's config and its diagnostics.
    pub fn from_json(json: &str, nl: &Netlist) -> Result<(Self, Vec<Diagnostic>), String> {
        parse(json, nl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EXT-25: a `Load` entry is an external load on its net, aF.
    #[test]
    fn load_entry_sets_a_net_load() {
        let nl = crate::tests::ota();
        let (cfg, d) = parse(r#"[{"constraint":"Load","net":"vout2","ff":1000}]"#, &nl).unwrap();
        assert_eq!(cfg.loads, [(NetId(4), 1e6)]);
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn unknown_constraint_is_diagnosed() {
        let nl = crate::tests::ota();
        let (_, d) = parse(r#"[{"constraint":"Align","instances":["XM1"]}]"#, &nl).unwrap();
        assert_eq!(d.iter().map(|d| d.kind).collect::<Vec<_>>(), ["sidecar_unsupported"]);
    }

    /// EXT-26: a `GroupBlocks` pull carries its entry as `Origin::User`, not
    /// the pattern or net class of its first device.
    #[test]
    fn group_blocks_batches_have_user_origin() {
        let nl = crate::tests::ota();
        let (cfg, _) = parse(r#"[{"constraint":"Align","instances":["XM1"]},{"constraint":"GroupBlocks","instances":["XM1","XM5"]}]"#, &nl).unwrap();
        let p = crate::annotate(&nl, &cfg);
        let user = |arm: &[Box<dyn analog::RuleBatch<analog::base::Layout>>]| {
            arm.iter()
                .filter(|b| b.meta().map(|m| m.origin) == Some(analog::intent::Origin::User { index: 1 }))
                .map(|b| b.kind().ends_with("Proximity"))
                .collect::<Vec<_>>()
        };
        assert_eq!((user(&p.placement.budget), user(&p.placement.cost)), (vec![true], vec![true]));
    }

    /// GAP-14: two NMOS on one bulk, tied to a supply.
    #[test]
    fn an_isolated_tub_parses() {
        let nl = crate::tests::ota();
        let (cfg, d) = parse(r#"[{"constraint":"IsolatedTub","instances":["XM1","XM2"],"tie":"vdd"}]"#, &nl).unwrap();
        assert!(d.is_empty(), "{d:?}");
        assert_eq!(cfg.tubs, [(vec![DeviceId(0), DeviceId(1)], NetId(7))]);
        assert_eq!(cfg.groups, [(0, vec![DeviceId(0), DeviceId(1)])], "a tub is also a proximity group");
    }

    /// The tie must be a quiet supply (GAP-14): a signal net is drawn as asked and reported.
    #[test]
    fn a_tub_tied_to_a_signal_is_reported() {
        let nl = crate::tests::ota();
        let note = ("IsolatedTub", "tie is not a supply net");
        for (tie, warned) in [("vdd", false), ("vtail", true)] {
            let (cfg, _) = parse(&format!(r#"[{{"constraint":"IsolatedTub","instances":["XM1","XM2"],"tie":"{tie}"}}]"#), &nl).unwrap();
            assert_eq!(crate::annotate(&nl, &cfg).missing.contains(&note), warned, "tie {tie}");
        }
    }

    /// A device in two tubs is refused, not silently given the first.
    #[test]
    fn overlapping_tubs_are_refused() {
        let nl = crate::tests::ota();
        let (cfg, d) = parse(
            r#"[{"constraint":"IsolatedTub","instances":["XM1","XM2"],"tie":"vdd"},{"constraint":"IsolatedTub","instances":["XM2"],"tie":"vdd"}]"#,
            &nl,
        )
        .unwrap();
        assert_eq!(d.iter().map(|d| d.kind).collect::<Vec<_>>(), ["sidecar_unsupported"]);
        assert_eq!(cfg.tubs.len(), 1);
    }

    #[test]
    fn a_tub_with_a_pmos_is_refused() {
        let nl = crate::tests::ota();
        let (cfg, d) = parse(r#"[{"constraint":"IsolatedTub","instances":["XM1","XM3"],"tie":"vdd"}]"#, &nl).unwrap();
        assert_eq!(d.iter().map(|d| d.kind).collect::<Vec<_>>(), ["sidecar_unsupported"]);
        assert!(cfg.tubs.is_empty());
    }

    #[test]
    fn unknown_name_is_diagnosed() {
        let nl = crate::tests::ota();
        let (c, d) = parse(r#"[{"constraint":"PowerPorts","ports":["nope"]}]"#, &nl).unwrap();
        assert_eq!(d.iter().map(|d| d.kind).collect::<Vec<_>>(), ["sidecar_unknown_name"]);
        assert!(c.supply_nets.is_empty());
    }

    #[test]
    fn not_json_is_an_error() {
        assert!(parse("{", &crate::tests::ota()).is_err());
        assert!(parse("{}", &crate::tests::ota()).is_err(), "not an array");
    }

    /// The issue's example entry (#78), its four names as `GroupBlocks`
    /// aliases over the OTA's devices.
    const SPEC_EXAMPLE: &str = r#"[
        {"constraint":"GroupBlocks","instances":["XM5"],"instance_name":"cdac"},
        {"constraint":"GroupBlocks","instances":["XM1","XM2"],"instance_name":"comparator"},
        {"constraint":"GroupBlocks","instances":["XM3"],"instance_name":"bias"},
        {"constraint":"GroupBlocks","instances":["XM4"],"instance_name":"sar_logic"},
        {"constraint":"Reference","source":"sar10 v1, gf180mcu, 2025 shuttle","aspect":1.6,
         "rows":[["cdac"],["comparator","bias"],["sar_logic"]],
         "extents":{"cdac":{"x":0.00,"y":0.00,"w":1.00,"h":0.55},
                    "comparator":{"x":0.00,"y":0.55,"w":0.60,"h":0.20},
                    "bias":{"x":0.60,"y":0.55,"w":0.40,"h":0.20},
                    "sar_logic":{"x":0.00,"y":0.75,"w":1.00,"h":0.25}},
         "unit_area":{"cdac":1.0},"area_ratio":1.0}]"#;

    /// Oracle: the values the issue's example writes.
    #[test]
    fn the_issue_example_reference_parses() {
        let nl = crate::tests::ota();
        let (cfg, d) = parse(SPEC_EXAMPLE, &nl).unwrap();
        assert!(d.is_empty(), "{d:?}");
        let r = cfg.reference.expect("a reference");
        assert_eq!((r.entry, r.source.as_str(), r.aspect, r.area_ratio), (4, "sar10 v1, gf180mcu, 2025 shuttle", 1.6, Some(1.0)));
        let ext = |x: f32, y: f32, w: f32, h: f32| analog::intent::Extent { x, y, w, h };
        let got: Vec<_> = r.blocks.iter().map(|b| (b.name.as_str(), b.devices.clone(), b.row, b.extent, b.unit_area)).collect();
        assert_eq!(
            got,
            [
                ("cdac", vec![DeviceId(4)], 0, ext(0.0, 0.0, 1.0, 0.55), Some(1.0)),
                ("comparator", vec![DeviceId(0), DeviceId(1)], 1, ext(0.0, 0.55, 0.6, 0.2), None),
                ("bias", vec![DeviceId(2)], 1, ext(0.6, 0.55, 0.4, 0.2), None),
                ("sar_logic", vec![DeviceId(3)], 2, ext(0.0, 0.75, 1.0, 0.25), None),
            ]
        );
    }

    /// Instance names resolve as aliases do; a name the netlist lacks is a
    /// diagnostic and drops that block only.
    #[test]
    fn a_reference_name_the_netlist_lacks_is_a_diagnostic() {
        let nl = crate::tests::ota();
        let json = r#"[{"constraint":"Reference","aspect":1,"rows":[["XM5"],["nope"]],
            "extents":{"XM5":{"x":0,"y":0,"w":1,"h":0.5},"nope":{"x":0,"y":0.5,"w":1,"h":0.5}}}]"#;
        let (cfg, d) = parse(json, &nl).unwrap();
        assert_eq!(d.iter().map(|d| (d.kind, d.message.as_str())).collect::<Vec<_>>(), [("sidecar_unknown_name", "entry 0 (Reference): unknown name nope")]);
        let r = cfg.reference.expect("the known block stands");
        assert_eq!(r.blocks.iter().map(|b| (b.name.as_str(), b.devices.clone(), b.row)).collect::<Vec<_>>(), [("XM5", vec![DeviceId(4)], 0)]);
    }

    /// Negative space: every shape the floorplan could not honour is refused
    /// with one diagnostic naming the entry, and no reference is kept.
    #[test]
    fn a_malformed_reference_is_refused() {
        let nl = crate::tests::ota();
        let two = r#""extents":{"XM1":{"x":0,"y":0,"w":0.5,"h":1},"XM3":{"x":0.5,"y":0,"w":0.5,"h":1}}"#;
        let cases = [
            ("no aspect", format!(r#""rows":[["XM1","XM3"]],{two}"#)),
            ("aspect 0", format!(r#""aspect":0,"rows":[["XM1","XM3"]],{two}"#)),
            ("no rows", format!(r#""aspect":1,"rows":[],{two}"#)),
            ("a name twice", format!(r#""aspect":1,"rows":[["XM1","XM3"],["XM1"]],{two}"#)),
            ("a name with no extent", r#""aspect":1,"rows":[["XM1"]],"extents":{}"#.to_string()),
            ("an extent no row lists", format!(r#""aspect":1,"rows":[["XM1"]],{two}"#)),
            ("past the outline", r#""aspect":1,"rows":[["XM1"]],"extents":{"XM1":{"x":0.5,"y":0,"w":0.6,"h":1}}"#.to_string()),
            ("zero width", r#""aspect":1,"rows":[["XM1"]],"extents":{"XM1":{"x":0,"y":0,"w":0,"h":1}}"#.to_string()),
            ("overlap in a row", r#""aspect":1,"rows":[["XM1","XM3"]],"extents":{"XM1":{"x":0,"y":0,"w":0.6,"h":1},"XM3":{"x":0.5,"y":0,"w":0.5,"h":1}}"#.to_string()),
            ("right to left", r#""aspect":1,"rows":[["XM3","XM1"]],"extents":{"XM1":{"x":0,"y":0,"w":0.5,"h":1},"XM3":{"x":0.5,"y":0,"w":0.5,"h":1}}"#.to_string()),
            ("rows out of order", r#""aspect":1,"rows":[["XM1"],["XM3"]],"extents":{"XM1":{"x":0,"y":0.5,"w":1,"h":0.5},"XM3":{"x":0,"y":0,"w":1,"h":0.5}}"#.to_string()),
            ("unit_area 0", format!(r#""aspect":1,"rows":[["XM1","XM3"]],{two},"unit_area":{{"XM1":0}}"#)),
            ("unit_area off the rows", format!(r#""aspect":1,"rows":[["XM1","XM3"]],{two},"unit_area":{{"XM5":1}}"#)),
            ("area_ratio -1", format!(r#""aspect":1,"rows":[["XM1","XM3"]],{two},"area_ratio":-1"#)),
            ("a device in two blocks", r#""aspect":1,"rows":[["XM1","xm1"]],"extents":{"XM1":{"x":0,"y":0,"w":0.5,"h":1},"xm1":{"x":0.5,"y":0,"w":0.5,"h":1}}"#.to_string()),
        ];
        for (what, body) in cases {
            let (cfg, d) = parse(&format!(r#"[{{"constraint":"Reference",{body}}}]"#), &nl).unwrap();
            assert_eq!(d.iter().map(|d| d.kind).collect::<Vec<_>>(), ["sidecar_unsupported"], "{what}: {d:?}");
            assert!(d[0].message.starts_with("entry 0 (Reference): "), "{what}: {}", d[0].message);
            assert!(cfg.reference.is_none(), "{what}: kept");
        }
        // A second entry is refused; the first stands.
        let one = format!(r#"{{"constraint":"Reference","aspect":1,"rows":[["XM1","XM3"]],{two}}}"#);
        let (cfg, d) = parse(&format!("[{one},{one}]"), &nl).unwrap();
        assert_eq!(d.iter().map(|d| d.kind).collect::<Vec<_>>(), ["sidecar_unsupported"]);
        assert_eq!(cfg.reference.map(|r| r.entry), Some(0));
    }

    /// `benchmarks/fixtures/dac4.reference.json`, the input of the gate's
    /// first check (it runs at M3, when `dac4` parses), against `dac4`'s
    /// instances as the engine names them: SCIR drops SPICE's element
    /// letter, so `XC0` is `C0`. Oracle: the fixture's own blocks.
    #[test]
    fn the_dac4_reference_resolves() {
        use analog::base::netlist::{Device, DeviceKind};
        let dev = |name: String, kind| Device { name, kind, model: String::new(), terminals: Vec::new(), params: Vec::new() };
        let mut devices: Vec<Device> = (0..5).map(|k| dev(format!("C{k}"), DeviceKind::Capacitor)).collect();
        for k in 0..4 {
            devices.extend([dev(format!("MP{k}"), DeviceKind::Pmos), dev(format!("MN{k}"), DeviceKind::Nmos)]);
        }
        devices.push(dev("MC".into(), DeviceKind::Nmos));
        let nl = Netlist { devices, ..Default::default() };
        let (cfg, d) = parse(include_str!("../../../../benchmarks/fixtures/dac4.reference.json"), &nl).unwrap();
        assert!(d.is_empty(), "{d:?}");
        let r = cfg.reference.expect("a reference");
        assert_eq!(r.blocks.iter().map(|b| (b.name.as_str(), b.row, b.devices.len())).collect::<Vec<_>>(), [("cdac", 0, 5), ("drivers", 0, 8), ("cmp", 1, 1)]);
        assert_eq!((r.aspect, r.area_ratio), (1.506, Some(1.0)));
    }

    /// A user class wins over the role, and an Exceptional Voltage set makes
    /// its compound Perfect.
    #[test]
    fn match_class_overrides_role() {
        let nl = crate::tests::ota();
        let (cfg, _) = parse(r#"[{"constraint":"Match","instances":["XM1","XM2"],"class":"exceptional"}]"#, &nl).unwrap();
        let p = crate::annotate(&nl, &cfg);
        let s = p.intent.sets.iter().find(|s| s.kind == MatchKind::Voltage).expect("the input pair");
        assert_eq!((s.class, s.class_source), (MatchClass::Exceptional, analog::intent::ClassSource::User));
        assert_eq!(p.intent.compounds[s.compound.unwrap() as usize].kind, analog::intent::SymKind::Perfect);
    }
}
