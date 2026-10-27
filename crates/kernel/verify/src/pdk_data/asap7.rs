//! asap7's Philis data: what no deck statement carries (layer roles, the
//! routing stack, generator dimensions, process coefficients, recipe tables),
//! each number beside its `_source`. Was `pdks/asap7.json`; the deck is
//! `pdks/gpurify/asap7.deck` (GPurify's plus `crate::overrides`). Registry and
//! validation: [`crate::sidecar`]; classification:
//! `docs/ref/research/sidecar-keys.md`.
//!
//! Philis sidecar for ASAP7, the predictive 7 nm FinFET PDK (ASU and ARM; GPurify vendors its deck as generic_finfet.deck): every rule, layer, device and PEX value comes from the GPurify deck named here; this file holds only what the checker does not read (layer roles, generator dimensions, process coefficients).

/// The `cell` table [`crate::Pdk::load`] reads.
#[allow(clippy::too_many_lines)]
pub(crate) fn cell() -> serde_json::Value {
    serde_json::json!({
        "bjt_ka_pct_um": 2.0,
        "bjt_ka_pct_um_source": "textbook, not ASAP7: Hastings, The Art of Analog Layout, §10.2.1 (PDF p.507), I_S mismatch area constant of a bipolar pair; replace with the PDK's own figure when one is published.",
        "vbe_tc_uv_per_k": 2000,
        "vbe_tc_uv_per_k_source": "textbook, not ASAP7: Hastings, The Art of Analog Layout, §9.1 (PDF p.433), |dV_BE/dT| ~ 2 mV/K.",
        "dummy_max_l_nm": null,
        "layers": {
            "diff": "ACTIVE",
            "tap": "ACTIVE",
            "poly": "GATE",
            "fin": "FIN",
            "sdt": "SDT",
            "lisd": "LISD",
            "lig": "LIG",
            "gcut": "GCUT",
            "li": "M1",
            "licon": "V0",
            "mcon": "V1",
            "met1": "M2",
            "nwell": "WELL",
            "nsdm": "NSELECT",
            "psdm": "PSELECT",
            "routing_metals": [
                "M1",
                "M2",
                "M3",
                "M4",
                "M5",
                "M6",
                "M7",
                "M8",
                "M9"
            ],
            "routing_vias": [
                "V1",
                "V2",
                "V3",
                "V4",
                "V5",
                "V6",
                "V7",
                "V8"
            ]
        },
        "sd_width": 34,
        "dummy_gates_per_end": 1,
        "max_finger_width": 2700,
        "plate_spacing": 200,
        "res_head": 300,
        "device_gap": 108,
        "res_min_segment": 10000,
        "res_seg_gap": 400,
        "bjt_min_emitter_side": 420,
        "diode_gap": 270,
        "min_guard_ring_width": 54,
        "guard_licon_pitch": 54,
        "substrate_kind": null,
        "epi_thickness_nm": null,
        "retrograde_pwell": false,
        "retrograde_pwell_source": "UNVERIFIED: no source was recorded with this value. Read by no code.",
        "cap_unit_side": 2000,
        "diode_l": 1000,
        "diode_w": 500,
        "mom_finger_space": 200,
        "mom_finger_width": 200,
        "res_min_width": 500,
        "res_value_tol_ppm": 5000,
    })
}
