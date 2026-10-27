//! Composites of five or more devices (priority 30–59): OTAs, Gilbert
//! cells, VCO cores, output stages. Each row cites its ALIGN template or
//! textbook.
//!
//! Contents:
//! - `pub const FIVE_TRANSISTOR_OTA`
//! - `pub const DIFF_PAIR_CASCODE_LOAD`
//! - `pub const DIFF_PAIR_CROSS_COUPLED_LOAD`
//! - `pub const WIDE_SWING_CASCODE_MIRROR_5`
//! - `pub const TELESCOPIC_OTA_CORE`
//! - `pub const FOLDED_CASCODE_CORE`
//! - `pub const GILBERT_CELL`
//! - `pub const VCO_CORE_WITH_TAILS`
//! - `pub const TELESCOPIC_OTA_FULL`
//! - `pub const OTA_SELF_BIASED_LOAD`
//! - `pub const CASCODED_DIFF_PAIR_WITH_TAIL`
//! - `pub const DIFF_OUTPUT_STAGE`

use crate::pattern::{DiodeReq, Pattern, SizeMatch, Slot, SlotKind};

use super::{comp, eq, ne, same0, same0_exact, same0_samel, same_as, same_exact, S_ANY};

/// Five-transistor OTA: M0+M1 diff pair, M2+M3 load (diode mirror, or a pair
/// biased from outside — fully differential), M4 tail current source.
/// Classic topology from Razavi Ch.9 / Allen-Holberg Ch.6.
pub const FIVE_TRANSISTOR_OTA: Pattern = Pattern {
    name: "five_transistor_ota",
    priority: 34,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY }, // M0: diff pair A
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true }, // M1: diff pair B
        comp(0),                                // M2: load (diode or biased)
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY }, // M3: load
        same0(),                                // M4: tail
    ],
    links: &[
        eq(0, "S", 1, "S"), // diff pair shared source
        ne(0, "G", 1, "G"), // different inputs
        eq(0, "D", 2, "D"), // M2 loads M0
        eq(1, "D", 3, "D"), // M3 loads M1
        eq(2, "G", 3, "G"), // mirror gate
        eq(2, "S", 3, "S"), // mirror source
        eq(0, "S", 4, "D"), // tail drives diff pair
    ],
};

/// Diff pair + cascode load: M0+M1 diff pair, M2+M3 cascode load pair
/// (diff pair drains connect to cascode SOURCES), M4 tail.
/// Used in high-gain single-stage amplifiers. The key distinction from
/// an active load is the drain-to-source (cascode) connection.
pub const DIFF_PAIR_CASCODE_LOAD: Pattern = Pattern {
    name: "diff_pair_cascode_load",
    priority: 33,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        comp(0), // M2: cascode load A
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY },
        same0(), // M4: tail
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "S"), // drain-to-source = cascode connection
        eq(1, "D", 3, "S"), // drain-to-source = cascode connection
        eq(2, "G", 3, "G"),
        eq(0, "S", 4, "D"),
    ],
};

/// Diff pair with tail and cross-coupled load: M0+M1 diff pair,
/// M2+M3 cross-coupled load (complement), M4 tail.
pub const DIFF_PAIR_CROSS_COUPLED_LOAD: Pattern = Pattern {
    name: "diff_pair_cross_coupled_load",
    priority: 35,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        comp(0),
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY },
        same0(),
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "D"),
        eq(1, "D", 3, "D"),
        eq(2, "G", 3, "D"), // cross-coupled
        eq(2, "D", 3, "G"), // cross-coupled
        eq(2, "S", 3, "S"),
        eq(0, "S", 4, "D"),
    ],
};

/// Wide-swing cascode mirror (5T): standard cascode mirror (4T) plus
/// a bias device. M0 diode, M1 output, M2 cascode diode, M3 cascode out,
/// M4 bias (diode, generates bottom gate voltage).
pub const WIDE_SWING_CASCODE_MIRROR_5: Pattern = Pattern {
    name: "wide_swing_cascode_mirror_5",
    priority: 30,
    slots: &[
        Slot { diode: DiodeReq::Required, ..S_ANY },
        same0_samel(),
        Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY },
        same0(),
        Slot { kind: SlotKind::SameTypeAs(0), diode: DiodeReq::Required, ..S_ANY },
    ],
    links: &[
        eq(0, "G", 1, "G"),
        eq(0, "S", 1, "S"),
        eq(0, "D", 2, "S"),
        eq(1, "D", 3, "S"),
        eq(2, "G", 3, "G"),
        eq(4, "G", 0, "G"), // M4 bias shares bottom gate
        eq(4, "S", 0, "S"), // M4 shares source with bottom
    ],
};

/// Telescopic OTA: M0+M1 diff pair, M2+M3 NMOS cascodes on diff pair,
/// M4+M5 PMOS cascode loads.
/// Topology: M0.D=M2.S, M1.D=M3.S (input cascodes), M2.D=M4.D, M3.D=M5.D.
pub const TELESCOPIC_OTA_CORE: Pattern = Pattern {
    name: "telescopic_ota_core",
    priority: 42,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY }, // M0: diff A
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true }, // M1: diff B
        same0(),                                // M2: cascode A
        same_exact(2),                          // M3: cascode B
        comp(0),                                // M4: load A
        Slot { kind: SlotKind::SameTypeAs(4), size_match: SizeMatch::ExactAs(4), ..S_ANY },
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "S"),
        eq(1, "D", 3, "S"),
        eq(2, "G", 3, "G"),
        eq(2, "D", 4, "D"),
        eq(3, "D", 5, "D"),
        eq(4, "G", 5, "G"),
        eq(4, "S", 5, "S"),
    ],
};

/// Folded cascode core: M0+M1 diff pair (e.g. PMOS), M2+M3 folding
/// devices (NMOS, cascode), M4+M5 current sources providing fold current.
/// M0.D = M2.S (fold point A), M1.D = M3.S (fold point B).
/// M4.D = M2.S, M5.D = M3.S (current injection into fold).
pub const FOLDED_CASCODE_CORE: Pattern = Pattern {
    name: "folded_cascode_core",
    priority: 43,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        comp(0), // M2: fold cascode A
        Slot { kind: SlotKind::SameTypeAs(2), size_match: SizeMatch::ExactAs(2), ..S_ANY },
        comp(0), // M4: fold current src A
        Slot { kind: SlotKind::SameTypeAs(4), size_match: SizeMatch::ExactAs(4), ..S_ANY },
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "S"), // fold point A
        eq(1, "D", 3, "S"), // fold point B
        eq(2, "G", 3, "G"), // cascode gate
        eq(4, "D", 2, "S"), // current injection A
        eq(5, "D", 3, "S"), // current injection B
        eq(4, "G", 5, "G"),
        eq(4, "S", 5, "S"),
    ],
};

/// Gilbert cell multiplier core: M0+M1 bottom diff pair (RF input),
/// M2+M3 top diff pair A (LO), M4+M5 top diff pair B (LO complement).
/// M0.D = shared source of M2+M3, M1.D = shared source of M4+M5.
/// M2.D = M5.D (IF+), M3.D = M4.D (IF-).
pub const GILBERT_CELL: Pattern = Pattern {
    name: "gilbert_cell",
    priority: 44,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY }, // M0: RF+
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true }, // M1: RF-
        same0(),                                // M2: LO quad A+
        same_exact(2),                          // M3: LO quad A-
        same_exact(2),                          // M4: LO quad B+
        same_exact(2),                          // M5: LO quad B-
    ],
    links: &[
        eq(0, "S", 1, "S"), // bottom pair source
        ne(0, "G", 1, "G"), // RF inputs differ
        eq(0, "D", 2, "S"), // M0 feeds quad A
        eq(0, "D", 3, "S"),
        eq(1, "D", 4, "S"), // M1 feeds quad B
        eq(1, "D", 5, "S"),
        eq(2, "G", 5, "G"),
        eq(3, "G", 4, "G"),
        ne(2, "G", 3, "G"), // quad A LO inputs differ
        eq(2, "D", 4, "D"), // cross-connect outputs
        eq(3, "D", 5, "D"),
    ],
};

/// Complementary cross-coupled VCO core: two NMOS cross-coupled +
/// two PMOS cross-coupled, forming LC-VCO negative resistance.
/// M0+M1 NMOS cross-coupled, M2+M3 PMOS cross-coupled,
/// M4+M5 optional tail current sources (one per type).
pub const VCO_CORE_WITH_TAILS: Pattern = Pattern {
    name: "vco_core_with_tails",
    priority: 44,
    slots: &[
        S_ANY,
        same0_exact(),
        comp(0),
        same_exact(2),
        same0(),    // M4: NMOS tail
        same_as(2), // M5: PMOS tail
    ],
    links: &[
        eq(0, "G", 1, "D"),
        eq(0, "D", 1, "G"),
        eq(2, "G", 3, "D"),
        eq(2, "D", 3, "G"),
        eq(0, "D", 2, "D"),
        eq(1, "D", 3, "D"),
        eq(0, "S", 4, "D"), // NMOS tail
        eq(1, "S", 4, "D"),
        eq(2, "S", 5, "D"), // PMOS tail
        eq(3, "S", 5, "D"),
    ],
};

/// Full telescopic OTA: 6-device telescopic core + 2 tail devices.
/// M0+M1 diff pair, M2+M3 input cascodes, M4+M5 load cascodes,
/// M6 tail current source, M7 load bias (diode-connected).
pub const TELESCOPIC_OTA_FULL: Pattern = Pattern {
    name: "telescopic_ota_full",
    priority: 52,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        same0(),
        same_exact(2),
        comp(0),
        Slot { kind: SlotKind::SameTypeAs(4), size_match: SizeMatch::ExactAs(4), ..S_ANY },
        same0(), // M6: tail
        Slot {
            // M7: load cascode bias
            kind: SlotKind::SameTypeAs(4),
            size_match: SizeMatch::Any,
            diode: DiodeReq::Any,
            gate_is_signal: false,
        },
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "S"),
        eq(1, "D", 3, "S"),
        eq(2, "G", 3, "G"),
        eq(2, "D", 4, "D"),
        eq(3, "D", 5, "D"),
        eq(4, "G", 5, "G"),
        eq(4, "S", 5, "S"),
        eq(0, "S", 6, "D"), // tail
        eq(4, "S", 7, "S"), // load bias shares supply
    ],
};

/// Diff pair with diode load + mirror load + tail (5T variant where
/// loads are same type as diff pair but diode/mirror).
pub const OTA_SELF_BIASED_LOAD: Pattern = Pattern {
    name: "ota_self_biased_load",
    priority: 32,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::Any, diode: DiodeReq::Required, gate_is_signal: false },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::SameLAs(2), ..S_ANY },
        comp(0), // tail is complementary type
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        ne(0, "D", 1, "D"),
        eq(0, "D", 2, "D"),
        eq(1, "D", 3, "D"),
        eq(2, "G", 3, "G"),
        eq(2, "S", 3, "S"),
        eq(0, "S", 4, "D"),
    ],
};

/// Symmetrical OTA (Razavi Fig. 9.35): M0+M1 diff pair, M2+M3
/// same-type cascode on diff outputs, M4 tail. Cascoded diff pair
/// for higher gain.
pub const CASCODED_DIFF_PAIR_WITH_TAIL: Pattern = Pattern {
    name: "cascoded_diff_pair_with_tail",
    priority: 32,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        same0(),
        same_exact(2),
        same0(), // tail
    ],
    links: &[
        eq(0, "S", 1, "S"),
        ne(0, "G", 1, "G"),
        eq(0, "D", 2, "S"),
        eq(1, "D", 3, "S"),
        eq(2, "G", 3, "G"),
        ne(2, "D", 3, "D"),
        eq(0, "S", 4, "D"),
    ],
};

/// Fully differential amplifier output stage: two complementary
/// source followers (M0+M1 for out+, M2+M3 for out-), biased by M4.
/// Outputs are at the source-source junction (complementary followers).
pub const DIFF_OUTPUT_STAGE: Pattern = Pattern {
    name: "diff_output_stage",
    priority: 30,
    slots: &[
        Slot { gate_is_signal: true, ..S_ANY },
        Slot { kind: SlotKind::ComplementOf(0), size_match: SizeMatch::Any, diode: DiodeReq::Forbidden, gate_is_signal: true },
        Slot { kind: SlotKind::SameTypeAs(0), size_match: SizeMatch::ExactAs(0), diode: DiodeReq::Any, gate_is_signal: true },
        Slot { kind: SlotKind::SameTypeAs(1), size_match: SizeMatch::ExactAs(1), ..S_ANY },
        same0(), // tail/bias
    ],
    links: &[
        eq(0, "S", 1, "S"), // out+ (source followers share output at source)
        eq(2, "S", 3, "S"), // out-
        eq(0, "G", 1, "G"), // same gate signal drives complementary pair
        eq(2, "G", 3, "G"), // same gate signal drives complementary pair
        ne(0, "G", 2, "G"), // different signal inputs
        ne(0, "D", 1, "D"), // different supply rails
        ne(0, "S", 2, "S"), // different outputs
    ],
};
