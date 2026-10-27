//! Three-device patterns (priority 14–19): three-transistor mirrors, diff
//! pairs with a tail or load, cascode and follower combinations. Each row
//! cites its ALIGN template or textbook.
//!
//! Contents:
//! - `pub const WILSON_MIRROR`
//! - `pub const IMPROVED_WILSON_MIRROR`
//! - `pub const CURRENT_MIRROR_3`
//! - `pub const DIFF_PAIR_WITH_TAIL`
//! - `pub const DIFF_PAIR_DIODE_LOAD`
//! - `pub const REGULATED_CASCODE`
//! - `pub const CASCODE_WITH_DEGENERATION`
//! - `pub const SOURCE_FOLLOWER_WITH_MIRROR`
//! - `pub const FLIPPED_VOLTAGE_FOLLOWER`
//! - `pub const ACTIVE_LOAD_3`
//! - `pub const CASCODE_WITH_MIRROR_BIAS`
//! - `pub const MIRROR_WITH_CASCODE_OUTPUT`
//! - `pub const SELF_BIASED_CASCODE`
//! - `pub const SOOCH_MIRROR`
//! - `pub const INVERTER_WITH_TAIL`
//! - `pub const PUSH_PULL_WITH_BIAS`
//! - `pub const BIAS_CHAIN_3`
//! - `pub const CMFB_SENSE_PAIR`
//! - `pub const BOOTSTRAPPED_SWITCH`
//! - `pub const TRIPLE_CASCODE`
//! - `pub const CURRENT_MIRROR_1_TO_2`
//! - `pub const SUPER_SOURCE_FOLLOWER`
//! - `pub const CASCODED_REFERENCE`
//! - `pub const ACTIVE_LOADED_INVERTER`
//! - `pub const DIFF_PAIR_WITH_REFERENCE`

use crate::pattern::{DiodeReq, Pattern, SizeMatch, Slot, SlotKind};

use super::{comp, eq, eq_sig, ne, same0, same0_exact, same0_samel, same_as, S_ANY};

// ── 2.1 Mirrors with 3 devices ──

/// Wilson current mirror: slot 0 = diode reference, slot 1 = input, slot 2 =
/// output. Topology: slots 0+1 share gate+source; slot 1's drain drives
/// slot 2's gate; slot 0's drain is slot 2's source (feedback).
pub const WILSON_MIRROR: Pattern = Pattern {
    name: "wilson_mirror",
    priority: 16,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), ..S_ANY },
    ],
    links: &[eq(0, "G", 1, "G"), eq(0, "S", 1, "S"), eq(1, "D", 2, "G"), eq(0, "D", 2, "S")],
};

/// Improved Wilson mirror: like Wilson but M2 is diode-connected and
/// M0 is NOT diode-connected. M2.D = M2.G provides the feedback,
/// M0 and M1 share gate (driven by M2), share source.
/// Gives better output impedance than basic Wilson.
pub const IMPROVED_WILSON_MIRROR: Pattern = Pattern {
    name: "improved_wilson_mirror",
    priority: 17,
    slots: &[
        Slot { diode: DiodeReq::Forbidden, ..S_ANY }, // M0: input, not diode
        same0_samel(),                                // M1: output, same L
        Slot {
            kind: SlotKind::SameTypeAs(0),
            size_match: SizeMatch::Any,
            diode: DiodeReq::Required, // M2: feedback diode
            gate_is_signal: false,
        },
    ],
    links: &[
        eq(0, "G", 1, "G"), // M0 M1 share gate
        eq(0, "S", 1, "S"), // M0 M1 share source
        eq(0, "D", 2, "S"), // M0.D = M2.S
        eq(2, "D", 0, "G"), // M2.D(=M2.G) drives M0+M1 gates
    ],
};

/// Three-output current mirror: diode ref + 2 output transistors.
/// All three share gate and source. M0 is diode-connected.
pub const CURRENT_MIRROR_3: Pattern = Pattern {
    name: "current_mirror_3",
    priority: 14,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0_samel(), same0_samel()],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "G", 2, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "S", 2, "S"),
        ne(0, "D", 1, "D"),
        ne(0, "D", 2, "D"),
        ne(1, "D", 2, "D"),
    ],
};

// ── 2.2 Diff pair combinations ──

/// Diff pair with tail current source.
/// M0+M1 form diff pair, M2 provides tail current (its drain = shared source).
pub const DIFF_PAIR_WITH_TAIL: Pattern = Pattern {
    name: "diff_pair_with_tail",
    priority: 18,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        Slot { kind: SlotKind::SameTypeAs(0), ..S_ANY },
    ],
    links: &[eq(0, "S", 1, "S"), ne(0, "G", 1, "G"), ne(0, "D", 1, "D"), eq(0, "S", 2, "D")],
};

/// Diff pair with diode-connected load on one side.
/// M0+M1 diff pair, M2 diode load on M0's drain (complementary type).
pub const DIFF_PAIR_DIODE_LOAD: Pattern = Pattern {
    name: "diff_pair_diode_load",
    priority: 15,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        Slot { kind: SlotKind::ComplementOf(0), size_match: SizeMatch::Any, diode: DiodeReq::Required, gate_is_signal: false },
    ],
    links: &[eq(0, "S", 1, "S"), ne(0, "G", 1, "G"), ne(0, "D", 1, "D"), eq(0, "D", 2, "D")],
};

// ── 2.3 Cascode combinations ──

/// Regulated cascode: M0 common-source, M1 cascode on top,
/// M2 amplifier providing feedback to M1's gate.
/// M0.D = M1.S, M0.D = M2.G (sense), M2.D = M1.G (regulate).
pub const REGULATED_CASCODE: Pattern = Pattern {
    name: "regulated_cascode",
    priority: 17,
    slots: &[
        S_ANY,   // M0: bottom CS
        same0(), // M1: cascode (top)
        same0(), // M2: regulation amp
    ],
    links: &[
        eq(0, "D", 1, "S"), // cascode stack
        eq(0, "D", 2, "G"), // M2 senses M0 drain voltage
        eq(2, "D", 1, "G"), // M2 output drives cascode gate
    ],
};

/// Source-degenerated cascode: M0 bottom, M1 top cascode,
/// M2 source degeneration (gate tied to supply/bias, in triode).
/// M2.D = M0.S, M0.D = M1.S.
pub const CASCODE_WITH_DEGENERATION: Pattern = Pattern {
    name: "cascode_with_degeneration",
    priority: 15,
    slots: &[S_ANY, same0(), same0()],
    links: &[
        eq(0, "D", 1, "S"), // cascode stack
        eq(2, "D", 0, "S"), // degeneration device under bottom
        ne(0, "G", 1, "G"), // input vs cascode bias: a shared gate is a series stack
        ne(0, "G", 2, "G"), // input vs degeneration bias
    ],
};

// ── 2.4 Source follower with bias ──

/// Source follower with current mirror bias: M0 is the follower (signal gate),
/// M1 is a diode-connected current source, M2 mirrors M1.
/// M0.S = M2.D (current source output), M1.G = M2.G (mirror gate).
pub const SOURCE_FOLLOWER_WITH_MIRROR: Pattern = Pattern {
    name: "source_follower_with_mirror",
    priority: 15,
    slots: &[
        Slot { kind: SlotKind::AnyFet, size_match: SizeMatch::Any, diode: DiodeReq::Forbidden, gate_is_signal: true },
        Slot { diode: DiodeReq::Required, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(1), size_match: SizeMatch::SameLAs(1), ..S_ANY },
    ],
    links: &[eq(0, "S", 2, "D"), eq(1, "G", 2, "G"), eq(1, "S", 2, "S")],
};

/// Flipped voltage follower: M0 (input), M1 (current source),
/// M2 (feedback amplifier). M0.S is the output, M1 forces constant
/// current. M1.D = M0.S, M2.G = M0.S (sense), M2.D = M1.G (feedback).
pub const FLIPPED_VOLTAGE_FOLLOWER: Pattern = Pattern {
    name: "flipped_voltage_follower",
    priority: 16,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY }, // M0: input device
        same0(),                                // M1: current source
        same0(),                                // M2: feedback amp
    ],
    links: &[
        eq(1, "D", 0, "S"), // M1 drives M0's source
        eq(0, "S", 2, "G"), // M2 senses output (M0.S)
        eq(2, "D", 1, "G"), // M2 regulates M1's gate
    ],
};

// ── 2.5 Three-output structures ──

/// Three-output active load: three same-type FETs, all sharing gate
/// and source, different drains. None diode-connected.
pub const ACTIVE_LOAD_3: Pattern = Pattern {
    name: "active_load_3",
    priority: 14,
    slots: &[
        Slot { diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Forbidden, gate_is_signal: false },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Forbidden, gate_is_signal: false },
    ],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "G", 2, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "S", 2, "S"),
        ne(0, "D", 1, "D"),
        ne(0, "D", 2, "D"),
        ne(1, "D", 2, "D"),
    ],
};

/// Cascode with mirror bias: M0 is the common-source device, M1 is
/// the cascode device on top, M2 is a diode-connected bias device
/// whose gate drives M1. M2.G = M2.D = M1.G.
pub const CASCODE_WITH_MIRROR_BIAS: Pattern = Pattern {
    name: "cascode_with_mirror_bias",
    priority: 15,
    slots: &[S_ANY, same0(), Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY }],
    links: &[
        eq(0, "D", 1, "S"),
        eq(2, "D", 1, "G"), // diode drives cascode gate
        eq(2, "S", 0, "S"), // bias device shares source rail
    ],
};

/// Current mirror with cascode on output: M0 diode ref, M1 output,
/// M2 cascode on M1. M0.G = M1.G, M0.S = M1.S, M1.D = M2.S.
/// This is the "half" of a cascode mirror (ref side only basic mirror).
pub const MIRROR_WITH_CASCODE_OUTPUT: Pattern = Pattern {
    name: "mirror_with_cascode_output",
    priority: 15,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0_samel(), same0()],
    links: &[eq(0, "G", 1, "G"), eq(0, "S", 1, "S"), eq(1, "D", 2, "S")],
};

/// Self-biased cascode: M0 bottom (diode-connected), M1 top cascode
/// (gate driven by M0's drain = M0's gate), M2 output mirror of M0.
/// M0.D = M0.G = M1.G, M0.D = M1.S, M0.G = M2.G.
pub const SELF_BIASED_CASCODE: Pattern = Pattern {
    name: "self_biased_cascode",
    priority: 16,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0(), same0_samel()],
    links: &[
        eq(0, "D", 1, "S"),
        eq(0, "D", 1, "G"), // M0 diode drives M1 gate (self-bias)
        eq(0, "G", 2, "G"),
        eq(0, "S", 2, "S"),
    ],
};

/// Sooch cascode mirror: M0 diode, M1 output, M2 cascode.
/// M0 is diode, M0.G = M1.G (basic mirror), M1.D = M2.S (cascode).
/// M2.G is driven by a separate bias (not from M0 or M1).
/// Distinguished from Wilson by: M2.G is NOT connected to M0.D.
pub const SOOCH_MIRROR: Pattern = Pattern {
    name: "sooch_mirror",
    priority: 15,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0_samel(), Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Forbidden, ..S_ANY }],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "S", 1, "S"),
        eq(1, "D", 2, "S"),
        ne(0, "D", 2, "G"), // NOT Wilson feedback
        ne(1, "D", 2, "G"),
    ],
};

/// CMOS inverter with tail current source: M0(N) + M1(P) inverter,
/// M2 tail current source below M0.
pub const INVERTER_WITH_TAIL: Pattern = Pattern {
    name: "inverter_with_tail",
    priority: 15,
    slots: &[S_ANY, comp(0), same0()],
    links: &[eq(0, "G", 1, "G"), eq(0, "D", 1, "D"), eq(0, "S", 2, "D")],
};

/// Push-pull output stage with bias: M0(N) and M1(P) share drain
/// (output), M2 is a bias/level-shift device connecting M0.G or M1.G.
pub const PUSH_PULL_WITH_BIAS: Pattern = Pattern {
    name: "push_pull_with_bias",
    priority: 14,
    slots: &[
        S_ANY,
        comp(0),
        same0(), // or same_as(1) — bias device
    ],
    links: &[
        eq(0, "D", 1, "D"), // shared output
        ne(0, "S", 1, "S"),
        eq(2, "D", 0, "G"), // bias drives one gate
    ],
};

/// Bias chain: three diode-connected FETs in series.
/// M0.D = M1.S, M1.D = M2.S. Generates multi-VGS bias voltage.
pub const BIAS_CHAIN_3: Pattern = Pattern {
    name: "bias_chain_3",
    priority: 14,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Required, gate_is_signal: false },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Required, gate_is_signal: false },
    ],
    links: &[eq(0, "D", 1, "S"), eq(1, "D", 2, "S")],
};

/// Common-mode feedback sensing pair with mirror: M0+M1 sense the two
/// diff outputs (gate=outp, gate=outn), M2 is a diode-connected reference.
/// All three share drain (summing node).
pub const CMFB_SENSE_PAIR: Pattern = Pattern {
    name: "cmfb_sense_pair",
    priority: 15,
    slots: &[Slot { gate_is_signal: true, ..S_ANY }, same0_exact(), Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY }],
    links: &[
        eq(0, "D", 1, "D"), // summing node
        eq(0, "D", 2, "D"),
        eq(0, "S", 1, "S"),
        eq(0, "S", 2, "S"),
        ne(0, "G", 1, "G"),
    ],
};

/// Sample-and-hold switch: transmission gate (M0+M1) with a bootstrap
/// device (M2) that boosts the gate voltage of the NMOS switch.
pub const BOOTSTRAPPED_SWITCH: Pattern = Pattern {
    name: "bootstrapped_switch",
    priority: 15,
    slots: &[S_ANY, comp(0), same0()],
    links: &[
        eq(0, "D", 1, "D"),
        eq(0, "S", 1, "S"),
        eq(2, "D", 0, "G"), // bootstrap drives NMOS gate
    ],
};

/// Triple cascode stack: three same-type FETs in series.
/// M0.D = M1.S, M1.D = M2.S. All different gates.
/// Used in high-voltage or ultra-high-output-impedance designs.
pub const TRIPLE_CASCODE: Pattern = Pattern {
    name: "triple_cascode",
    priority: 16,
    slots: &[S_ANY, same0(), same0()],
    links: &[eq(0, "D", 1, "S"), eq(1, "D", 2, "S"), ne(0, "G", 1, "G"), ne(1, "G", 2, "G")],
};

/// Current mirror with two outputs: M0 diode ref, M1+M2 two output
/// transistors sharing gate with M0, all sharing source.
/// Distinct from CURRENT_MIRROR_3 in that M1 and M2 may have
/// different W (only same L required).
pub const CURRENT_MIRROR_1_TO_2: Pattern = Pattern {
    name: "current_mirror_1_to_2",
    priority: 14,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Forbidden, ..S_ANY },
    ],
    links: &[eq(0, "G", 1, "G"), eq(0, "G", 2, "G"), eq(0, "S", 1, "S"), eq(0, "S", 2, "S"), ne(0, "D", 1, "D"), ne(0, "D", 2, "D")],
};

/// Super source follower: M0 (follower, signal gate), M1 (current source),
/// M2 (feedback amp sensing M0.S and driving M1.G).
/// Provides very low output impedance.
pub const SUPER_SOURCE_FOLLOWER: Pattern = Pattern {
    name: "super_source_follower",
    priority: 16,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY }, // M0: follower
        same0(),                                // M1: current source
        same0(),                                // M2: feedback amp
    ],
    links: &[
        eq(1, "D", 0, "S"), // M1 current source to follower source
        eq(0, "S", 2, "S"), // M2 source = output (sensing)
        eq(2, "D", 1, "G"), // M2 output drives M1 gate
    ],
};

/// Three-transistor current source: M0 diode ref, M1 mirror output,
/// M2 cascode on M0 (diode-connected). Provides cascoded reference.
pub const CASCODED_REFERENCE: Pattern = Pattern {
    name: "cascoded_reference",
    priority: 15,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0_samel(), Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY }],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "D", 2, "S"), // cascode on ref
    ],
};

/// Active-loaded inverter: CMOS inverter (M0+M1) with current source tail
/// on the PMOS side. M0(N)+M1(P) share gate and drain, M2(P) provides
/// bias current with its drain = M1's source.
pub const ACTIVE_LOADED_INVERTER: Pattern = Pattern {
    name: "active_loaded_inverter",
    priority: 15,
    slots: &[
        S_ANY,
        comp(0),
        same_as(1), // same type as M1
    ],
    links: &[eq(0, "G", 1, "G"), eq(0, "D", 1, "D"), eq(2, "D", 1, "S")],
};

/// Diff pair with common-mode input: M0+M1 diff pair where one input
/// gate is driven by a diode-connected M2 (reference level).
pub const DIFF_PAIR_WITH_REFERENCE: Pattern = Pattern {
    name: "diff_pair_with_reference",
    priority: 15,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: false },
        Slot { kind: SlotKind::ComplementOf(0), size_match: SizeMatch::Any, diode: DiodeReq::Required, gate_is_signal: false },
    ],
    links: &[
        eq_sig(0, "S", 1, "S"), // a rail-sourced pair is not differential (EXT-04)
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
        eq(1, "G", 2, "D"), // M2 diode drives M1 gate
    ],
};
