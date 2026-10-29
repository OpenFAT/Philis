//! # `engine` — Philis 2.0's spines (`docs/ARCHITECTURE.md`)
//!
//! Recipe mode: `parse → annotate → plan_recipes → build_tiles → arrange →
//! route → signoff → report`, each step a free function over the tables the
//! one before returned, and [`run_with`] those lines pasted. M0 holds one set
//! recipe ([`RecipeKind::MosAdjacent`]) and a one-row floorplan, or the
//! sidecar `Reference`'s floorplan ([`arrange_reference`]); the steps the
//! spine names but M0 does not need yet (`gate_confidence`, `compact`,
//! `score_claims`, `print`) land with the milestone that needs them.

pub mod route;

use std::collections::{HashMap, HashSet};

use analog::base::netlist::{Device, Net, Netlist as Flat};
use analog::base::{DeviceGroup, DeviceId, DeviceKind, Layout, Macro, NetId, Orient, Report, Routes, Shape};
use substrate3::cells::finfet::FinFet;
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
    /// `parse`, `sidecar`, `no-recipe`, `floorplan`, `floorplan/reference`,
    /// `reference` (the LVS reference), or a foundry engine's `foundry/…`
    /// ([`verify::Refused`]).
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

/// [`annotation`] plus the sidecar's entries (Q7), their names resolved
/// against `netlist` (Q31); every entry-level problem is a diagnostic the
/// intent carries. `[]` gives exactly [`annotation`].
///
/// # Errors
/// `sidecar`: text that is not JSON, or not a top-level array.
pub fn annotation_with(pdk: &Pdk, netlist: &Netlist, sidecar: &str) -> Result<annotator::AnnotationConfig, Refusal> {
    let (mut base, diags) = annotator::AnnotationConfig::from_json(sidecar, &netlist.flat).map_err(|e| refuse("sidecar", e))?;
    base.sidecar_diags = diags;
    Ok(route::annotation(pdk, &base))
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
    /// Append a call at ladder step 0. `unitization`: the annotator's row it
    /// draws; `None` draws the members at their schematic fingers.
    pub fn push(&mut self, kind: RecipeKind, members: &[DeviceId], unitization: Option<u16>) {
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
                None => as_drawn(flat, members),
            };
            let model = &flat.devices[members[0].0 as usize].model;
            let overlay = pdk.mos_overlay(model);
            let process: &dyn analog::base::Process = overlay.as_ref().map_or(pdk as &dyn analog::base::Process, |o| o);
            // A fin deck draws fins (each member on its own active), any
            // other the planar block's `Single` style: members side by side.
            let drawn: Vec<Macro> = match calls.kind[i] {
                RecipeKind::MosAdjacent if process.layer("fin").is_some() => FinFet::enumerate(&group, &constraints, process)
                    .into_iter()
                    .filter(|v| !v.shared)
                    .map(|v| v.draw(&group, &constraints, process))
                    .collect(),
                // Blocks may abut on one shared diffusion only where every
                // member has the same source and bulk (the block style's
                // boundaries sit on sources); else each member is its own
                // block, side by side.
                RecipeKind::MosAdjacent if common(flat, members, "S") && common(flat, members, "B") => {
                    Mosfet::enumerate(&group, &constraints, process)
                        .into_iter()
                        .filter(|v| v.style == Pattern::Single && v.rows == 1)
                        .map(|v| v.draw(&group, &constraints, process))
                        .chain(apart(members, &constraints, process, pdk))
                        .collect()
                }
                RecipeKind::MosAdjacent => apart(members, &constraints, process, pdk).into_iter().collect(),
            };
            let mut m = drawn
                .into_iter()
                .min_by_key(|m| i64::from(m.bbox.w) * i64::from(m.bbox.h))
                .ok_or_else(|| refuse("no-recipe", format!("{}: no {:?} variant draws it", names(flat, members), calls.kind[i])))?;
            bind_pins(&mut m, flat, members, ground);
            join_members(&mut m, pdk);
            Ok(m)
        })
        .collect()
}

/// Whether every member's terminal `t` is on one net.
fn common(flat: &Flat, members: &[DeviceId], t: &str) -> bool {
    let net = |d: &DeviceId| flat.devices[d.0 as usize].terminals.iter().find(|(n, _)| n == t).map(|&(_, n)| n);
    members.windows(2).all(|w| net(&w[0]) == net(&w[1]))
}

/// The members as separate planar blocks in one row, left to right, the
/// widest drawn-layer spacing apart: member `k`'s pins renamed `d{k}:`.
/// `None` when a member has no single-block variant.
fn apart(members: &[DeviceId], c: &analog::Constraints, process: &dyn analog::base::Process, pdk: &Pdk) -> Option<Macro> {
    let u = c.unitization.iter().find(|u| u.devices == members)?;
    let lattice = substrate3::cells::builder::cut_lattice(process).max(1);
    let mut row = Macro::default();
    let mut cursor = 0;
    let mut first = true;
    for (k, &d) in members.iter().enumerate() {
        let group = DeviceGroup { devices: vec![d] };
        let one = analog::Constraints {
            unitization: vec![analog::cell::Unitization { devices: vec![d], dev_nf: vec![u.dev_nf[k]], target_ratio: vec![1], ..u.clone() }],
            ..analog::Constraints::default()
        };
        let m = Mosfet::enumerate(&group, &one, process)
            .into_iter()
            .filter(|v| v.style == Pattern::Single && v.rows == 1)
            .map(|v| v.draw(&group, &one, process))
            .min_by_key(|m| i64::from(m.bbox.w) * i64::from(m.bbox.h))?;
        // Left edge on `cursor`, on the cut lattice.
        let dx = cursor - m.bbox.x;
        let shift = |r: analog::base::Rect| analog::base::Rect { x: r.x + dx, ..r };
        row.shapes.extend(m.shapes.iter().map(|s| Shape { layer: s.layer, rect: shift(s.rect) }));
        row.pins.extend(m.pins.iter().map(|p| analog::base::Pin {
            name: p.name.replacen("d0:", &format!("d{k}:"), 1),
            at: shift(p.at),
            ..p.clone()
        }));
        row.units.extend(m.units.iter().map(|un| analog::base::Unit { owner: k as u8, x: un.x + dx, ..*un }));
        row.dummies.extend(m.dummies.iter().map(|du| analog::base::Dummy { owner: k as u8, ..*du }));
        let b = shift(m.bbox);
        row.bbox = if first { b } else { union(row.bbox, b) };
        first = false;
        cursor = b.x + b.w + widest_spacing(std::slice::from_ref(&m), pdk);
        cursor = (cursor + lattice - 1).div_euclid(lattice) * lattice;
    }
    Some(row)
}

fn union(a: analog::base::Rect, b: analog::base::Rect) -> analog::base::Rect {
    let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
    let (x1, y1) = ((a.x + a.w).max(b.x + b.w), (a.y + a.h).max(b.y + b.h));
    analog::base::Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
}

/// The widest spacing among the layers `tiles` draw: neighbours that far
/// apart never interact.
fn widest_spacing(tiles: &[Macro], pdk: &Pdk) -> i32 {
    let mut layers: Vec<u16> = tiles.iter().flat_map(|m| m.shapes.iter().map(|s| s.layer.0)).collect();
    layers.sort_unstable();
    layers.dedup();
    layers.iter().filter_map(|&l| pdk.min_spacing(l)).max().unwrap_or(0)
}

fn names(flat: &Flat, members: &[DeviceId]) -> String {
    members.iter().map(|d| flat.devices[d.0 as usize].name.as_str()).collect::<Vec<_>>().join(", ")
}

/// The unitization a call with no annotator row draws (as `substrate3`'s
/// adapter synthesises one): every member at its schematic fingers and the
/// first member's finger size. ponytail: members of unequal size draw at the
/// first's; the annotator's rows carry real ratios.
fn as_drawn(flat: &Flat, members: &[DeviceId]) -> analog::Constraints {
    let first = &flat.devices[members[0].0 as usize];
    let s = first.mos_size();
    let nf = |d: &DeviceId| flat.devices[d.0 as usize].mos_size().map_or(1, |s| s.fingers().clamp(1, u32::from(u16::MAX)) as u16);
    analog::Constraints {
        unitization: vec![analog::cell::Unitization {
            devices: members.to_vec(),
            device_type: first.kind,
            dev_nf: members.iter().map(nf).collect(),
            target_ratio: vec![1; members.len()],
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

/// Union-find over a macro's shapes: shapes on one layer that touch share an
/// island. `islands[i]` is shape `i`'s root.
#[must_use]
pub fn islands(shapes: &[Shape]) -> Vec<usize> {
    fn find(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    let mut p: Vec<usize> = (0..shapes.len()).collect();
    for i in 0..shapes.len() {
        for j in i + 1..shapes.len() {
            if shapes[i].layer == shapes[j].layer && touch(shapes[i].rect, shapes[j].rect) {
                let (a, b) = (find(&mut p, i), find(&mut p, j));
                p[a] = b;
            }
        }
    }
    (0..shapes.len()).map(|i| find(&mut p, i)).collect()
}

fn touch(a: analog::base::Rect, b: analog::base::Rect) -> bool {
    a.x <= b.x + b.w && b.x <= a.x + a.w && a.y <= b.y + b.h && b.y <= a.y + a.h
}

/// The island a pin sits on: the first shape on its layer it touches.
#[must_use]
pub fn pin_island(p: &analog::base::Pin, shapes: &[Shape], islands: &[usize]) -> Option<usize> {
    shapes.iter().position(|s| s.layer == p.layer && touch(s.rect, p.at)).map(|i| islands[i])
}

/// The set recipe draws the joins among its own members (`TRAITS.md`): two
/// islands of one net on one layer whose shapes face each other across a
/// straight gap (overlapping on the other axis by the layer's width) get a
/// bar across it, the layer's width wide, unless the bar comes within the
/// layer's spacing of a third island. Whatever stays apart goes to the router.
fn join_members(m: &mut Macro, pdk: &Pdk) {
    use analog::base::Rect;
    let pins = m.pins.clone();
    for (n, a) in pins.iter().enumerate() {
        for b in &pins[n + 1..] {
            if a.net != b.net || a.layer != b.layer {
                continue;
            }
            let isl = islands(&m.shapes);
            let (Some(ia), Some(ib)) = (pin_island(a, &m.shapes, &isl), pin_island(b, &m.shapes, &isl)) else { continue };
            if ia == ib {
                continue;
            }
            let layer = a.layer;
            let (w, space) = (pdk.min_width(layer.0).unwrap_or(1).max(1), pdk.min_spacing(layer.0).unwrap_or(0));
            let of = |i: usize| m.shapes.iter().zip(&isl).filter(move |(s, &k)| s.layer == layer && k == i).map(|(s, _)| s.rect);
            let near = |bar: Rect, r: Rect| {
                let dx = (r.x - (bar.x + bar.w)).max(bar.x - (r.x + r.w));
                let dy = (r.y - (bar.y + bar.h)).max(bar.y - (r.y + r.h));
                dx.max(dy) < space
            };
            let clear = |bar: Rect| m.shapes.iter().zip(&isl).all(|(s, &k)| s.layer != layer || k == ia || k == ib || !near(bar, s.rect));
            // Every straight bar between a shape of each island, shortest first.
            let mut bars: Vec<Rect> = Vec::new();
            for p in of(ia) {
                for q in of(ib) {
                    let (lo, hi) = (p.x.max(q.x), (p.x + p.w).min(q.x + q.w));
                    if hi - lo >= w {
                        let (y0, y1) = if p.y < q.y { (p.y + p.h, q.y) } else { (q.y + q.h, p.y) };
                        bars.push(Rect { x: lo, y: y0, w, h: y1 - y0 });
                    }
                    let (lo, hi) = (p.y.max(q.y), (p.y + p.h).min(q.y + q.h));
                    if hi - lo >= w {
                        let (x0, x1) = if p.x < q.x { (p.x + p.w, q.x) } else { (q.x + q.w, p.x) };
                        bars.push(Rect { x: x0, y: lo, w: x1 - x0, h: w });
                    }
                }
            }
            bars.retain(|r| r.w > 0 && r.h > 0);
            bars.sort_by_key(|r| r.w.max(r.h));
            if let Some(bar) = bars.into_iter().find(|&r| clear(r)) {
                m.shapes.push(Shape { layer, rect: bar });
            }
        }
    }
}

/// The floorplan: one row, left to right in call order, `gap` apart, as a
/// [`Layout`] the router's `place_macros` stamps. `gap`: the widest spacing
/// among the layers the tiles draw, so neighbours never interact.
/// ponytail: one row; the symmetric row stack is M2's floorplan recipe.
///
/// # Errors
/// Several tiles on a deck with fins: exact gate and fin pitches (ASAP7
/// GATE.S.1, FIN.S.1) need the gate-lattice glue row between tiles (M2).
pub fn arrange(tiles: &[Macro], pdk: &Pdk) -> Result<Layout, Refusal> {
    one_tile_on_fins(tiles, pdk)?;
    let gap = widest_spacing(tiles, pdk);
    // Tile origins on the cut lattice, as the cells draw their cuts.
    let lattice = substrate3::cells::builder::cut_lattice(pdk).max(1);
    let mut x = Vec::with_capacity(tiles.len());
    let mut cursor = 0;
    for m in tiles {
        x.push(cursor);
        cursor += (m.bbox.w + gap + lattice - 1) / lattice * lattice;
    }
    Ok(layout_at(tiles, &x, &vec![0; tiles.len()]))
}

/// The reference floorplan (#78, [`placement::reference`]): each tile in
/// its block's target box, the tiles of no block in a row above the
/// outline, `gap` as [`arrange`]'s. Returns the layout, which block each
/// tile took (and the conflicts that degraded), and the fit.
///
/// # Errors
/// `floorplan/reference`: the reference splits a common-centroid, Precise or
/// User matched set or contradicts a user `Order` (each named with the
/// entry), or its extents are too small for their tiles; `floorplan` as
/// [`arrange`].
pub fn arrange_reference(
    reference: &analog::intent::Reference,
    calls: &RecipeCalls,
    tiles: &[Macro],
    problem: &annotator::Problem,
    flat: &Flat,
    pdk: &Pdk,
) -> Result<(Layout, placement::reference::Assignment, placement::reference::Fit), Refusal> {
    assert_eq!(calls.len(), tiles.len(), "one tile per call");
    one_tile_on_fins(tiles, pdk)?;
    let names: Vec<&str> = flat.devices.iter().map(|d| d.name.as_str()).collect();
    let members: Vec<&[DeviceId]> = (0..calls.len()).map(|i| calls.members_of(i)).collect();
    let refused = |why: String| refuse("floorplan/reference", why);
    let assignment = placement::reference::assign(reference, &problem.intent, &members, &names).map_err(refused)?;
    let sizes: Vec<(i32, i32)> = tiles.iter().map(|m| (m.bbox.w, m.bbox.h)).collect();
    let lattice = substrate3::cells::builder::cut_lattice(pdk).max(1);
    let fit = placement::reference::fit(reference, &assignment.block, &sizes, widest_spacing(tiles, pdk), lattice).map_err(refused)?;
    Ok((layout_at(tiles, &fit.x, &fit.y), assignment, fit))
}

/// [`arrange`]'s refusal: exact gate and fin pitches need M2's glue row
/// between tiles, so a fin deck takes one tile.
fn one_tile_on_fins(tiles: &[Macro], pdk: &Pdk) -> Result<(), Refusal> {
    if tiles.len() > 1 && analog::base::Process::layer(pdk, "fin").is_some() {
        return Err(refuse("floorplan", format!("{} tiles on a fin deck: a row needs the gate-lattice glue row (M2)", tiles.len())));
    }
    Ok(())
}

/// A [`Layout`] with tile `i`'s lower-left corner at `(x[i], y[i])`:
/// `place_macro` puts it at `x − hw`, `y − hh`.
fn layout_at(tiles: &[Macro], x: &[i32], y: &[i32]) -> Layout {
    assert!(x.len() == tiles.len() && y.len() == tiles.len(), "one corner per tile");
    let n = tiles.len();
    Layout {
        x: tiles.iter().zip(x).map(|(m, &x)| x + m.bbox.w / 2).collect(),
        y: tiles.iter().zip(y).map(|(m, &y)| y + m.bbox.h / 2).collect(),
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
    /// The sidecar `Reference`'s floorplan, which block each call took and
    /// where ([`arrange_reference`]); `None` without a reference.
    pub reference: Option<(placement::reference::Assignment, placement::reference::Fit)>,
}

/// Flat geometry: every placed tile's shapes, then every routed wire.
#[must_use]
pub fn geometry(placed: &[Macro], routes: &Routes) -> Vec<Shape> {
    placed.iter().flat_map(|m| m.shapes.iter().copied()).chain(routes.wires.iter().flatten().copied()).collect()
}

/// Sign the routed layout off against the schematic with `engine`
/// ([`verify::SignoffEngine`]: GPurify in process, or the foundry's KLayout
/// or magic/netgen decks). Labels: one per net, on the first pin that sits
/// on drawn metal.
///
/// # Errors
/// The schematic's LVS reference cannot be built, or a foundry engine
/// refused (no tool, no deck for this PDK, no verdict): its code is kept.
pub fn signoff(netlist: &Netlist, placed: &[Macro], routes: &Routes, pdk: &Pdk, engine: verify::SignoffEngine) -> Result<verify::Signoff, Refusal> {
    let shapes = geometry(placed, routes);
    let names: Vec<String> = netlist.flat.nets.iter().map(|n| n.name.clone()).collect();
    let pins = labeled_pins(placed, &names, pdk, &shapes);
    let mut reference = substrate3::signoff::reference(&netlist.scir, pdk).map_err(|e| refuse("reference", e.to_string()))?;
    reference.devices.extend(dummy_cards(placed, &names, &reference.devices));
    reference.external_ports = Some(reference.ports.clone());
    reference.ports = pins.iter().map(|p| p.name.clone()).collect();
    verify::foundry::signoff(engine, &shapes, &pins, &reference, &verify::Intent::default(), pdk)
        .map_err(|r| Refusal { code: r.code, message: r.message })
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
        let on_drawn = shapes
            .iter()
            .any(|s| s.layer == p.layer && (s.rect.x..=s.rect.x + s.rect.w).contains(&x) && (s.rect.y..=s.rect.y + s.rect.h).contains(&y));
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

/// The report's reference section (#78): the reference quoted, its aspect
/// against the outline's and the drawn layout's (tiles and routes), each
/// block's target extent against the box its tiles drew, the calls outside
/// the reference, the targets M0 does not apply, and every conflict that
/// degraded. Fractions are of the outline, from its lower-left corner.
#[must_use]
pub fn reference_report(
    r: &analog::intent::Reference,
    a: &placement::reference::Assignment,
    fit: &placement::reference::Fit,
    placed: &[Macro],
    routes: &Routes,
) -> String {
    use std::fmt::Write as _;
    let (w, h) = (f64::from(fit.outline.0.max(1)), f64::from(fit.outline.1.max(1)));
    let pct = |got: f64, want: f64| 100.0 * (got / want - 1.0);
    let aspect = f64::from(r.aspect);
    let mut s = format!("reference: sidecar entry {}, {:?}\n", r.entry, r.source);
    let _ = write!(s, "  aspect: target {aspect:.3}, outline {:.3} ({:.2} x {:.2} um)", w / h, w / 1e3, h / 1e3);
    match geometry(placed, routes).into_iter().map(|g| g.rect).reduce(union).filter(|d| d.h > 0) {
        Some(d) => {
            let got = f64::from(d.w) / f64::from(d.h);
            let _ = writeln!(s, ", drawn {got:.3} ({:+.1} %, {:.2} x {:.2} um)", pct(got, aspect), f64::from(d.w) / 1e3, f64::from(d.h) / 1e3);
        }
        None => s.push_str(", nothing drawn\n"),
    }
    for (b, k) in r.blocks.iter().enumerate() {
        let e = k.extent;
        let calls: Vec<usize> = (0..a.block.len()).filter(|&t| a.block[t] == Some(b as u16)).collect();
        let _ = write!(s, "  {} (row {}): target x {:.3} y {:.3} w {:.3} h {:.3}", k.name, k.row, e.x, e.y, e.w, e.h);
        match calls.iter().map(|&t| placed[t].bbox).reduce(union) {
            None => s.push_str(", no tile\n"),
            Some(d) => {
                let (dw, dh) = (f64::from(d.w) / w, f64::from(d.h) / h);
                let _ = writeln!(
                    s,
                    "; calls {calls:?} drew x {:.3} y {:.3} w {dw:.3} h {dh:.3} (w {:+.1} %, h {:+.1} %)",
                    f64::from(d.x) / w,
                    f64::from(d.y) / h,
                    pct(dw, f64::from(e.w)),
                    pct(dh, f64::from(e.h))
                );
            }
        }
    }
    let outside: Vec<usize> = (0..a.block.len()).filter(|&t| a.block[t].is_none()).collect();
    if !outside.is_empty() {
        let _ = writeln!(s, "  outside the reference: calls {outside:?}, a row above the outline");
    }
    for k in &r.blocks {
        if let Some(u) = k.unit_area {
            let _ = writeln!(s, "  unit_area {} {u:.3}: not applied, no M0 set recipe takes a unit target", k.name);
        }
    }
    if let Some(ratio) = r.area_ratio {
        let _ = writeln!(s, "  area_ratio {ratio:.3}: not applied, the entry gives no reference area to scale (Q61, open)");
    }
    for d in &a.degraded {
        let _ = writeln!(s, "  degraded: {d}");
    }
    s
}

/// Recipe mode, the whole spine, signed off by `engine`: [`run_with`] and
/// no sidecar.
///
/// # Errors
/// Any step's [`Refusal`].
pub fn run(spice: &str, pdk: &Pdk, engine: verify::SignoffEngine) -> Result<Outcome, Refusal> {
    run_with(spice, None, pdk, engine)
}

/// Recipe mode with the user's constraint sidecar (Q7), the whole spine,
/// signed off by `engine`.
///
/// # Errors
/// Any step's [`Refusal`].
pub fn run_with(spice: &str, sidecar: Option<&str>, pdk: &Pdk, engine: verify::SignoffEngine) -> Result<Outcome, Refusal> {
    let netlist = parse(spice, pdk)?;
    let cfg = match sidecar {
        Some(json) => annotation_with(pdk, &netlist, json)?,
        None => annotation(pdk),
    };
    let problem = annotator::annotate(&netlist.flat, &cfg);
    let calls = plan_recipes(&problem, &netlist.flat)?;
    run_calls(&netlist, &problem, &cfg, calls, pdk, engine)
}

/// The spine from a call table on: what [`run`] does after `plan_recipes`,
/// for a caller (the conformance harness) that writes its own calls.
///
/// # Errors
/// Any step's [`Refusal`].
pub fn run_calls(
    netlist: &Netlist,
    problem: &annotator::Problem,
    cfg: &annotator::AnnotationConfig,
    calls: RecipeCalls,
    pdk: &Pdk,
    engine: verify::SignoffEngine,
) -> Result<Outcome, Refusal> {
    let tiles = build_tiles(&calls, problem, &netlist.flat, pdk)?;
    let (layout, reference) = match &cfg.reference {
        None => (arrange(&tiles, pdk)?, None),
        Some(r) => {
            let (layout, assignment, fit) = arrange_reference(r, &calls, &tiles, problem, &netlist.flat, pdk)?;
            (layout, Some((r, assignment, fit)))
        }
    };
    let (placed, routes, route_report) = route::route(&tiles, &layout, problem, cfg, netlist.flat.nets.len(), pdk);
    let signoff = signoff(netlist, &placed, &routes, pdk, engine)?;
    let mut report = report(&calls, &netlist.flat, &signoff);
    if let Some((r, assignment, fit)) = &reference {
        report.push_str(&reference_report(r, assignment, fit, &placed, &routes));
    }
    let reference = reference.map(|(_, assignment, fit)| (assignment, fit));
    Ok(Outcome { calls, placed, routes, route_report, signoff, report, reference })
}
