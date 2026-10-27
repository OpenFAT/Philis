//! Extraction's contract (EXT-12): batch identity and provenance (EXT-10),
//! and everything the annotator infers about a circuit (matched sets,
//! symmetry compounds, the group tree, net and device facts), independent of
//! how placement turns it into rules. Reached by path only: `verify::Intent`
//! shares the name.
//!
//! In → out: none; the annotator writes [`Intent`], placement and the recipes
//! read it. Cites: EXT-06, EXT-10, EXT-12–EXT-15, EXT-19–EXT-21, EXT-26–EXT-28,
//! MAT-04, MAT-08, GAP-03, PLC-25; the analog layout synthesis survey §3.2.1
//! (requirement order).
//!
//! Contents:
//! - `pub struct ConstraintId(pub u32)`
//! - `pub enum Origin`
//! - `pub struct BatchMeta`
//! - `pub enum ClassSource`
//! - `pub enum Half`
//! - `pub enum Term`
//! - `pub enum ArrayStyle`
//! - `pub struct Member`
//! - `pub struct UnitGeom`
//! - `pub struct MatchSpec`
//! - `pub enum SymKind`
//! - `pub struct Compound`
//! - `pub enum ReqType`
//! - `pub enum GroupKind`
//! - `pub struct GroupNode`
//! - `pub struct Order`
//! - `pub enum Region`
//! - `pub enum DeviceRole`
//! - `pub struct DeviceFacts`
//! - `pub enum RcClass`
//! - `pub enum EvidenceLevel`
//! - `pub struct NetFacts`
//! - `pub enum Inject`
//! - `pub struct Aggressor`
//! - `pub struct Victim`
//! - `pub struct CommonNodeReq`
//! - `pub struct StarReq`
//! - `pub struct KelvinReq`
//! - `pub struct Diagnostic`
//! - `pub struct Intent`

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
        /// The recognised template's name.
        template: &'static str,
    },
    /// Emitted from net classes, the substrate, or a cross-block rule.
    NetClass,
    /// Propagated from a symmetry seed (EXT-14).
    Symmetry {
        /// The seed constraint it was propagated from.
        seed: ConstraintId,
    },
    /// A shared-bias group (EXT-15).
    SharedBias,
    /// A passive or bipolar set rule (EXT-19), e.g. `"divider"`, `"dac_bank"`.
    PassiveSet {
        /// The set rule's name.
        rule: &'static str,
    },
    /// Emitted from spec sensitivities.
    Sensitivity,
    /// Index into the user sidecar (EXT-26).
    User {
        /// Row of the sidecar entry.
        index: u32,
    },
    /// Derived from another constraint.
    Derived {
        /// The constraint it was derived from.
        parent: ConstraintId,
    },
}

/// A batch's identity and provenance (EXT-10), carried by `rule::Tagged`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BatchMeta {
    /// Dense id; `gp::Prices` re-finds a batch by it.
    pub id: ConstraintId,
    /// Where the batch came from.
    pub origin: Origin,
}

/// User and Spec count as explicit for MAT-08 (`MatchedSet::class_explicit = source != Role`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassSource {
    /// The user's sidecar named the class.
    User,
    /// Derived from a spec (EXT-16).
    Spec,
    /// The role default (Moderate); not explicit.
    Role,
}

/// Which side of a symmetric set a member sits on.
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

/// How a matched set's units are arranged; `Any` leaves it to the generator.
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
    /// The schematic device.
    pub device: DeviceId,
    /// Units in parallel; ≥ 1.
    pub parallel: u16,
    /// Units in series; ≥ 1.
    pub series: u16,
    /// Its side of a symmetric set; `None` outside one.
    pub half: Option<Half>,
}

/// The unit every member is built from, nm.
#[derive(Clone, Copy, Debug)]
pub struct UnitGeom {
    /// Unit width, nm.
    pub w_nm: i32,
    /// Unit length, nm.
    pub l_nm: i32,
    /// The device model, as the annotator interned it.
    pub model: u16,
}

/// What extraction knows about one matched set. The rule that scores it is MAT-04's
/// `analog::placement::matched_set::MatchedSet`, built from this by EXT-20 (different type, different module).
#[derive(Clone, Debug)]
pub struct MatchSpec {
    /// Dense id within the problem.
    pub id: ConstraintId,
    /// Where the set came from.
    pub origin: Origin,
    /// At least two.
    pub members: Vec<Member>,
    /// A mirror's diode reference or a ratio bank's unit member (index into `members`); MAT-04 puts it in slot 0.
    pub reference: Option<usize>,
    /// The members' device family.
    pub family: Family,
    /// Voltage, current or ratio matching.
    pub kind: MatchKind,
    /// The precision tier.
    pub class: MatchClass,
    /// Whether `class` was given (User, Spec) or defaulted (Role).
    pub class_source: ClassSource,
    /// `None`: the ratio is not an integer multiple of one unit (CELL/MAT handle it).
    pub unit: Option<UnitGeom>,
    /// 1σ systematic allowance for the whole set (mV for Voltage, % for Current/Ratio), from EXT-21. `None` = not allocated.
    pub allowance: Option<f32>,
    /// Share of the spec variance this set explains, 0..=1; `None` without sensitivities.
    pub weight: Option<f32>,
    /// The array arrangement asked for.
    pub style: ArrayStyle,
    /// Index into `Intent.compounds` of the compound holding a member.
    pub compound: Option<u16>,
}

/// How a compound's partners are drawn (as `placement::SymMode`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SymKind {
    /// Partner reflected about the axis.
    Mirror,
    /// Partner drawn with the same orient.
    Perfect,
}

/// Devices symmetric about one axis (EXT-14).
#[derive(Clone, Debug)]
pub struct Compound {
    /// Dense id within the problem.
    pub id: ConstraintId,
    /// The axis slot every pair shares.
    pub axis: AxisId,
    /// Vertical or horizontal axis.
    pub dir: AxisDir,
    /// How partners are drawn.
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

/// What binds a group node of the HSMPG tree.
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
    /// What binds the node.
    pub kind: GroupKind,
    /// Direct device members only.
    pub devices: Vec<DeviceId>,
    /// Indices of child nodes in `Intent.tree`.
    pub children: Vec<u32>,
}

/// A placement order (EXT-28, EXT-27 arrays, sidecar `Order`). `steps[0]` sits at the low
/// coordinate of `dir` (bottom for `V`, left for `H`); each step is a set of devices
/// (PLC-25 takes the bbox of a step's devices).
#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    /// Device sets, low coordinate first.
    pub steps: Vec<Vec<DeviceId>>,
    /// The axis the order runs along.
    pub dir: AxisDir,
    /// The sense along `dir` is placement's choice (extracted orders); `false` for a user order.
    pub reversible: bool,
    /// Path current / the largest extracted path current, 0..=1; 1.0 without an op point.
    pub weight: f32,
}

/// A MOS device's operating region at the op point; `Unknown` without one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Region {
    Unknown,
    Off,
    Subthreshold,
    Triode,
    Saturation,
}

/// A device's circuit role, as the annotator recognised it.
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

/// What the annotator knows about one device, indexed by `DeviceId`.
#[derive(Clone, Copy, Debug)]
pub struct DeviceFacts {
    /// Operating region at the op point.
    pub region: Region,
    /// Recognised role.
    pub role: DeviceRole,
}

/// Which passives load a net: none, resistors, capacitors or both.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RcClass {
    Unknown,
    None,
    R,
    C,
    Rc,
}

/// What decided a net's class, for the report.
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

/// What the annotator knows about one net, indexed by `NetId`.
#[derive(Clone, Debug)]
pub struct NetFacts {
    /// What decided the net's class.
    pub evidence: EvidenceLevel,
    /// A port of the top sub-circuit.
    pub port: bool,
    /// DC voltage at the op point, mV; both entries equal from one op point.
    pub dc_mv: Option<(i32, i32)>,
    /// Which passives load it.
    pub rc: RcClass,
    /// Small-signal impedance to AC ground, Ω; `None` unless every FET on it has a known gds.
    pub z_ohm: Option<f32>,
    /// The quiet net a shield for it ties to; `None` = not a shielded victim.
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

/// A device that injects into the substrate.
#[derive(Clone, Debug)]
pub struct Aggressor {
    /// The injecting device.
    pub device: DeviceId,
    /// How it injects.
    pub inject: Inject,
    /// Why it was marked, for the report.
    pub reason: &'static str,
}

/// A device sensitive to substrate noise.
#[derive(Clone, Debug)]
pub struct Victim {
    /// The sensitive device.
    pub device: DeviceId,
    /// Relative sensitivity; larger is more sensitive.
    pub weight: f32,
    /// Why it was marked, for the report.
    pub reason: &'static str,
}

/// A matched set's shared source (or emitter) net, whose routed R to each side
/// must balance (`routing::CommonNode`).
#[derive(Clone, Debug)]
pub struct CommonNodeReq {
    /// The shared net.
    pub net: NetId,
    /// Index into `Intent.sets`.
    pub set: u16,
    /// One side's devices.
    pub a: Vec<DeviceId>,
    /// The other side's devices.
    pub b: Vec<DeviceId>,
    /// The terminal on the shared net (S, or E for a BJT).
    pub term: Term,
}

/// A net routed as a star: branches meet only at the root.
#[derive(Clone, Debug)]
pub struct StarReq {
    /// The star net.
    pub net: NetId,
    /// The feed pin; `None` = the net's centre.
    pub root: Option<(DeviceId, Term)>,
    /// One pin per branch.
    pub branches: Vec<(DeviceId, Term)>,
}

/// A Kelvin connection: the sense leads carry no force current.
#[derive(Clone, Debug)]
pub struct KelvinReq {
    /// The device whose terminal is force-connected.
    pub device: DeviceId,
    /// That terminal.
    pub term: Term,
    /// The pins that sense it.
    pub sense: Vec<(DeviceId, Term)>,
}

/// A finding extraction reports instead of guessing, e.g. `"ambiguous_symmetry"`.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// Stable code, e.g. `"ambiguous_symmetry"`.
    pub kind: &'static str,
    /// The devices it concerns.
    pub devices: Vec<DeviceId>,
    /// Human-readable detail.
    pub message: String,
}

/// Everything the annotator infers about one circuit; every table is in
/// emission order, so it is permutation-invariant (EXT-06).
#[derive(Clone, Debug, Default)]
pub struct Intent {
    /// Matched sets.
    pub sets: Vec<MatchSpec>,
    /// Symmetry compounds.
    pub compounds: Vec<Compound>,
    /// HSMPG tree, `Root` last (EXT-13).
    pub tree: Vec<GroupNode>,
    /// User orders, current paths, signal stages, then instance arrays (EXT-27/28).
    pub order: Vec<Order>,
    /// Indexed by `NetId`.
    pub nets: Vec<NetFacts>,
    /// Indexed by `DeviceId`.
    pub devices: Vec<DeviceFacts>,
    /// Substrate injectors.
    pub aggressors: Vec<Aggressor>,
    /// Substrate-sensitive devices.
    pub victims: Vec<Victim>,
    /// Shared nodes to balance.
    pub common_nodes: Vec<CommonNodeReq>,
    /// Star-routed nets.
    pub stars: Vec<StarReq>,
    /// Kelvin connections.
    pub kelvins: Vec<KelvinReq>,
    /// Findings reported instead of guessed.
    pub diagnostics: Vec<Diagnostic>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: spec example, Hastings' tier order (MIN < MOD < EXC).
    #[test]
    fn match_class_orders() {
        assert!(MatchClass::Minimal < MatchClass::Moderate);
        assert!(MatchClass::Moderate < MatchClass::Exceptional);
        assert_eq!(MatchClass::default(), MatchClass::Moderate);
    }

    /// Oracle: spec example, the survey's §3.2.1 importance order.
    #[test]
    fn req_type_importance() {
        assert!(ReqType::MatchSym < ReqType::MatchBlock);
        assert!(ReqType::MatchBlock < ReqType::ProxBlock);
        assert!(ReqType::ProxBlock < ReqType::Sym);
        assert!(ReqType::Sym < ReqType::ProxNet);
    }
}
