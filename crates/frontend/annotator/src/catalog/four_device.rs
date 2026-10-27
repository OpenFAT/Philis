//! Four-device patterns (priority 20–29): cascode and Wilson mirrors, diff
//! pairs with loads, cross-coupled and cascode pairs, level shifters,
//! charge pumps. Each row cites its ALIGN template or textbook.
//!
//! Contents:
//! - `pub const SERIES_STACK_4`
//! - `pub const CASCODE_MIRROR`
//! - `pub const WIDE_SWING_CASCODE_MIRROR`
//! - `pub const LOW_VOLTAGE_CASCODE_MIRROR`
//! - `pub const WILSON_MIRROR_4`
//! - `pub const IMPROVED_WILSON_MIRROR_4`
//! - `pub const DIFF_PAIR_WITH_ACTIVE_LOAD`
//! - `pub const DIFF_PAIR_WITH_MIRROR_LOAD`
//! - `pub const CROSS_COUPLED_INVERTERS`
//! - `pub const CASCODE_PAIR`
//! - `pub const CASCODE_PAIR_SYMMETRIC`
//! - `pub const LEVEL_SHIFTER`
//! - `pub const CHARGE_PUMP_CELL`
//! - `pub const CURRENT_MIRROR_4`
//! - `pub const DIFF_PAIR_WITH_CASCODES`
//! - `pub const MIRROR_WITH_DUAL_OUTPUT`
//! - `pub const REGULATED_CASCODE_MIRROR`
//! - `pub const TRANSMISSION_GATE_PAIR`
//! - `pub const DAC_CURRENT_CELL`
//! - `pub const FEEDBACK_PAIR`
//! - `pub const SCHMITT_TRIGGER`
//! - `pub const BANDGAP_MIRROR_PAIR`
//! - `pub const COMPLEMENTARY_DIFF_PAIR`
//! - `pub const NAND_CROSS_COUPLED`
//! - `pub const NAND_GATE`
//! - `pub const NOR_GATE`
//! - `pub const DIFF_PAIR_WITH_SPLIT_CASCODES`
//! - `pub const FOLDED_LOAD_PAIR`
//! - `pub const INVERTER_BUFFER`
//! - `pub const CASCODE_CURRENT_SOURCE_PAIR`

use crate::pattern::{DiodeReq, Pattern, SizeMatch, Slot, SlotKind};

use super::{comp, eq, eq_sig, ne, same0, same0_exact, same0_samel, same_as, same_exact, S_ANY};

/// Four-high series stack (one long device split in four, or a chain of
/// pass devices): drain→source chained, one gate, exact sizes. Held as one
/// group so the whole chain may share diffusion.
pub const SERIES_STACK_4: Pattern = Pattern {
    name: "series_stack_4",
    priority: 20,
    slots: &[S_ANY, same0_exact(), same0_exact(), same0_exact()],
    links: &[eq(0, "D", 1, "S"), eq(1, "D", 2, "S"), eq(2, "D", 3, "S"), eq(0, "G", 1, "G"), eq(0, "G", 2, "G"), eq(0, "G", 3, "G")],
};

// ── 3.1 Cascode mirrors ──

/// Cascode current mirror (standard): M0 diode + M1 output form the
/// bottom mirror, M2 diode + M3 cascode form the top pair.
/// M0.D = M2.S, M1.D = M3.S, M2.G = M3.G.
pub const CASCODE_MIRROR: Pattern = Pattern {
    name: "cascode_mirror",
    priority: 24,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY }, // M0: bottom ref (diode)
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(0), ..S_ANY }, // M1: bottom out
        Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY }, // M2: top ref (diode)
        same0(),                                     // M3: top out
    ],
    links: &[
        eq(0, "G", 1, "G"), // bottom pair share gate
        eq(0, "S", 1, "S"), // bottom pair share source
        eq(0, "D", 2, "S"), // M0 stacks under M2
        eq(1, "D", 3, "S"), // M1 stacks under M3
        eq(2, "G", 3, "G"), // top pair share gate
    ],
};

/// Wide-swing cascode mirror: like cascode mirror but the top gate bias
/// is derived differently. M0 diode, M1 output, M2 cascode (diode),
/// M3 cascode output. The key difference is M2's gate connects to M0's
/// drain (not to M3's gate independently) providing wide output swing.
/// M2.G = M0.D = M2.S is what makes it "wide-swing" — the diode cascode
/// self-biases at VGS above the bottom mirror.
pub const WIDE_SWING_CASCODE_MIRROR: Pattern = Pattern {
    name: "wide_swing_cascode_mirror",
    priority: 25,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        same0_samel(),
        Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY },
        same0(),
    ],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "D", 2, "S"), // M0 drain = M2 source
        eq(1, "D", 3, "S"),
        eq(2, "G", 3, "G"),
        eq(2, "D", 0, "G"), // M2.D feeds back to bottom mirror gate (wide-swing bias)
    ],
};

/// Low-voltage cascode mirror: M0+M1 bottom pair, M2+M3 top pair.
/// Bottom gates are driven by the cascode voltage (M2 or M3 drain).
/// Allows operation at lower supply than standard cascode.
/// M0.G = M1.G (shared), M2.G = M3.G, M0.D=M2.S, M1.D=M3.S.
/// The distinction: neither M2 nor M3 needs to be diode-connected.
pub const LOW_VOLTAGE_CASCODE_MIRROR: Pattern = Pattern {
    name: "low_voltage_cascode_mirror",
    priority: 23,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        same0_samel(),
        Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Forbidden, ..S_ANY },
    ],
    links: &[eq(0, "G", 1, "G"), eq(0, "S", 1, "S"), eq(0, "D", 2, "S"), eq(1, "D", 3, "S"), eq(2, "G", 3, "G")],
};

// ── 3.2 Wilson mirrors (4-device) ──

/// Four-transistor Wilson mirror: M0+M1 bottom pair (cross-coupled gates
/// to drains), M2+M3 top cascode pair.
/// M0.G = M3.D, M1.G = M2.D, M0.D = M2.S, M1.D = M3.S.
pub const WILSON_MIRROR_4: Pattern = Pattern {
    name: "wilson_mirror_4",
    priority: 24,
    slots: &[S_ANY, same0_samel(), same0(), same0()],
    links: &[
        eq(0, "S", 1, "S"),
        eq(0, "D", 2, "S"),
        eq(1, "D", 3, "S"),
        eq(0, "G", 3, "D"), // cross-feedback
        eq(1, "G", 2, "D"), // cross-feedback
    ],
};

/// Improved Wilson mirror (4T): M0+M1 bottom, M2+M3 top.
/// M2 is diode-connected. M0.G = M1.G = M2.D.
/// M0.D = M2.S, M1.D = M3.S, M2.G = M3.G.
pub const IMPROVED_WILSON_MIRROR_4: Pattern = Pattern {
    name: "improved_wilson_mirror_4",
    priority: 25,
    slots: &[S_ANY, same0_samel(), Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY }, same0()],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "D", 2, "S"),
        eq(1, "D", 3, "S"),
        eq(2, "G", 3, "G"),
        eq(2, "D", 0, "G"), // M2 diode drives bottom gates
    ],
};

// ── 3.3 Diff pair with loads ──

/// Diff pair + active load: M0+M1 diff pair (same type, signal gates),
/// M2+M3 active load (complement type, shared gate, biased).
pub const DIFF_PAIR_WITH_ACTIVE_LOAD: Pattern = Pattern {
    name: "diff_pair_with_active_load",
    priority: 22,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        comp(0),
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY },
    ],
    links: &[eq(0, "S", 1, "S"), ne(0, "G", 1, "G"), eq(0, "D", 2, "D"), eq(1, "D", 3, "D"), eq(2, "G", 3, "G")],
};

/// Diff pair + current mirror load: M0+M1 diff pair, M2 diode load
/// on one drain, M3 mirrors M2 on the other drain.
/// This is the classic single-ended output diff amp.
pub const DIFF_PAIR_WITH_MIRROR_LOAD: Pattern = Pattern {
    name: "diff_pair_with_mirror_load",
    priority: 23,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        Slot { kind: SlotKind::ComplementOf(0), size_match: SizeMatch::Any, diode: DiodeReq::Required, gate_is_signal: false },
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY },
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "D"), // M2 diode on M0's drain
        eq(1, "D", 3, "D"), // M3 mirror on M1's drain
        eq(2, "G", 3, "G"), // mirror gate tie
        eq(2, "S", 3, "S"), // mirror source tie
    ],
};

// ── 3.4 Cross-coupled structures ──

/// Cross-coupled inverter pair (SRAM latch core): two CMOS inverters
/// where output of each drives input of the other.
/// M0(N)+M2(P) = inv1, M1(N)+M3(P) = inv2.
/// M0.D = M2.D = M1.G = M3.G, M1.D = M3.D = M0.G = M2.G.
pub const CROSS_COUPLED_INVERTERS: Pattern = Pattern {
    name: "cross_coupled_inverters",
    priority: 26,
    slots: &[S_ANY, same0_exact(), comp(0), Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY }],
    links: &[
        eq(0, "D", 2, "D"), // inv1 output
        eq(1, "D", 3, "D"), // inv2 output
        eq(0, "G", 2, "G"), // inv1 input
        eq(1, "G", 3, "G"), // inv2 input
        eq(0, "D", 1, "G"), // cross-couple 1
        eq(1, "D", 0, "G"), // cross-couple 2
    ],
};

// ── 3.5 Cascode pairs ──

/// Matched cascode pair: two independent cascode stacks (4 FETs total)
/// where the bottom pair shares gate/source and the top pair shares gate.
/// This is the core of a cascode amplifier stage.
/// M0+M2 = stack A (M0 bottom, M2 top), M1+M3 = stack B.
pub const CASCODE_PAIR: Pattern = Pattern {
    name: "cascode_pair",
    priority: 22,
    slots: &[
        S_ANY,         // M0: bottom A
        same0_exact(), // M1: bottom B
        same0(),       // M2: top A
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY },
    ],
    links: &[
        eq(0, "G", 1, "G"), // bottom pair share gate
        eq(0, "S", 1, "S"), // bottom pair share source
        eq(0, "D", 2, "S"), // stack A
        eq(1, "D", 3, "S"), // stack B
        eq(2, "G", 3, "G"), // top pair share gate
        ne(2, "D", 3, "D"), // different outputs
    ],
};

/// Symmetric cascode pair with split sources on top: bottom pair shares
/// gate+source, top pair shares gate but has separate outputs AND sources
/// coming from different bottom devices.
pub const CASCODE_PAIR_SYMMETRIC: Pattern = Pattern {
    name: "cascode_pair_symmetric",
    priority: 21,
    slots: &[S_ANY, same0_exact(), same0(), same_exact(2)],
    links: &[eq(0, "S", 1, "S"), eq(0, "D", 2, "S"), eq(1, "D", 3, "S"), eq(2, "G", 3, "G"), ne(0, "G", 2, "G")],
};

// ── 3.6 Level shifter ──

/// Level shifter: M0 (input, signal gate) stacked on M1 (current source),
/// M2 (complementary, diode load) stacked on M3 (supply).
/// M0.D = M2.D (shared output node). Used to shift DC level between stages.
pub const LEVEL_SHIFTER: Pattern = Pattern {
    name: "level_shifter",
    priority: 20,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY }, // M0: input device
        same0(),                                // M1: current source
        comp(0),                                // M2: diode load (complement)
        Slot { kind: SlotKind::SameTypeAs(2), ..S_ANY },
    ],
    links: &[
        eq(0, "S", 1, "D"), // M0 stacks on M1
        eq(0, "D", 2, "D"), // shared output node
        eq(2, "S", 3, "D"), // M2 stacks on M3
    ],
};

// ── 3.7 Charge pump ──

/// Basic charge pump cell: M0 (PMOS switch, up), M1 (current source up),
/// M2 (NMOS switch, down), M3 (current source down).
/// M0.D = M2.D (output), M0.S = M1.D (up current), M2.S = M3.D (down current).
pub const CHARGE_PUMP_CELL: Pattern = Pattern {
    name: "charge_pump_cell",
    priority: 22,
    slots: &[
        S_ANY,   // M0: up switch
        same0(), // M1: up current source
        comp(0), // M2: down switch
        Slot { kind: SlotKind::SameTypeAs(2), ..S_ANY },
    ],
    links: &[
        eq(0, "D", 2, "D"), // shared output
        eq(0, "S", 1, "D"), // up path
        eq(2, "S", 3, "D"), // down path
        ne(0, "G", 2, "G"), // complementary clocks
    ],
};

// ── 3.8 Four-output mirror ──

/// Four-output current mirror: diode ref + 3 matched outputs.
pub const CURRENT_MIRROR_4: Pattern = Pattern {
    name: "current_mirror_4",
    priority: 20,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0_samel(), same0_samel(), same0_samel()],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "G", 2, "G"),
        eq(0, "G", 3, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "S", 2, "S"),
        eq(0, "S", 3, "S"),
        ne(0, "D", 1, "D"),
        ne(0, "D", 2, "D"),
        ne(0, "D", 3, "D"),
    ],
};

/// Diff pair with matched cascode on both outputs:
/// M0+M1 diff pair, M2 cascode on M0.D, M3 cascode on M1.D.
pub const DIFF_PAIR_WITH_CASCODES: Pattern = Pattern {
    name: "diff_pair_with_cascodes",
    priority: 22,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        same0(),
        same_exact(2),
    ],
    links: &[eq(0, "S", 1, "S"), ne(0, "G", 1, "G"), eq(0, "D", 2, "S"), eq(1, "D", 3, "S"), eq(2, "G", 3, "G"), ne(2, "D", 3, "D")],
};

/// Current mirror with two cascoded outputs: M0 diode ref,
/// M1+M2 two output transistors, M3 cascode on one output.
/// Or alternatively: mirror pair + one cascode.
pub const MIRROR_WITH_DUAL_OUTPUT: Pattern = Pattern {
    name: "mirror_with_dual_output",
    priority: 21,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0_samel(), same0_samel(), same0()],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "G", 2, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "S", 2, "S"),
        ne(0, "D", 1, "D"),
        ne(0, "D", 2, "D"),
        ne(1, "D", 2, "D"),
        eq(1, "D", 3, "S"), // cascode on M1
    ],
};

/// Regulated cascode mirror: M0 diode ref, M1 output, M2 cascode,
/// M3 regulation amplifier. M0.G=M1.G, M0.S=M1.S, M1.D=M2.S,
/// M1.D=M3.G (sense), M3.D=M2.G (regulate).
pub const REGULATED_CASCODE_MIRROR: Pattern = Pattern {
    name: "regulated_cascode_mirror",
    priority: 24,
    slots: &[Slot { diode: DiodeReq::Required, ..S_ANY }, same0_samel(), same0(), same0()],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "S", 1, "S"),
        eq(1, "D", 2, "S"),
        eq(1, "D", 3, "G"), // sense
        eq(3, "D", 2, "G"), // regulate
    ],
};

/// CMOS transmission gate pair: two TGs in parallel or series.
/// M0(N)+M1(P) = TG1, M2(N)+M3(P) = TG2.
pub const TRANSMISSION_GATE_PAIR: Pattern = Pattern {
    name: "transmission_gate_pair",
    priority: 21,
    slots: &[S_ANY, comp(0), same0_exact(), Slot { kind: SlotKind::SameTypeAs(1), size_match: SizeMatch::ExactAs(1), ..S_ANY }],
    links: &[
        eq(0, "D", 1, "D"), // TG1 shared D
        eq(0, "S", 1, "S"), // TG1 shared S
        ne(0, "G", 1, "G"), // TG1 complementary clocks
        eq(2, "D", 3, "D"), // TG2 shared D
        eq(2, "S", 3, "S"), // TG2 shared S
        ne(2, "G", 3, "G"), // TG2 complementary clocks
    ],
};

/// Four-transistor current-steering DAC cell: M0+M1 diff switch pair
/// (signal gates, shared source), M2 current source (tail),
/// M3 cascode on current source.
pub const DAC_CURRENT_CELL: Pattern = Pattern {
    name: "dac_current_cell",
    priority: 22,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Forbidden, gate_is_signal: true },
        same0(), // M2: current source
        same0(), // M3: cascode on source
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
        eq(0, "S", 3, "D"), // cascode output to diff pair source
        eq(3, "S", 2, "D"), // current source under cascode
    ],
};

/// Feedback amplifier pair: M0+M1 common-source gain stage,
/// M2+M3 feedback devices (gate of M2 = drain of M0, etc).
/// Four same-type FETs forming a two-stage feedback loop.
pub const FEEDBACK_PAIR: Pattern = Pattern {
    name: "feedback_pair",
    priority: 20,
    slots: &[S_ANY, same0_exact(), same0(), same_exact(2)],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
        eq(0, "D", 2, "G"), // forward
        eq(1, "D", 3, "G"), // forward
        eq(2, "S", 3, "S"),
    ],
};

/// Schmitt trigger input: M0+M1 NMOS stack, M2+M3 PMOS stack.
/// M0 and M2 share gate (input), M1 gate and M3 gate provide hysteresis
/// feedback from output.
pub const SCHMITT_TRIGGER: Pattern = Pattern {
    name: "schmitt_trigger",
    priority: 22,
    slots: &[
        S_ANY,      // M0: NMOS input
        same0(),    // M1: NMOS feedback
        comp(0),    // M2: PMOS input
        same_as(2), // M3: PMOS feedback
    ],
    links: &[
        eq(0, "G", 2, "G"), // shared input
        eq(0, "D", 1, "S"), // NMOS stack
        eq(2, "D", 3, "S"), // PMOS stack
        eq(1, "D", 3, "D"), // shared output
    ],
};

/// Bandgap core (transistor part): M0+M1 form a current mirror,
/// M2+M3 form another mirror or diff pair sensing the PTAT/CTAT voltages.
/// The resistors are outside the FET pattern.
pub const BANDGAP_MIRROR_PAIR: Pattern = Pattern {
    name: "bandgap_mirror_pair",
    priority: 20,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        same0_samel(),
        comp(0),
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), diode: DiodeReq::Required, gate_is_signal: false },
    ],
    links: &[eq(0, "G", 1, "G"), eq(0, "S", 1, "S"), eq(2, "G", 3, "G"), eq(2, "S", 3, "S"), eq(0, "D", 2, "D"), eq(1, "D", 3, "D")],
};

/// Complementary diff pair: NMOS diff pair + PMOS diff pair sharing
/// the same input signals but providing rail-to-rail input range.
/// M0+M1 NMOS DP, M2+M3 PMOS DP.
pub const COMPLEMENTARY_DIFF_PAIR: Pattern = Pattern {
    name: "complementary_diff_pair",
    priority: 23,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        Slot { kind: SlotKind::ComplementOf(0), size_match: SizeMatch::Any, diode: DiodeReq::Any, gate_is_signal: true },
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), diode: DiodeReq::Any, gate_is_signal: true },
    ],
    links: &[
        eq_sig(0, "S", 1, "S"), // a tail node, not a rail (as `diff_pair`)
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
        eq_sig(2, "S", 3, "S"),
        eq(0, "G", 2, "G"), // same signal inputs
        eq(1, "G", 3, "G"),
        ne(2, "D", 3, "D"),
    ],
};

/// Non-overlapping clock generator element: two cross-coupled NAND gates
/// implemented at transistor level. M0+M1 = first NAND (NMOS),
/// M2+M3 = second NAND (PMOS portion). Structural match only.
pub const NAND_CROSS_COUPLED: Pattern = Pattern {
    name: "nand_cross_coupled",
    priority: 22,
    slots: &[
        S_ANY,      // M0: NAND1 pull-down A
        same0(),    // M1: NAND1 pull-down B (series)
        comp(0),    // M2: NAND1 pull-up A
        same_as(2), // M3: NAND1 pull-up B
    ],
    links: &[
        eq(0, "D", 1, "S"), // series NMOS
        eq(1, "D", 2, "D"), // output node
        eq(1, "D", 3, "D"),
        eq(2, "S", 3, "S"),
        eq(0, "G", 2, "G"), // input A shared
    ],
};

/// NAND gate (transistor level): M0+M1 series NMOS, M2+M3 parallel PMOS.
/// M0.G = M2.G (input A), M1.G = M3.G (input B).
/// Output at M1.D = M2.D = M3.D.
pub const NAND_GATE: Pattern = Pattern {
    name: "nand_gate",
    priority: 22,
    slots: &[
        S_ANY,      // M0: NMOS A
        same0(),    // M1: NMOS B
        comp(0),    // M2: PMOS A
        same_as(2), // M3: PMOS B
    ],
    links: &[
        eq(0, "D", 1, "S"), // series NMOS
        eq(1, "D", 2, "D"), // output
        eq(1, "D", 3, "D"), // output
        eq(0, "G", 2, "G"), // input A
        eq(1, "G", 3, "G"), // input B
        eq(2, "S", 3, "S"), // shared PMOS source (VDD)
    ],
};

/// NOR gate (transistor level): M0+M1 parallel NMOS, M2+M3 series PMOS.
/// M0.G = M2.G (input A), M1.G = M3.G (input B).
/// Output at M0.D = M1.D = M3.D.
pub const NOR_GATE: Pattern = Pattern {
    name: "nor_gate",
    priority: 22,
    slots: &[
        S_ANY,      // M0: NMOS A
        same0(),    // M1: NMOS B
        comp(0),    // M2: PMOS A
        same_as(2), // M3: PMOS B
    ],
    links: &[
        eq(0, "S", 1, "S"), // parallel NMOS share source
        eq(0, "D", 1, "D"), // parallel NMOS share drain
        eq(2, "D", 3, "S"), // series PMOS
        eq(3, "D", 0, "D"), // output = NMOS drain = bottom PMOS drain
        eq(0, "G", 2, "G"), // input A
        eq(1, "G", 3, "G"), // input B
    ],
};

/// Cascode diff pair with independent biases: M0+M1 diff pair,
/// M2 cascode on M0 with bias voltage A, M3 cascode on M1 with
/// different bias voltage B. Used when cascode devices need
/// different bias points.
pub const DIFF_PAIR_WITH_SPLIT_CASCODES: Pattern = Pattern {
    name: "diff_pair_with_split_cascodes",
    priority: 21,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        same0(),
        same_exact(2),
    ],
    links: &[
        eq_sig(0, "S", 1, "S"), // a rail-sourced pair is not differential (EXT-04)
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "S"),
        eq(1, "D", 3, "S"),
        ne(2, "G", 3, "G"), // different cascode biases
        ne(2, "D", 3, "D"),
    ],
};

/// Folded pair: diff pair where outputs connect to sources (not drains)
/// of complement-type devices. M0+M1 diff (same type),
/// M2+M3 complement loads. M0.D = M2.S, M1.D = M3.S.
pub const FOLDED_LOAD_PAIR: Pattern = Pattern {
    name: "folded_load_pair",
    priority: 22,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        comp(0),
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY },
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "S"), // folded connection
        eq(1, "D", 3, "S"), // folded connection
        eq(2, "G", 3, "G"),
    ],
};

/// Two-stage inverter buffer: inv1 (M0+M1) drives inv2 (M2+M3).
/// M0.D = M1.D = M2.G = M3.G.
pub const INVERTER_BUFFER: Pattern = Pattern {
    name: "inverter_buffer",
    priority: 23,
    slots: &[
        S_ANY,      // M0: inv1 N
        comp(0),    // M1: inv1 P
        same0(),    // M2: inv2 N
        same_as(1), // M3: inv2 P
    ],
    links: &[
        eq(0, "G", 1, "G"), // inv1 input
        eq(0, "D", 1, "D"), // inv1 output
        eq(2, "G", 3, "G"), // inv2 input
        eq(2, "D", 3, "D"), // inv2 output
        eq(0, "D", 2, "G"), // chain
    ],
};

/// Cascode current source pair: two independent cascode current sources
/// (4 FETs) where bottom pair shares gate+source and top pair shares gate.
/// All four are NOT diode-connected (externally biased).
pub const CASCODE_CURRENT_SOURCE_PAIR: Pattern = Pattern {
    name: "cascode_current_source_pair",
    priority: 21,
    slots: &[
        Slot { diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Forbidden, gate_is_signal: false },
        Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Forbidden, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), diode: DiodeReq::Forbidden, gate_is_signal: false },
    ],
    links: &[eq(0, "G", 1, "G"), eq(0, "S", 1, "S"), eq(0, "D", 2, "S"), eq(1, "D", 3, "S"), eq(2, "G", 3, "G"), ne(2, "D", 3, "D")],
};
