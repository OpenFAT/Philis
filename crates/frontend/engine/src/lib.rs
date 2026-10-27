//! # `engine` — Philis 2.0's spines (`docs/ARCHITECTURE.md`)
//!
//! Recipe mode: `parse → annotate → plan_recipes → build_tiles → arrange →
//! route → signoff → report`, each step a free function over the tables the
//! one before returned, and [`run`] those lines pasted. M0 holds one set
//! recipe ([`RecipeKind::MosAdjacent`]) and a one-row floorplan; the steps the
//! spine names but M0 does not need yet (`gate_confidence`, `compact`,
//! `score_claims`, `print`) land with the milestone that needs them.

pub mod claims;
pub mod floorplan;
pub mod interface;
pub mod route;

use std::collections::{HashMap, HashSet};

use analog::base::netlist::{Device, Net, Netlist as Flat};
use analog::base::{DeviceGroup, DeviceId, DeviceKind, Layout, Macro, NetId, Report, Routes, Shape};
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
    /// `parse`, `no-recipe`, `reference`, or a foundry engine's `foundry/…` ([`verify::Refused`]).
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
    /// ABBA palindrome row, boundaries on a shared source, end dummies at the
    /// class reach, split gates where the gates are private (`RECIPES.md`
    /// §2.1; 1.0 `Mosfet` `Cc1d`; on fins, FinFet's shared CC row).
    MosCc1d,
    /// Two rows sharing one tap strip: cross-quad or the swapped ABBA/BAAB
    /// (`RECIPES.md` §2.1; 1.0 `Mosfet` `rows = 2`). Planar only.
    MosCc2d,
    /// Members' drain pairs interleaved in one row, every boundary on a
    /// shared source (Hastings Table 13.3, `(ₛAᴅAₛBᴅBₛ)ⁱ`): equal LOD and
    /// source resistance, not common-centroid (`RECIPES.md` §2.1). Planar.
    MosInterdigitated,
    /// A ratioed mirror's members (one kind, model and L) drawn from one unit
    /// finger ([`ratioed`]) in one diffusion-legal common-centroid row with
    /// end dummies (`RECIPES.md` §2.1 MOS-Ratioed; 1.0 `diffusion_cc_row`
    /// and `sets::unitize`). The row is a palindrome, so every member's
    /// centroid sits on the row's centre; counts with no such row (an odd
    /// member at the gcd unit whose half the deck cannot draw, or no
    /// diffusion-legal order) draw nothing and the ladder steps down.
    MosRatioed,
}

impl RecipeKind {
    /// Every kind, in ladder order from the most precise.
    pub const ALL: [RecipeKind; 5] =
        [RecipeKind::MosCc2d, RecipeKind::MosCc1d, RecipeKind::MosRatioed, RecipeKind::MosInterdigitated, RecipeKind::MosAdjacent];
}

/// A MOS set's degradation ladder (Q19, `RECIPES.md` §1), most precise
/// first. `None` class reads Moderate (C16), `None` kind Voltage. An
/// Exceptional set whose ladder is exhausted refuses (Q13); the others end on
/// Adjacent, which every MOS set can draw. A Moderate Current set (a mirror
/// or a load) climbs MOS-Ratioed first, a Moderate Voltage set Cc1d.
#[must_use]
pub fn ladder(class: Option<analog::intent::MatchClass>, kind: Option<analog::intent::MatchKind>) -> &'static [RecipeKind] {
    use analog::intent::{MatchClass as C, MatchKind as K};
    use RecipeKind::{MosAdjacent as Adj, MosCc1d as Cc1, MosCc2d as Cc2, MosInterdigitated as Int, MosRatioed as Rat};
    match (class.unwrap_or(C::Moderate), kind == Some(K::Current)) {
        (C::Exceptional, _) => &[Cc2, Cc1],
        (C::Moderate, true) => &[Rat, Int, Adj],
        (C::Moderate, false) => &[Cc1, Int, Adj],
        // MIN Current: Adjacent only (≤ 3–4 mirrors from the reference).
        (C::Minimal, true) => &[Adj],
        (C::Minimal, false) => &[Adj, Int],
    }
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

/// One call per matched MOS set the annotator left without a unitization row
/// (its members' fingers differ) that MOS-Ratioed draws from one unit, then
/// one per unitization row (a matched set), then one per MOS left over. Every
/// device lands in exactly one call, or the plan refuses.
///
/// # Errors
/// A device no recipe draws (M0: anything but a MOS).
pub fn plan_recipes(problem: &annotator::Problem, flat: &Flat, pdk: &Pdk) -> Result<RecipeCalls, Refusal> {
    let mut calls = RecipeCalls::default();
    let mut taken = vec![false; flat.devices.len()];
    // The first rung of `rungs` that draws `members` on this deck.
    let first_rung = |rungs: &[RecipeKind], members: &[DeviceId], row: Option<usize>| {
        let process = Mos::new(flat, members, pdk);
        rungs.iter().position(|&k| {
            constraints_of(k, &problem.constraints, row, flat, members, pdk, process.get())
                .is_some_and(|c| !mos_variants(k, flat, members, &c, process.get()).is_empty())
        })
    };
    for s in &problem.intent.sets {
        let members: Vec<DeviceId> = s.members.iter().map(|m| m.device).collect();
        let mos = members.iter().all(|d| matches!(flat.devices[d.0 as usize].kind, DeviceKind::Nmos | DeviceKind::Pmos));
        let has_row =
            problem.constraints.unitization.iter().any(|u| u.devices.len() == members.len() && u.devices.iter().all(|d| members.contains(d)));
        if !mos || has_row || members.len() < 2 || members.iter().any(|d| taken[d.0 as usize]) {
            continue;
        }
        let rungs = ladder(Some(s.class), Some(s.kind));
        // Only Ratioed draws members of unequal fingers: no other rung is tried.
        let Some(step) = rungs.iter().position(|&k| k == RecipeKind::MosRatioed) else { continue };
        if first_rung(&rungs[step..=step], &members, None).is_none() {
            continue;
        }
        members.iter().for_each(|d| taken[d.0 as usize] = true);
        calls.push(RecipeKind::MosRatioed, &members, None);
        *calls.ladder_step.last_mut().expect("just pushed") = step as u8;
    }
    for (i, u) in problem.constraints.unitization.iter().enumerate() {
        if u.devices.iter().any(|d| taken[d.0 as usize]) || !matches!(u.device_type, DeviceKind::Nmos | DeviceKind::Pmos) {
            continue;
        }
        u.devices.iter().for_each(|d| taken[d.0 as usize] = true);
        let rungs = ladder(u.class, u.kind);
        let Some(step) = first_rung(rungs, &u.devices, Some(i)) else {
            return Err(refuse(
                "no-recipe",
                format!("{}: no rung of {rungs:?} draws this {:?} set", names(flat, &u.devices), u.class.unwrap_or_default()),
            ));
        };
        calls.push(rungs[step], &u.devices, Some(i as u16));
        *calls.ladder_step.last_mut().expect("just pushed") = step as u8;
    }
    for (i, d) in flat.devices.iter().enumerate().filter(|(i, _)| !taken[*i]) {
        match d.kind {
            DeviceKind::Nmos | DeviceKind::Pmos => calls.push(RecipeKind::MosAdjacent, &[DeviceId(i as u16)], None),
            k => return Err(refuse("no-recipe", format!("{}: no M0 recipe draws a {k:?}", d.name))),
        }
    }
    Ok(calls)
}

/// A MOS the deck can draw exactly as the netlist sizes it: each finger at
/// least the deck's shortest legal channel, and on a fin deck a whole number
/// of fins wide. Anything else is a coded refusal, never a silent resize
/// (the generator would round fins and LVS would see another W).
///
/// # Errors
/// `device`: the size and the deck limit it misses.
pub fn drawable_size(d: &Device, pdk: &Pdk) -> Result<(), Refusal> {
    let Some(s) = d.mos_size() else { return Ok(()) };
    drawable_finger(s.w_finger_nm(), s.l_nm, d, pdk).map_err(|why| refuse("device", format!("{}: {why}", d.name)))
}

/// Whether one `w` × `l` nm finger of `d`'s kind and model is drawable, as
/// [`drawable_size`] asks; `Err` names the limit it misses.
fn drawable_finger(w: i64, l: i64, d: &Device, pdk: &Pdk) -> Result<(), String> {
    use analog::base::Process;
    let (l_min, w_min) = pdk.min_channel(d.kind == DeviceKind::Pmos, &d.model);
    // A transistor's diffusion can be wider than plain diffusion's minimum
    // (sky130 difftap.2: 420 nm).
    let w_min = w_min.max(pdk.rule("min_finger_width", 0));
    if l < i64::from(l_min) || w < i64::from(w_min) {
        return Err(format!("finger W/L {w}/{l} nm is below the deck's shortest legal channel {w_min}/{l_min} nm"));
    }
    if pdk.layer("fin").is_some() {
        let fin_p = i64::from(pdk.width("fin").unwrap_or(0) + pdk.space("fin").unwrap_or(0));
        if fin_p > 0 && w % fin_p != 0 {
            return Err(format!("finger W {w} nm is not a whole number of {fin_p} nm fins"));
        }
    }
    Ok(())
}

/// One drawn tile per call, its pins bound to the netlist's nets.
///
/// # Errors
/// A call no variant of its recipe can draw.
pub fn build_tiles(calls: &RecipeCalls, problem: &annotator::Problem, flat: &Flat, pdk: &Pdk) -> Result<Vec<Macro>, Refusal> {
    build_tiles_within(calls, problem, flat, pdk, None)
}

/// [`build_tiles`] under a die's width targets ([`floorplan::targets`], one
/// per call): each call's least-area variant no wider than its target, an
/// Adjacent block then free to fold into more rows. `None`: [`build_tiles`].
///
/// # Errors
/// `interface`: a call none of whose variants fits its target, never a
/// silently wider tile.
fn build_tiles_within(
    calls: &RecipeCalls,
    problem: &annotator::Problem,
    flat: &Flat,
    pdk: &Pdk,
    max_w: Option<&[i32]>,
) -> Result<Vec<Macro>, Refusal> {
    let ground = problem.net_classes.iter().find(|c| c.class == analog::metadata::NetClass::Ground).map(|c| c.net);
    (0..calls.len())
        .map(|i| {
            let members = calls.members_of(i);
            let group = DeviceGroup { devices: members.to_vec() };
            let mos = Mos::new(flat, members, pdk);
            let process = mos.get();
            let constraints = constraints_of(calls.kind[i], &problem.constraints, calls.unitization[i].map(usize::from), flat, members, pdk, process)
                .ok_or_else(|| refuse("no-recipe", format!("{}: no one unit draws this {:?} set", names(flat, members), calls.kind[i])))?;
            let drawn: Vec<Macro> = match mos_variants(calls.kind[i], flat, members, &constraints, process) {
                Variants::Fin(vs) => vs.into_iter().map(|v| v.draw(&group, &constraints, process)).collect(),
                Variants::Planar(mut vs, apart_too) => {
                    if max_w.is_some() && calls.kind[i] == RecipeKind::MosAdjacent && !vs.is_empty() {
                        vs.extend(Mosfet::enumerate(&group, &constraints, process).into_iter().filter(|v| v.style == Pattern::Single && v.rows > 1));
                    }
                    vs.into_iter()
                        .map(|v| v.draw(&group, &constraints, process))
                        .chain(apart_too.then(|| apart(members, &constraints, process, pdk)).flatten())
                        .collect()
                }
            };
            let narrowest = drawn.iter().map(|m| m.bbox.w).min();
            let target = max_w.map(|w| w[i]);
            let mut m = drawn
                .into_iter()
                .filter(|m| target.is_none_or(|w| m.bbox.w <= w))
                .min_by_key(|m| i64::from(m.bbox.w) * i64::from(m.bbox.h))
                .ok_or_else(|| match (target, narrowest) {
                    (Some(w), Some(n)) => refuse(
                        "interface",
                        format!(
                            "call {i} {:?} [{}]: its narrowest variant is {n} nm, over the die's {w} nm target",
                            calls.kind[i],
                            names(flat, members)
                        ),
                    ),
                    _ => refuse("no-recipe", format!("{}: no {:?} variant draws it", names(flat, members), calls.kind[i])),
                })?;
            bind_pins(&mut m, flat, members, ground);
            join_members(&mut m, pdk);
            Ok(m)
        })
        .collect()
}

/// The process a MOS set draws on: its model's overlay (a flavoured model's
/// markers) or the deck itself.
struct Mos<'a> {
    pdk: &'a Pdk,
    overlay: Option<verify::pdk::Overlay<'a>>,
}

impl<'a> Mos<'a> {
    fn new(flat: &Flat, members: &[DeviceId], pdk: &'a Pdk) -> Self {
        Mos { pdk, overlay: pdk.mos_overlay(&flat.devices[members[0].0 as usize].model) }
    }
    fn get(&self) -> &dyn analog::base::Process {
        self.overlay.as_ref().map_or(self.pdk as &dyn analog::base::Process, |o| o)
    }
}

/// The unitization a `kind` call over `members` draws: MOS-Ratioed's one unit
/// ([`ratioed`], over annotator row `row` when it has one), else row `row`
/// folded ([`folded`]), else the members as drawn ([`as_drawn`]). `None`:
/// Ratioed finds no unit the deck draws.
fn constraints_of(
    kind: RecipeKind,
    c: &analog::Constraints,
    row: Option<usize>,
    flat: &Flat,
    members: &[DeviceId],
    pdk: &Pdk,
    process: &dyn analog::base::Process,
) -> Option<analog::Constraints> {
    match (kind, row) {
        (RecipeKind::MosRatioed, _) => ratioed(flat, members, row.map(|r| &c.unitization[r]), pdk, process),
        (RecipeKind::MosAdjacent | RecipeKind::MosCc1d | RecipeKind::MosCc2d | RecipeKind::MosInterdigitated, Some(r)) => Some(folded(c, r, process)),
        (RecipeKind::MosAdjacent | RecipeKind::MosCc1d | RecipeKind::MosCc2d | RecipeKind::MosInterdigitated, None) => {
            Some(as_drawn(flat, members, process))
        }
    }
}

/// MOS-Ratioed's unitization (`RECIPES.md` §2.1; 1.0 `sets::unitize`): every
/// member drawn from one unit finger, the gcd of the members' fingers folded
/// to the tap reach ([`fold`]), member `k` as that unit's integer count of its
/// own total W, with end dummies. A member with an odd count would put a
/// device boundary on a drain, so then the unit halves, every count even.
/// `base` (the annotator's row) lends its class and route matching.
///
/// `None`, never a rounded or resized unit: members of another kind, model or
/// L, or a unit (or the half) off the grid or below what the deck draws
/// ([`drawable_size`]).
fn ratioed(
    flat: &Flat,
    members: &[DeviceId],
    base: Option<&analog::cell::Unitization>,
    pdk: &Pdk,
    process: &dyn analog::base::Process,
) -> Option<analog::Constraints> {
    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    assert!(!members.is_empty(), "a call has members");
    let devs: Vec<&Device> = members.iter().map(|d| &flat.devices[d.0 as usize]).collect();
    let sizes: Vec<analog::base::MosSize> = devs.iter().map(|d| d.mos_size()).collect::<Option<_>>()?;
    let (d0, l) = (devs[0], sizes[0].l_nm);
    if devs.iter().zip(&sizes).any(|(d, s)| d.kind != d0.kind || d.model != d0.model || s.l_nm != l) {
        return None;
    }
    let total: Vec<i64> = sizes.iter().map(|s| s.w_finger_nm() * i64::from(s.fingers())).collect();
    let g = sizes.iter().map(|s| s.w_finger_nm() / i64::from(fold(s.w_finger_nm(), l, process))).fold(0, gcd);
    let odd = total.iter().any(|t| (t / g.max(1)) % 2 == 1);
    let unit = if odd { g / 2 } else { g };
    if (odd && g % 2 != 0) || unit <= 0 || unit % i64::from(process.grid().max(1)) != 0 || drawable_finger(unit, l, d0, pdk).is_err() {
        return None;
    }
    let dev_nf: Vec<u16> = total.iter().map(|t| u16::try_from(t / unit).ok()).collect::<Option<_>>()?;
    assert!(total.iter().zip(&dev_nf).all(|(t, &n)| i64::from(n) * unit == *t), "every member is a whole number of units");
    let r = dev_nf.iter().fold(0, |a, &n| gcd(a, i64::from(n))) as u16;
    Some(analog::Constraints {
        unitization: vec![analog::cell::Unitization {
            devices: members.to_vec(),
            device_type: d0.kind,
            target_ratio: dev_nf.iter().map(|&n| n / r).collect(),
            dev_nf,
            unit_w: i32::try_from(unit).ok()?,
            unit_l: i32::try_from(l).ok()?,
            series_parallel: analog::cell::SeriesParallel::Parallel,
            dummy_required: true,
            route_matching_required: base.is_some_and(|u| u.route_matching_required),
            class: base.and_then(|u| u.class),
            kind: base.and_then(|u| u.kind),
            series: Vec::new(),
            style: base.and_then(|u| u.style),
        }],
        ..analog::Constraints::default()
    })
}

/// A recipe's candidate variants for a set; empty when the kind cannot draw
/// it on this process (the ladder then steps down).
enum Variants {
    Fin(Vec<FinFet>),
    /// Generator variants, and whether the members-apart row is a candidate.
    Planar(Vec<Mosfet>, bool),
}

impl Variants {
    fn is_empty(&self) -> bool {
        match self {
            Variants::Fin(v) => v.is_empty(),
            Variants::Planar(v, apart) => v.is_empty() && !apart,
        }
    }
}

/// `kind`'s variants for `members`. Styles that abut members on one shared
/// diffusion (Single blocks, Cc1d, Cc2d) need one source and one bulk net;
/// a centroid row with private gates needs split gates.
fn mos_variants(kind: RecipeKind, flat: &Flat, members: &[DeviceId], c: &analog::Constraints, process: &dyn analog::base::Process) -> Variants {
    let group = DeviceGroup { devices: members.to_vec() };
    let shared = common(flat, members, "S") && common(flat, members, "B");
    let private_gates = !common(flat, members, "G");
    if process.layer("fin").is_some() {
        let all = FinFet::enumerate(&group, c, process);
        return Variants::Fin(match kind {
            RecipeKind::MosAdjacent => all.into_iter().filter(|v| !v.shared).collect(),
            RecipeKind::MosCc1d | RecipeKind::MosRatioed if shared => all.into_iter().filter(|v| v.shared).collect(),
            RecipeKind::MosCc1d | RecipeKind::MosRatioed | RecipeKind::MosCc2d | RecipeKind::MosInterdigitated => Vec::new(),
        });
    }
    let all = || Mosfet::enumerate(&group, c, process);
    match kind {
        RecipeKind::MosAdjacent if shared => {
            Variants::Planar(all().into_iter().filter(|v| v.style == Pattern::Single && v.rows == 1).collect(), true)
        }
        RecipeKind::MosAdjacent => Variants::Planar(Vec::new(), true),
        RecipeKind::MosCc1d | RecipeKind::MosCc2d | RecipeKind::MosInterdigitated | RecipeKind::MosRatioed if !shared || members.len() < 2 => {
            Variants::Planar(Vec::new(), false)
        }
        RecipeKind::MosInterdigitated => Variants::Planar(
            all().into_iter().filter(|v| v.style == Pattern::Interdig && v.rows == 1 && (v.split_gates || !private_gates)).collect(),
            false,
        ),
        // Ratioed is the one-row centroid over its own unit ([`ratioed`]).
        RecipeKind::MosCc1d | RecipeKind::MosCc2d | RecipeKind::MosRatioed => {
            let rows = if kind == RecipeKind::MosCc2d { 2 } else { 1 };
            Variants::Planar(
                all().into_iter().filter(|v| v.style == Pattern::Cc1d && v.rows == rows && (v.split_gates || !private_gates)).collect(),
                false,
            )
        }
    }
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
fn as_drawn(flat: &Flat, members: &[DeviceId], process: &dyn analog::base::Process) -> analog::Constraints {
    let first = &flat.devices[members[0].0 as usize];
    let s = first.mos_size();
    // Fold a finger longer than the deck's tap reach (LU.2/LU.3) into k equal
    // grid-aligned fingers: the same total W, which LVS merges back.
    let k = s.map_or(1, |s| fold(s.w_finger_nm(), s.l_nm, process));
    let nf = |d: &DeviceId| flat.devices[d.0 as usize].mos_size().map_or(1, |s| (s.fingers() * k).clamp(1, u32::from(u16::MAX)) as u16);
    analog::Constraints {
        unitization: vec![analog::cell::Unitization {
            devices: members.to_vec(),
            device_type: first.kind,
            dev_nf: members.iter().map(nf).collect(),
            target_ratio: vec![1; members.len()],
            unit_w: s.map_or(0, |s| (s.w_finger_nm() / i64::from(k)) as i32),
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

/// The fold count for a finger `w` × `l` nm: the least `k` whose `w / k` is
/// within the deck's tap reach and a whole number of grid steps; 1 when `w`
/// already fits or the deck states no reach.
fn fold(w: i64, l: i64, process: &dyn analog::base::Process) -> u32 {
    let taps = i64::from(substrate3::cells::mosfet::max_finger_for_taps(process, l as i32));
    let deck = i64::from(process.rule("max_finger_width", 0));
    let reach = if deck > 0 { taps.min(deck) } else { taps };
    let grid = i64::from(process.grid().max(1));
    if reach == i64::from(i32::MAX) || w <= reach {
        return 1;
    }
    (1..=1024).find(|&k| w / k <= reach && w % (k * grid) == 0).map_or(1, |k| k as u32)
}

/// `c` with unitization row `row` folded (each finger to the tap reach and
/// the deck's widest finger, [`fold`]): the same total W per member.
fn folded(c: &analog::Constraints, row: usize, process: &dyn analog::base::Process) -> analog::Constraints {
    let mut c = c.clone();
    let u = &mut c.unitization[row];
    let k = fold(i64::from(u.unit_w), i64::from(u.unit_l), process);
    if k > 1 {
        u.unit_w /= k as i32;
        u.dev_nf.iter_mut().for_each(|n| *n = n.saturating_mul(k as u16));
    }
    c
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

/// The floorplan: one glue row (Q20, `placement::floorplan::glue`), tiles
/// left to right in call order. Every gap is sized from the shapes that face
/// across it, layer pair by layer pair (sky130 NMOS beside PMOS: 330 nm, not
/// the widest spacing, nwell's 1270); on a fin deck the tiles share one gate
/// lattice with field-gate fill. The fill macro is already in world
/// coordinates, so it goes after the tiles, past the layout's rows, where
/// `place_macros` leaves it as drawn.
/// ponytail: one row; the symmetric row stack is M2's floorplan recipe.
///
/// # Errors
/// `floorplan`: the glue row refuses (tiles whose gates span other heights
/// on a fin deck).
pub fn arrange(tiles: &mut Vec<Macro>, pdk: &Pdk) -> Result<Layout, Refusal> {
    let roles: Vec<&str> = pdk.roles.iter().map(|(r, _)| r.as_str()).collect();
    let spacing = placement::floorplan::Spacing::of(pdk, &roles);
    // Tile origins on the cut lattice, as the cells draw their cuts.
    let lattice = substrate3::cells::builder::cut_lattice(pdk).max(1);
    let (floorplan, fill) = placement::floorplan::glue(tiles, pdk, &spacing, lattice).map_err(|e| refuse("floorplan", format!("{e:?}")))?;
    if !fill.shapes.is_empty() {
        tiles.push(fill);
    }
    Ok(floorplan.layout())
}

/// Everything signoff reads, and its verdict.
pub struct Outcome {
    pub calls: RecipeCalls,
    pub claims: claims::Claims,
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

/// Sign the routed layout off against the schematic with `engine`
/// ([`verify::SignoffEngine`]: GPurify in process, or the foundry's KLayout
/// or magic/netgen decks). Labels: one per net, on the first pin that sits
/// on drawn metal.
///
/// # Errors
/// The schematic's LVS reference cannot be built, or a foundry engine
/// refused (no tool, no deck for this PDK, no verdict): its code is kept.
pub fn signoff(netlist: &Netlist, placed: &[Macro], routes: &Routes, pdk: &Pdk, engine: verify::SignoffEngine) -> Result<verify::Signoff, Refusal> {
    let names: Vec<String> = netlist.flat.nets.iter().map(|n| n.name.clone()).collect();
    let pins = labeled_pins(placed, &names, pdk, &geometry(placed, routes));
    signoff_labeled(netlist, placed, routes, pins, pdk, engine)
}

/// [`signoff`] with the caller's labels: LVS ports are exactly `pins`.
fn signoff_labeled(
    netlist: &Netlist,
    placed: &[Macro],
    routes: &Routes,
    pins: Vec<LabeledPin>,
    pdk: &Pdk,
    engine: verify::SignoffEngine,
) -> Result<verify::Signoff, Refusal> {
    let shapes = geometry(placed, routes);
    let names: Vec<String> = netlist.flat.nets.iter().map(|n| n.name.clone()).collect();
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
pub fn report(calls: &RecipeCalls, claims: &claims::Claims, flat: &Flat, signoff: &verify::Signoff) -> String {
    let mut s = String::new();
    for i in 0..calls.len() {
        s.push_str(&format!("call {i}: {:?} [{}] ladder step {}\n", calls.kind[i], names(flat, calls.members_of(i)), calls.ladder_step[i]));
    }
    s.push_str(&claims.to_string());
    let hard = &signoff.report.hard_violations;
    s.push_str(&format!("signoff: {} hard violations\n", hard.len()));
    for v in hard {
        s.push_str(&format!("  {} ({})\n", v.rule, v.margin));
    }
    s
}

/// Recipe mode, the whole spine, signed off by `engine`.
///
/// # Errors
/// Any step's [`Refusal`].
pub fn run(spice: &str, pdk: &Pdk, engine: verify::SignoffEngine) -> Result<Outcome, Refusal> {
    run_in(spice, None, pdk, engine)
}

/// [`run`] inside a block harness: the floorplan centred in `interface`'s
/// die, its boundary pins routed and labelled as the LVS ports
/// ([`interface`]).
///
/// # Errors
/// Any step's [`Refusal`]; `interface` for a pin on a non-port net or a
/// non-routing layer, or a block that does not fit the die.
pub fn run_in(spice: &str, interface: Option<&interface::Interface>, pdk: &Pdk, engine: verify::SignoffEngine) -> Result<Outcome, Refusal> {
    let netlist = parse(spice, pdk)?;
    let cfg = annotation(pdk);
    let problem = annotator::annotate(&netlist.flat, &cfg);
    let calls = plan_recipes(&problem, &netlist.flat, pdk)?;
    run_calls_in(&netlist, &problem, &cfg, calls, interface, pdk, engine)
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
    run_calls_in(netlist, problem, cfg, calls, None, pdk, engine)
}

fn run_calls_in(
    netlist: &Netlist,
    problem: &annotator::Problem,
    cfg: &annotator::AnnotationConfig,
    calls: RecipeCalls,
    interface: Option<&interface::Interface>,
    pdk: &Pdk,
    engine: verify::SignoffEngine,
) -> Result<Outcome, Refusal> {
    let bound = interface.map(|i| interface::bind(i, &netlist.flat, pdk)).transpose()?;
    netlist.flat.devices.iter().try_for_each(|d| drawable_size(d, pdk))?;
    // A die's width targets, inside its pins' reach on every side.
    let targets = interface.zip(bound.as_deref()).and_then(|(i, b)| {
        let (w, h) = i.die_nm?;
        let r = 2 * interface::reach(b, pdk);
        Some(floorplan::targets(&calls, problem, &netlist.flat, (w - r, h - r), pdk).w)
    });
    let tiles = build_tiles_within(&calls, problem, &netlist.flat, pdk, targets.as_deref())?;
    let mut tiles = tiles;
    let layout = floorplan::arrange_block(&mut tiles, &calls, problem, &netlist.flat, pdk)?;
    // The glue fill (if any) rides after the call tiles; claims read the calls.
    let claims = claims::score_claims(&calls, &tiles[..calls.len()], &layout, problem, pdk);
    let io = match interface.zip(bound) {
        Some((i, b)) => interface::boundary(i, &b, &analog::base::place_macros(&tiles, &layout), pdk)?,
        None => Vec::new(),
    };
    tiles.extend(io.iter().cloned());
    let (placed, routes, route_report) = route::route(&tiles, &layout, problem, cfg, netlist.flat.nets.len(), pdk);
    let signoff = match interface {
        Some(_) => {
            let names: Vec<String> = netlist.flat.nets.iter().map(|n| n.name.clone()).collect();
            // The harness pins, then each port it leaves unpinned at its first cell pin.
            let mut pins = interface::labels(&io, &names);
            let ports: Vec<&String> = netlist.flat.ports.iter().map(|p| &names[p.0 as usize]).collect();
            let rest: Vec<LabeledPin> = labeled_pins(&placed, &names, pdk, &geometry(&placed, &routes))
                .into_iter()
                .filter(|l| ports.contains(&&l.name) && pins.iter().all(|p| p.name != l.name))
                .collect();
            pins.extend(rest);
            signoff_labeled(netlist, &placed, &routes, pins, pdk, engine)?
        }
        None => signoff(netlist, &placed, &routes, pdk, engine)?,
    };
    let report = report(&calls, &claims, &netlist.flat, &signoff);
    Ok(Outcome { calls, claims, placed, routes, route_report, signoff, report })
}
