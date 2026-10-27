//! Drawn geometry → the circuit GPurify extracts from it, as SPICE text
//! ([`crate::gpurify::netlist`], `pex --format spice`). No schematic is
//! consulted.
//!
//! Contents:
//! - `pub enum Detail`
//! - `pub fn extract_spice(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk, detail: Detail) -> Result<String, String>`
//! - `pub fn device_count(shapes: &[Shape], pdk: &Pdk) -> Option<usize>`

use analog::base::Shape;

use crate::geom::LabeledPin;
use crate::pdk::Pdk;

/// Whether the netlist keeps PEX's parasitic R/C cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    /// Device cards only, every node folded to its net (`out#3` → `out`).
    Schematic,
    /// GPurify's netlist as written: devices on their nodes, then R and C.
    WithParasitics,
}

/// Extract `shapes` and write the recognised circuit as a SPICE `.subckt`.
///
/// `pins` name the nets that become subcircuit ports; an unlabelled net is
/// GPurify's `n<id>`. A device card is `M3 d g s b model …`, its family's
/// letter and layout index.
///
/// # Errors
/// What [`crate::gpurify::netlist`] refuses: `gpurify` missing, a label
/// short, a deck whose PEX GPurify refuses (asap7's middle of line).
pub fn extract_spice(shapes: &[Shape], pins: &[LabeledPin], pdk: &Pdk, detail: Detail) -> Result<String, String> {
    let text = crate::gpurify::netlist(shapes, pins, pdk)?;
    Ok(match detail {
        Detail::WithParasitics => text,
        Detail::Schematic => text.lines().filter(|l| !is_parasitic(l)).map(fold_nodes).collect::<Vec<_>>().join("\n") + "\n",
    })
}

/// How many devices GPurify extracts from `shapes`; `None` when it writes
/// no netlist — for the merge oracle, itself the ambiguity signal.
#[must_use]
pub fn device_count(shapes: &[Shape], pdk: &Pdk) -> Option<usize> {
    let text = extract_spice(shapes, &[], pdk, Detail::Schematic).ok()?;
    Some(text.lines().filter(|l| l.starts_with(|c: char| c.is_ascii_alphabetic())).count())
}

// ---- private ----

/// An R, C, L or K card of four fields (`R1 a a#1 2.5`): a parasitic, where
/// a device card carries its model.
fn is_parasitic(line: &str) -> bool {
    line.starts_with(['R', 'C', 'L', 'K']) && line.split_whitespace().count() == 4
}

/// `line` with every `net#k` node written `net`.
fn fold_nodes(line: &str) -> String {
    line.split(' ').map(|f| f.split('#').next().unwrap_or(f)).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: GPurify's documented `pex --format spice` card shapes
    /// (docs/usage.md, "A simulator-ready netlist"): device cards carry a
    /// model, a parasitic R or C card is four fields.
    #[test]
    fn schematic_keeps_device_cards_on_their_nets() {
        let text = "* pex\n.subckt top out inp\nM0 out#3 inp#1 tail#2 vss sky130_fd_pr__nfet_01v8 w=4.2 l=0.5\nR1 out out#1 12.4\nC1 out#1 0 3.1f\nR0 a b sky130_fd_pr__res_high_po w=0.35 l=2\n.ends top\n";
        let kept: Vec<String> = text.lines().filter(|l| !is_parasitic(l)).map(fold_nodes).collect();
        assert_eq!(kept[2], "M0 out inp tail vss sky130_fd_pr__nfet_01v8 w=4.2 l=0.5");
        assert_eq!(kept[3], "R0 a b sky130_fd_pr__res_high_po w=0.35 l=2");
        assert_eq!(kept.len(), 5);
    }
}
