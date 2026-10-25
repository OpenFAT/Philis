//! # `engine` — Philis 2.0's spines (`docs/ARCHITECTURE.md`)
//!
//! Recipe mode: `parse → annotate → plan_recipes → build_tiles → arrange →
//! route → signoff → report`, each step a free function over the tables the
//! one before returned, and [`run`] those lines pasted. M0 holds one set
//! recipe ([`RecipeKind::MosAdjacent`]) and a one-row floorplan; the steps the
//! spine names but M0 does not need yet (`gate_confidence`, `compact`,
//! `score_claims`, `print`) land with the milestone that needs them.

pub mod route;

use std::collections::{HashMap, HashSet};

use analog::base::netlist::{Device, Net, Netlist as Flat};
use analog::base::{DeviceGroup, DeviceId, DeviceKind, Layout, Macro, NetId, Orient, Report, Routes, Shape};
use substrate3::cells::mosfet::Mosfet;
use substrate3::cells::{Cell, Pattern};
use substrate3::scir::{self, ChildId, ParamValue};
use substrate3::spice::{self, Spice};
use verify::reference::{RefDeviceIn, RefKind};
use verify::{LabeledPin, Pdk};

/// A coded refusal (Q18): what could not be done, naming the devices, never a
/// fallback to another spine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// `parse`, `no-recipe`, `reference`.
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

fn refuse(code: &'static str, message: impl Into<String>) -> Refusal {
    Refusal { code, message: message.into() }
}

/// The parsed netlist: the SCIR library (the LVS reference reads it) and the
/// flat device tables the annotator reads, on one net numbering by name.
pub struct Netlist {
    pub scir: scir::Library<Spice>,
    pub flat: Flat,
}

/// Parse `text` (one top `.subckt`, the cell no other instantiates) with the
/// fork's SPICE parser (Q37) and flatten it.
///
/// # Errors
/// A parse or conversion failure, no top cell, or a device the deck has no
/// recogniser for.
pub fn parse(text: &str, pdk: &Pdk) -> Result<Netlist, Refusal> {
    let parsed = spice::parser::Parser::parse(spice::parser::Dialect::Spice, text).map_err(|e| refuse("parse", e.to_string()))?;
    let lib = parsed.to_scir().map_err(|e| refuse("parse", e.to_string()))?;
    let children: HashSet<scir::CellId> = lib
        .cells()
        .flat_map(|(_, c)| c.instances().filter_map(|(_, i)| if let ChildId::Cell(c) = i.child() { Some(c) } else { None }).collect::<Vec<_>>())
        .collect();
    let roots: Vec<scir::CellId> = lib.cells().map(|(id, _)| id).filter(|id| !children.contains(id)).collect();
    let [top] = roots[..] else { return Err(refuse("parse", format!("{} top-level cells; one .subckt must hold the circuit", roots.len()))) };
    let mut b = lib.into_builder();
    b.set_top(top);
    let lib = b.build().map_err(|e| refuse("parse", format!("{e:?}")))?;
    let mut flat = Flat::default();
    let mut ids: HashMap<String, NetId> = HashMap::new();
    let cell = lib.cell(top);
    for p in cell.ports() {
        let name = cell.signal(p.signal()).name.to_string();
        let id = intern(&mut flat, &mut ids, &name);
        flat.ports.push(id);
    }
    flatten(&lib, top, "", &HashMap::new(), pdk, &mut flat, &mut ids)?;
    Ok(Netlist { scir: lib, flat })
}

fn intern(flat: &mut Flat, ids: &mut HashMap<String, NetId>, name: &str) -> NetId {
    *ids.entry(name.to_string()).or_insert_with(|| {
        flat.nets.push(Net { name: name.to_string() });
        NetId((flat.nets.len() - 1) as u16)
    })
}

/// SCIR → flat devices, as `substrate3::signoff::reference` walks it for LVS.
/// ponytail: scalar signals only; a bus port refuses until a fixture has one.
fn flatten(
    lib: &scir::Library<Spice>,
    id: scir::CellId,
    prefix: &str,
    bound: &HashMap<String, String>,
    pdk: &Pdk,
    flat: &mut Flat,
    ids: &mut HashMap<String, NetId>,
) -> Result<(), Refusal> {
    let cell = lib.cell(id);
    let net_of = |inst: &scir::Instance, port: &str| -> Result<String, Refusal> {
        let c = inst.connections().get(port).ok_or_else(|| refuse("parse", format!("{prefix}{}: port `{port}` unconnected", inst.name())))?;
        let mut parts = c.parts();
        match (parts.next(), parts.next()) {
            (Some(s), None) if s.range().is_none() => {
                let name = cell.signal(s.signal()).name.to_string();
                Ok(bound.get(&name).cloned().unwrap_or_else(|| format!("{prefix}{name}")))
            }
            _ => Err(refuse("parse", format!("{prefix}{}.{port}: a bus connection", inst.name()))),
        }
    };
    for (_, inst) in cell.instances() {
        match inst.child() {
            ChildId::Cell(child) => {
                let c = lib.cell(child);
                let mut inner = HashMap::new();
                for p in c.ports() {
                    let name = c.signal(p.signal()).name.to_string();
                    inner.insert(name.clone(), net_of(inst, &name)?);
                }
                flatten(lib, child, &format!("{prefix}{}/", inst.name()), &inner, pdk, flat, ids)?;
            }
            ChildId::Primitive(pid) => {
                let (model, ports, params) = match lib.primitive(pid) {
                    spice::Primitive::RawInstance { ports, cell, params } | spice::Primitive::RawInstanceWithCell { ports, cell, params, .. } => {
                        (cell.to_string(), ports.iter().map(ToString::to_string).collect::<Vec<_>>(), params)
                    }
                    spice::Primitive::Mos { model, params } => (model.to_string(), ["D", "G", "S", "B"].map(String::from).to_vec(), params),
                    other => return Err(refuse("parse", format!("{prefix}{}: {other:?} is not a device M0 reads", inst.name()))),
                };
                let kind = match pdk.model_kind(&model) {
                    Some(RefKind::Nmos) => DeviceKind::Nmos,
                    Some(RefKind::Pmos) => DeviceKind::Pmos,
                    Some(k) => return Err(refuse("parse", format!("{prefix}{}: {k:?} `{model}`: M0 reads MOS only", inst.name()))),
                    None => return Err(refuse("parse", format!("{prefix}{}: no deck recogniser for `{model}`", inst.name()))),
                };
                if ports.len() != 4 {
                    return Err(refuse("parse", format!("{prefix}{}: a MOS has 4 terminals, `{model}` lists {}", inst.name(), ports.len())));
                }
                // Stored G, D, S, B, as 1.0's parser does.
                let mut terminals = Vec::with_capacity(4);
                for (term, i) in [("G", 1), ("D", 0), ("S", 2), ("B", 3)] {
                    let name = net_of(inst, &ports[i])?;
                    terminals.push((term.to_string(), intern(flat, ids, &name)));
                }
                let num = |k: &str| {
                    params.iter().find(|(n, _)| n.as_ref().eq_ignore_ascii_case(k)).and_then(|(_, v)| match v {
                        ParamValue::Numeric(d) => d.to_string().parse::<f64>().ok(),
                        ParamValue::String(s) => s.parse::<f64>().ok(),
                    })
                };
                // A SPICE length: `>= 0.01` is bare micrometres, smaller is metres.
                let nm = |v: f64| (if v.abs() >= 0.01 { v * 1e3 } else { v * 1e9 }).round() as i64;
                let mut p: Vec<(String, i64)> = Vec::new();
                p.extend(num("w").map(|v| ("w".to_string(), nm(v))));
                p.extend(num("l").map(|v| ("l".to_string(), nm(v))));
                p.extend(num("nf").map(|v| ("nf".to_string(), v.round() as i64)));
                p.extend(num("m").map(|v| ("m".to_string(), v.round() as i64)));
                flat.devices.push(Device {
                    name: format!("{prefix}{}", inst.name()),
                    kind,
                    model: pdk.deck_model(&model).unwrap_or(model),
                    terminals,
                    params: p,
                });
            }
        }
    }
    Ok(())
}

/// The annotator's configuration for `pdk`: the deck numbers it reads.
#[must_use]
pub fn annotation(pdk: &Pdk) -> annotator::AnnotationConfig {
    route::annotation(pdk, &annotator::AnnotationConfig::default())
}

/// Recipe kinds (S1: a closed enum, matched exhaustively).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RecipeKind {
    /// Members as compact blocks side by side, same orientation, no
    /// interleave (`RECIPES.md` §2.1; 1.0 `Mosfet` style `Single`).
    MosAdjacent,
}

/// The recipe call table (Q32), SoA: one row per call; `members[start..start
/// + count]` are its devices. M0's calls are all roots of one row floorplan.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecipeCalls {
    pub kind: Vec<RecipeKind>,
    pub member_start: Vec<u32>,
    pub member_count: Vec<u16>,
    /// The annotator's unitization row the call draws, if it set one.
    pub unitization: Vec<Option<u16>>,
    /// 0 = the ladder's first choice (Q19); M0 has one rung.
    pub ladder_step: Vec<u8>,
    pub members: Vec<DeviceId>,
}

impl RecipeCalls {
    #[must_use]
    pub fn len(&self) -> usize {
        self.kind.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kind.is_empty()
    }
    /// Call `i`'s devices.
    #[must_use]
    pub fn members_of(&self, i: usize) -> &[DeviceId] {
        let s = self.member_start[i] as usize;
        &self.members[s..s + usize::from(self.member_count[i])]
    }
    fn push(&mut self, kind: RecipeKind, members: &[DeviceId], unitization: Option<u16>) {
        self.kind.push(kind);
        self.member_start.push(self.members.len() as u32);
        self.member_count.push(members.len() as u16);
        self.unitization.push(unitization);
        self.ladder_step.push(0);
        self.members.extend_from_slice(members);
    }
}

/// One call per unitization row (a matched set), then one per MOS left over.
/// Every device lands in exactly one call, or the plan refuses.
///
/// # Errors
/// A device no recipe draws (M0: anything but a MOS).
pub fn plan_recipes(problem: &annotator::Problem, flat: &Flat) -> Result<RecipeCalls, Refusal> {
    let mut calls = RecipeCalls::default();
    let mut taken = vec![false; flat.devices.len()];
    for (i, u) in problem.constraints.unitization.iter().enumerate() {
        if u.devices.iter().any(|d| taken[d.0 as usize]) || !matches!(u.device_type, DeviceKind::Nmos | DeviceKind::Pmos) {
            continue;
        }
        u.devices.iter().for_each(|d| taken[d.0 as usize] = true);
        calls.push(RecipeKind::MosAdjacent, &u.devices, Some(i as u16));
    }
    for (i, d) in flat.devices.iter().enumerate().filter(|(i, _)| !taken[*i]) {
        match d.kind {
            DeviceKind::Nmos | DeviceKind::Pmos => calls.push(RecipeKind::MosAdjacent, &[DeviceId(i as u16)], None),
            k => return Err(refuse("no-recipe", format!("{}: no M0 recipe draws a {k:?}", d.name))),
        }
    }
    Ok(calls)
}

/// One drawn tile per call, its pins bound to the netlist's nets.
///
/// # Errors
/// A call no variant of its recipe can draw.
pub fn build_tiles(calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat, pdk: &Pdk) -> Result<Vec<Macro>, Refusal> {
    let ground = problem.net_classes.iter().find(|c| c.class == analog::metadata::NetClass::Ground).map(|c| c.net);
    (0..calls.len())
        .map(|i| {
            let members = calls.members_of(i);
            let group = DeviceGroup { devices: members.to_vec() };
            let constraints = match calls.unitization[i] {
                Some(_) => problem.constraints.clone(),
                None => single(&flat.devices[members[0].0 as usize], members[0]),
            };
            let model = &flat.devices[members[0].0 as usize].model;
            let overlay = pdk.mos_overlay(model);
            let process: &dyn analog::base::Process = overlay.as_ref().map_or(pdk as &dyn analog::base::Process, |o| o);
            let mut m = match calls.kind[i] {
                RecipeKind::MosAdjacent => Mosfet::enumerate(&group, &constraints, process)
                    .into_iter()
                    .filter(|v| v.style == Pattern::Single && v.rows == 1)
                    .map(|v| v.draw(&group, &constraints, process))
                    .min_by_key(|m| i64::from(m.bbox.w) * i64::from(m.bbox.h))
                    .ok_or_else(|| refuse("no-recipe", format!("{}: no MosAdjacent variant draws it", names(flat, members))))?,
            };
            bind_pins(&mut m, flat, members, ground);
            Ok(m)
        })
        .collect()
}

fn names(flat: &Flat, members: &[DeviceId]) -> String {
    members.iter().map(|d| flat.devices[d.0 as usize].name.as_str()).collect::<Vec<_>>().join(", ")
}

/// The one-device unitization a lone MOS draws from (as `substrate3`'s
/// adapter synthesises it): its schematic fingers at their own width.
fn single(d: &Device, id: DeviceId) -> analog::Constraints {
    let s = d.mos_size();
    let nf = s.map_or(1, |s| s.fingers().clamp(1, u32::from(u16::MAX)) as u16);
    analog::Constraints {
        unitization: vec![analog::cell::Unitization {
            devices: vec![id],
            device_type: d.kind,
            dev_nf: vec![nf],
            target_ratio: vec![1],
            unit_w: s.map_or(0, |s| s.w_finger_nm() as i32),
            unit_l: s.map_or(0, |s| s.l_nm as i32),
            series_parallel: analog::cell::SeriesParallel::Parallel,
            dummy_required: false,
            route_matching_required: false,
            class: None,
            kind: None,
            series: Vec::new(),
            style: None,
        }],
        ..analog::Constraints::default()
    }
}

/// Rebind a drawn macro's pins to the netlist (1.0 `cellgen::bind_pins`):
/// `d{k}:T` is terminal `T` of `members[k]`; a bare `GND` (a dummy tie) is
/// the classified ground, else member 0's `S`.
fn bind_pins(m: &mut Macro, flat: &Flat, members: &[DeviceId], ground: Option<NetId>) {
    let terminal = |d: &Device, t: &str| d.terminals.iter().find(|(n, _)| n == t).map(|&(_, n)| n);
    for pin in &mut m.pins {
        let (k, term) = match pin.name.split_once(':') {
            Some((d, t)) => (d.strip_prefix('d').and_then(|s| s.parse::<usize>().ok()).unwrap_or(0), t),
            None => (0, pin.name.as_str()),
        };
        let dev = members.get(k).and_then(|d| flat.devices.get(d.0 as usize));
        let net = if pin.name == "GND" { ground.or_else(|| dev.and_then(|d| terminal(d, "S"))) } else { dev.and_then(|d| terminal(d, term)) };
        if let Some(net) = net {
            pin.net = net;
        }
    }
}

/// The floorplan: one row, left to right in call order, `gap` apart, as a
/// [`Layout`] the router's `place_macros` stamps. `gap`: the widest spacing
/// among the layers the tiles draw, so neighbours never interact.
/// ponytail: one row; the symmetric row stack is M2's floorplan recipe.
#[must_use]
pub fn arrange(tiles: &[Macro], pdk: &Pdk) -> Layout {
    let mut layers: Vec<u16> = tiles.iter().flat_map(|m| m.shapes.iter().map(|s| s.layer.0)).collect();
    layers.sort_unstable();
    layers.dedup();
    let gap = layers.iter().filter_map(|&l| pdk.min_spacing(l)).max().unwrap_or(0);
    // Tile origins on the cut lattice, as the cells draw their cuts.
    let lattice = substrate3::cells::builder::cut_lattice(pdk).max(1);
    let n = tiles.len();
    let mut x = Vec::with_capacity(n);
    let mut cursor = 0;
    for m in tiles {
        // `place_macro` puts the lower-left at `x − hw`: here, `cursor`.
        x.push(cursor + m.bbox.w / 2);
        cursor += (m.bbox.w + gap + lattice - 1) / lattice * lattice;
    }
    Layout {
        x,
        y: tiles.iter().map(|m| m.bbox.h / 2).collect(),
        hw: tiles.iter().map(|m| m.bbox.w / 2).collect(),
        hh: tiles.iter().map(|m| m.bbox.h / 2).collect(),
        axis: vec![0],
        groups: Vec::new(),
        orient: vec![Orient::default(); n],
        variant: vec![0; n],
        branch: Vec::new(),
        power_uw: vec![0; n],
        temp_mc: vec![0; n],
        units: Default::default(),
    }
}

/// Everything signoff reads, and its verdict.
pub struct Outcome {
    pub calls: RecipeCalls,
    pub placed: Vec<Macro>,
    pub routes: Routes,
    pub route_report: Report,
    pub signoff: verify::Signoff,
    pub report: String,
}

/// Flat geometry: every placed tile's shapes, then every routed wire.
#[must_use]
pub fn geometry(placed: &[Macro], routes: &Routes) -> Vec<Shape> {
    placed.iter().flat_map(|m| m.shapes.iter().copied()).chain(routes.wires.iter().flatten().copied()).collect()
}

/// DRC, ERC, LVS and PEX of the routed layout against the schematic, in
/// process. Labels: one per net, on the first pin that sits on drawn metal.
///
/// # Errors
/// The schematic's LVS reference cannot be built.
pub fn signoff(netlist: &Netlist, placed: &[Macro], routes: &Routes, pdk: &Pdk) -> Result<verify::Signoff, Refusal> {
    let shapes = geometry(placed, routes);
    let names: Vec<String> = netlist.flat.nets.iter().map(|n| n.name.clone()).collect();
    let pins = labeled_pins(placed, &names, pdk, &shapes);
    let mut reference = substrate3::signoff::reference(&netlist.scir, pdk).map_err(|e| refuse("reference", e.to_string()))?;
    reference.devices.extend(dummy_cards(placed, &names, &reference.devices));
    reference.external_ports = Some(reference.ports.clone());
    reference.ports = pins.iter().map(|p| p.name.clone()).collect();
    Ok(verify::signoff_checked(&shapes, &pins, &reference, &verify::Intent::default(), pdk))
}

/// One label per net (1.0 `labeled_pins`): the first placed pin on a label
/// layer whose centre lies on drawn conductor of that layer.
fn labeled_pins(placed: &[Macro], nets: &[String], pdk: &Pdk, shapes: &[Shape]) -> Vec<LabeledPin> {
    let mut out: Vec<LabeledPin> = Vec::new();
    let mut labelled: Vec<u16> = Vec::new();
    // ponytail: O(pins × shapes), once per signoff.
    for p in placed.iter().flat_map(|m| &m.pins) {
        if labelled.contains(&p.net.0) || verify::geom::label_layer(&pdk.deck, p.layer.0).is_none() {
            continue;
        }
        let (x, y) = (p.at.x + p.at.w / 2, p.at.y + p.at.h / 2);
        let on_drawn = shapes.iter().any(|s| s.layer == p.layer && (s.rect.x..=s.rect.x + s.rect.w).contains(&x) && (s.rect.y..=s.rect.y + s.rect.h).contains(&y));
        let Some(name) = nets.get(p.net.0 as usize).filter(|_| on_drawn) else { continue };
        labelled.push(p.net.0);
        out.push(LabeledPin { name: name.clone(), layer: p.layer.0, x, y });
    }
    out
}

/// One LVS card per dummy gate a tile drew (1.0 `cellgen::dummy_cards`): the
/// near side on its owner's `S`/`D` net, gate, far side and body on `B`.
fn dummy_cards(placed: &[Macro], nets: &[String], schematic: &[RefDeviceIn]) -> Vec<RefDeviceIn> {
    let sized = schematic.iter().any(|c| matches!(c.kind, RefKind::Nmos | RefKind::Pmos) && !c.params.is_empty());
    let mut out = Vec::new();
    for m in placed {
        let net = |owner: u8, t: &str| {
            let name = format!("d{owner}:{t}");
            m.pins.iter().find(|p| p.name == name).and_then(|p| nets.get(p.net.0 as usize)).cloned()
        };
        for d in &m.dummies {
            let (Some(near), Some(bulk)) = (net(d.owner, d.edge), net(d.owner, "B")) else { continue };
            out.push(RefDeviceIn {
                kind: if d.pmos { RefKind::Pmos } else { RefKind::Nmos },
                model: None,
                terminals: vec![near, bulk.clone(), bulk.clone(), bulk],
                params: if sized { vec![("w".into(), f64::from(d.w) * 1e-9), ("l".into(), f64::from(d.l) * 1e-9)] } else { Vec::new() },
            });
        }
    }
    out
}

/// The report: one line per call, then the signoff verdict.
#[must_use]
pub fn report(calls: &RecipeCalls, flat: &Flat, signoff: &verify::Signoff) -> String {
    let mut s = String::new();
    for i in 0..calls.len() {
        s.push_str(&format!("call {i}: {:?} [{}] ladder step {}\n", calls.kind[i], names(flat, calls.members_of(i)), calls.ladder_step[i]));
    }
    let hard = &signoff.report.hard_violations;
    s.push_str(&format!("signoff: {} hard violations\n", hard.len()));
    for v in hard {
        s.push_str(&format!("  {} ({})\n", v.rule, v.margin));
    }
    s
}

/// Recipe mode, the whole spine.
///
/// # Errors
/// Any step's [`Refusal`].
pub fn run(spice: &str, pdk: &Pdk) -> Result<Outcome, Refusal> {
    let netlist = parse(spice, pdk)?;
    let cfg = annotation(pdk);
    let problem = annotator::annotate(&netlist.flat, &cfg);
    let calls = plan_recipes(&problem, &netlist.flat)?;
    let tiles = build_tiles(&calls, &problem, &netlist.flat, pdk)?;
    let layout = arrange(&tiles, pdk);
    let (placed, routes, route_report) = route::route(&tiles, &layout, &problem, &cfg, netlist.flat.nets.len(), pdk);
    let signoff = signoff(&netlist, &placed, &routes, pdk)?;
    let report = report(&calls, &netlist.flat, &signoff);
    Ok(Outcome { calls, placed, routes, route_report, signoff, report })
}
