//! What a terminal is, by its name and its device's kind (AA-16): never by position.
//!
//! Contents:
//! - `pub enum TermRole`
//! - `pub fn term_role(kind: DeviceKind, term: &str) -> TermRole`

use analog::base::DeviceKind;

/// A terminal's electrical role, which decides how its net is classified.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TermRole {
    /// A FET gate: capacitive, no DC path.
    FetGate,
    /// A FET drain or source, or a BJT collector or emitter: a DC path.
    Channel,
    /// A FET bulk.
    Body,
    /// A BJT base: a DC path, but not a gate.
    BjtBase,
    /// A capacitor plate: capacitive, no DC path.
    Plate,
    /// A resistor, diode, inductor or unknown pin: a DC path.
    Passive,
}

/// FET `G`/`D`,`S`/`B`; bipolar `B` = base, `C`/`E` = channel; capacitor pins are plates;
/// anything else (resistor, diode, inductor, unknown pin names) is a passive DC path.
#[must_use]
pub fn term_role(kind: DeviceKind, term: &str) -> TermRole {
    use DeviceKind::{Capacitor, Nmos, Npn, Pmos, Pnp};
    match (kind, term) {
        (Nmos | Pmos, "G") => TermRole::FetGate,
        (Nmos | Pmos, "D" | "S") => TermRole::Channel,
        (Nmos | Pmos, "B") => TermRole::Body,
        (Npn | Pnp, "B") => TermRole::BjtBase,
        (Npn | Pnp, _) => TermRole::Channel,
        (Capacitor, _) => TermRole::Plate,
        _ => TermRole::Passive,
    }
}
