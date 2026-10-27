//! Two-device patterns (priority 5–13): differential pairs, mirrors,
//! cross-coupled pairs, cascodes, followers, switches, and the bipolar
//! pairs (EXT-19; Hastings §9). Each row cites its ALIGN template or textbook.
//!
//! Contents:
//! - `pub const DIFF_PAIR`
//! - `pub const CURRENT_MIRROR`
//! - `pub const CURRENT_MIRROR_OUTPUT_PAIR`
//! - `pub const CROSS_COUPLED`
//! - `pub const CROSS_COUPLED_SPLIT_SOURCE`
//! - `pub const CASCODE`
//! - `pub const CASCODE_MATCHED`
//! - `pub const DIODE_LOAD_PAIR`
//! - `pub const SOURCE_FOLLOWER`
//! - `pub const CMOS_INVERTER`
//! - `pub const TRANSMISSION_GATE`
//! - `pub const PUSH_PULL_PAIR`
//! - `pub const LATCH_HALF`
//! - `pub const SERIES_STACK`
//! - `pub const DEGENERATION_PAIR`
//! - `pub const COMMON_GATE_PAIR`
//! - `pub const SWITCH_PAIR`
//! - `pub const BETA_MULTIPLIER_CORE`
//! - `pub const COMPLEMENTARY_SOURCE_FOLLOWER`
//! - `pub const COMMON_MODE_SENSE_PAIR`
//! - `pub const ESD_DIODE_CLAMP`
//! - `pub const CASCODE_DIODE_TOP`
//! - `pub const ANTI_PARALLEL_SWITCH`
//! - `pub const DIFF_SWITCH`

use crate::pattern::{DiodeReq, Pattern, SizeMatch, Slot, SlotKind};

use super::{comp, eq, eq_sig, ne, same0_exact, S_ANY};

// ── 1.1 Differential pairs ──

/// Classic differential pair: same-type, exact W+L match, shared source,
/// different gates (signal inputs), different drains.
/// Explicitly excludes cross-coupled configurations (G_A != D_B).
/// Matches both NMOS and PMOS variants.
/// ALIGN: DP_NMOS / DP_PMOS
pub const DIFF_PAIR: Pattern = Pattern {
    name: "diff_pair",
    priority: 10,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
    ],
    links: &[
        eq_sig(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
        ne(0, "G", 1, "D"), // exclude cross-coupled
        ne(1, "G", 0, "D"), // exclude cross-coupled
    ],
};

// ── 1.2 Current mirrors ──

/// Simple current mirror: diode-connected reference + output transistor.
/// Shared gate and source, reference is diode-connected.
/// ALIGN: SCM_NMOS / SCM_PMOS
pub const CURRENT_MIRROR: Pattern = Pattern {
    name: "current_mirror",
    priority: 8,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), ..S_ANY }],
    links: &[eq(0, "G", 1, "G"), eq(0, "S", 1, "S")],
};

/// Current mirror output pair (no diode): two transistors sharing gate
/// and source, both driven by an external bias. Neither is diode-connected.
/// ALIGN: CMC_NMOS / CMC_PMOS
pub const CURRENT_MIRROR_OUTPUT_PAIR: Pattern = Pattern {
    name: "current_mirror_output_pair",
    priority: 7,
    slots: &[
        Slot { diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Forbidden, gate_is_signal: false },
    ],
    links: &[eq(0, "G", 1, "G"), eq(0, "S", 1, "S"), ne(0, "D", 1, "D")],
};

// ── 1.3 Cross-coupled pairs ──

/// Cross-coupled pair: gate of A = drain of B and vice versa.
/// Forms positive feedback (latch). Shared source.
/// ALIGN: CCP_NMOS / CCP_PMOS
pub const CROSS_COUPLED: Pattern = Pattern {
    name: "cross_coupled",
    priority: 9,
    slots: &[S_ANY, Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY }],
    links: &[eq(0, "G", 1, "D"), eq(0, "D", 1, "G"), eq(0, "S", 1, "S")],
};

/// Cross-coupled pair with split sources: gate-drain cross but sources
/// on different nets (e.g. in SRAM cells or sense amps).
/// ALIGN: CCP_S_NMOS / CCP_S_PMOS
pub const CROSS_COUPLED_SPLIT_SOURCE: Pattern = Pattern {
    name: "cross_coupled_split_source",
    priority: 9,
    slots: &[S_ANY, Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY }],
    links: &[eq(0, "G", 1, "D"), eq(0, "D", 1, "G"), ne(0, "S", 1, "S")],
};

// ── 1.4 Cascode structures ──

/// Simple cascode: drain of bottom connects to source of top.
/// Same type, distinct gates (a shared gate is a series stack). Used for high
/// output impedance.
pub const CASCODE: Pattern = Pattern {
    name: "cascode",
    priority: 7,
    slots: &[S_ANY, Slot { kind: SlotKind::SameTypeAs(0), ..S_ANY }],
    links: &[eq(0, "D", 1, "S"), ne(0, "G", 1, "G")],
};

/// Cascode with same L constraint. Bottom and top share channel length
/// (typical in matched cascode stacks).
pub const CASCODE_MATCHED: Pattern = Pattern {
    name: "cascode_matched",
    priority: 8,
    slots: &[S_ANY, Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), ..S_ANY }],
    links: &[eq(0, "D", 1, "S"), ne(0, "G", 1, "G")],
};

// ── 1.5 Active loads / matched pairs ──

/// Diode-connected load pair: two diode-connected FETs sharing source,
/// different drains. Used as matched diode loads in DAC reference ladders.
pub const DIODE_LOAD_PAIR: Pattern = Pattern {
    name: "diode_load_pair",
    priority: 8,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Required, gate_is_signal: false },
    ],
    links: &[eq(0, "S", 1, "S"), ne(0, "D", 1, "D")],
};

// ── 1.6 Source follower / common drain ──

/// Source follower pair: two same-type FETs where drain of bottom
/// connects to source of top AND top has signal gate input.
/// The top FET is the follower, bottom is the current source.
pub const SOURCE_FOLLOWER: Pattern = Pattern {
    name: "source_follower",
    priority: 8,
    slots: &[
        S_ANY,                                                                                                                // current source (bottom)
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::Any, diode: DiodeReq::Forbidden, gate_is_signal: true }, // follower (top)
    ],
    links: &[
        eq(0, "D", 1, "S"), // current source drain = follower source
        ne(0, "G", 1, "G"), // bias vs input: a shared gate is a series stack
    ],
};

// ── 1.7 Complementary pairs ──

/// CMOS inverter: complementary pair sharing gate and drain.
/// ALIGN: inv
pub const CMOS_INVERTER: Pattern =
    Pattern { name: "cmos_inverter", priority: 11, slots: &[S_ANY, comp(0)], links: &[eq(0, "G", 1, "G"), eq(0, "D", 1, "D")] };

/// Transmission gate: complementary pair sharing drain and source
/// but with different gates (complementary clock signals).
/// ALIGN: tgate
pub const TRANSMISSION_GATE: Pattern = Pattern {
    name: "transmission_gate",
    priority: 11,
    slots: &[S_ANY, comp(0)],
    links: &[eq(0, "D", 1, "D"), eq(0, "S", 1, "S"), ne(0, "G", 1, "G")],
};

/// Complementary common-source pair: NMOS and PMOS sharing gate,
/// different drains and sources. Push-pull output stage element.
pub const PUSH_PULL_PAIR: Pattern =
    Pattern { name: "push_pull_pair", priority: 10, slots: &[S_ANY, comp(0)], links: &[eq(0, "G", 1, "G"), ne(0, "D", 1, "D"), ne(0, "S", 1, "S")] };

// ── 1.8 Latch / self-biased structures ──

/// Latch half: M0 is diode-connected (gate=drain), M0's drain drives
/// M1's gate, separate sources. Used in sense amp latches.
/// ALIGN: LS_S_NMOS / LS_S_PMOS
pub const LATCH_HALF: Pattern = Pattern {
    name: "latch_half",
    priority: 8,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY }],
    links: &[
        eq(0, "D", 1, "G"), // diode drain drives mirror gate
        ne(0, "S", 1, "S"), // separate sources
        ne(0, "D", 1, "D"), // separate drains
    ],
};

// ── 1.9 Series-connected FETs ──

/// Series stack: same-type, source of top = drain of bottom,
/// SAME gate (series combination for effective L doubling or
/// high-voltage tolerance). Different from cascode where gates differ.
/// Outranks `mirror_pair_split_source`, which a shared-gate chain also fits.
pub const SERIES_STACK: Pattern = Pattern {
    name: "series_stack",
    priority: 7,
    slots: &[S_ANY, Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY }],
    links: &[eq(0, "D", 1, "S"), eq(0, "G", 1, "G")],
};

/// Degeneration pair: two same-type FETs sharing drain but with
/// different gates and different sources. Used when a degeneration
/// resistor is replaced by a FET in triode region.
pub const DEGENERATION_PAIR: Pattern = Pattern {
    name: "degeneration_pair",
    priority: 5,
    slots: &[S_ANY, Slot { kind: SlotKind::SameTypeAs(0), ..S_ANY }],
    links: &[eq(0, "D", 1, "D"), ne(0, "G", 1, "G"), ne(0, "S", 1, "S")],
};

// These catch common wiring patterns not covered by the primary set.

/// Common-gate pair: two same-type FETs sharing gate (non-signal bias)
/// and drain, with different sources. Used as common-gate input stage
/// in transimpedance amplifiers.
pub const COMMON_GATE_PAIR: Pattern = Pattern {
    name: "common_gate_pair",
    priority: 6,
    slots: &[S_ANY, Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY }],
    links: &[eq(0, "G", 1, "G"), eq(0, "D", 1, "D"), ne(0, "S", 1, "S")],
};

/// Switch pair: two same-type FETs, different gates (complementary
/// control), shared source, shared drain. Parallel switches for
/// reduced on-resistance or redundancy.
pub const SWITCH_PAIR: Pattern = Pattern {
    name: "switch_pair",
    priority: 5,
    slots: &[S_ANY, same0_exact()],
    links: &[eq(0, "D", 1, "D"), eq(0, "S", 1, "S"), ne(0, "G", 1, "G")],
};

/// Beta multiplier reference: M0 (diode-connected, narrow) and M1 (wide)
/// with a degeneration element between M1's source and the shared rail.
/// They share gate, but M1 is wider than M0 (SameLAs but not ExactAs).
/// The source resistor is outside the FET pattern.
pub const BETA_MULTIPLIER_CORE: Pattern = Pattern {
    name: "beta_multiplier_core",
    priority: 9,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Forbidden, gate_is_signal: false },
    ],
    links: &[
        eq(0, "G", 1, "G"),
        ne(0, "S", 1, "S"), // different sources (degeneration resistor on M1)
        ne(0, "D", 1, "D"),
    ],
};

/// Complementary source follower: NMOS follower + PMOS follower,
/// sharing drain(=output). Class AB output.
pub const COMPLEMENTARY_SOURCE_FOLLOWER: Pattern = Pattern {
    name: "complementary_source_follower",
    priority: 10,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::ComplementOf(0), size_match: SizeMatch::Any, diode: DiodeReq::Forbidden, gate_is_signal: true },
    ],
    links: &[
        eq(0, "S", 1, "S"), // shared output at sources
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
    ],
};

/// Common-mode pair with split source: two same-type, same gate,
/// different drains AND different sources. Used as input pair
/// for common-mode feedback sensing.
pub const COMMON_MODE_SENSE_PAIR: Pattern = Pattern {
    name: "common_mode_sense_pair",
    priority: 6,
    slots: &[
        Slot { diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Forbidden, gate_is_signal: false },
    ],
    links: &[
        eq(0, "G", 1, "G"),
        ne(0, "S", 1, "S"),
        eq(0, "D", 1, "D"), // drains tied (sensing common mode)
    ],
};

/// ESD clamp: two diode-connected FETs in series (NMOS),
/// forming a diode voltage clamp between supply and ground.
/// M0 diode source=gnd, M1 diode source=M0.drain.
pub const ESD_DIODE_CLAMP: Pattern = Pattern {
    name: "esd_diode_clamp",
    priority: 10,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::Any, diode: DiodeReq::Required, gate_is_signal: false },
    ],
    links: &[eq(0, "D", 1, "S")],
};

/// Cascode with diode on top: bottom device (not diode-connected),
/// top device is diode-connected. Used for generating cascode gate bias.
pub const CASCODE_DIODE_TOP: Pattern = Pattern {
    name: "cascode_diode_top",
    priority: 9,
    slots: &[
        Slot { diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), diode: DiodeReq::Required, gate_is_signal: false },
    ],
    links: &[eq(0, "D", 1, "S")],
};

/// Anti-parallel pair: two same-type FETs where source of A = drain of B
/// AND drain of A = source of B. Bidirectional switch.
pub const ANTI_PARALLEL_SWITCH: Pattern = Pattern {
    name: "anti_parallel_switch",
    priority: 10,
    slots: &[S_ANY, same0_exact()],
    links: &[eq(0, "D", 1, "S"), eq(0, "S", 1, "D"), ne(0, "G", 1, "G")],
};

/// Differential switch (analog mux): two same-type FETs sharing source,
/// different gates (select signals), different drains (outputs).
/// Similar to diff pair but gates are digital select, not analog signal.
/// `gate_is_signal: false` means the gate is not checked, so this also matches
/// signal-gated pairs; `diff_pair` (priority 10) wins those.
pub const DIFF_SWITCH: Pattern = Pattern {
    name: "diff_switch",
    priority: 8,
    slots: &[
        Slot { gate_is_signal: false, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Forbidden, gate_is_signal: false },
    ],
    links: &[
        eq_sig(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
        ne(0, "G", 1, "D"), // exclude cross-coupled
        ne(1, "G", 0, "D"), // exclude cross-coupled
    ],
};

/// The three bipolar 2-device patterns for one polarity: slot 0 of kind `$k`,
/// slot 1 the same kind with an exact emitter (`ExactAs`: W/L or a
/// fixed-geometry model).
macro_rules! bjt_patterns {
    ($ratio:ident, $diff:ident, $mirror:ident, $k:expr, $sfx:literal) => {
        /// Ratioed pair (bandgap ΔV_BE core): shared base, distinct emitters; the
        /// base may be a rail (`bgr_core`). Ratio from `m`.
        pub const $ratio: Pattern = Pattern {
            name: concat!("bjt_ratioed_pair_", $sfx),
            priority: 9,
            slots: &[Slot { kind: SlotKind::Kind($k), ..S_ANY }, Slot { kind: SlotKind::SameKindAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY }],
            links: &[eq(0, "B", 1, "B"), ne(0, "E", 1, "E")],
        };
        /// Differential pair: shared signal emitter node, distinct bases and collectors.
        pub const $diff: Pattern = Pattern {
            name: concat!("bjt_diff_pair_", $sfx),
            priority: 10,
            slots: &[Slot { kind: SlotKind::Kind($k), ..S_ANY }, Slot { kind: SlotKind::SameKindAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY }],
            links: &[eq_sig(0, "E", 1, "E"), ne(0, "B", 1, "B"), ne(0, "C", 1, "C")],
        };
        /// Mirror: diode-connected reference (C == B), shared base and emitter.
        pub const $mirror: Pattern = Pattern {
            name: concat!("bjt_mirror_", $sfx),
            priority: 8,
            slots: &[
                Slot { kind: SlotKind::Kind($k), diode: DiodeReq::Required, ..S_ANY },
                Slot { kind: SlotKind::SameKindAs(0), size_match: SizeMatch::ExactAs(0), ..S_ANY },
            ],
            links: &[eq(0, "B", 1, "B"), eq(0, "E", 1, "E")],
        };
    };
}
bjt_patterns!(BJT_RATIOED_PAIR_NPN, BJT_DIFF_PAIR_NPN, BJT_MIRROR_NPN, analog::base::netlist::DeviceKind::Npn, "npn");

bjt_patterns!(BJT_RATIOED_PAIR_PNP, BJT_DIFF_PAIR_PNP, BJT_MIRROR_PNP, analog::base::netlist::DeviceKind::Pnp, "pnp");
