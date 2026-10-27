//! The LVS reference netlist: plain [`RefInput`] (verify cannot depend on
//! `frontend/library`) → the SPICE text `gpurify lvs --reference` reads
//! (Q61), one `X` card per device calling its deck model.
//!
//! - **Terminals are in SPICE card order** — GPurify assigns roles by
//!   position: MOS `d g s [b]` (bulk only if the recogniser declares 4),
//!   BJT `c b e`, R/C/D `a b`.
//! - **Params are SI** (metres). A param named on one side only is an
//!   `lvs.undeclared_param`. The caller expands m/nf to one card per drawn
//!   finger.
//! - **Kinds the deck cannot recognise are skipped** (returned by index):
//!   the layout extracts none, so keeping them would mismatch unconditionally.
//!   The caller counts them as LVS-unverified ([`crate::Coverage`]), never as
//!   matched. A MOM capacitor (no deck recognises one: a comb draws each
//!   electrode as many polygons and `DeviceRecognition` binds one polygon per
//!   terminal), a BJT on a deck without one, and every inductor are the cases.
//!
//! Contents:
//! - `pub enum RefKind`, `pub struct RefDeviceIn`, `pub struct RefInput`
//! - `pub fn spice(input: &RefInput, pdk: &Pdk) -> Result<(String, Vec<usize>), String>`

use crate::deck::{Deck, DeviceKind};
use crate::pdk::Pdk;

/// A schematic device kind, polarity included (recognisers are per-polarity).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefKind {
    Nmos,
    Pmos,
    Npn,
    Pnp,
    Resistor,
    Capacitor,
    Diode,
    /// No deck recognises one: always skipped.
    Inductor,
}

/// One schematic device.
#[derive(Clone, Debug)]
pub struct RefDeviceIn {
    pub kind: RefKind,
    /// Deck model name; selects the matching recogniser row. `None` takes the
    /// first recogniser of the kind/polarity (a capacitor is skipped); a name
    /// no row matches takes it too for MOS/BJT, and is skipped for R/C/D (a
    /// MOM is not a MIM).
    pub model: Option<String>,
    /// Terminal net names in card order; at least the recogniser's arity,
    /// extras (e.g. a bulk the recogniser does not extract) ignored.
    pub terminals: Vec<String>,
    /// `(spice name, value)` in SI units, one card per drawn finger.
    pub params: Vec<(String, f64)>,
}

/// The whole reference: devices plus the cell's port net names (which must
/// match the [`crate::geom::LabeledPin`] names on the geometry).
#[derive(Clone, Debug, Default)]
pub struct RefInput {
    pub devices: Vec<RefDeviceIn>,
    pub ports: Vec<String>,
    /// The nets that leave the block (the `.subckt` port list), apart from
    /// `ports` (every labelled net, for LVS naming): only these are exempt
    /// from `floating_gate`/`unconnected_pin`. `None`: no port list, every
    /// labelled net is exempt and reported so in the coverage (AV-04).
    pub external_ports: Option<Vec<String>>,
}

/// `input` as the SPICE netlist GPurify's LVS reads: one `.subckt top
/// <ports>`, one `X` card per device calling its deck recogniser's model,
/// on every terminal the card states. A length (`w`, `l`, ...)
/// is written in µm with a `u` suffix and an area in µm² with `p`, so the
/// deck's `netlist_scale` cannot rescale it; any other value is written
/// bare. A FinFET recogniser measures fins, not width: its `w` is written
/// as `nfin`, the width over [`Pdk::fin_pitch`] rounded as the generator
/// rounds it. Returns the text and the indices into `input.devices` of the
/// devices skipped for want of a recogniser.
///
/// # Errors
/// A device whose terminals are fewer than its recogniser's arity, or a
/// FinFET on a PDK that states no fin pitch.
pub fn spice(input: &RefInput, pdk: &Pdk) -> Result<(String, Vec<usize>), String> {
    let deck = &pdk.deck;
    let mut out = format!(".subckt top {}\n", input.ports.join(" "));
    let mut skipped = Vec::new();
    for (index, dev) in input.devices.iter().enumerate() {
        let Some(d) = recogniser_for(dev, deck) else {
            // No marker layer in this deck extracts this device from the
            // layout, so the reference must not expect it either.
            skipped.push(index);
            continue;
        };
        let arity = d.terminals.len();
        if dev.terminals.len() < arity {
            return Err(format!(
                "reference device {index} ({:?}) states {} terminals, its deck recogniser needs {arity}",
                dev.kind,
                dev.terminals.len()
            ));
        }
        // Every card terminal: GPurify reads a MOS's bulk off the card even
        // where the deck extracts none.
        out.push_str(&format!("X{index} {} {}", dev.terminals.join(" "), d.model));
        for (name, v) in &dev.params {
            if d.fins.is_some() && name == "w" {
                let pitch =
                    pdk.fin_pitch().ok_or_else(|| format!("reference device {index}: {} counts fins, the PDK states no fin pitch", d.model))?;
                let w_nm = (v * 1e9).round() as i64;
                out.push_str(&format!(" nfin={}", ((w_nm + i64::from(pitch) / 2) / i64::from(pitch)).max(1)));
                continue;
            }
            match name.as_str() {
                // Rounded to the pm: no float noise past the nm grid.
                "w" | "l" | "ps" | "pd" | "pe" | "perim" => out.push_str(&format!(" {name}={}u", (v * 1e12).round() / 1e6)),
                "area" | "as" | "ad" | "ae" => out.push_str(&format!(" {name}={}p", v * 1e12)),
                _ => out.push_str(&format!(" {name}={v}")),
            }
        }
        out.push('\n');
    }
    out.push_str(".ends top\n");
    debug_assert!(skipped.windows(2).all(|w| w[0] < w[1]), "skipped indices ascend");
    Ok((out, skipped))
}

// ---- private ----

/// The deck recogniser row for one schematic device, `None` when the deck has
/// no marker for its kind/polarity. Polarity is read off the marker layer name
/// ([`Deck::polarity`]: `ngate`/`pgate`, `npn`/`pnp`, asap7's `PMOS_RVT`). For R/C/D a model
/// hint no row names (exactly or as a `__` vendor suffix) is `None`: sky130's
/// `cap_generic_m1m2` (MOM) must not be compared as its `capm` (MIM). So is a
/// capacitor with no model (an elaborated composition's card): every deck's
/// capacitor row is a MIM or MOS cap, never the MOM the generators draw. A
/// model-less R/D still takes the first row (antenna diodes carry no model).
fn recogniser_for<'d>(dev: &RefDeviceIn, deck: &'d Deck) -> Option<&'d crate::deck::Device> {
    let (kind, polarity) = match dev.kind {
        RefKind::Inductor => return None,
        RefKind::Capacitor if dev.model.is_none() => return None,
        RefKind::Nmos => (DeviceKind::Mos, Some(false)),
        RefKind::Pmos => (DeviceKind::Mos, Some(true)),
        RefKind::Npn => (DeviceKind::Bjt, Some(false)),
        RefKind::Pnp => (DeviceKind::Bjt, Some(true)),
        RefKind::Resistor => (DeviceKind::Resistor, None),
        RefKind::Capacitor => (DeviceKind::Capacitor, None),
        RefKind::Diode => (DeviceKind::Diode, None),
    };
    let mut fallback = None;
    for d in deck.devices.iter().filter(|d| d.kind == kind) {
        let p = deck.polarity(d);
        if polarity.is_some_and(|want_p| p != Some(want_p)) {
            continue;
        }
        let vendor = dev.model.as_deref().is_some_and(|m| d.model.ends_with(&format!("__{m}")) || m.ends_with(&format!("__{}", d.model)));
        if dev.model.is_none() || dev.model.as_deref() == Some(d.model.as_str()) || vendor {
            return Some(d);
        }
        // The hint names no deck model (yet): a MOS/BJT falls back to the
        // first match whose marker states its polarity (IHP's `esd_vdd` bjt
        // is an ESD diode, no NPN a schematic may mean).
        if polarity.is_some() && p.is_some() {
            fallback.get_or_insert(d);
        }
    }
    fallback
}
