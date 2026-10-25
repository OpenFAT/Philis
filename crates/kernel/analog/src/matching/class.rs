//! The matching class's one home (GAP-01): the device family a matched set
//! belongs to, the environment each class asks of the layout (read from the
//! sidecar's `[MIN, MOD, EXC]` tier arrays) and Hastings' class limit table.

pub use crate::matching::mismatch::MatchKind;
pub use crate::base::MatchClass;

use crate::base::DeviceKind;

/// The device family a class limit is tabulated for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Mos,
    Bipolar,
    Diode,
    Resistor,
    Capacitor,
}

impl Family {
    /// `None` for an inductor: no class table, never matched.
    #[must_use]
    pub fn of(k: DeviceKind) -> Option<Family> {
        match k {
            DeviceKind::Nmos | DeviceKind::Pmos => Some(Family::Mos),
            DeviceKind::Npn | DeviceKind::Pnp => Some(Family::Bipolar),
            DeviceKind::Diode => Some(Family::Diode),
            DeviceKind::Resistor => Some(Family::Resistor),
            DeviceKind::Capacitor => Some(Family::Capacitor),
            DeviceKind::Inductor => None,
        }
    }
}

/// How matched gates are strapped: MIN a poly bar, MOD a poly bar ≥ 1 µm
/// from the gates (CELL-11 step 4), EXC each gate contacted in metal with
/// no shared poly (Hastings §13.3 rule 22, hastings.txt L42638–42647).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GateStrap {
    #[default]
    PolyBar,
    PolyBarFar,
    MetalIsolated,
}

/// What a MOS matched set of one class asks of its surroundings, nm.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MosEnv {
    /// Outer dummy poly edge to the nearest active gate (rule 12).
    pub dummy_reach_nm: i32,
    /// Diffusion run past the outer dummy (LOD moat, §13.2.2 and rule 12).
    pub moat_nm: i32,
    /// Well edge to the matched gates (rule 19).
    pub wpe_nm: i32,
    /// Gate poly extension beyond the deck rule (rule 21).
    pub gate_ext_extra_nm: i32,
    pub gate_strap: GateStrap,
}

/// Hastings' MOS tiers, `[MIN, MOD, EXC]`, nm; class policy, the same on
/// every process (Q59).
/// - dummy reach, §13.3 rule 12 (H13-47): MIN dummies optional, MOD one full
///   dummy for L < ~3 µm, EXC the outer dummy edge ≥ 10 µm from the gates.
/// - LOD moat, §13.2.2 (H13-29) "two or three microns" at MIN; §13.3 rule 12
///   MOD STI ≥ 5 µm, EXC ≥ 10 µm.
/// - well edge to the matched gates, §13.3 rule 19 (H13-53): EXC the lower
///   bound of 5–10 µm.
/// - gate poly past the rule, §13.3 rule 21 (H13-54): 1–2 µm at MOD and EXC,
///   the lower end taken.
const DUMMY_REACH_NM: [i32; 3] = [0, 3000, 10000];
const LOD_MOAT_EXT_NM: [i32; 3] = [3000, 5000, 10000];
const WPE_CLEARANCE_NM: [i32; 3] = [2000, 3000, 5000];
const GATE_EXT_EXTRA_NM: [i32; 3] = [0, 1000, 1000];

/// Hastings §8's resistor tiers, `[MIN, MOD, EXC]` (H08-13, H08-41): dummies
/// spanning ≥ 10 µm at EXC; width floors 150/200/400 % of the minimum; length
/// floors 3×/5×/10× the minimum.
const RES_DUMMY_SPAN_NM: [i32; 3] = [0, 0, 10000];
const RES_WIDTH_FLOOR_PERMILLE: [i32; 3] = [1500, 2000, 4000];
const RES_LENGTH_FLOOR_X: [i32; 3] = [3, 5, 10];

/// `c`'s MOS environment.
#[must_use]
pub fn mos_env(c: MatchClass) -> MosEnv {
    let i = c as usize;
    MosEnv {
        dummy_reach_nm: DUMMY_REACH_NM[i],
        moat_nm: LOD_MOAT_EXT_NM[i],
        wpe_nm: WPE_CLEARANCE_NM[i],
        gate_ext_extra_nm: GATE_EXT_EXTRA_NM[i],
        gate_strap: match c {
            MatchClass::Minimal => GateStrap::PolyBar,
            MatchClass::Moderate => GateStrap::PolyBarFar,
            MatchClass::Exceptional => GateStrap::MetalIsolated,
        },
    }
}

/// What a matched resistor array of one class asks (Hastings §8, H08-13,
/// H08-41).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PassiveEnv {
    /// Dummies per array end: one on every class; EXC also spans
    /// `dummy_span_nm`, so its count is `ceil(span / pitch)` (the caller's).
    pub min_dummies: u8,
    pub dummy_span_nm: i32,
    /// Segment width floor, ‰ of the deck's minimum width.
    pub width_floor_permille: i32,
    /// Segment length floor, multiples of the deck's minimum length.
    pub length_floor_x: i32,
}

/// `c`'s resistor environment.
#[must_use]
pub fn resistor_env(c: MatchClass) -> PassiveEnv {
    let i = c as usize;
    PassiveEnv {
        min_dummies: 1,
        dummy_span_nm: RES_DUMMY_SPAN_NM[i],
        width_floor_permille: RES_WIDTH_FLOOR_PERMILLE[i],
        length_floor_x: RES_LENGTH_FLOOR_X[i],
    }
}

/// Whether a matched pair's Φ (common-orientation) check is armed: hard at
/// EXC, a budget at MOD, off at MIN (Hastings §13.3, hastings.txt
/// L42155–42186: orientation matters at the millivolt level).
#[must_use]
pub fn phi_arm(c: MatchClass) -> Option<bool> {
    match c {
        MatchClass::Exceptional => Some(true),
        MatchClass::Moderate => Some(false),
        MatchClass::Minimal => None,
    }
}

/// A class limit: an offset in mV or a mismatch in %, both 6σ.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClassLimit {
    Mv(f32),
    Pct(f32),
}

/// Hastings' class limit for `(f, k)` at `c` (C2): MOS §13.3 (hastings.txt
/// L42327–42350), bipolar §9 (L31244–31275), R/C ratio §8 (L25121–25133,
/// the tight end of each range). `None` where the book gives none (a diode
/// has ideality limits, not offset limits) or the pair is meaningless.
#[must_use]
pub fn limit(f: Family, k: MatchKind, c: MatchClass) -> Option<ClassLimit> {
    use ClassLimit::{Mv, Pct};
    let i = c as usize;
    Some(match (f, k) {
        (Family::Mos, MatchKind::Voltage) => Mv([10.0, 3.0, 1.0][i]),
        (Family::Mos, MatchKind::Current) => Pct([10.0, 3.0, 1.0][i]),
        (Family::Bipolar, MatchKind::Voltage) => Mv([2.0, 0.5, 0.1][i]),
        (Family::Bipolar, MatchKind::Current) => Pct([8.0, 2.0, 0.5][i]),
        (Family::Resistor | Family::Capacitor, MatchKind::Ratio) => Pct([1.0, 0.1, 0.01][i]),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClassLimit::{Mv, Pct};
    use MatchClass::{Exceptional as Exc, Minimal as Min, Moderate as Mod};

    #[test]
    fn tiers_are_hastings() {
        assert_eq!(mos_env(Mod), MosEnv { dummy_reach_nm: 3000, moat_nm: 5000, wpe_nm: 3000, gate_ext_extra_nm: 1000, gate_strap: GateStrap::PolyBarFar });
        assert_eq!(resistor_env(Exc), PassiveEnv { min_dummies: 1, dummy_span_nm: 10000, width_floor_permille: 4000, length_floor_x: 10 });
        assert_eq!(mos_env(Min).dummy_reach_nm, 0);
    }

    #[test]
    fn limits_match_hastings() {
        let cases = [
            (Family::Mos, MatchKind::Voltage, [Mv(10.0), Mv(3.0), Mv(1.0)]),
            (Family::Mos, MatchKind::Current, [Pct(10.0), Pct(3.0), Pct(1.0)]),
            (Family::Bipolar, MatchKind::Voltage, [Mv(2.0), Mv(0.5), Mv(0.1)]),
            (Family::Bipolar, MatchKind::Current, [Pct(8.0), Pct(2.0), Pct(0.5)]),
            (Family::Resistor, MatchKind::Ratio, [Pct(1.0), Pct(0.1), Pct(0.01)]),
            (Family::Capacitor, MatchKind::Ratio, [Pct(1.0), Pct(0.1), Pct(0.01)]),
        ];
        for (f, k, want) in cases {
            for (c, w) in [Min, Mod, Exc].into_iter().zip(want) {
                assert_eq!(limit(f, k, c), Some(w), "{f:?} {k:?} {c:?}");
            }
        }
        for c in [Min, Mod, Exc] {
            assert_eq!(limit(Family::Diode, MatchKind::Voltage, c), None);
            assert_eq!(limit(Family::Mos, MatchKind::Ratio, c), None);
        }
    }

    #[test]
    fn phi_arm_follows_hastings() {
        assert_eq!(phi_arm(Exc), Some(true));
        assert_eq!(phi_arm(Mod), Some(false));
        assert_eq!(phi_arm(Min), None);
    }

    #[test]
    fn family_of_kind() {
        assert_eq!(Family::of(DeviceKind::Nmos), Some(Family::Mos));
        assert_eq!(Family::of(DeviceKind::Pnp), Some(Family::Bipolar));
        assert_eq!(Family::of(DeviceKind::Inductor), None);
    }
}
