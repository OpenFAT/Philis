//! The corpus tables: each circuit's one-line netlist, and the leaves, sets
//! and compounds the annotator gives for it today (EXT-01, plan-01 T2/T3).
//!
//! Contents:
//! - `pub(super) const CIRCUITS`
//! - `pub(super) const NEGATIVE`
//! - `pub(super) type Row`
//! - `pub(super) type Set`
//! - `pub(super) const EXPECTED`
//! - `pub(super) type CompoundRow`
//! - `pub(super) const COMPOUNDS`

use super::common::STRONGARM;

/// `(name, netlist)`: repository fixtures transcribed without `.subckt`, then
/// constructed circuits (exact text of plan-01 EXT-01).
pub(super) const CIRCUITS: [(&str, &str); 18] = [
    // backend/annotator/src/tests.rs `ota()`.
    (
        "ota5t",
        "XM1 vout1 vinp vtail VSS nfet w=10u l=1u | XM2 vout2 vinm vtail VSS nfet w=10u l=1u
               XM3 vout1 vbias VDD VDD pfet w=20u l=1u | XM4 vout2 vbias VDD VDD pfet w=20u l=1u
               XM5 vtail vbn VSS VSS nfet w=40u l=2u",
    ),
    // examples/three_stage_opamp/src/main.rs.
    (
        "three_stage",
        "M1 n1 vin_p tail vss nfet w=4u l=0.5u | M2 n2 vin_n tail vss nfet w=4u l=0.5u
                     M3 tail vbias vss vss nfet w=8u l=0.5u
                     M4 n1 n1 vdd vdd pfet w=6u l=0.5u | M5 n2 n1 vdd vdd pfet w=6u l=0.5u
                     M6 n3 n2 vdd vdd pfet w=12u l=0.5u | M7 n3 vbias vss vss nfet w=6u l=0.5u
                     M8 vout n3 vdd vdd pfet w=40u l=0.5u | M9 vout vbias vss vss nfet w=20u l=0.5u",
    ),
    // benchmarks/fixtures/*.spice.
    (
        "dac4",
        "XC0 top VSS cap_generic_m1m2 W=2u L=2u m=1
              XC1 top b0 cap_generic_m1m2 W=2u L=2u m=1
              XC2 top b1 cap_generic_m1m2 W=2u L=2u m=2
              XC3 top b2 cap_generic_m1m2 W=2u L=2u m=4
              XC4 top b3 cap_generic_m1m2 W=2u L=2u m=8
              XMP0 b0 d0 VDD VDD pfet_01v8 W=1u L=0.15u | XMN0 b0 d0 VSS VSS nfet_01v8 W=0.5u L=0.15u
              XMP1 b1 d1 VDD VDD pfet_01v8 W=1u L=0.15u | XMN1 b1 d1 VSS VSS nfet_01v8 W=0.5u L=0.15u
              XMP2 b2 d2 VDD VDD pfet_01v8 W=1u L=0.15u | XMN2 b2 d2 VSS VSS nfet_01v8 W=0.5u L=0.15u
              XMP3 b3 d3 VDD VDD pfet_01v8 W=1u L=0.15u | XMN3 b3 d3 VSS VSS nfet_01v8 W=0.5u L=0.15u
              XMC cmp top VSS VSS nfet_01v8 W=1u L=0.15u",
    ),
    ("bgr_core", "XQ1 VSS VSS e1 pnp_05v5_W3p40L3p40 m=1 | XQ2 VSS VSS e2 pnp_05v5_W3p40L3p40 m=8"),
    ("bjt_mirror", "XQ1 outn in VSS npn_05v5_1x1 W=1u L=1u | XQ2 outp in VDD pnp_05v5_W3p40L3p40"),
    (
        "chain4",
        "XM1 a g b VSS nfet_01v8 W=2u L=0.5u | XM2 b g c VSS nfet_01v8 W=2u L=0.5u
                XM3 c g d VSS nfet_01v8 W=2u L=0.5u | XM4 d g e VSS nfet_01v8 W=2u L=0.5u",
    ),
    ("pair", "XM1 d g VSS VSS nfet_01v8 W=2u L=0.5u | XM2 d g VSS VSS nfet_01v8 W=2u L=0.5u"),
    (
        "quad",
        "XM1 n g VSS VSS nfet_01v8 W=2u L=0.5u | XM2 n g VSS VSS nfet_01v8 W=2u L=0.5u
              XM3 n g VSS VSS nfet_01v8 W=2u L=0.5u | XM4 n g VSS VSS nfet_01v8 W=2u L=0.5u",
    ),
    (
        "folded",
        "M1 x1 vinp tail VSS nfet w=10u l=1u | M2 x2 vinn tail VSS nfet w=10u l=1u | M0 tail vbn VSS VSS nfet w=20u l=1u
                M3 x1 vbp1 VDD VDD pfet w=20u l=1u | M4 x2 vbp1 VDD VDD pfet w=20u l=1u
                M5 o1 vbp2 x1 VDD pfet w=10u l=1u | M6 out vbp2 x2 VDD pfet w=10u l=1u
                M7 o1 vbn2 y1 VSS nfet w=5u l=1u | M8 out vbn2 y2 VSS nfet w=5u l=1u
                M9 y1 o1 VSS VSS nfet w=5u l=1u | M10 y2 o1 VSS VSS nfet w=5u l=1u",
    ),
    (
        "gilbert",
        "M1 x1 rfp tail VSS nfet w=8u l=0.5u | M2 x2 rfn tail VSS nfet w=8u l=0.5u | M0 tail vb VSS VSS nfet w=16u l=0.5u
                 M3 outp lop x1 VSS nfet w=4u l=0.5u | M4 outn lon x1 VSS nfet w=4u l=0.5u
                 M5 outp lon x2 VSS nfet w=4u l=0.5u | M6 outn lop x2 VSS nfet w=4u l=0.5u",
    ),
    (
        "rail2rail",
        "MN1 xn1 vinp tn VSS nfet w=4u l=0.5u | MP1 xp1 vinp tp VDD pfet w=8u l=0.5u
                   MN2 xn2 vinn tn VSS nfet w=4u l=0.5u | MP2 xp2 vinn tp VDD pfet w=8u l=0.5u
                   MN0 tn vbn VSS VSS nfet w=8u l=0.5u | MP0 tp vbp VDD VDD pfet w=16u l=0.5u",
    ),
    (
        "latch",
        "MN1 q qb VSS VSS nfet w=2u l=0.15u | MP1 q qb VDD VDD pfet w=4u l=0.15u
               MN2 qb q VSS VSS nfet w=2u l=0.15u | MP2 qb q VDD VDD pfet w=4u l=0.15u",
    ),
    (
        "mirror6",
        "MR ref ref VSS VSS nfet w=2u l=1u | MO1 o1 ref VSS VSS nfet w=2u l=1u | MO2 o2 ref VSS VSS nfet w=4u l=1u
                 MO3 o3 ref VSS VSS nfet w=2u l=1u | MO4 o4 ref VSS VSS nfet w=8u l=1u | MO5 o5 ref VSS VSS nfet w=2u l=1u",
    ),
    (
        "brokaw",
        "Q1 c1 vb x npn m=1 | Q2 c2 vb y npn m=8 | R2 y x rpoly w=2u l=20u | R1 x VSS rpoly w=2u l=80u
                MP1 c1 c1 VDD VDD pfet w=4u l=1u | MP2 c2 c1 VDD VDD pfet w=4u l=1u",
    ),
    ("rdiv", "RA top mid rpoly w=2u l=10u | RB mid VSS rpoly w=2u l=40u"),
    (
        "splitdac",
        "C0 tl VSS cap w=3u l=3u m=1 | C1 tl b0 cap w=3u l=3u m=1 | C2 tl b1 cap w=3u l=3u m=2
                  C3 tm b2 cap w=3u l=3u m=1 | C4 tm b3 cap w=3u l=3u m=2 | CA tl tm cap w=3u l=4u m=1",
    ),
    ("strongarm", STRONGARM),
    // EXT-19 step 5: a source-degenerated diff pair (M1 regression note).
    (
        "degen_pair",
        "M1 o1 inp s1 VSS nfet w=4u l=0.5u | M2 o2 inn s2 VSS nfet w=4u l=0.5u | R1 s1 tail rpoly w=2u l=10u
                    R2 s2 tail rpoly w=2u l=10u | M0 tail vb VSS VSS nfet w=8u l=0.5u",
    ),
];

/// Negative circuits: nothing in them should be matched (T3). `bjt_mirror` is
/// the fourth, shared with [`CIRCUITS`].
pub(super) const NEGATIVE: [(&str, &str); 3] = [
    ("sc_switches", "MS1 a1 p1 b1 VSS nfet w=1u l=0.15u | MS2 a2 p2 b2 VSS nfet w=1u l=0.15u"),
    ("equal_fets", "MA da ga VSS VSS nfet w=2u l=0.5u | MB db gb VSS VSS nfet w=2u l=0.5u"),
    ("inv_chain", "MN1 a in VSS VSS nfet w=1u l=0.15u | MP1 a in VDD VDD pfet w=2u l=0.15u | MN2 b a VSS VSS nfet w=1u l=0.15u | MP2 b a VDD VDD pfet w=2u l=0.15u | MN3 c b VSS VSS nfet w=1u l=0.15u | MP3 c b VDD VDD pfet w=2u l=0.15u | MN4 out c VSS VSS nfet w=1u l=0.15u | MP4 out c VDD VDD pfet w=2u l=0.15u"),
];

/// Today's decisions per circuit: leaves `(kind, sorted member names)`, then the
/// [`Canon`] fields `pairs`, `selfs`, `net_pairs`, `axes`, then `sets` (EXT-15).
pub(super) type Row = (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
    &'static [(&'static str, &'static str)],
    &'static [&'static str],
    &'static [(&'static str, &'static str)],
    usize,
    &'static [Set],
);

/// `(members (name, parallel, series), kind, class)`.
pub(super) type Set = (&'static [(&'static str, u16, u16)], &'static str, &'static str);

pub(super) const EXPECTED: [Row; 18] = [
    // EXT-05: declared roles: five_transistor_ota's slots 2,3 are a Load.
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    (
        "ota5t",
        &[("Load", &["XM3", "XM4"]), ("DiffPair", &["XM1", "XM2"])],
        &[("XM1", "XM2"), ("XM3", "XM4")],
        &["XM5"],
        &[("vinm", "vinp"), ("vout1", "vout2")],
        1,
        &[(&[("XM1", 1, 1), ("XM2", 1, 1)], "Voltage", "Moderate"), (&[("XM3", 2, 1), ("XM4", 2, 1)], "Current", "Moderate")],
    ),
    // EXT-05: declared roles: diff_pair_with_mirror_load's slots 2,3 are a Load.
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    (
        "three_stage",
        &[("DiffPair", &["M1", "M2"]), ("Group", &["M6", "M8", "M9"]), ("Load", &["M4", "M5"])],
        &[("M1", "M2"), ("M4", "M5")],
        &["M3"],
        &[("n1", "n2"), ("vin_n", "vin_p")],
        1,
        &[
            (&[("M1", 1, 1), ("M2", 1, 1)], "Voltage", "Moderate"),
            (&[("M3", 4, 1), ("M7", 3, 1), ("M9", 10, 1)], "Current", "Minimal"),
            (&[("M4", 1, 1), ("M5", 1, 1)], "Current", "Moderate"),
        ],
    ),
    // EXT-05 review: `cmos_inverter`'s declared prox makes each switch inverter a Stack leaf.
    // EXT-19: the capacitor bank is one binary DacBank set, Exceptional Ratio.
    (
        "dac4",
        &[("Stack", &["XMN0", "XMP0"]), ("Stack", &["XMN1", "XMP1"]), ("Stack", &["XMN2", "XMP2"]), ("Stack", &["XMN3", "XMP3"])],
        &[],
        &[],
        &[],
        0,
        &[(&[("XC0", 1, 1), ("XC1", 1, 1), ("XC2", 2, 1), ("XC3", 4, 1), ("XC4", 8, 1)], "Ratio", "Exceptional")],
    ),
    // EXT-19: bjt_ratioed_pair (a CurrentMirror leaf, matched but no hard Symmetry: a 1:N
    // centroid array, `emit.rs`, review fixes 1); the set is Voltage (ΔV_BE) Moderate (BandgapCore).
    ("bgr_core", &[("CurrentMirror", &["XQ1", "XQ2"])], &[], &[], &[], 0, &[(&[("XQ1", 1, 1), ("XQ2", 8, 1)], "Voltage", "Moderate")]),
    ("bjt_mirror", &[], &[], &[], &[], 0, &[]),
    // EXT-05: series_stack_4 declares no roles, so no re-searched Stack children.
    ("chain4", &[("Group", &["XM1", "XM2", "XM3", "XM4"])], &[], &[], &[], 0, &[]),
    ("pair", &[], &[], &[], &[], 0, &[]),
    ("quad", &[], &[], &[], &[], 0, &[]),
    // EXT-04: DIFF_PAIR_SPLIT_SOURCE (any-pins-differ, no shared net) was a dead
    // pattern that falsely matched M7/M9 and M10/M8 as DiffPair; deleted, so
    // they fall back to their real Stack structure.
    // EXT-05: folded_cascode_core's declared pairs and prox; the M7-M10 composite
    // declares no roles.
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    // EXT-20: Symmetry per compound: M7/M8, M9/M10 and the self M0 join.
    (
        "folded",
        &[
            ("CascodePair", &["M5", "M6"]),
            ("DiffPair", &["M1", "M2"]),
            ("Group", &["M10", "M7", "M8", "M9"]),
            ("Load", &["M3", "M4"]),
            ("Stack", &["M3", "M5"]),
            ("Stack", &["M4", "M6"]),
        ],
        &[("M1", "M2"), ("M10", "M9"), ("M3", "M4"), ("M5", "M6"), ("M7", "M8")],
        &["M0"],
        &[("o1", "out"), ("vinn", "vinp"), ("x1", "x2"), ("y1", "y2")],
        1,
        &[
            (&[("M1", 1, 1), ("M2", 1, 1)], "Voltage", "Moderate"),
            (&[("M10", 1, 1), ("M9", 1, 1)], "Current", "Minimal"),
            (&[("M3", 2, 1), ("M4", 2, 1)], "Current", "Moderate"),
            (&[("M5", 1, 1), ("M6", 1, 1)], "Current", "Moderate"),
            (&[("M7", 1, 1), ("M8", 1, 1)], "Current", "Minimal"),
        ],
    ),
    // EXT-04: GILBERT_CELL's link fix makes it match the whole 6-device cell
    // (today's child re-search, max_slots=2, decomposes it into 3 DiffPair legs).
    // EXT-05: declared pairs are the mirror images (M3,M6), (M4,M5).
    // EXT-15: matched sets; the switching quad M3-M6 is one set (the overlapping diff_pair
    // matches (M3,M4), (M5,M6) join the declared mirror pairs, AA-01).
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    // EXT-20: the compound's self M0 joins the Symmetry.
    (
        "gilbert",
        &[("DiffPair", &["M1", "M2"]), ("DiffPair", &["M3", "M6"]), ("DiffPair", &["M4", "M5"])],
        &[("M1", "M2"), ("M3", "M6"), ("M4", "M5")],
        &["M0"],
        &[("outn", "outp"), ("rfn", "rfp"), ("x1", "x2")],
        1,
        &[(&[("M1", 1, 1), ("M2", 1, 1)], "Voltage", "Moderate"), (&[("M3", 1, 1), ("M4", 1, 1), ("M5", 1, 1), ("M6", 1, 1)], "Voltage", "Moderate")],
    ),
    // EXT-05: complementary_diff_pair declares both polarities' DiffPairs.
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    // EXT-20: the compound's selfs (both tails) join the Symmetry.
    (
        "rail2rail",
        &[("DiffPair", &["MN1", "MN2"]), ("DiffPair", &["MP1", "MP2"])],
        &[("MN1", "MN2"), ("MP1", "MP2")],
        &["MN0", "MP0"],
        &[("vinn", "vinp"), ("xn1", "xn2"), ("xp1", "xp2")],
        1,
        &[(&[("MN1", 1, 1), ("MN2", 1, 1)], "Voltage", "Moderate"), (&[("MP1", 1, 1), ("MP2", 1, 1)], "Voltage", "Moderate")],
    ),
    // EXT-05 (AA-03): cross_coupled_inverters' declared pairs and prox, not inverters.
    // EXT-09: `net_pairs` moves — Differential now comes from the DiffPair leaves, and
    // the cross-coupled MN1/MN2 pair's drains (q, qb) differ, so it yields one.
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    (
        "latch",
        &[("DiffPair", &["MN1", "MN2"]), ("DiffPair", &["MP1", "MP2"]), ("Stack", &["MN1", "MP1"]), ("Stack", &["MN2", "MP2"])],
        &[("MN1", "MN2"), ("MP1", "MP2")],
        &[],
        &[("q", "qb")],
        1,
        &[(&[("MN1", 1, 1), ("MN2", 1, 1)], "Voltage", "Moderate"), (&[("MP1", 1, 1), ("MP2", 1, 1)], "Voltage", "Moderate")],
    ),
    // EXT-05: current_mirror_4 declares (ref, k) per output; only the first pair
    // holding the shared reference gets a Symmetry.
    // EXT-06: equal-priority current_mirror_4 matches tie on canonical labels, not
    // device ids: MR takes the three identical 2 µm outputs, MO2/MO4 pair up.
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    // EXT-20: no compound holds the mirror (one ratioed set), so no Symmetry (AA-24).
    (
        "mirror6",
        &[
            ("CurrentMirror", &["MO1", "MR"]),
            ("CurrentMirror", &["MO3", "MR"]),
            ("CurrentMirror", &["MO5", "MR"]),
            ("CurrentMirror", &["MO2", "MO4"]),
        ],
        &[],
        &[],
        &[],
        0,
        &[(&[("MO1", 1, 1), ("MO2", 2, 1), ("MO3", 1, 1), ("MO4", 4, 1), ("MO5", 1, 1), ("MR", 1, 1)], "Current", "Minimal")],
    ),
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    // EXT-19: Q1/Q2 ratioed pair (Voltage Moderate) and the bandgap resistors R1:R2 = 4:1 in
    // series units of 20 µm (Ratio Moderate).
    // EXT-20: MP1/MP2 is in no compound (a CurrentMirror is no seed), so no Symmetry.
    (
        "brokaw",
        &[("CurrentMirror", &["MP1", "MP2"]), ("CurrentMirror", &["Q1", "Q2"])],
        &[],
        &[],
        &[],
        0,
        &[
            (&[("MP1", 1, 1), ("MP2", 1, 1)], "Current", "Minimal"),
            (&[("Q1", 1, 1), ("Q2", 8, 1)], "Voltage", "Moderate"),
            (&[("R1", 1, 4), ("R2", 1, 1)], "Ratio", "Moderate"),
        ],
    ),
    // EXT-19: the divider is one FeedbackRatio set, RB four 10 µm series units.
    ("rdiv", &[], &[], &[], &[], 0, &[(&[("RA", 1, 1), ("RB", 1, 4)], "Ratio", "Moderate")]),
    // EXT-19: both banks and the bridge CA are one split_dac set (CA outside the unit).
    (
        "splitdac",
        &[],
        &[],
        &[],
        &[],
        0,
        &[(&[("C0", 1, 1), ("C1", 1, 1), ("C2", 2, 1), ("C3", 1, 1), ("C4", 2, 1), ("CA", 1, 1)], "Ratio", "Exceptional")],
    ),
    // EXT-05: five_transistor_ota's roles (0,1,DiffPair), (2,3,Load), self 4 (mn0,
    // not mp8); cross_coupled_inverters' pairs; complementary_diff_pair's sources must be a
    // signal, so the output inverters no longer match it and mp9/mp10 join
    // undeclared 3-device groups (their CurrentMirror was a misrecognition).
    // EXT-05 review: the out-of-plan `diff_pair_cross_coupled_load` entry is gone; the
    // row is unchanged, as `five_transistor_ota` (EXT-06 selection) claims mn1/mn2/mp7/mp8/mn0.
    // EXT-09: `net_pairs` gains (vin_o, vip_o) — the second DiffPair leaf (mn3, mn4)
    // now also yields a Differential; the old device-pair scan missed it.
    // EXT-15: matched sets (components of MatchSym ∪ MatchBlock over every match, shared-bias groups), unitized.
    // EXT-16: kind and class per set (DiffPair leaf → Voltage; role defaults: input/load Moderate, bias Minimal).
    // EXT-24: net pairs are every compound's mirrored non-rail nets (Differential), not DiffPair drains only.
    // EXT-20: Symmetry per compound: every ALIGN gold pair, one axis.
    (
        "strongarm",
        &[
            ("DiffPair", &["mn1", "mn2"]),
            ("DiffPair", &["mn3", "mn4"]),
            ("DiffPair", &["mp5", "mp6"]),
            ("Group", &["mn13", "mp10", "mp11"]),
            ("Group", &["mn14", "mp12", "mp9"]),
            ("Load", &["mp7", "mp8"]),
            ("Stack", &["mn3", "mp5"]),
            ("Stack", &["mn4", "mp6"]),
        ],
        &[("mn1", "mn2"), ("mn13", "mn14"), ("mn3", "mn4"), ("mp10", "mp9"), ("mp11", "mp12"), ("mp5", "mp6"), ("mp7", "mp8")],
        &["mn0"],
        &[("vin", "vip"), ("vin_d", "vip_d"), ("vin_o", "vip_o"), ("von", "vop")],
        1,
        &[
            (&[("mn1", 2, 1), ("mn2", 2, 1)], "Voltage", "Moderate"),
            (&[("mn13", 1, 1), ("mn14", 1, 1)], "Current", "Minimal"),
            (&[("mn3", 1, 1), ("mn4", 1, 1)], "Voltage", "Moderate"),
            (&[("mp10", 1, 1), ("mp7", 1, 1), ("mp8", 1, 1), ("mp9", 1, 1)], "Current", "Moderate"),
            (&[("mp11", 1, 1), ("mp12", 1, 1), ("mp5", 4, 1), ("mp6", 4, 1)], "Voltage", "Moderate"),
        ],
    ),
    // EXT-19 step 5: the degenerated pair is a DiffPair (`passive::degenerated_pairs`), its
    // resistors one degeneration Ratio set (Moderate, the pair's role).
    // EXT-20: the compound's equal couples (R1/R2 too) and its self M0.
    (
        "degen_pair",
        &[("DiffPair", &["M1", "M2"])],
        &[("M1", "M2"), ("R1", "R2")],
        &["M0"],
        &[("inn", "inp"), ("o1", "o2"), ("s1", "s2")],
        1,
        &[(&[("M1", 1, 1), ("M2", 1, 1)], "Voltage", "Moderate"), (&[("R1", 1, 1), ("R2", 1, 1)], "Ratio", "Moderate")],
    ),
];

/// EXT-14 compounds, exact, on [`common::canon_intent`]: `(name, pairs, selfs,
/// net_pairs, axes)`; every circuit not listed has none.
pub(super) type CompoundRow =
    (&'static str, &'static [(&'static str, &'static str)], &'static [&'static str], &'static [(&'static str, &'static str)], usize);

pub(super) const COMPOUNDS: [CompoundRow; 8] = [
    ("ota5t", &[("XM1", "XM2"), ("XM3", "XM4")], &["XM5"], &[("vinm", "vinp"), ("vout1", "vout2")], 1),
    (
        "folded",
        &[("M1", "M2"), ("M3", "M4"), ("M5", "M6"), ("M7", "M8"), ("M10", "M9")],
        &["M0"],
        &[("vinn", "vinp"), ("x1", "x2"), ("o1", "out"), ("y1", "y2")],
        1,
    ),
    ("gilbert", &[("M1", "M2"), ("M3", "M6"), ("M4", "M5")], &["M0"], &[("rfn", "rfp"), ("x1", "x2"), ("outn", "outp")], 1),
    ("rail2rail", &[("MN1", "MN2"), ("MP1", "MP2")], &["MN0", "MP0"], &[("vinn", "vinp"), ("xn1", "xn2"), ("xp1", "xp2")], 1),
    ("latch", &[("MN1", "MN2"), ("MP1", "MP2")], &[], &[("q", "qb")], 1),
    ("three_stage", &[("M1", "M2"), ("M4", "M5")], &["M3"], &[("n1", "n2"), ("vin_n", "vin_p")], 1),
    // T1's circuit: align_gold checks it against the ALIGN gold.
    // EXT-19: the degeneration resistors mirror through s1/s2; the tail M0 is on the axis.
    ("degen_pair", &[("M1", "M2"), ("R1", "R2")], &["M0"], &[("inn", "inp"), ("o1", "o2"), ("s1", "s2")], 1),
    (
        "strongarm",
        &[("mn1", "mn2"), ("mn3", "mn4"), ("mp5", "mp6"), ("mp7", "mp8"), ("mp10", "mp9"), ("mp11", "mp12"), ("mn13", "mn14")],
        &["mn0"],
        &[("vin", "vip"), ("vin_d", "vip_d"), ("vin_o", "vip_o"), ("von", "vop")],
        1,
    ),
];
