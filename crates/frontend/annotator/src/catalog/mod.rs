//! Analog primitive pattern catalog.
//!
//! ~100 structural patterns for automatic netlist annotation.
//! Derived from ALIGN's basic_template library, Razavi/Allen-Holberg/
//! Gray-Meyer canonical topologies, and the GANA primitive taxonomy.
//!
//! Add a new pattern: define a `const Pattern` in its device-count file
//! (`two_device`, `three_device`, `four_device`, `five_plus`), add it to `PATTERNS`.
//! The engine in `pattern.rs` picks it up automatically — no switch
//! statements, no registration, no match arms.
//!
//! ## Priority scheme
//!
//! Larger patterns (more devices) get higher priority so they match first,
//! preventing their sub-structures from being consumed as smaller patterns.
//!
//!   8-device composites : 50–59
//!   6-device composites : 40–49
//!   5-device composites : 30–39
//!   4-device composites : 20–29
//!   3-device patterns   : 14–19
//!   2-device patterns   :  5–13
//!
//! Within a size tier, more-constrained patterns (exact sizing, diode
//! requirements) get higher priority than loosely-constrained ones.
//!
//! Contents:
//! - `pub use five_plus::*`
//! - `pub use four_device::*`
//! - `pub use three_device::*`
//! - `pub use two_device::*`
//! - `pub const PATTERNS`
//! - `pub struct Roles`
//! - `pub const ROLES`
//! - `pub fn roles_of(p: &Pattern) -> Roles`

mod five_plus;
mod four_device;
mod three_device;
mod two_device;

pub use five_plus::*;
pub use four_device::*;
pub use three_device::*;
pub use two_device::*;

use crate::block::BlockKind::{self, CascodePair, CurrentMirror, DiffPair, Load};
use crate::pattern::{DiodeReq, Pattern, PinLink, PinRel, SizeMatch, Slot, SlotKind};

/// All registered patterns, ordered by priority (highest first).
/// The engine applies greedy non-overlapping selection, so larger/
/// more-specific patterns consume devices before smaller ones can.
pub const PATTERNS: &[Pattern] = &[
    // ── 8-device composites (50–59) ──
    TELESCOPIC_OTA_FULL,
    // ── 6-device composites (40–49) ──
    GILBERT_CELL,
    VCO_CORE_WITH_TAILS,
    FOLDED_CASCODE_CORE,
    TELESCOPIC_OTA_CORE,
    // ── 5-device composites (30–39) ──
    DIFF_PAIR_CROSS_COUPLED_LOAD,
    FIVE_TRANSISTOR_OTA,
    DIFF_PAIR_CASCODE_LOAD,
    OTA_SELF_BIASED_LOAD,
    CASCODED_DIFF_PAIR_WITH_TAIL,
    WIDE_SWING_CASCODE_MIRROR_5,
    DIFF_OUTPUT_STAGE,
    // ── 4-device composites (20–29) ──
    CROSS_COUPLED_INVERTERS,
    WIDE_SWING_CASCODE_MIRROR,
    IMPROVED_WILSON_MIRROR_4,
    CASCODE_MIRROR,
    REGULATED_CASCODE_MIRROR,
    WILSON_MIRROR_4,
    INVERTER_BUFFER,
    DIFF_PAIR_WITH_MIRROR_LOAD,
    COMPLEMENTARY_DIFF_PAIR,
    DIFF_PAIR_WITH_ACTIVE_LOAD,
    DIFF_PAIR_WITH_CASCODES,
    FOLDED_LOAD_PAIR,
    CASCODE_PAIR,
    CHARGE_PUMP_CELL,
    DAC_CURRENT_CELL,
    SCHMITT_TRIGGER,
    NAND_GATE,
    NOR_GATE,
    NAND_CROSS_COUPLED,
    LOW_VOLTAGE_CASCODE_MIRROR,
    CASCODE_PAIR_SYMMETRIC,
    DIFF_PAIR_WITH_SPLIT_CASCODES,
    CASCODE_CURRENT_SOURCE_PAIR,
    TRANSMISSION_GATE_PAIR,
    MIRROR_WITH_DUAL_OUTPUT,
    LEVEL_SHIFTER,
    CURRENT_MIRROR_4,
    SERIES_STACK_4,
    BANDGAP_MIRROR_PAIR,
    FEEDBACK_PAIR,
    // ── 3-device patterns (14–19) ──
    DIFF_PAIR_WITH_TAIL,
    IMPROVED_WILSON_MIRROR,
    WILSON_MIRROR,
    TRIPLE_CASCODE,
    REGULATED_CASCODE,
    SUPER_SOURCE_FOLLOWER,
    FLIPPED_VOLTAGE_FOLLOWER,
    SELF_BIASED_CASCODE,
    SOURCE_FOLLOWER_WITH_MIRROR,
    DIFF_PAIR_DIODE_LOAD,
    DIFF_PAIR_WITH_REFERENCE,
    MIRROR_WITH_CASCODE_OUTPUT,
    CASCODED_REFERENCE,
    SOOCH_MIRROR,
    CASCODE_WITH_MIRROR_BIAS,
    CASCODE_WITH_DEGENERATION,
    INVERTER_WITH_TAIL,
    ACTIVE_LOADED_INVERTER,
    BOOTSTRAPPED_SWITCH,
    CMFB_SENSE_PAIR,
    CURRENT_MIRROR_3,
    CURRENT_MIRROR_1_TO_2,
    ACTIVE_LOAD_3,
    PUSH_PULL_WITH_BIAS,
    BIAS_CHAIN_3,
    // ── 2-device patterns (5–13) ──
    CMOS_INVERTER,
    TRANSMISSION_GATE,
    DIFF_PAIR,
    PUSH_PULL_PAIR,
    COMPLEMENTARY_SOURCE_FOLLOWER,
    ANTI_PARALLEL_SWITCH,
    CASCODE_DIODE_TOP,
    CROSS_COUPLED,
    CROSS_COUPLED_SPLIT_SOURCE,
    BETA_MULTIPLIER_CORE,
    ESD_DIODE_CLAMP,
    CASCODE_MATCHED,
    CURRENT_MIRROR,
    DIFF_SWITCH,
    CURRENT_MIRROR_OUTPUT_PAIR,
    SOURCE_FOLLOWER,
    DIODE_LOAD_PAIR,
    LATCH_HALF,
    COMMON_MODE_SENSE_PAIR,
    COMMON_GATE_PAIR,
    SERIES_STACK,
    CASCODE,
    DEGENERATION_PAIR,
    SWITCH_PAIR,
    BJT_DIFF_PAIR_NPN,
    BJT_DIFF_PAIR_PNP,
    BJT_RATIOED_PAIR_NPN,
    BJT_RATIOED_PAIR_PNP,
    BJT_MIRROR_NPN,
    BJT_MIRROR_PNP,
];

// ═══════════════════════════════════════════════════════════════════════
//  Declared slot roles (plan-01 EXT-05; survey Fig. 3.11 block → requirement)
// ═══════════════════════════════════════════════════════════════════════

/// What a pattern's slots mean for placement: the composite's children and
/// on-axis members, built by [`crate::Block::from_match`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Roles {
    /// Constraint-bearing couples: (slot a, slot b, leaf kind).
    pub pairs: &'static [(u8, u8, BlockKind)],
    /// Slots on the structure's symmetry axis (tail, shared bias device).
    pub selfs: &'static [u8],
    /// Adjacent-but-not-matched couples (cascode over its source): a `Stack` leaf.
    pub prox: &'static [(u8, u8)],
}

/// Composite declarations keyed by `Pattern::name`. A composite absent here is
/// recognised and reported but yields no constraint until a source-backed
/// declaration is added.
pub const ROLES: &[(&str, Roles)] = &[
    ("cmos_inverter", roles(&[], &[], &[(0, 1)])),
    ("five_transistor_ota", roles(&[(0, 1, DiffPair), (2, 3, Load)], &[4], &[])),
    ("diff_pair_with_active_load", roles(&[(0, 1, DiffPair), (2, 3, Load)], &[], &[])),
    ("diff_pair_with_mirror_load", roles(&[(0, 1, DiffPair), (2, 3, Load)], &[], &[])),
    ("diff_pair_with_tail", roles(&[(0, 1, DiffPair)], &[2], &[])),
    ("cascoded_diff_pair_with_tail", roles(&[(0, 1, DiffPair), (2, 3, CascodePair)], &[4], &[(0, 2), (1, 3)])),
    ("diff_pair_cascode_load", roles(&[(0, 1, DiffPair), (2, 3, CascodePair)], &[4], &[(0, 2), (1, 3)])),
    ("telescopic_ota_core", roles(&[(0, 1, DiffPair), (2, 3, CascodePair), (4, 5, Load)], &[], &[(0, 2), (1, 3), (2, 4), (3, 5)])),
    // Slot 7 (load cascode bias) gets no role: it is not on the axis.
    ("telescopic_ota_full", roles(&[(0, 1, DiffPair), (2, 3, CascodePair), (4, 5, Load)], &[6], &[(0, 2), (1, 3), (2, 4), (3, 5)])),
    ("folded_cascode_core", roles(&[(0, 1, DiffPair), (2, 3, CascodePair), (4, 5, Load)], &[], &[(4, 2), (5, 3)])),
    ("cross_coupled_inverters", roles(&[(0, 1, DiffPair), (2, 3, DiffPair)], &[], &[(0, 2), (1, 3)])),
    ("vco_core_with_tails", roles(&[(0, 1, DiffPair), (2, 3, DiffPair)], &[4, 5], &[])),
    // The mirror images (M3,M6), (M4,M5); the switching quad is two 2-member sets
    // until a quad-set rule exists (plan-01 "Cut or deferred items").
    ("gilbert_cell", roles(&[(0, 1, DiffPair), (2, 5, DiffPair), (3, 4, DiffPair)], &[], &[])),
    ("cascode_mirror", roles(&[(0, 1, CurrentMirror), (2, 3, CascodePair)], &[], &[(0, 2), (1, 3)])),
    ("wide_swing_cascode_mirror", roles(&[(0, 1, CurrentMirror), (2, 3, CascodePair)], &[], &[(0, 2), (1, 3)])),
    ("low_voltage_cascode_mirror", roles(&[(0, 1, CurrentMirror), (2, 3, CascodePair)], &[], &[(0, 2), (1, 3)])),
    ("current_mirror_3", roles(&[(0, 1, CurrentMirror), (0, 2, CurrentMirror)], &[], &[])),
    ("current_mirror_4", roles(&[(0, 1, CurrentMirror), (0, 2, CurrentMirror), (0, 3, CurrentMirror)], &[], &[])),
    ("current_mirror_1_to_2", roles(&[(0, 1, CurrentMirror), (0, 2, CurrentMirror)], &[], &[])),
    // Slots 0,1 = one polarity's pair, 2,3 = the complement's (rail-to-rail input).
    ("complementary_diff_pair", roles(&[(0, 1, DiffPair), (2, 3, DiffPair)], &[], &[])),
    // EXT-19: a ratioed pair is matched (a MatchBlock edge); `class::kind_of` makes it Voltage (ΔV_BE).
    ("bjt_ratioed_pair_npn", roles(&[(0, 1, CurrentMirror)], &[], &[])),
    ("bjt_ratioed_pair_pnp", roles(&[(0, 1, CurrentMirror)], &[], &[])),
    ("bjt_diff_pair_npn", roles(&[(0, 1, DiffPair)], &[], &[])),
    ("bjt_diff_pair_pnp", roles(&[(0, 1, DiffPair)], &[], &[])),
    ("bjt_mirror_npn", roles(&[(0, 1, CurrentMirror)], &[], &[])),
    ("bjt_mirror_pnp", roles(&[(0, 1, CurrentMirror)], &[], &[])),
];

// ── Shorthand constructors ──

const S_ANY: Slot = Slot { kind: SlotKind::AnyFet, size_match: SizeMatch::Any, diode: DiodeReq::Any, gate_is_signal: false };

/// `ROLES` entry; else for a 2-slot pattern today's [`BlockKind::from_template`]
/// mapping (DiffPair/CurrentMirror/Load → pairs (0,1,k); Stack → prox (0,1));
/// else none.
#[must_use]
pub fn roles_of(p: &Pattern) -> Roles {
    if let Some(&(_, r)) = ROLES.iter().find(|(n, _)| *n == p.name) {
        return r;
    }
    if p.slots.len() != 2 {
        return Roles::default();
    }
    match BlockKind::from_template(p.name, 2) {
        DiffPair => roles(&[(0, 1, DiffPair)], &[], &[]),
        CurrentMirror => roles(&[(0, 1, CurrentMirror)], &[], &[]),
        Load => roles(&[(0, 1, Load)], &[], &[]),
        BlockKind::Stack => roles(&[], &[], &[(0, 1)]),
        _ => Roles::default(),
    }
}

// ---- private ----

/// Same type as slot 0, exact W+L match, no diode constraint.
const fn same0_exact() -> Slot {
    Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: false }
}

/// Same type as slot 0, same L only.
const fn same0_samel() -> Slot {
    Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Any, gate_is_signal: false }
}

/// Same type as slot 0, any sizing.
const fn same0() -> Slot {
    Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::Any, diode: DiodeReq::Any, gate_is_signal: false }
}

/// Same type as a given slot, any sizing.
const fn same_as(r: u8) -> Slot {
    Slot { kind: SlotKind::SameTypeAs(r), size_match: SizeMatch::Any, diode: DiodeReq::Any, gate_is_signal: false }
}

/// Same type as a given slot, exact W+L match with that slot.
const fn same_exact(r: u8) -> Slot {
    Slot { kind: SlotKind::SameTypeAs(r), size_match: SizeMatch::ExactAs(r), diode: DiodeReq::Any, gate_is_signal: false }
}

/// Complement (NMOS<->PMOS) of a given slot.
const fn comp(r: u8) -> Slot {
    Slot { kind: SlotKind::ComplementOf(r), size_match: SizeMatch::Any, diode: DiodeReq::Any, gate_is_signal: false }
}

/// Shorthand link: two pins on the same net.
const fn eq(a: u8, pa: &'static str, b: u8, pb: &'static str) -> PinLink {
    PinLink { a, pin_a: pa, b, pin_b: pb, rel: PinRel::Same }
}

/// Shorthand link: two pins on different nets.
const fn ne(a: u8, pa: &'static str, b: u8, pb: &'static str) -> PinLink {
    PinLink { a, pin_a: pa, b, pin_b: pb, rel: PinRel::Diff }
}

/// Shorthand link: two pins on the same net, and that net's role is `Signal`.
const fn eq_sig(a: u8, pa: &'static str, b: u8, pb: &'static str) -> PinLink {
    PinLink { a, pin_a: pa, b, pin_b: pb, rel: PinRel::SameSignal }
}

const fn roles(pairs: &'static [(u8, u8, BlockKind)], selfs: &'static [u8], prox: &'static [(u8, u8)]) -> Roles {
    Roles { pairs, selfs, prox }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{DiodeReq, PinRel, SizeMatch, SlotKind};
    use analog::base::ids::NetId;
    use analog::base::netlist::{Device, DeviceKind, Net, Netlist};
    use std::collections::{HashMap, HashSet};

    /// The slots `kind` and `size_match` refer to (0, 1 or 2 of them).
    fn back_refs(slot: &Slot) -> impl Iterator<Item = u8> {
        let kr = match slot.kind {
            SlotKind::SameTypeAs(r) | SlotKind::ComplementOf(r) | SlotKind::SameKindAs(r) => Some(r),
            SlotKind::AnyFet | SlotKind::Kind(_) => None,
        };
        let sr = match slot.size_match {
            SizeMatch::ExactAs(r) | SizeMatch::SameLAs(r) => Some(r),
            _ => None,
        };
        [kr, sr].into_iter().flatten()
    }

    /// Oracle: structural invariants of the table: unique names, slot references in range.
    #[test]
    fn catalog_is_well_formed() {
        let mut names = HashSet::new();
        let mut shapes = HashSet::new();
        for p in PATTERNS {
            assert!(names.insert(p.name), "duplicate pattern name {}", p.name);
            for (k, slot) in p.slots.iter().enumerate() {
                for r in back_refs(slot) {
                    assert!((r as usize) < k, "{}: slot {k} refers forward to slot {r}", p.name);
                }
            }
            for l in p.links {
                assert!((l.a as usize) < p.slots.len() && (l.b as usize) < p.slots.len(), "{}: link slot out of range", p.name);
                for (slot, pin) in [(l.a, l.pin_a), (l.b, l.pin_b)] {
                    // A bipolar slot (EXT-19) is `Kind(Npn|Pnp)` or `SameKindAs` one.
                    let bjt = matches!(p.slots[slot as usize].kind, SlotKind::Kind(DeviceKind::Npn | DeviceKind::Pnp) | SlotKind::SameKindAs(_));
                    let ok: &[&str] = if bjt { &["C", "B", "E"] } else { &["G", "D", "S", "B"] };
                    assert!(ok.contains(&pin), "{}: link pin {pin} not {ok:?}", p.name);
                }
            }
            let mut link_keys: Vec<String> = p.links.iter().map(|l| format!("{l:?}")).collect();
            link_keys.sort_unstable();
            let shape = (format!("{:?}", p.slots), link_keys);
            assert!(shapes.insert(shape), "{}: duplicate of another pattern's shape", p.name);
        }
    }

    /// Terminal index within a slot's 4-pin block: FET `G/D/S/B = 0..3`, BJT
    /// `B/C/E = 0..2` (base, collector, emitter in the gate, drain, source places).
    fn pin_idx(slot: u8, pin: &str, bjt: bool) -> u32 {
        slot as u32 * 4
            + match (bjt, pin) {
                (false, "G") | (true, "B") => 0,
                (false, "D") | (true, "C") => 1,
                (false, "S") | (true, "E") => 2,
                (false, "B") => 3,
                _ => unreachable!("not a pin of this family"),
            }
    }

    /// The smallest netlist a pattern can match itself against: one device per
    /// slot, pins joined exactly as the pattern's links (and diode requirement)
    /// demand, every other `B` tied to its device's own rail.
    fn minimal_netlist(p: &'static Pattern) -> Netlist {
        let n = p.slots.len();
        let mut kinds = vec![DeviceKind::Nmos; n];
        for (k, slot) in p.slots.iter().enumerate() {
            kinds[k] = match slot.kind {
                SlotKind::AnyFet => DeviceKind::Nmos,
                SlotKind::SameTypeAs(r) => kinds[r as usize],
                SlotKind::ComplementOf(r) => {
                    if kinds[r as usize] == DeviceKind::Nmos {
                        DeviceKind::Pmos
                    } else {
                        DeviceKind::Nmos
                    }
                }
                SlotKind::Kind(k) => k,
                SlotKind::SameKindAs(r) => kinds[r as usize],
            };
        }

        let bjt = |k: u8| matches!(kinds[k as usize], DeviceKind::Npn | DeviceKind::Pnp);
        let mut uf = analog::base::UnionFind::new(n * 4);
        for l in p.links {
            if matches!(l.rel, PinRel::Same | PinRel::SameSignal) {
                uf.union(pin_idx(l.a, l.pin_a, bjt(l.a)), pin_idx(l.b, l.pin_b, bjt(l.b)));
            }
        }
        for (k, slot) in p.slots.iter().enumerate() {
            if matches!(slot.diode, DiodeReq::Required) {
                uf.union(pin_idx(k as u8, "G", false), pin_idx(k as u8, "D", false));
            }
        }
        let linked_b: Vec<bool> = (0..n)
            .map(|k| p.links.iter().any(|l| !bjt(k as u8) && ((l.a as usize == k && l.pin_a == "B") || (l.b as usize == k && l.pin_b == "B"))))
            .collect();

        let mut nets: Vec<Net> = Vec::new();
        let mut net_of_root: HashMap<u32, NetId> = HashMap::new();
        let mut ensure = |root: u32, nets: &mut Vec<Net>| -> NetId {
            *net_of_root.entry(root).or_insert_with(|| {
                let id = NetId(nets.len() as u16);
                nets.push(Net { name: format!("n{}", nets.len()) });
                id
            })
        };
        let mut rail: HashMap<&'static str, NetId> = HashMap::new();
        let mut rail_net = |name: &'static str, nets: &mut Vec<Net>| -> NetId {
            *rail.entry(name).or_insert_with(|| {
                let id = NetId(nets.len() as u16);
                nets.push(Net { name: name.to_string() });
                id
            })
        };

        let devices = (0..n)
            .map(|k| {
                let pins: &[&str] = if bjt(k as u8) { &["C", "B", "E"] } else { &["G", "D", "S", "B"] };
                let terminals = pins
                    .iter()
                    .map(|&pin| {
                        let net = if pin == "B" && !bjt(k as u8) && !linked_b[k] {
                            let name = if kinds[k] == DeviceKind::Nmos { "VSS" } else { "VDD" };
                            rail_net(name, &mut nets)
                        } else {
                            let root = uf.find(pin_idx(k as u8, pin, bjt(k as u8)));
                            ensure(root, &mut nets)
                        };
                        (pin.to_string(), net)
                    })
                    .collect();
                Device {
                    name: format!("D{k}"),
                    kind: kinds[k],
                    model: String::new(),
                    terminals,
                    params: vec![("w".into(), 1_000), ("l".into(), 1_000)],
                }
            })
            .collect();
        Netlist { devices, nets, ..Default::default() }
    }

    /// Oracle: structural invariants: every ROLES row names a pattern and slots in range.
    #[test]
    fn roles_are_well_formed() {
        let mut names = HashSet::new();
        for &(name, r) in ROLES {
            assert!(names.insert(name), "duplicate ROLES entry {name}");
            let p = PATTERNS.iter().find(|p| p.name == name).unwrap_or_else(|| panic!("ROLES {name} not in PATTERNS"));
            let n = p.slots.len() as u8;
            let paired: Vec<u8> = r.pairs.iter().flat_map(|&(a, b, _)| [a, b]).collect();
            let slots = paired.iter().chain(r.selfs).chain(r.prox.iter().flat_map(|(a, b)| [a, b]));
            assert!(slots.clone().all(|&s| s < n), "{name}: slot index out of range");
            // A mirror's reference (slot 0) is in every output's pair; nothing else repeats.
            let shared_ref = r.pairs.iter().all(|p| p.0 == 0);
            for (i, s) in paired.iter().enumerate() {
                assert!(shared_ref || !paired[..i].contains(s), "{name}: slot {s} in two pairs");
            }
            assert!(r.selfs.iter().all(|s| !paired.contains(s)), "{name}: a self is also paired");
        }
    }

    /// Oracle: round-trip: a netlist built from each pattern's slots and links is matched by that pattern.
    #[test]
    fn every_pattern_matches_its_own_minimal_netlist() {
        for p in PATTERNS {
            let nl = minimal_netlist(p);
            let hg = analog::base::BipartiteHypergraph::from_netlist(&nl);
            let cfg = crate::netrole::AnnotationConfig::default();
            let roles = crate::netrole::classify_nets(&hg, &cfg);
            let mut models = Vec::new();
            let drawn: Vec<_> = nl.devices.iter().map(|d| crate::size::drawn(d, &mut models)).collect();
            let allowed = vec![true; p.slots.len()];
            let rank: Vec<u32> = (0..p.slots.len() as u32).collect();
            let pins = crate::pattern::pins(&hg);
            let ms = crate::pattern::matches(p, &hg, &pins, &crate::pattern::on_pin(&hg, &pins), &drawn, &roles, &allowed, &rank);
            let want: Vec<u32> = (0..p.slots.len() as u32).collect();
            assert!(
                ms.iter().any(|m| {
                    let mut s = m.instances.clone();
                    s.sort_unstable();
                    s == want
                }),
                "{}: no match of its own minimal netlist ({:?})",
                p.name,
                ms
            );
        }
    }
}
