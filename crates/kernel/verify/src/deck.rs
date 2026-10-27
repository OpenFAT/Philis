//! GPurify's deck dump (`gpurify deck <deck> --format json`, format
//! `gpurify-deck/1`, Q61) → [`Deck`]: the deck's layers, rules,
//! connectivity, devices and PEX stack, and the answers of GPurify's own
//! deck queries. Philis parses no deck text (Q55): this is GPurify's one
//! parse, read with serde.
//!
//! Units as the dump writes them: lengths whole nm (`_nm`), areas whole nm²
//! (`_nm2`), every other number in the unit its key or `unit` field names.
//! `null` is "the deck states none", read as no rule, never as 0. A layer
//! id is an index into [`Deck::layers`].
//!
//! Contents:
//! - `pub const FORMAT: &str`
//! - `pub struct Deck`, `pub struct Layer`, `pub struct Derived`, `pub struct Rule`, `pub struct Param`, `pub struct Connectivity`, `pub struct Via`, `pub struct Label`, `pub enum DeviceKind`, `pub struct Device`, `pub struct Pex`, `pub struct PexRow`, `pub struct Answers`, `pub struct LayerAnswers`, `pub struct Spacing`, `pub struct Enclosure`, `pub struct ModelAnswers`
//! - `pub fn parse(json: &str) -> Result<Deck, String>`
//! - `Deck::{layer_id, op, operands, enclosure, polarity, model}`, `Rule::{param, nm, number, layer_param}`

use serde::Deserialize;

/// The dump format this reader pins; another version is refused.
pub const FORMAT: &str = "gpurify-deck/1";

/// One deck, as GPurify compiled it.
#[derive(Clone, Debug, Deserialize)]
pub struct Deck {
    pub format: String,
    /// One per layer id, in id order: drawn layers first, then derived ones.
    pub layers: Vec<Layer>,
    pub rules: Vec<Rule>,
    pub connectivity: Connectivity,
    pub devices: Vec<Device>,
    pub pex: Pex,
    pub answers: Answers,
}

/// A drawn layer (`gds`) or a derived one (`derived`).
#[derive(Clone, Debug, Deserialize)]
pub struct Layer {
    pub id: u16,
    pub name: String,
    /// `(layer, datatype)` of a drawn layer.
    pub gds: Option<(u16, u16)>,
    pub derived: Option<Derived>,
}

/// How a derived layer is computed.
#[derive(Clone, Debug, Deserialize)]
pub struct Derived {
    /// As the deck spells it: `and`, `not`, `sized`, `vertical`, ...
    pub op: String,
    /// The layer ids it reads, receiver first.
    pub operands: Vec<u16>,
    /// `sized`'s growth, nm.
    pub dx_nm: Option<i64>,
    pub dy_nm: Option<i64>,
}

/// One rule.
#[derive(Clone, Debug, Deserialize)]
pub struct Rule {
    pub id: String,
    /// The engine kind (`min_width`, `noncoincident_enclosure`, ...).
    pub kind: String,
    /// `error` or `warning`.
    pub severity: String,
    /// Layer ids in the order the kind takes them (an enclosure: outer, inner).
    pub layers: Vec<u16>,
    pub params: Vec<Param>,
}

/// One rule parameter: a number with its unit, a flag, a layer or a model.
#[derive(Clone, Debug, Deserialize)]
pub struct Param {
    pub name: String,
    /// A number (with `unit`) or a boolean flag; `null` for a list's `none`.
    pub value: Option<serde_json::Value>,
    pub unit: Option<String>,
    pub layer: Option<u16>,
    pub model: Option<String>,
}

/// How conductors join.
#[derive(Clone, Debug, Deserialize)]
pub struct Connectivity {
    pub conductors: Vec<u16>,
    pub vias: Vec<Via>,
    pub global: Vec<u16>,
    pub labels: Vec<Label>,
}

/// A cut and the two conductors it joins.
#[derive(Clone, Debug, Deserialize)]
pub struct Via {
    pub cut: u16,
    pub connects: (u16, u16),
}

/// A text layer and the conductor whose net it names.
#[derive(Clone, Debug, Deserialize)]
pub struct Label {
    pub layer: u16,
    pub names: u16,
}

/// A device recogniser's family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceKind {
    Mos,
    Bjt,
    Resistor,
    Capacitor,
    Diode,
    Inductor,
}

/// One device recogniser.
#[derive(Clone, Debug, Deserialize)]
pub struct Device {
    pub kind: DeviceKind,
    pub model: String,
    pub marker: u16,
    /// MOS gate, source, drain [, bulk]; BJT base, emitter, collector [,
    /// bulk]; a two-pin kind pin 0, pin 1.
    pub terminals: Vec<u16>,
    /// The fin layer of a FinFET recogniser: it measures `nfin`, no `w`.
    pub fins: Option<u16>,
}

/// The parasitic stack.
#[derive(Clone, Debug, Deserialize)]
pub struct Pex {
    pub stack: Vec<PexRow>,
}

/// One layer's `pex` line.
#[derive(Clone, Debug, Deserialize)]
pub struct PexRow {
    pub layer: u16,
    pub thickness_nm: f64,
    pub height_nm: f64,
    pub sheet_ohm_sq: f64,
    pub dielectric_k: Option<f64>,
    pub area_cap_af_um2: f64,
    pub fringe_cap_af_um: f64,
}

/// GPurify's own deck queries, answered for every layer, pair and model.
#[derive(Clone, Debug, Deserialize)]
pub struct Answers {
    /// One per layer id, in id order.
    pub layers: Vec<LayerAnswers>,
    pub enclosures: Vec<Enclosure>,
    pub models: Vec<ModelAnswers>,
}

/// One layer's answers.
#[derive(Clone, Debug, Deserialize)]
pub struct LayerAnswers {
    pub layer: u16,
    pub min_width_nm: Option<i64>,
    pub min_area_nm2: Option<i64>,
    pub spacing_nm: Spacing,
}

/// `requiredSpacing` at GPurify's two documented queries.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Spacing {
    /// Two minimum-width shapes with no parallel run.
    pub min_width: Option<i64>,
    /// The strictest case the deck states (wide metal, long runs).
    pub strictest: Option<i64>,
}

/// The enclosure rules on one (inner, outer) pair, nm.
#[derive(Clone, Debug, Deserialize)]
pub struct Enclosure {
    pub inner: u16,
    pub outer: u16,
    pub all_sides_nm: Option<i64>,
    pub opposite_nm: Option<i64>,
    pub full_width: bool,
}

/// One device model's ratings.
#[derive(Clone, Debug, Deserialize)]
pub struct ModelAnswers {
    pub model: String,
    pub gate_oxide_mv: Option<f64>,
    pub drain_source_mv: Option<f64>,
}

/// The dump in `json`.
///
/// # Errors
/// Not JSON of the schema, a `format` other than [`FORMAT`], or a layer or
/// answer row out of id order.
pub fn parse(json: &str) -> Result<Deck, String> {
    let deck: Deck = serde_json::from_str(json).map_err(|e| format!("deck dump: {e}"))?;
    if deck.format != FORMAT {
        return Err(format!("deck dump is {:?}, this reader takes {FORMAT:?}", deck.format));
    }
    if let Some(i) = (0..deck.layers.len()).find(|&i| usize::from(deck.layers[i].id) != i) {
        return Err(format!("deck dump: layer row {i} has id {}", deck.layers[i].id));
    }
    if deck.answers.layers.len() != deck.layers.len() || deck.answers.layers.iter().enumerate().any(|(i, a)| usize::from(a.layer) != i) {
        return Err("deck dump: answers.layers is not one row per layer id".into());
    }
    Ok(deck)
}

impl Deck {
    /// The id of the layer named `name`.
    #[must_use]
    pub fn layer_id(&self, name: &str) -> Option<u16> {
        self.layers.iter().find(|l| l.name == name).map(|l| l.id)
    }

    /// The op computing layer `x`; `None` for a drawn layer.
    ///
    /// # Panics
    /// `x` is not a layer id of this deck.
    #[must_use]
    pub fn op(&self, x: u16) -> Option<&str> {
        self.layers[usize::from(x)].derived.as_ref().map(|d| d.op.as_str())
    }

    /// The layers `x` is computed from, receiver first; empty for a drawn one.
    ///
    /// # Panics
    /// `x` is not a layer id of this deck.
    #[must_use]
    pub fn operands(&self, x: u16) -> &[u16] {
        self.layers[usize::from(x)].derived.as_ref().map_or(&[], |d| &d.operands)
    }

    /// The enclosure answers for `inner` inside `outer`; `None` when no
    /// enclosure rule names the pair.
    #[must_use]
    pub fn enclosure(&self, inner: u16, outer: u16) -> Option<&Enclosure> {
        self.answers.enclosures.iter().find(|e| e.inner == inner && e.outer == outer)
    }

    /// `d`'s polarity as its marker layer's name states it: a leading `p`
    /// or `P` is P-type (`pgate`, asap7's `PMOS_RVT`), `n`/`N` N-type,
    /// anything else neither.
    #[must_use]
    pub fn polarity(&self, d: &Device) -> Option<bool> {
        match self.layers[usize::from(d.marker)].name.as_bytes().first()?.to_ascii_lowercase() {
            b'p' => Some(true),
            b'n' => Some(false),
            _ => None,
        }
    }

    /// The recogniser row for deck model `model` (the first of several).
    #[must_use]
    pub fn model(&self, model: &str) -> Option<&Device> {
        self.devices.iter().find(|d| d.model == model)
    }
}

impl Rule {
    /// The first parameter named `name`.
    #[must_use]
    pub fn param(&self, name: &str) -> Option<&Param> {
        self.params.iter().find(|p| p.name == name)
    }

    /// Parameter `name` as a length, nm; `None` when absent or not a length.
    #[must_use]
    pub fn nm(&self, name: &str) -> Option<i64> {
        let p = self.param(name)?;
        (p.unit.as_deref() == Some("nm")).then(|| p.value.as_ref()?.as_i64())?
    }

    /// Every value of the length list `name`, nm, in order.
    pub fn nm_list<'a>(&'a self, name: &'a str) -> impl Iterator<Item = i64> + 'a {
        self.params.iter().filter(move |p| p.name == name && p.unit.as_deref() == Some("nm")).filter_map(|p| p.value.as_ref()?.as_i64())
    }

    /// Parameter `name` as a number in its own unit; `None` when absent or
    /// not a number.
    #[must_use]
    pub fn number(&self, name: &str) -> Option<f64> {
        self.param(name)?.value.as_ref()?.as_f64()
    }

    /// Parameter `name` as a layer id.
    #[must_use]
    pub fn layer_param(&self, name: &str) -> Option<u16> {
        self.param(name)?.layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: the dump contract (GPurify `src/gpurify/export/deck.zig`):
    /// another format is refused, a rule's length param reads in nm, a
    /// `null` reads as no rule.
    #[test]
    fn parse_pins_the_format_and_reads_lengths() {
        let doc = |format: &str| {
            format!(
                r#"{{"format":"{format}","deck":"d","grid_nm":null,"layers":[{{"id":0,"name":"m1","gds":[68,20]}}],
                "rules":[{{"id":"m1.1","kind":"min_width","keyword":"width","domain":"drc","severity":"error","layers":[0],"params":[{{"name":"limit","value":140,"unit":"nm"}}]}}],
                "connectivity":{{"conductors":[0],"vias":[],"touch_within_layer":true,"global":[],"labels":[]}},"devices":[],"matching":[],"lvs":{{}},
                "pex":{{"stack":[]}},"answers":{{"layers":[{{"layer":0,"min_width_nm":140,"min_area_nm2":null,"spacing_nm":{{"min_width":null,"strictest":null}}}}],
                "enclosures":[],"projection_spacing":[],"contains":[],"minimum_cuts":[],"models":[]}}}}"#
            )
        };
        assert!(parse(&doc("gpurify-deck/2")).is_err());
        let d = parse(&doc(FORMAT)).unwrap();
        assert_eq!(d.rules[0].nm("limit"), Some(140));
        assert_eq!(d.answers.layers[0].min_area_nm2, None);
        assert_eq!(d.layers[0].gds, Some((68, 20)));
    }
}
