//! Extraction's contract (EXT-12): batch identity and provenance (EXT-10),
//! and everything the annotator infers about a circuit (matched sets,
//! symmetry compounds, the group tree, net and device facts), independent of
//! how placement turns it into rules. Reached by path only: `verify::Intent`
//! shares the name.

use crate::base::ids::{AxisId, DeviceId, NetId};
pub use crate::matching::class::{Family, MatchClass, MatchKind};
pub use crate::placement::symmetry::AxisDir;

/// Dense id within one `annotator::Problem`, assigned in emission order:
/// permutation-invariant because emission order is (EXT-06).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ConstraintId(pub u32);

/// Where a batch or an inferred constraint came from. `Copy` because
/// `rule::Tagged` copies its [`BatchMeta`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Origin {
    /// Emitted for a recognised block of this template.
    Pattern {
        template: &'static str,
    },
    /// Emitted from net classes, the substrate, or a cross-block rule.
    NetClass,
    /// Propagated from a symmetry seed (EXT-14).
    Symmetry {
        seed: ConstraintId,
    },
    /// A shared-bias group (EXT-15).
    SharedBias,
    /// A passive or bipolar set rule (EXT-19), e.g. `"divider"`, `"dac_bank"`.
    PassiveSet {
        rule: &'static str,
    },
    Sensitivity,
    /// Index into the user sidecar (EXT-26).
    User {
        index: u32,
    },
    Derived {
        parent: ConstraintId,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BatchMeta {
    pub id: ConstraintId,
    pub origin: Origin,
}

/// User and Spec count as explicit for MAT-08 (`MatchedSet::class_explicit = source != Role`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassSource {
    User,
    Spec,
    Role,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Half {
    A,
    B,
}

/// Terminal names, the pin-table columns of `annotator::pattern`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Term {
    G,
    D,
    S,
    B,
    C,
    E,
    P,
    N,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArrayStyle {
    Adjacent,
    Interdigitated,
    CommonCentroid1d,
    CommonCentroid2d,
    Any,
}

/// One device of a matched set, in units of the set's [`UnitGeom`].
#[derive(Clone, Copy, Debug)]
pub struct Member {
    pub device: DeviceId,
    pub parallel: u16,
    pub series: u16,
    pub half: Option<Half>,
}

/// The unit every member is built from, nm.
#[derive(Clone, Copy, Debug)]
pub struct UnitGeom {
    pub w_nm: i32,
    pub l_nm: i32,
    pub model: u16,
}

/// What extraction knows about one matched set. The rule that scores it is MAT-04's
/// `analog::placement::matched_set::MatchedSet`, built from this by EXT-20 (different type, different module).
#[derive(Clone, Debug)]
pub struct MatchSpec {
    pub id: ConstraintId,
    pub origin: Origin,
    pub members: Vec<Member>,
    /// A mirror's diode reference or a ratio bank's unit member (index into `members`); MAT-04 puts it in slot 0.
    pub reference: Option<usize>,
    pub family: Family,
    pub kind: MatchKind,
    pub class: MatchClass,
    pub class_source: ClassSource,
    /// `None`: the ratio is not an integer multiple of one unit (CELL/MAT handle it).
    pub unit: Option<UnitGeom>,
    /// 1σ systematic allowance for the whole set (mV for Voltage, % for Current/Ratio), from EXT-21. `None` = not allocated.
    pub allowance: Option<f32>,
    /// Share of the spec variance this set explains, 0..=1; `None` without sensitivities.
    pub weight: Option<f32>,
    pub style: ArrayStyle,
    /// Index into `Intent.compounds` of the compound holding a member.
    pub compound: Option<u16>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SymKind {
    Mirror,
    Perfect,
}

/// Devices symmetric about one axis (EXT-14).
#[derive(Clone, Debug)]
pub struct Compound {
    pub id: ConstraintId,
    pub axis: AxisId,
    pub dir: AxisDir,
    pub kind: SymKind,
    /// Mirror couples, half A first.
    pub pairs: Vec<(DeviceId, DeviceId)>,
    /// Devices on the axis.
    pub selfs: Vec<DeviceId>,
    /// Mirrored nets, half A first.
    pub net_pairs: Vec<(NetId, NetId)>,
    /// Self-symmetric non-rail nets.
    pub self_nets: Vec<NetId>,
    /// Indices into `Intent.sets` of spec pairs that mirror each other (nested symmetry).
    pub set_pairs: Vec<(u16, u16)>,
}

/// Requirement types in importance order (survey §3.2.1): `MatchSym < MatchBlock < ProxBlock < Sym < ProxNet`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ReqType {
    MatchSym,
    MatchBlock,
    ProxBlock,
    Sym,
    ProxNet,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupKind {
    Matching,
    Symmetry,
    Proximity,
    Root,
}

/// One node of the HSMPG tree (EXT-13). `devices` lists only the node's
/// direct device members; `children` index group nodes in `Intent.tree`.
#[derive(Clone, Debug)]
pub struct GroupNode {
    pub kind: GroupKind,
    pub devices: Vec<DeviceId>,
    pub children: Vec<u32>,
}

/// A placement order (EXT-28, EXT-27 arrays, sidecar `Order`). `steps[0]` sits at the low
/// coordinate of `dir` (bottom for `V`, left for `H`); each step is a set of devices
/// (PLC-25 takes the bbox of a step's devices).
#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    pub steps: Vec<Vec<DeviceId>>,
    pub dir: AxisDir,
    /// The sense along `dir` is placement's choice (extracted orders); `false` for a user order.
    pub reversible: bool,
    /// Path current / the largest extracted path current, 0..=1; 1.0 without an op point.
    pub weight: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Region {
    Unknown,
    Off,
    Subthreshold,
    Triode,
    Saturation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeviceRole {
    Unknown,
    Amplifier,
    CurrentSource,
    Cascode,
    Load,
    Switch,
    Diode,
    Passive,
}

#[derive(Clone, Copy, Debug)]
pub struct DeviceFacts {
    pub region: Region,
    pub role: DeviceRole,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RcClass {
    Unknown,
    None,
    R,
    C,
    Rc,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvidenceLevel {
    User,
    Port,
    Name,
    Testbench,
    Structure,
    OpPoint,
    Default,
}

#[derive(Clone, Debug)]
pub struct NetFacts {
    pub evidence: EvidenceLevel,
    pub port: bool,
    pub dc_mv: Option<(i32, i32)>,
    pub rc: RcClass,
    pub z_ohm: Option<f32>,
    pub shield_ref: Option<NetId>,
}

/// How an aggressor injects into the substrate (GAP-03 adds the minority kinds, C4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Inject {
    Switching,
    Capacitive,
    /// n+ diffusion in p-sub on a pin (forward-biased below ground).
    MinorityElectron,
    /// p+ in n-well (above supply).
    MinorityHole,
}

#[derive(Clone, Debug)]
pub struct Aggressor {
    pub device: DeviceId,
    pub inject: Inject,
    pub reason: &'static str,
}

#[derive(Clone, Debug)]
pub struct Victim {
    pub device: DeviceId,
    pub weight: f32,
    pub reason: &'static str,
}

#[derive(Clone, Debug)]
pub struct CommonNodeReq {
    pub net: NetId,
    pub set: u16,
    pub a: Vec<DeviceId>,
    pub b: Vec<DeviceId>,
    pub term: Term,
}

#[derive(Clone, Debug)]
pub struct StarReq {
    pub net: NetId,
    pub root: Option<(DeviceId, Term)>,
    pub branches: Vec<(DeviceId, Term)>,
}

#[derive(Clone, Debug)]
pub struct KelvinReq {
    pub device: DeviceId,
    pub term: Term,
    pub sense: Vec<(DeviceId, Term)>,
}

/// A finding extraction reports instead of guessing, e.g. `"ambiguous_symmetry"`.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub kind: &'static str,
    pub devices: Vec<DeviceId>,
    pub message: String,
}

/// A box as fractions of a reference outline, from its lower-left corner:
/// `x`, `y` ≥ 0, `w`, `h` > 0, `x + w` and `y + h` at most 1 (the sidecar
/// reader admits nothing else).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// One block of a [`Reference`]: the devices a sidecar name resolves to (a
/// `GroupBlocks` alias or an instance) and where the prior design had them.
#[derive(Clone, Debug, PartialEq)]
pub struct RefBlock {
    /// The name as the sidecar writes it; the report quotes it.
    pub name: String,
    pub devices: Vec<DeviceId>,
    /// 0 is the bottom row.
    pub row: u16,
    pub extent: Extent,
    /// The block's unit (a capacitor unit, a mirror unit) over the
    /// reference's, > 0; `None` when the entry gives none.
    pub unit_area: Option<f32>,
}

/// A prior design's floorplan, the sidecar's `Reference` entry
/// (UW-ASIC/Philis#78, Q61): its row order, extents and aspect are the
/// floorplan's targets in place of the stage order (Q21, Q22). `blocks` run
/// row by row, bottom first, left to right within a row; their extents
/// stack in that order and never overlap, and no device is in two blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    /// Index of the sidecar entry, as [`Origin::User`] counts it.
    pub entry: u32,
    /// The prior design, free text the report quotes.
    pub source: String,
    /// Outline width over height, > 0.
    pub aspect: f32,
    pub blocks: Vec<RefBlock>,
    /// Target block area over the reference's after scaling by the ratio
    /// of total device area, > 0; `None` when the entry gives none.
    pub area_ratio: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct Intent {
    pub sets: Vec<MatchSpec>,
    pub compounds: Vec<Compound>,
    /// HSMPG tree, `Root` last (EXT-13).
    pub tree: Vec<GroupNode>,
    /// User orders, current paths, signal stages, then instance arrays (EXT-27/28).
    pub order: Vec<Order>,
    pub nets: Vec<NetFacts>,
    pub devices: Vec<DeviceFacts>,
    pub aggressors: Vec<Aggressor>,
    pub victims: Vec<Victim>,
    pub common_nodes: Vec<CommonNodeReq>,
    pub stars: Vec<StarReq>,
    pub kelvins: Vec<KelvinReq>,
    pub diagnostics: Vec<Diagnostic>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_class_orders() {
        assert!(MatchClass::Minimal < MatchClass::Moderate && MatchClass::Moderate < MatchClass::Exceptional);
        assert_eq!(MatchClass::default(), MatchClass::Moderate);
    }

    #[test]
    fn req_type_importance() {
        assert!(
            ReqType::MatchSym < ReqType::MatchBlock
                && ReqType::MatchBlock < ReqType::ProxBlock
                && ReqType::ProxBlock < ReqType::Sym
                && ReqType::Sym < ReqType::ProxNet
        );
    }
}
