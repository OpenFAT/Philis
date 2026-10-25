//! Philis's edits to GPurify's shipped decks, as [`DeckBuilder`] overrides
//! (Q59): the upstream text is the pinned dependency's own
//! (`gdsverify::pdks::*`, byte for byte GPurify's `pdks/`), and each edit
//! below is one that 1.0 made by hand in a vendored copy under a
//! `# PHILIS:` comment. Each carries that comment's reason; an edit leaves
//! here once GPurify ships it (inventory: `docs/ref/research/deck-overrides.md`).

use crate::pdk::nm_grid;
use gdsverify::check::deck::{print, rules, DeckBuilder, LayerExpr, LayerRef};
use gdsverify::geom::StrTable;
use gdsverify::ingest::deck::{DeviceKind, DeviceValue, StackSource};

/// `(PDK name, GPurify's deck text)`: the upstream deck each built-in PDK
/// starts from. ASAP7 is GPurify's `generic_finfet.deck`.
pub(crate) const UPSTREAM: &[(&str, &str)] = &[
    ("sky130", gdsverify::pdks::SKY130),
    ("gf180mcu", gdsverify::pdks::GF180MCU),
    ("ihp_sg13g2", gdsverify::pdks::IHP_SG13G2),
    ("asap7", gdsverify::pdks::GENERIC_FINFET),
];

/// The deck text of built-in PDK `name`: GPurify's deck with Philis's
/// overrides applied, compiled on [`nm_grid`] and printed back.
///
/// # Errors
/// An unknown name, or what GPurify refuses in the deck or an override.
pub fn deck_text(name: &str) -> Result<String, String> {
    let (_, upstream) = UPSTREAM.iter().find(|(n, _)| *n == name).ok_or_else(|| format!("no upstream deck for {name:?}"))?;
    let mut strings = StrTable::default();
    let base = gdsverify::check::deck::compile(upstream, nm_grid(), &mut strings).map_err(|e| format!("{name}: upstream deck rejected: {e}"))?;
    let mut d = DeckBuilder::from_deck(&base, &mut strings);
    match name {
        "sky130" => sky130(&mut d),
        "asap7" => asap7(&mut d),
        _ => {} // gf180mcu, ihp_sg13g2: no edits.
    }
    let deck = d.finish().map_err(|e| format!("{name}: deck overrides rejected: {e}"))?;
    print(&deck, &strings).map_err(|e| format!("{name}: {e}"))
}

fn e(layer: LayerRef) -> LayerExpr {
    LayerExpr::from(layer)
}

/// A `pex` row as the deck writes it (`dielectric` k, caps in aF/µm², aF/µm).
fn pex(thickness_nm: f64, height_nm: f64, sheet: f64, k: f64, area: f64, fringe: f64) -> StackSource {
    StackSource { thickness_nm, height_nm, sheet_res_ohm_sq: sheet, area_cap_af_um2: area, fringe_cap_af_um: fringe, dielectric_k: k, tc: None }
}

fn sky130(d: &mut DeckBuilder<'_>) {
    // npn marker (VOL/klayout/lvs/sky130.lvs:744).
    let npn = d.layer("npn", (82, 20));

    // GPurify measures an inner no host covers as 0 (a violation), so the
    // lvtn off every such well (an LVT nfet) is selected out here.
    let (lvtn, novar) = (d.get("lvtn"), d.get("nwell_novar"));
    let w = d.length(380.0);
    d.replace("lvtn.10", rules::noncoincident_enclosure(e(lvtn).interacting(novar), novar, w));

    // Antenna sidewall = the pex thickness of met3/met4/met5 in this deck, and
    // VOL/magic/sky130A.tech:4890 'height allm3 2.7861 0.845'.
    let (gate, diode, exempt) = (d.get("gate"), d.get("antenna_diode"), d.get("pantenna_short"));
    for (id, metal, sidewall_nm) in [("ar.met3.1", "met3", 845.0), ("ar.met4.1", "met4", 845.0), ("ar.met5.1", "met5", 1260.0)] {
        let metal = d.get(metal);
        let sidewall = d.length(sidewall_nm);
        let row = rules::antenna_electrical(gate, &[metal], 400.0, Some(sidewall), 1.0, Some(diode), 400.0, 2200.0, 0.0, 0.0, &[], &[], Some(exempt));
        d.replace(id, row);
    }

    // psub is soft too. GPurify keeps a global layer in the hard
    // connectivity (its docs say it drops it), so a supply tied to the
    // substrate through ptap read as two pieces, its strap and the substrate.
    let soft: Vec<LayerRef> = ["ntap", "ptap", "nwell", "pwell_iso", "psub"].map(|n| d.get(n)).into();
    d.replace("soft_connection", rules::soft_connection(&soft));

    // Latch-up, VOL/magic/sky130A.tech:4163-4170 (LU.2, LU.2.1, LU.3:
    // "< 15.0um"); operands as magic's ptap_missing (*ndiff and-not dnwell),
    // dptap_missing (*ndiff and dnwell) and ntap_missing (*pdiff).
    let [nsd, psd, nwell, dnwell, ptap, ntap] = ["nsd", "psd", "nwell", "dnwell", "ptap", "ntap"].map(|n| d.get(n));
    let lu_ndiff = d.derived("lu_ndiff", e(nsd).not(nwell).group().not(dnwell));
    let lu_ndiff_dnw = d.derived("lu_ndiff_dnw", e(nsd).not(nwell).group().and(dnwell));
    let lu_pdiff = d.derived("lu_pdiff", e(psd).and(nwell));
    let reach = d.length(15_000.0);
    d.add("LU.2", rules::max_distance_to_tap(lu_ndiff, ptap, reach));
    d.add("LU.2.1", rules::max_distance_to_tap(lu_ndiff_dnw, ptap, reach));
    d.add("LU.3", rules::max_distance_to_tap(lu_pdiff, ntap, reach));

    // Vertical PNP (sky130.lvs:745, 1089-1091) and NPN (:744, 1063-1065),
    // terminals Base, Emitter, Collector. One size each: two rows on one
    // marker, the earlier wins, and there is no emitter-area select.
    let pnp = d.get("pnp");
    d.device(DeviceKind::Bjt, pnp, "sky130_fd_pr__pnp_05v5_W3p40L3p40", &[ntap, psd, ptap], DeviceValue::default());
    d.device(DeviceKind::Bjt, npn, "sky130_fd_pr__npn_05v5_W1p00L1p00", &[ptap, nsd, ntap], DeviceValue::default());

    // Poly resistor bodies: sheet = the model card's rsh (generic_po 48.2,
    // res_high_po 317.3885, res_xhigh_po 2000; VOL/ngspice and libs.ref
    // model cards); geometry columns = poly_c's.
    for (body, sheet) in [("rbody_po", 48.2), ("rbody_high_po", 317.3885), ("rbody_xhigh_po", 2000.0)] {
        let body = d.get(body);
        d.pex(body, pex(180.0, 326.2, sheet, 3.9, 106.13, 55.27));
    }
}

fn asap7(d: &mut DeckBuilder<'_>) {
    // The front end is connected, so LVS extracts transistors. SD, GATE_C,
    // LISD and the SDT and V0 cuts have no pex row: their resistance is
    // unknown, never zero. A source/drain is diffusion off the gate that a
    // trench contacts; a gate is GATE plus the LIG on it, when one is.
    let [active, gate, sdt, gate_uncut, lig, channel, lisd, m1, v0] =
        ["ACTIVE", "GATE", "SDT", "GATE_UNCUT", "LIG", "CHANNEL", "LISD", "M1", "V0"].map(|n| d.get(n));
    let sd = d.derived("SD", e(active).not(gate).group().interacting(sdt));
    let gate_c = d.derived("GATE_C", e(gate_uncut).or(lig).group().interacting(lig));
    // A channel is a transistor only under a contacted gate: an uncontacted
    // gate over a diffusion edge is PODE and no FinFET LVS deck extracts it.
    let channel_c = d.derived("CHANNEL_C", e(channel).interacting(gate_c));
    d.conductors(&[sd, gate_c, lisd]);
    // The front-end cuts [DRM 2.2]: SDT from diffusion to LISD, V0 from LISD
    // and from LIG up to M1.
    d.via(sdt, (sd, lisd));
    d.via(v0, (lisd, m1));
    d.via(v0, (gate_c, m1));

    // MOS recognisers ([DRM] 2.1 layers, models per cdslib/asap7_TechLib_10).
    // Three terminals: well and substrate are not conductors, so the bulk is
    // not compared.
    let [nselect, pselect, slvt, lvt, sramvt] = ["NSELECT", "PSELECT", "SLVT", "LVT", "SRAMVT"].map(|n| d.get(n));
    for (polarity, select) in [("nmos", nselect), ("pmos", pselect)] {
        let rvt = e(channel_c).and(select).group().not(e(slvt).or(lvt).or(sramvt).group());
        for (vt, expr) in [
            ("rvt", rvt),
            ("lvt", e(channel_c).and(select).and(lvt)),
            ("slvt", e(channel_c).and(select).and(slvt)),
            ("sram", e(channel_c).and(select).and(sramvt)),
        ] {
            let name = format!("{polarity}_{vt}");
            let marker = d.derived(&name, expr);
            d.device(DeviceKind::Mos, marker, &name, &[gate_c, sd, sd], DeviceValue::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdsverify::ingest::deck::DeckSource;

    /// Every list of a deck source in one order, so two decks that state the
    /// same things in a different order compare equal. ponytail: sorts by
    /// `Debug` text; the order the builder appends in (layers, rules,
    /// devices last) is the only difference it has to absorb.
    fn canon(text: &str) -> DeckSource {
        fn sort<T: std::fmt::Debug>(v: &mut [T]) {
            v.sort_by_cached_key(|x| format!("{x:?}"));
        }
        let mut strings = StrTable::default();
        let deck = gdsverify::check::deck::compile(text, nm_grid(), &mut strings).expect("deck compiles");
        let mut s = deck.source(&strings);
        sort(&mut s.layers);
        sort(&mut s.derived);
        sort(&mut s.rules);
        sort(&mut s.connectivity.conductors);
        sort(&mut s.connectivity.vias);
        sort(&mut s.devices);
        sort(&mut s.pex);
        s
    }

    /// The override-built deck states exactly what 1.0's hand-edited
    /// vendored deck does: every layer, rule, connection, device and pex row.
    #[test]
    fn overrides_equal_the_edited_decks() {
        for (name, edited) in crate::decks::DECKS {
            let pdk = name.trim_end_matches(".deck");
            let built = deck_text(pdk).unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(canon(&built), canon(edited), "{pdk}: overrides differ from pdks/decks/{name}");
        }
    }
}
