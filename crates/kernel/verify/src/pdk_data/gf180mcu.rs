//! gf180mcu's Philis data: what no deck statement carries (layer roles, the
//! routing stack, generator dimensions, process coefficients, recipe tables),
//! each number beside its `_source`. Was `pdks/gf180mcu.json`; the deck is
//! `pdks/gpurify/gf180mcu.deck` (GPurify's plus `crate::overrides`). Registry and
//! validation: [`crate::sidecar`]; classification:
//! `docs/ref/research/sidecar-keys.md`.

/// The `cell` table [`crate::Pdk::load`] reads.
#[allow(clippy::too_many_lines)]
pub(crate) fn cell() -> serde_json::Value {
    serde_json::json!({
        "bjt_ka_pct_um": 2.0,
        "bjt_ka_pct_um_source": "textbook, not gf180mcu: Hastings, The Art of Analog Layout, §10.2.1 (PDF p.507), I_S mismatch area constant of a bipolar pair; replace with the PDK's own figure when one is published.",
        "vbe_tc_uv_per_k": 2000,
        "vbe_tc_uv_per_k_source": "textbook, not gf180mcu: Hastings, The Art of Analog Layout, §9.1 (PDF p.433), |dV_BE/dT| ~ 2 mV/K.",
        "li_encloses_licon_one_side": 60,
        "cap_strap_via_enclosure_nm": 400,
        "cap_strap_via_enclosure_nm_source": "Philis margin, not a deck rule: how far the sandwich capacitor's strap-column metals pass its cuts. The deck states 10 nm (V1.4a, Vn.3b) and 60 nm end of line (Vn.3c/Vn.4b); 400 nm is MIMTM.2's via5-on-MIM enclosure, kept stricter by owner decision, here and not in the deck, since a 400 nm via rule would fail every routed via. Read by cells::capacitor Geom::new, through builder::dim, which maxes it with the deck's via enclosure.",
        "dummy_max_l_nm": 3000,
        "dummy_max_l_nm_source": "microloading reach 3-5 um, hastings.txt L41097-41104 (H13-26); Philis policy: the lower end",
        "em_derating": {
            "t_ref_k": 358.15,
            "ea_ev": null,
            "n": null
        },
        "em_derating_source": "pdks/gpurify/gf180mcu.deck:486 \"Electromigration at 85 C\" [DRM 54, tables 14.3/14.4]; Ea and n not stated",
        "avt_n_mv_um": 8.9,
        "avt_n_mv_um_source": "nfet_03v3 Monte Carlo of the installed models: benchmarks/characterize_mismatch.py gf180mcu 250 1|2, ngspice-44.2, design.ngspice + '.lib sm141064.ngspice typical' with sw_stat_mismatch=1, ngbehavior=hsa (volare gf180mcu c6d73a35 (gf180mcuD)); 250 pairs x 6 W/L (1/0.28..8/2 um), diode-connected, constant-current VT at 0.1uA*W/L, sigma(dVT) fit through origin vs 1/sqrt(WL); seeds 1,2 gave 8.88, 8.9 -> 8.9. Model card (fets_mm): par_vth 7.148 mV.um on effective area (L-0.15um); mulu0 mismatch adds the rest.",
        "avt_p_mv_um": 8.5,
        "avt_p_mv_um_source": "pfet_03v3, same run: seeds 1,2 gave 8.47, 8.48 -> 8.5. Model card par_vth 6.66 mV.um on effective area.",
        "vt_tc_uv_per_k": 1130.0,
        "vt_tc_uv_per_k_source": "nfet_03v3 benchmarks/characterize_mismatch.py tc gf180mcu, ngspice-44.2, '.lib sm141064.ngspice typical' (volare gf180mcu c6d73a35 (gf180mcuD)), diode-connected, constant-current VT at 0.1uA*W/L, temp 0/27/85/125 C, least-squares slope: 2/0.5 um -1150, 4/1 um -1108 uV/K -> 1130 (magnitude; VT falls with T). The library reads this single key; pfet in vt_tc_uv_per_k_p.",
        "vt_tc_uv_per_k_p": 1140.0,
        "vt_tc_uv_per_k_p_source": "pfet_03v3, same run: 2/0.5 um -1183, 4/1 um -1098 uV/K -> 1140.",
        "svt_uv_per_um": 0.941,
        "svt_uv_per_um_source": "PROXY, not gf180mcu data: Schaper & Linnenbank, 'Comparison of distance mismatch and pair matching of CMOS devices' (Infineon 130 nm, IEEE), Table 1, thin-oxide nMOS S_D [mV/mm = uV/um]: L/W 0.12/1.6 1.63, 0.24/1.6 0.780, 0.24/12.8 0.941, 0.48/1.6 0.582, 0.48/3.2 0.644; S_D grows as L shrinks (Fig. 8, S_D^2 ~ 1/Leff^2). gf180 3.3V Lmin 0.28 um lies between the 0.24 and 0.48 rows; took the larger of the two L=0.24 values (0.941), an upper bound for L>=0.24 given S_D falls with L. Planar 180 nm vs 130 nm source process.",
        "layers": {
            "diff": "comp",
            "poly": "poly2",
            "rpoly": "resistor",
            "li": "metal1",
            "licon": "contact",
            "mcon": "via1",
            "met1": "metal2",
            "nwell": "nwell",
            "nsdm": "nplus",
            "psdm": "pplus",
            "tap": "comp",
            "routing_metals": [
                "metal1",
                "metal2",
                "metal3",
                "metal4",
                "metal5",
                "metaltop"
            ],
            "routing_vias": [
                "via1",
                "via2",
                "via3",
                "via4",
                "via5"
            ]
        },
        "sd_width": 260,
        "dummy_gates_per_end": 1,
        "max_finger_width": 100000,
        "plate_spacing": 1200,
        "res_head": 360,
        "device_gap": 860,
        "res_min_segment": 10000,
        "res_seg_gap": 400,
        "bjt_min_emitter_side": 540,
        "diode_gap": 280,
        "min_guard_ring_width": 220,
        "guard_licon_pitch": 470,
        "substrate_kind": null,
        "epi_thickness_nm": null,
        "retrograde_pwell": false,
        "retrograde_pwell_source": "UNVERIFIED: no source was recorded with this value. Read by no code.",
        "cap_unit_side": 5000,
        "diode_l": 1000,
        "diode_w": 220,
        "mom_finger_space": 230,
        "mom_finger_width": 230,
        "res_min_width": 800,
        "res_value_tol_ppm": 5000,
        "licon_poly_enc": 70,
        "polycon_to_diff_spacing": 170,
        "resistors": {
            "default": "ppolyf_u",
            "recipes": {
                "ppolyf_u": {
                    "model": "ppolyf_u",
                    "layers": {
                        "rpoly": "res_mk",
                        "res_block": "sab",
                        "res_implant": "pplus"
                    }
                },
                "npolyf_u": {
                    "model": "npolyf_u",
                    "layers": {
                        "rpoly": "res_mk",
                        "res_block": "sab",
                        "res_implant": "nplus"
                    }
                }
            },
            "note": "GPurify gf180mcu.deck: ppolyf_u_r = (poly2 and pplus and sab and res_mk) not interacting resistor, npolyf_u_r = poly2 and nplus and sab and res_mk; res_mk cuts poly2_con, so the heads are two nets."
        },
        "waivers": {
            "multiple_drivers": "Analog: a node driven by two drains (a differential pair's output and its load, a mirror's input) is the circuit, not a fault. The rule counts distinct gate nets driving a net (a digital fan-in check)."
        }
    })
}
