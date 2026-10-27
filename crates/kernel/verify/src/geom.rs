//! The pin labels sign-off draws and the deck's label layers: what names an
//! extracted net (`connect label` rows of the deck).
//!
//! Contents:
//! - `pub struct LabeledPin`
//! - `pub fn label_layer(deck: &Deck, drawn: u16) -> Option<u16>`

use crate::deck::Deck;

/// A named pin label. The point `(x, y)` (nm) must lie on a shape of `layer`,
/// the pin's drawing layer; the label goes on whichever layer the deck's
/// `connectivity.labels` binds to the conductor built from it ([`label_layer`]),
/// and that binding names the extracted net (and makes it a port).
#[derive(Clone, Debug)]
pub struct LabeledPin {
    pub name: String,
    /// Deck layer id (`analog::base::LayerId.0`).
    pub layer: u16,
    pub x: i32,
    pub y: i32,
}

/// The label layer that names the conductor drawn on `drawn`: a label row
/// whose conductor is `drawn` or is derived from it (sky130 `li` names
/// `li_c = li not li_rs`; ihp labels only through `metal1_label`). `drawn`
/// itself when it is such a label layer. `None` when no label reaches it.
#[must_use]
pub fn label_layer(deck: &Deck, drawn: u16) -> Option<u16> {
    let rows: Vec<u16> =
        deck.connectivity.labels.iter().filter(|l| l.names == drawn || deck.operands(l.names).contains(&drawn)).map(|l| l.layer).collect();
    rows.iter().find(|&&l| l == drawn).or_else(|| rows.first()).copied()
}
