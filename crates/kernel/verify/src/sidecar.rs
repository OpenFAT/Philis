//! The registry of every `cell.*` key a Philis sidecar may carry: its kind,
//! whether loading requires it, whether it is process data that must name its
//! source, and who reads it.
//!
//! `required` is the set generators read through `Process::rule` with a
//! compiled default, less keys whose reader only raises a deck-derived value
//! and the `npn_isolation` and `pnp_cells_only` flags (kernel/cells/tests/deck_keys.rs checks both
//! directions against a recording run; `max_finger_width`, read by the
//! library's cellgen, by name), so a shipped deck never silently builds to a
//! number compiled into Philis. `sourced` keys are measurements of the process
//! (mismatch, temperature, capacitance, well geometry): each non-null value
//! needs a non-empty `<key>_source`; one starting `UNVERIFIED` is accepted
//! and listed by [`crate::Pdk::unverified`]. Generator construction
//! dimensions are Philis choices bounded below by deck rules
//! (kernel/cells/src/builder.rs `dim`), so they are not sourced.

use serde_json::Value;

/// The JSON shape a key's value must have. `null` is accepted for every kind
/// (the process does not state it) except on a `required` key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Integer length, nm.
    Nm,
    /// Non-negative integer (a count or 0/1 flag).
    Count,
    Bool,
    /// Any number.
    Real,
    Text,
    /// Array of strings.
    List,
    /// Object whose shape its reader checks.
    Table,
    /// `cell.layers`: role → layer name plus the routing stack, checked by
    /// `parse_roles`.
    Layers,
}

/// One registry row.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub name: &'static str,
    pub kind: Kind,
    /// Missing (or null) fails `Pdk::load`.
    pub required: bool,
    /// Process data: a non-null value needs a `<name>_source`.
    pub sourced: bool,
    /// Where it is read, or `unread`.
    pub reader: &'static str,
}

const fn k(name: &'static str, kind: Kind, required: bool, sourced: bool, reader: &'static str) -> Key {
    Key { name, kind, required, sourced, reader }
}

use Kind::{Bool, Count, Layers, List, Nm, Real, Table, Text};

/// Every key, alphabetical. `<name>_source` of a registered key is implied.
pub const KEYS: &[Key] = &[
    k("abeta_n_pct_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("abeta_p_pct_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("antenna_source", Text, false, false, "provenance of the deck's antenna rules"),
    k("avt_n_mv_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("avt_p_mv_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("bjt_ka_pct_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("bjt_min_emitter_side", Nm, true, false, "kernel/cells/src/bjt.rs"),
    k("bjts", Table, false, false, "backend/verify/src/pdk.rs recipe"),
    k("cap_strap_via_enclosure_nm", Nm, false, false, "kernel/substrate3/src/cells/capacitor.rs Geom::new via builder.rs dim (a Philis margin over the deck's via enclosure, the sandwich strap's cuts only)"),
    k("cap_strap_via_spacing_nm", Nm, false, false, "kernel/substrate3/src/cells/capacitor.rs Geom::new (a Philis margin over the deck's via spacing, the sandwich strap's cuts only)"),
    k("cap_unit_side", Nm, true, false, "kernel/cells/src/capacitor.rs, cap_array.rs"),
    k("capacitors", Table, false, false, "backend/verify/src/pdk.rs recipe"),
    k("density_source", Text, false, false, "provenance of the deck's density rules"),
    k("device_gap", Nm, true, false, "kernel/cells/src/capacitor.rs; frontend/library/src/emit.rs"),
    k("diode_gap", Nm, true, false, "kernel/cells/src/diode.rs"),
    k("diode_l", Nm, true, false, "kernel/cells/src/diode.rs; frontend/library/src/elaborate.rs"),
    k("diode_w", Nm, true, false, "kernel/cells/src/diode.rs; frontend/library/src/elaborate.rs"),
    k("dummy_gates_per_end", Count, true, false, "kernel/cells/src/mosfet.rs (dummy gates each end of a matched row)"),
    k("dummy_max_l_nm", Nm, false, true, "kernel/cells/src/mosfet.rs (dummy gate length cap)"),
    k("em_current_density_source", Text, false, false, "provenance of the deck's EM rules"),
    k("em_derating", Table, false, true, "backend/verify/src/pdk.rs em_limit (when the deck rule states no Black parameters)"),
    k("em_front_row_cuts", Bool, false, true, "frontend/library/src/lib.rs em_rules; backend/dr DetailedCfg (REL-12)"),
    k("epi_thickness_nm", Nm, false, true, "frontend/library/src/lib.rs annotation (isolation distance on epi_on_pplus)"),
    k("erc_rules_note", Text, false, false, "documentation"),
    k("finfet_note", Text, false, false, "documentation"),
    k("gate_cap_af_um2", Count, false, true, "frontend/library/src/lib.rs annotation"),
    k("guard_licon_pitch", Nm, true, false, "kernel/cells/src/mosfet.rs, post_cell.rs"),
    k("inapplicable_rules", List, false, false, "backend/verify/src/pdk.rs"),
    k("inapplicable_rules_note", Text, false, false, "documentation"),
    k("latent_merge_nm", Nm, false, true, "frontend/library/src/elaborate.rs stack (GAP-13; default the layer's min spacing)"),
    k("layers", Layers, true, false, "backend/verify/src/pdk.rs parse_roles"),
    k("li_encloses_licon_one_side", Nm, false, false, "kernel/cells/src/mosfet.rs"),
    k("licon_poly_enc", Nm, false, false, "kernel/cells/src/mosfet.rs"),
    k("licon_to_gate_spacing", Nm, false, false, "kernel/cells/src/mosfet.rs"),
    k("lod_kvth0_n_mv_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("lod_kvth0_p_mv_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("max_finger_width", Nm, true, false, "frontend/library/src/cellgen.rs folds (0 = no limit)"),
    k("metal_family", Text, false, true, "kernel/analog/src/routing/em.rs metal_family (REL-17 ESD width; \"al\" | \"cu\")"),
    k("min_guard_ring_width", Nm, true, false, "kernel/cells/src/bjt.rs, post_cell.rs"),
    k("mom_finger_space", Nm, true, false, "kernel/cells/src/capacitor.rs"),
    k("mom_finger_width", Nm, true, false, "kernel/cells/src/capacitor.rs"),
    k("npn_isolation", Count, false, false, "kernel/cells/src/bjt.rs"),
    k("npn_isolation_note", Text, false, false, "documentation"),
    k("plate_spacing", Nm, true, false, "kernel/cells/src/capacitor.rs, cap_array.rs"),
    k("pnp_cells_only", Count, false, false, "kernel/substrate3/src/cells/bjt.rs"),
    k("pnp_cells_only_note", Text, false, false, "documentation"),
    k("polycon_to_diff_spacing", Nm, false, false, "kernel/cells/src/mosfet.rs"),
    k("polycon_to_pdiff_spacing", Nm, false, false, "kernel/cells/src/mosfet.rs"),
    k("res_head", Nm, true, false, "kernel/cells/src/resistor.rs"),
    k("res_min_segment", Nm, true, false, "kernel/cells/src/resistor.rs"),
    k("res_min_width", Nm, true, false, "kernel/cells/src/resistor.rs"),
    k("res_seg_gap", Nm, true, false, "kernel/cells/src/resistor.rs"),
    k("res_self_heat_dt_k", Real, false, true, "kernel/cells/src/resistor.rs self_heating_min_width_nm (CELL-23; default 5 K, Hastings eq. 5.8)"),
    k("res_tox_nm", Real, false, true, "kernel/cells/src/resistor.rs self_heating_min_width_nm (CELL-23; else pex height_nm of the body layer)"),
    k("res_value_tol_ppm", Count, true, false, "kernel/cells/src/resistor.rs"),
    k("resistors", Table, false, false, "backend/verify/src/pdk.rs recipe"),
    k("retrograde_pwell", Bool, false, true, "kernel/cells/src/post_cell.rs drawable"),
    k("sd_width", Nm, true, false, "kernel/cells/src/mosfet.rs"),
    k("substrate_kind", Text, false, true, "frontend/library/src/lib.rs annotation (\"bulk\" | \"epi_on_pplus\" | null)"),
    k("svt_a_uv2_per_um2", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("svt_b_uv2", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("svt_uv_per_um", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("vbe_tc_uv_per_k", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("vt_tc_uv_per_k", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("vt_tc_uv_per_k_p", Real, false, true, "frontend/library/src/lib.rs annotation"),
    k("waivers", Table, false, false, "backend/verify/src/checker.rs"),
];

/// The registry row for `name`.
fn key(name: &str) -> Option<&'static Key> {
    KEYS.iter().find(|k| k.name == name)
}

/// Every problem at once: unknown key (a typo), wrong kind, a required key
/// missing, a sourced key with a non-null value and no non-empty
/// `<name>_source`.
#[must_use]
pub fn validate(cell: &Value) -> Vec<String> {
    let Some(obj) = cell.as_object() else { return vec!["the sidecar's `cell` is not an object".into()] };
    let mut bad = Vec::new();
    for (name, v) in obj {
        if name.strip_suffix("_source").is_some_and(|b| key(b).is_some()) {
            if !v.is_string() {
                bad.push(format!("cell.{name}: a source is text, got {v}"));
            }
            continue;
        }
        let Some(row) = key(name) else {
            bad.push(format!("cell.{name}: unknown key (a typo? register it in backend/verify/src/sidecar.rs)"));
            continue;
        };
        let ok = match row.kind {
            _ if v.is_null() => true,
            Nm => v.is_i64(),
            Count => v.is_u64(),
            Bool => v.is_boolean(),
            Real => v.is_number(),
            Text => v.is_string(),
            List => v.as_array().is_some_and(|a| a.iter().all(Value::is_string)),
            Table | Layers => v.is_object(),
        };
        if !ok {
            bad.push(format!("cell.{name}: expected {:?}, got {v}", row.kind));
        }
    }
    for row in KEYS {
        let v = obj.get(row.name).filter(|v| !v.is_null());
        if row.required && v.is_none() {
            bad.push(format!("cell.{}: required ({} reads it with a compiled default; add this process's value)", row.name, row.reader));
        }
        if row.sourced && v.is_some() && source(obj, row.name).is_none() {
            bad.push(format!("cell.{}: process data with no `{}_source`", row.name, row.name));
        }
    }
    bad
}

/// `<name>_source`, when non-empty text.
pub(crate) fn source<'a>(cell: &'a serde_json::Map<String, Value>, name: &str) -> Option<&'a str> {
    cell.get(&format!("{name}_source"))?.as_str().filter(|s| !s.trim().is_empty())
}
