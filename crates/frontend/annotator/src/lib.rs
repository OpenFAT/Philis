//! `annotator`: netlist → recognised analog blocks → constraints.
//!
//! [`annotate`] runs the [`pattern`] catalog over the netlist, turns the
//! non-overlapping matches into blocks (one group each, plus a trailing glue
//! block for the unclaimed devices), gives each composite its declared children
//! ([`catalog::roles_of`]), and emits placement ([`emit`]), routing ([`extract`]), net-class
//! ([`classify`]) and cell-tier ([`constraints`]) constraints from them, then
//! reports conflicts ([`conflict`]).
//!
//! Contents:
//! - `pub use block::{Block, BlockKind}`
//! - `pub use evidence::{Evidence, OpFacts}`
//! - `pub use netrole::{rail_of, AnnotationConfig, NetRole, ProcessNumbers}`
//! - `pub use policy::Policy`
//! - `pub struct Problem`
//! - `pub enum Coverage`
//! - `pub fn annotate(netlist: &Netlist, cfg: &AnnotationConfig) -> Problem`
//! - `pub fn annotate_with(netlist: &Netlist, cfg: &AnnotationConfig, ev: &Evidence) -> Problem`
//! - `pub(crate) fn gate_um2(dev: &analog::base::netlist::Device) -> f32`
//! - `pub(crate) fn param(dev: &analog::base::netlist::Device, key: &str, default: i64) -> i64`

#![warn(missing_docs)]

pub mod allocate;
pub mod audit;
pub mod block;
pub mod budget;
pub mod catalog;
pub mod class;
pub mod classify;
pub mod conflict;
pub mod constraints;
pub mod emit;
pub mod evidence;
pub mod extract;
pub mod flow;
pub mod graph;
pub mod hier;
pub mod ir;
pub mod netrole;
pub mod passive;
pub mod pattern;
pub mod policy;
pub mod rings;
mod seeds;
pub mod sets;
pub mod sidecar;
pub mod size;
pub mod substrate;
pub mod symmetry;
pub mod terms;

pub use block::{Block, BlockKind};
pub use evidence::{Evidence, OpFacts};
pub use netrole::{rail_of, AnnotationConfig, NetRole, ProcessNumbers};
pub use policy::Policy;

use analog::base::ids::DeviceId;
use analog::base::Netlist;

/// Everything the annotator hands the generators, placer and router.
pub struct Problem {
    /// Recognised blocks, then the glue block. Index = [`analog::base::GroupId`] and
    /// the stage's symmetry [`analog::base::ids::AxisId`].
    pub blocks: Vec<Block>,
    /// Cell-tier directives (unitization, dummies, guard rings), read by `cells`.
    pub constraints: analog::Constraints,
    /// Placement rules, every batch tagged with a stable id in emission order (EXT-10).
    pub placement: analog::Requirements<analog::base::Layout>,
    /// Routing rules, tagged after the placement batches.
    pub routing: analog::Requirements<analog::base::Routes>,
    /// Per-net class + budgets, indexed by [`analog::base::NetId`].
    pub net_classes: Vec<analog::metadata::NetClassification>,
    /// `blocks[i].devices` — feeds [`analog::base::Layout::groups`]. Composites may mix
    /// polarities.
    pub groups: Vec<Vec<DeviceId>>,
    /// Diffusion-sharing permission, parallel to `groups`: a mixed-polarity group is
    /// cut to one member (NMOS/PMOS abutment merges implants — DRC-clean, LVS-fatal).
    /// One member, not zero: an empty group panics `Layout::bbox`.
    pub abutment: Vec<Vec<DeviceId>>,
    /// `(rule kind, missing input)` for every family left unemitted because the
    /// deck lacks a number: **unknown**, never a pass.
    pub missing: Vec<(&'static str, &'static str)>,
    /// Every device, by id, with how recognition covers it (T7).
    pub coverage: Vec<(DeviceId, Coverage)>,
    /// Extraction's contract (EXT-12); filled from EXT-13 on.
    pub intent: analog::intent::Intent,
    /// Symmetry axes the placement emits: one per compound, at least one (EXT-20).
    pub axis_count: usize,
}

/// How a device is covered: the one report that it got a constraint or why not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Touched by a placement batch.
    Constrained,
    /// In a recognised block of this template whose roles emit nothing for it.
    Grouped(&'static str),
    /// In no constraint: `"do_not_identify"`, `"unknown size"` or `"no pattern"`.
    Unconstrained(&'static str),
}

/// Assemble the [`Problem`] from structure alone: [`annotate_with`] and no evidence.
///
/// # Panics
/// When the netlist has more than 65535 devices or nets: ids are `u16` (AA-35).
#[must_use]
pub fn annotate(netlist: &Netlist, cfg: &AnnotationConfig) -> Problem {
    annotate_with(netlist, cfg, &Evidence::default())
}

/// Assemble the [`Problem`], reading simulation evidence (EXT-17): testbench
/// rails and clocks join the name pre-pass, the op point sets device regions
/// and roles. Deterministic.
///
/// # Panics
/// When the netlist has more than 65535 devices or nets: ids are `u16` (AA-35).
#[must_use]
pub fn annotate_with(netlist: &Netlist, cfg: &AnnotationConfig, ev: &Evidence) -> Problem {
    for n in [netlist.devices.len(), netlist.nets.len()] {
        assert!(n <= usize::from(u16::MAX), "annotator: {n} devices/nets exceed the u16 id space (65535)");
    }
    let hg = analog::base::BipartiteHypergraph::from_netlist(netlist);
    let mut models = Vec::new();
    let drawn: Vec<size::Drawn> = netlist.devices.iter().map(|d| size::drawn(d, &mut models)).collect();
    let named = |n: &String| [&cfg.supply_nets, &cfg.ground_nets, &cfg.clock_nets].iter().any(|l| l.iter().any(|s| s.eq_ignore_ascii_case(n)));
    let user: Vec<bool> = hg.net_names.iter().map(named).collect();
    let cfg = &netrole::with_testbench(netlist, cfg, ev);
    let roles = netrole::classify_nets(&hg, cfg);

    // Recognised blocks in selection order, each composite with its primitive children.
    let canon = pattern::canonical_labels(&hg, &drawn, &models, &roles);
    let names: Vec<&str> = netlist.devices.iter().map(|d| d.name.as_str()).collect();
    let all = pattern::recognize_all(&hg, &drawn, &roles, cfg, &canon, &names);
    let mut claimed = vec![false; netlist.devices.len()];
    let mut blocks: Vec<Block> = pattern::select_disjoint(&all, &canon, &names)
        .into_iter()
        .map(|m| {
            for &d in &m.instances {
                claimed[d as usize] = true;
            }
            Block::from_match(&m)
        })
        .collect();
    // A source-degenerated diff pair no joined-source pattern sees (EXT-19 step 5).
    for (a, b) in passive::degenerated_pairs(&hg, &drawn, &roles) {
        if !claimed[a.0 as usize] && !claimed[b.0 as usize] {
            (claimed[a.0 as usize], claimed[b.0 as usize]) = (true, true);
            blocks.push(Block {
                kind: BlockKind::DiffPair,
                template: "diff_pair_with_degen",
                devices: vec![a, b],
                injected: false,
                sub_blocks: Vec::new(),
                selfs: Vec::new(),
            });
        }
    }
    let glue = (0..netlist.devices.len() as u16).filter(|&d| !claimed[d as usize]).map(DeviceId);
    blocks.push(Block {
        kind: BlockKind::Glue,
        template: "glue",
        devices: glue.collect(),
        injected: false,
        sub_blocks: Vec::new(),
        selfs: Vec::new(),
    });

    let groups: Vec<Vec<DeviceId>> = blocks.iter().map(|b| b.devices.clone()).collect();
    let abutment = groups
        .iter()
        .map(|g| {
            let k0 = g.first().map(|d| netlist.devices[d.0 as usize].kind);
            if g.iter().all(|d| Some(netlist.devices[d.0 as usize].kind) == k0) {
                g.clone()
            } else {
                g[..1].to_vec()
            }
        })
        .collect();

    // A net feeding a matched device's gate is the small-signal path whose coupling
    // shows up as offset: classified Sensitive, tight budgets.
    let mut sensitive = vec![false; netlist.devices.len()];
    for b in block::leaves(&blocks) {
        if b.kind.is_sensitive() {
            for d in &b.devices {
                sensitive[d.0 as usize] = true;
            }
        }
    }
    let gates: Vec<f32> = netlist.devices.iter().map(gate_um2).collect();
    let mut net_classes = classify::classify(&hg, &roles, &sensitive, &gates, cfg.process.gate_af_per_um2, &cfg.loads);

    let fet = |k: analog::base::DeviceKind| matches!(k, analog::base::DeviceKind::Nmos | analog::base::DeviceKind::Pmos);
    let needs = netrole::Needs {
        matched: block::leaves(&blocks)
            .iter()
            .any(|b| matches!(b.kind, BlockKind::DiffPair | BlockKind::CurrentMirror | BlockKind::Load | BlockKind::CascodePair)),
        gate_nets: netlist.devices.iter().any(|d| fet(d.kind)),
        budgeted_nets: gates.iter().any(|&g| g > 0.0),
    };
    let mut missing = netrole::missing(&cfg.process, &needs);
    // The matcher pairs FETs only: an unsized FET is a pair nobody could check.
    if netlist.devices.iter().zip(&drawn).any(|(d, s)| fet(d.kind) && size::unknown_size(d.kind, s)) {
        missing.push(("MatchedSet", "device W/L"));
    }
    if netlist.devices.iter().any(|d| d.kind == analog::base::DeviceKind::Capacitor) {
        missing.push(("ParasiticBudget", "capacitor-plate nets: settling / code-error spec (ARR-03, ARR-05)"));
    }
    let mut intent = analog::intent::Intent::default();
    let (seeds, hier_couples) = seeds::seeds(netlist, &drawn, &blocks, &canon, &names, &cfg.seeds);
    let (compounds, diags) = symmetry::analyze(&hg, &drawn, &net_classes, &seeds, &canon);
    intent.compounds = compounds;
    if let (Some(dir), [c]) = (cfg.symmetry_dir, intent.compounds.as_mut_slice()) {
        c.dir = dir;
    }
    intent.diagnostics.extend(cfg.sidecar_diags.iter().cloned());
    intent.diagnostics.extend(diags);
    let shared = sets::shared_bias_groups(&hg, &drawn, &net_classes);
    // EXT-19: passive, bipolar-core and diode sets.
    let pair_of = |m: &pattern::PatternMatch| (DeviceId(m.instances[0] as u16), DeviceId(m.instances[1] as u16));
    let bjt_ratioed: Vec<(DeviceId, DeviceId)> = all.iter().filter(|m| m.template.starts_with("bjt_ratioed_pair")).map(pair_of).collect();
    let mut pairs: Vec<(DeviceId, DeviceId)> =
        block::leaves(&blocks).iter().filter(|b| b.kind == BlockKind::DiffPair).map(|b| (b.devices[0], b.devices[1])).collect();
    pairs.extend(all.iter().filter(|m| m.template.starts_with("bjt_")).map(pair_of));
    // Degeneration first: its role (the pair's) wins over a plain symmetric couple.
    let mut passive_sets = passive::degeneration(&hg, &drawn, &pairs, &mut intent.diagnostics);
    passive_sets.extend(passive::resistor_sets(&hg, &drawn, &net_classes, &intent.compounds));
    passive_sets.extend(passive::bandgap_cores(&hg, &drawn, &bjt_ratioed));
    passive_sets.extend(passive::capacitor_sets(&hg, &drawn, &net_classes, &mut intent.diagnostics));
    passive_sets.extend(passive::diode_sets(&hg, &drawn));
    let passive_groups: Vec<Vec<DeviceId>> = passive_sets.iter().map(|p| p.devices.clone()).collect();
    // EXT-27 arrays: one ProxBlock group and one horizontal, reversible order step per instance.
    let array_steps = hier::array_steps(netlist, &drawn, &hg, &net_classes, &canon);
    let array_devs: Vec<Vec<DeviceId>> = array_steps.iter().map(|a| a.concat()).collect();
    let array_orders: Vec<analog::intent::Order> = array_steps
        .into_iter()
        .map(|steps| analog::intent::Order { steps, dir: analog::intent::AxisDir::H, reversible: true, weight: 1.0 })
        .collect();
    let reqs = graph::requirements(
        &all,
        &intent.compounds,
        &shared,
        &passive_groups,
        &cfg.groups,
        &hier_couples,
        &array_devs,
        &netlist.device_inst,
        &hg,
        &net_classes,
        &canon,
        &cfg.policy,
    );
    intent.tree = graph::hsmpg(netlist.devices.len(), &reqs, &canon);
    intent.sets = sets::matched_sets(
        &reqs,
        &intent.compounds,
        &shared,
        &passive_sets,
        &block::leaves(&blocks),
        &canon,
        &drawn,
        &hg,
        &cfg.process.unit,
        &mut intent.diagnostics,
    );
    // EXT-16: kind, class and style per set; the unit floors depend on the class.
    let leaves = block::leaves(&blocks);
    let n = netlist.devices.len();
    let leaf_idx = sets::device_index(n, leaves.iter().map(|b| b.devices.as_slice()));
    let passive_idx = sets::device_index(n, passive_sets.iter().map(|p| p.devices.as_slice()));
    let mut roles = Vec::new();
    for s in &intent.sets {
        let has = |d: DeviceId| s.members.iter().any(|m| m.device == d);
        let pair = |b: usize| leaves[b].devices.len() == 2 && leaves[b].devices.iter().all(|&d| has(d));
        let kinds: Vec<BlockKind> =
            sets::inside(&leaf_idx, s.members.iter().map(|m| m.device.0 as usize), pair).into_iter().map(|b| leaves[b].kind).collect();
        roles.push(kinds);
    }
    // The sidecar entry whose devices cover a set's members (EXT-26).
    let covering = |ds: &[DeviceId], s: &analog::intent::MatchSpec| s.members.iter().all(|m| ds.contains(&m.device));
    for (s, kinds) in intent.sets.iter_mut().zip(&roles) {
        let user_kind = cfg.classes.iter().find(|c| covering(&c.0, s)).and_then(|c| c.2);
        s.kind = user_kind.unwrap_or_else(|| class::kind_of(s, kinds, netlist.devices[s.members[0].device.0 as usize].kind));
    }
    // EXT-21: allowances split from spec sensitivities over the MOS sets a sidecar
    // OffsetBudget does not cover (a user budget wins over a computed one).
    if let Some(sens) = &ev.sens {
        let (ix, ins): (Vec<usize>, Vec<allocate::SetIn>) = (0..intent.sets.len())
            .filter(|&i| !cfg.offset_budgets.iter().any(|b| covering(&b.0, &intent.sets[i])))
            .filter_map(|i| allocate::set_in(&intent.sets[i], netlist, cfg.process.avt_mv_um).map(|s| (i, s)))
            .unzip();
        let (out, diags) = allocate::allocate(&ins, sens, cfg.policy.beta_target, cfg.policy.max_eta);
        for (i, (a, w)) in ix.into_iter().zip(out) {
            (intent.sets[i].allowance, intent.sets[i].weight) = (a, w);
        }
        intent.diagnostics.extend(diags);
    }
    let input = |i: usize| roles[i].contains(&BlockKind::DiffPair);
    let mut set_roles = Vec::with_capacity(intent.sets.len());
    let input_compounds: std::collections::HashSet<u16> =
        (0..intent.sets.len()).filter(|&j| input(j)).filter_map(|j| intent.sets[j].compound).collect();
    assert_eq!(roles.len(), intent.sets.len(), "one role list per set");
    for (i, kinds) in roles.iter().enumerate() {
        let s = &intent.sets[i];
        let role = if input(i) {
            class::SetRole::InputPair
        } else if kinds.iter().any(|k| matches!(k, BlockKind::Load | BlockKind::CascodePair))
            && s.compound.is_some_and(|c| input_compounds.contains(&c))
        {
            class::SetRole::LoadOfPair
        } else if let Some(&p) = sets::inside(&passive_idx, s.members.iter().map(|m| m.device.0 as usize), |p| {
            passive_sets[p].devices.iter().all(|d| s.members.iter().any(|m| m.device == *d))
        })
        .first()
        {
            passive_sets[p].role
        } else if matches!(s.origin, analog::intent::Origin::Pattern { template } if template.starts_with("bjt_ratioed_pair")) {
            // A ratioed pair's ΔV_BE is a bandgap core (H09-01).
            class::SetRole::BandgapCore
        } else if kinds.contains(&BlockKind::CurrentMirror) || s.origin == analog::intent::Origin::SharedBias {
            class::SetRole::BiasMirror
        } else {
            class::SetRole::Other
        };
        set_roles.push(role);
        let user = cfg.classes.iter().find(|c| covering(&c.0, s)).map(|c| c.1);
        let sigma = cfg.offset_budgets.iter().find(|b| covering(&b.0, s)).map(|b| b.1).or(cfg.offset_sigma_mv);
        // class_of reads only mV limits, so an allocated allowance sets a Voltage set's 6σ target.
        let alloc = s.allowance.filter(|_| s.kind == analog::intent::MatchKind::Voltage);
        let mut ctx = class::ClassCtx { user, spec_6sigma: sigma.map(|v| 6.0 * v).or(alloc.map(|a| 6.0 * a)), role, diags: &mut intent.diagnostics };
        let (mut c, mut src) = class::class_of(s, &mut ctx);
        // EXT-21: a set explaining under `minor_weight` of every spec's variance is Minimal.
        if src != analog::intent::ClassSource::User && s.weight.is_some_and(|w| w < cfg.policy.minor_weight) {
            (c, src) = (analog::intent::MatchClass::Minimal, analog::intent::ClassSource::Spec);
        }
        let source = |d: DeviceId| {
            let i = d.0 as usize;
            hg.terminals[i].iter().position(|t| t == "S" || t == "E").map(|k| hg.device_nets[i][k])
        };
        let shares_source = s.members.iter().all(|m| source(m.device).is_some() && source(m.device) == source(s.members[0].device));
        let s = &mut intent.sets[i];
        (s.class, s.class_source, s.style) = (c, src, class::style_of(c, s.kind, shares_source));
        let ids: Vec<DeviceId> = s.members.iter().map(|m| m.device).collect();
        if let Ok((u, units)) = sets::unitize_set(&ids, &passive_sets, &drawn, netlist.devices[ids[0].0 as usize].kind, c, &cfg.process.unit) {
            s.unit = Some(u);
            for (m, (p, ser)) in s.members.iter_mut().zip(units) {
                (m.parallel, m.series) = (p, ser);
            }
            if c == analog::intent::MatchClass::Exceptional && s.members.iter().any(|m| m.series > 1) {
                intent.diagnostics.push(analog::intent::Diagnostic {
                    kind: "series_units_exceptional",
                    devices: ids,
                    message: "series units carry inherent mismatch (Hastings H13-22)".into(),
                });
            }
        }
    }
    // EXT-14 step 9 (card D-d): a compound holding an Exceptional Voltage set is Perfect.
    let mut perfect = vec![false; intent.compounds.len()];
    for s in intent.sets.iter().filter(|s| s.kind == analog::intent::MatchKind::Voltage && s.class == analog::intent::MatchClass::Exceptional) {
        s.compound.and_then(|c| perfect.get_mut(c as usize)).into_iter().for_each(|p| *p = true);
    }
    for (c, &perfect) in intent.compounds.iter_mut().zip(&perfect) {
        c.kind = if perfect { analog::intent::SymKind::Perfect } else { analog::intent::SymKind::Mirror };
    }
    sets::set_pairs(&mut intent.compounds, &intent.sets);
    let load_leaf = {
        let mut v = vec![false; n];
        leaves.iter().filter(|b| b.kind == BlockKind::Load).flat_map(|b| &b.devices).for_each(|d| v[d.0 as usize] = true);
        v
    };
    // EXT-18: classes that need sets, then device roles, then current-source gates.
    let load_af = classify::net_load_af(&hg, &gates, cfg.process.gate_af_per_um2, &cfg.loads);
    // AA-25: a budgeted-class net on a channel and no plate whose load is off-netlist.
    if cfg.process.gate_af_per_um2.is_some() {
        let (mut channel, mut plate) = (vec![false; load_af.len()], vec![false; load_af.len()]);
        for (d, nets) in hg.device_nets.iter().enumerate() {
            for (t, net) in hg.terminals[d].iter().zip(nets) {
                match terms::term_role(hg.kinds[d], t) {
                    terms::TermRole::Channel => channel[net.0 as usize] = true,
                    terms::TermRole::Plate => plate[net.0 as usize] = true,
                    _ => {}
                }
            }
        }
        let rail = |i: usize| {
            matches!(
                net_classes[i].class,
                analog::metadata::NetClass::Supply | analog::metadata::NetClass::Ground | analog::metadata::NetClass::Substrate
            )
        };
        if (0..load_af.len()).any(|i| channel[i] && !plate[i] && load_af[i].is_none() && !rail(i)) {
            missing.push(("ParasiticBudget", "external load of drain-only nets: sidecar Load (AA-25)"));
        }
    }
    intent.nets = classify::refine(
        &mut net_classes,
        &classify::RefineCtx {
            hg: &hg,
            matches: &all,
            sets: &intent.sets,
            set_roles: &set_roles,
            passive: &passive_sets,
            user: &user,
            ev,
            user_classes: &cfg.net_classes,
            ports: &netlist.ports,
            load_af: &load_af,
        },
    );
    intent.devices = evidence::device_facts(netlist, ev.op.as_ref(), &net_classes, &shared, &load_leaf);
    // EXT-28: user orders, then current chains a user order does not already give (either sense),
    // then the signal stages, then the EXT-27 arrays.
    intent.order.extend(cfg.order.iter().cloned());
    let norm = |steps: &[Vec<DeviceId>]| -> Vec<Vec<u16>> {
        steps
            .iter()
            .map(|s| {
                let mut v: Vec<u16> = s.iter().map(|d| d.0).collect();
                v.sort_unstable();
                v
            })
            .collect()
    };
    let user_steps: Vec<Vec<Vec<u16>>> = cfg.order.iter().map(|o| norm(&o.steps)).collect();
    let chains = flow::current_paths(&hg, ev.op.as_ref(), &net_classes, &canon);
    let i_max = chains.iter().map(|c| c.1).fold(0.0, f64::max);
    for (steps, i) in chains {
        let (fwd, mut rev) = (norm(&steps), norm(&steps));
        rev.reverse();
        if !user_steps.iter().any(|u| *u == fwd || *u == rev) {
            let weight = if i_max > 0.0 { (i / i_max) as f32 } else { 1.0 };
            intent.order.push(analog::intent::Order { steps, dir: analog::intent::AxisDir::V, reversible: true, weight });
        }
    }
    let stages = flow::stage_order(&hg, &net_classes, &netlist.ports, &canon);
    if stages.len() >= 2 {
        intent.order.push(analog::intent::Order { steps: stages, dir: analog::intent::AxisDir::H, reversible: true, weight: 1.0 });
    }
    intent.order.extend(array_orders);
    // EXT-29: bias and structure audit over the final sets; cascode mirrors deduplicated.
    let cascodes: std::collections::BTreeSet<[u16; 4]> = all
        .iter()
        .filter(|m| matches!(m.template, "cascode_mirror" | "wide_swing_cascode_mirror" | "low_voltage_cascode_mirror"))
        .map(|m| std::array::from_fn(|k| m.instances[k] as u16))
        .collect();
    let cascodes: Vec<[DeviceId; 4]> = cascodes.into_iter().map(|c| c.map(DeviceId)).collect();
    intent.diagnostics.extend(audit::audit(&intent, netlist, &cascodes, ev.op.as_ref()));
    // The gate of every CurrentSource is a bias line. Cascode/CurrentSource-by-class
    // gates already are, so one pass is a fixpoint. A sidecar class (User) wins (EXT-26 step 4).
    for (d, f) in intent.devices.iter().enumerate() {
        let g = (f.role == analog::intent::DeviceRole::CurrentSource).then(|| pattern::pin_net(&hg, d as u32, "G")).flatten();
        let g = g.filter(|g| intent.nets[g.0 as usize].evidence != analog::intent::EvidenceLevel::User);
        if let Some(g) =
            g.filter(|g| matches!(net_classes[g.0 as usize].class, analog::metadata::NetClass::Signal | analog::metadata::NetClass::Sensitive))
        {
            classify::set_class(&mut net_classes[g.0 as usize], analog::metadata::NetClass::Bias, load_af[g.0 as usize]);
            intent.nets[g.0 as usize].evidence = analog::intent::EvidenceLevel::OpPoint;
        }
    }
    // EXT-23: substrate tags; an aggressor is never a victim.
    (intent.aggressors, intent.victims) = substrate::tag(netlist, &net_classes, &intent.sets);
    intent.aggressors.extend(substrate::injectors(netlist, &net_classes, ev.op.as_ref(), cfg.policy.inj_series_ohm, &mut missing));
    let victim = substrate::victim_mask(n, &intent.victims);
    let mut aggressor = vec![false; n];
    intent
        .aggressors
        .iter()
        .filter(|a| matches!(a.inject, analog::intent::Inject::Switching | analog::intent::Inject::Capacitive))
        .for_each(|a| aggressor[a.device.0 as usize] = true);
    let mut routing = extract::routing(&hg, &net_classes, &gates, &cfg.process, &mut intent, &set_roles, &cfg.policy, &netlist.ports, ev.op.as_ref());
    // EXT-25: R/C classes from the spec sensitivities (the rows themselves are the
    // library's, which owns the post-layout loop); evidence wins only where it is not Unknown.
    match (&ev.sens, cfg.process.wire_af_per_um) {
        (Some(sens), Some(af_per_um)) => {
            let (_, rc, diags) = budget::rows(sens, af_per_um / 1000.0, cfg.process.wire_ohm_per_um, &cfg.policy);
            for (n, c) in rc {
                if let Some(f) = intent.nets.get_mut(n.0 as usize) {
                    f.rc = c;
                }
            }
            intent.diagnostics.extend(diags);
        }
        (Some(_), None) => missing.push(("PerformanceBudget", "deck wire C")),
        _ => {}
    }
    intent.kelvins.extend(cfg.kelvins.iter().cloned());
    if ev.op.is_some() && ev.probe_bias {
        intent.diagnostics.push(analog::intent::Diagnostic {
            kind: "probe_bias",
            devices: vec![],
            message: "device regions from a synthesised probe bench".into(),
        });
    }
    let mut placement = emit::placement(&intent, &blocks, &cfg.groups, netlist, &drawn, &cfg.process, cfg.offset_sigma_mv, &cfg.policy);
    // Same entry of `blocks`, glue excluded: glue is no stage.
    let mut block_of = vec![usize::MAX; netlist.devices.len()];
    for (bi, b) in blocks.iter().enumerate().filter(|(_, b)| b.kind != BlockKind::Glue) {
        b.devices.iter().for_each(|d| block_of[d.0 as usize] = bi);
    }
    // Related devices sit together by design (AA-13): same block, same matched
    // set, or same symmetry compound (EXT-23 step 4: strongarm's `mn0` on the axis).
    let set_devs: Vec<Vec<DeviceId>> = intent.sets.iter().map(|s| s.members.iter().map(|m| m.device).collect()).collect();
    let compound_devs: Vec<Vec<DeviceId>> =
        intent.compounds.iter().map(|c| c.pairs.iter().flat_map(|&(a, b)| [a, b]).chain(c.selfs.iter().copied()).collect()).collect();
    let in_set = sets::device_index(n, set_devs.iter().map(Vec::as_slice));
    let in_compound = sets::device_index(n, compound_devs.iter().map(Vec::as_slice));
    let shares = |idx: &[Vec<usize>], a: usize, v: usize| idx[a].iter().any(|i| idx[v].contains(i));
    let related =
        |a: usize, v: usize| (block_of[a] != usize::MAX && block_of[a] == block_of[v]) || shares(&in_set, a, v) || shares(&in_compound, a, v);
    let p = &cfg.process;
    // GAP-09 (b): a sidecar GroupBlocks pull (≤ proximity_nm) on an aggressor–victim
    // pair that Isolation would push ≥ isolation_min_nm apart: the user pull wins.
    let iso_nm = emit::isolation_min_nm(p.substrate, p.epi_nm);
    let clash: Vec<(usize, usize, u32)> = cfg
        .groups
        .iter()
        .flat_map(|(e, g)| g.iter().skip(1).map(move |m| (g[0].0 as usize, m.0 as usize, *e)))
        .filter(|&(x, y, _)| ((aggressor[x] && victim[y]) || (aggressor[y] && victim[x])) && !related(x, y) && cfg.policy.proximity_nm < iso_nm)
        .collect();
    let related = |a: usize, v: usize| related(a, v) || clash.iter().any(|&(x, y, _)| (x, y) == (a, v) || (y, x) == (a, v));
    if let Some(why) = emit::isolation(&aggressor, &victim, &related, p.substrate, p.epi_nm, &mut placement) {
        missing.push(("Isolation", why));
    }
    if std::env::var("LOCAL_NO_SB").is_err() {
        emit::substrate_balance(&aggressor, &blocks, &block_of, &mut placement);
    }

    let touched = emit::tag_batches(&mut placement, &mut routing, &blocks, &block_of);

    // GAP-09 (b) report: the user pull's id, then the Isolation batch's (left
    // out when the dropped pairs were its only ones, so no batch was emitted).
    let first_id = |f: &dyn Fn(&dyn analog::RuleBatch<analog::base::Layout>) -> bool| {
        placement.budget.iter().chain(&placement.cost).find(|b| f(b.as_ref())).and_then(|b| b.meta()).map(|m| m.id)
    };
    let iso_id = first_id(&|b| b.kind().ends_with("::Isolation"));
    for &(x, y, e) in &clash {
        let user_id = first_id(&|b| b.meta().is_some_and(|m| m.origin == analog::intent::Origin::User { index: e }));
        let ids: Vec<_> = user_id.into_iter().chain(iso_id).collect();
        let what = format!("GroupBlocks Proximity ≤ {} nm vs Isolation ≥ {iso_nm} nm; Isolation dropped", cfg.policy.proximity_nm);
        intent.diagnostics.push(conflict::diag(&ids, vec![DeviceId(x as u16), DeviceId(y as u16)], what));
    }

    let coverage = (0..netlist.devices.len())
        .map(|d| {
            let c = if touched[d] {
                Coverage::Constrained
            } else if block_of[d] != usize::MAX {
                Coverage::Grouped(blocks[block_of[d]].template)
            } else if cfg.do_not_identify.contains(&(d as u32)) {
                Coverage::Unconstrained("do_not_identify")
            } else if size::unknown_size(netlist.devices[d].kind, &drawn[d]) {
                Coverage::Unconstrained("unknown size")
            } else {
                Coverage::Unconstrained("no pattern")
            };
            (DeviceId(d as u16), c)
        })
        .collect();

    // REL-07: guard rings by role, from the EXT-23 tags.
    let mut constraints = constraints::assemble(netlist, &drawn, &blocks, &intent.sets);
    let (rings, notes) = rings::plan_tagged(netlist, &net_classes, &intent, &aggressor, &victim, cfg);
    constraints.guard_rings.extend(rings);
    missing.extend(notes);

    let mut out = Problem {
        // One axis per compound (`Compound.axis`); a spare one when there is none.
        axis_count: intent.compounds.len().max(1),
        intent,
        placement,
        routing,
        coverage,
        constraints,
        net_classes,
        groups,
        abutment,
        blocks,
        missing,
    };
    let d = conflict::check(&out, netlist);
    out.intent.diagnostics.extend(d);
    out
}

/// Gate area `W_total·L·m` of a FET, µm²; `0` for anything else or when the
/// netlist omits W/L.
pub(crate) fn gate_um2(dev: &analog::base::netlist::Device) -> f32 {
    dev.gate_area_um2() as f32
}

/// SPICE param `key` of `dev`, or `default`.
pub(crate) fn param(dev: &analog::base::netlist::Device, key: &str, default: i64) -> i64 {
    dev.params.iter().find(|(k, _)| k == key).map_or(default, |&(_, v)| v)
}

#[cfg(test)]
mod tests;
