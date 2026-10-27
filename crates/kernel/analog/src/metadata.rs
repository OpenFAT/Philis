//! Net classification tags the annotator turns into routing rules.
//! In → out: none; plain rows the annotator writes. Cites: EXT-18 (bias nets).
//!
//! Contents:
//! - `pub struct NetClassification`
//! - `pub enum NetClass`

use crate::base::ids::NetId;

/// A net's functional class and the routing budgets it implies, aF;
/// `None` = unbudgeted.
#[derive(Clone, Debug)]
pub struct NetClassification {
    /// The classified net.
    pub net: NetId,
    /// Its functional class.
    pub class: NetClass,
    /// Max wire capacitance to ground (lowered to a drawn-length cap).
    pub c_budget_af: Option<i64>,
    /// Max total coupling onto this net.
    pub max_coupling_af: Option<i64>,
}

/// A net's functional class: which routing rules it gets and whether it is a
/// victim, an aggressor or a quiet rail.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetClass {
    /// Ordinary analog signal.
    Signal,
    /// A clock: an aggressor.
    Clock,
    /// A supply rail: quiet, never an aggressor.
    Supply,
    /// A ground rail: quiet, never an aggressor.
    Ground,
    /// Sensitive reference (bias, bandgap, ADC reference).
    Sensitive,
    /// The substrate tie: quiet, never an aggressor.
    Substrate,
    /// Bias line: gates only (plus diode D=G), or the gate of a current source/cascode (EXT-18).
    Bias,
    /// Reference voltage: a bandgap core node, a cascoded reference, a DAC reference plate.
    Reference,
    /// Logic level that does not toggle with a clock (logic G/D nets).
    DigitalStatic,
    /// Logic driven by a clock or by other switching logic: an aggressor.
    DigitalSwitching,
    /// Noisy node (charge-pump output, `_n` suffix without a `_p` twin): an aggressor.
    Noisy,
}
