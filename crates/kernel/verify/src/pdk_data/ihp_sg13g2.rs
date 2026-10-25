//! ihp_sg13g2's Philis data: what no deck statement carries (layer roles, the
//! routing stack, generator dimensions, process coefficients, recipe tables),
//! each number beside its `_source`. Was `pdks/ihp_sg13g2.json`; the deck is
//! `pdks/ihp_sg13g2.deck` (GPurify's plus `crate::overrides`). Registry and
//! validation: [`crate::sidecar`]; classification:
//! `docs/ref/research/sidecar-keys.md`.

/// The `cell` table [`crate::Pdk::load`] reads.
#[allow(clippy::too_many_lines)]
pub(crate) fn cell() -> serde_json::Value {
    serde_json::json!({
    "bjt_ka_pct_um": 2.0,
    "bjt_ka_pct_um_source": "textbook, not ihp_sg13g2: Hastings, The Art of Analog Layout, §10.2.1 (PDF p.507), I_S mismatch area constant of a bipolar pair; replace with the PDK's own figure when one is published.",
    "vbe_tc_uv_per_k": 2000,
    "vbe_tc_uv_per_k_source": "textbook, not ihp_sg13g2: Hastings, The Art of Analog Layout, §9.1 (PDF p.433), |dV_BE/dT| ~ 2 mV/K.",
    "via_spacing": 420,
    "via_enclosure": 360,
    "dummy_max_l_nm": 3000,
    "dummy_max_l_nm_source": "microloading reach 3-5 um, hastings.txt L41097-41104 (H13-26); Philis policy: the lower end",
    "abeta_n_pct_um": 0.88,
    "abeta_n_pct_um_source": "Monte Carlo of the installed models: benchmarks/characterize_mismatch.py bpv ihp_sg13g2 250 1 (MM-17), cornerMOSlv.lib mos_tt_mismatch, 250 voltage-biased pairs x 6 W/L (1/0.13..8/2 um) x Vov 0.1, 0.4 V (VGS = VDS = |VT|tt + Vov), G = gm/I from a tt +-1 mV difference; sigma(dI/I)^2 = A_VT^2 (0.1 G)^2/WL + A_beta^2/WL fit over the 12 points. One seed. sg13_lv_nmos: A_beta 0.877 %.um, refit A_VT 5.02 mV.um (avt_n 5.2 agrees).",
    "abeta_p_pct_um": 1.23,
    "abeta_p_pct_um_source": "Monte Carlo of the installed models: benchmarks/characterize_mismatch.py bpv ihp_sg13g2 250 1 (MM-17), cornerMOSlv.lib mos_tt_mismatch, 250 voltage-biased pairs x 6 W/L (1/0.13..8/2 um) x Vov 0.1, 0.4 V (VGS = VDS = |VT|tt + Vov), G = gm/I from a tt +-1 mV difference; sigma(dI/I)^2 = A_VT^2 (0.1 G)^2/WL + A_beta^2/WL fit over the 12 points. One seed. sg13_lv_pmos: A_beta 1.227 %.um, refit A_VT 2.70 mV.um (avt_p 2.9, 7% apart: within 1.6x the 4.5% per-point error).",
    "avt_n_mv_um": 5.2,
    "avt_n_mv_um_source": "sg13_lv_nmos Monte Carlo of the installed models: benchmarks/characterize_mismatch.py ihp_sg13g2 250 1|2, ngspice-44.2, '.lib cornerMOSlv.lib mos_tt_mismatch' with mm_ok=1, PSP103 OSDI compiled by openvaf-r from the repo's verilog-a/psp103/psp103.va (~/pdks/IHP-Open-PDK (git 5e6d592e)); 250 pairs x 6 W/L (1/0.13..8/2 um), diode-connected, constant-current VT at 0.1uA*W/L, sigma(dVT) fit through origin vs 1/sqrt(WL); seeds 1,2 gave 5.1, 5.33 -> 5.2. Model card sg13g2_moslv_mismatch.lib: delvto_mm 3.9 mV.um per device (x sqrt2 = 5.5 per pair). The process spec (libs.doc SG13G2_os_process_spec.pdf) gives no MOS A_VT.",
    "avt_p_mv_um": 2.9,
    "avt_p_mv_um_source": "sg13_lv_pmos, same run: seeds 1,2 gave 2.90, 2.99 -> 2.9. Model card delvto_mm 2.2 mV.um per device (x sqrt2 = 3.1).",
    "vt_tc_uv_per_k": 875.0,
    "vt_tc_uv_per_k_source": "sg13_lv_nmos benchmarks/characterize_mismatch.py tc ihp_sg13g2, ngspice-44.2, '.lib cornerMOSlv.lib mos_tt' (~/pdks/IHP-Open-PDK (git 5e6d592e)), diode-connected, constant-current VT at 0.1uA*W/L, temp 0/27/85/125 C, least-squares slope: 2/0.5 um -873, 4/1 um -878 uV/K -> 875 (magnitude; VT falls with T). The library reads this single key; pfet in vt_tc_uv_per_k_p.",
    "vt_tc_uv_per_k_p": 757.0,
    "vt_tc_uv_per_k_p_source": "sg13_lv_pmos, same run: 2/0.5 um -762, 4/1 um -751 uV/K -> 757.",
    "svt_uv_per_um": 1.63,
    "svt_uv_per_um_source": "PROXY, not ihp_sg13g2 data: Schaper & Linnenbank, 'Comparison of distance mismatch and pair matching of CMOS devices' (Infineon 130 nm, IEEE), Table 1, thin-oxide nMOS S_D [mV/mm = uV/um]: L/W 0.12/1.6 1.63, 0.24/1.6 0.780, 0.24/12.8 0.941, 0.48/1.6 0.582, 0.48/3.2 0.644; S_D grows as L shrinks (Fig. 8, S_D^2 ~ 1/Leff^2). IHP lv Lmin 0.13 um >= 0.12 row: took 1.63, an upper bound at 0.13 um. Same node as the source process.",
    "em_derating": {
        "t_ref_k": 378.15,
        "ea_ev": null,
        "n": null
    },
    "em_derating_source": "pdks/ihp_sg13g2.deck:601 [PS] §2.15 \"11 years at 105 C\"; Ea and n not stated",
    "layers": {
        "diff": "activ",
        "poly": "gatpoly",
        "rpoly": "res",
        "li": "metal1",
        "licon": "cont",
        "mcon": "via1",
        "met1": "metal2",
        "nwell": "nwell",
        "nsdm": "nsd",
        "psdm": "psd",
        "tap": "activ",
        "routing_metals": [
            "metal1",
            "metal2",
            "metal3",
            "metal4",
            "metal5"
        ],
        "routing_vias": [
            "via1",
            "via2",
            "via3",
            "via4"
        ]
    },
    "sd_width": 190,
    "dummy_gates_per_end": 1,
    "max_finger_width": 100000,
    "plate_spacing": 600,
    "res_head": 300,
    "device_gap": 620,
    "res_min_segment": 500,
    "res_seg_gap": 180,
    "bjt_min_emitter_side": 900,
    "diode_gap": 210,
    "min_guard_ring_width": 150,
    "guard_licon_pitch": 340,
    "substrate_kind": null,
    "epi_thickness_nm": null,
    "retrograde_pwell": false,
    "retrograde_pwell_source": "UNVERIFIED: no source was recorded with this value. Read by no code.",
    "cap_unit_side": 1140,
    "diode_l": 1000,
    "diode_w": 150,
    "mom_finger_space": 180,
    "mom_finger_width": 160,
    "res_min_width": 500,
    "res_value_tol_ppm": 5000,
    "tie_max_dist_nm": 20000,
    "tie_max_dist_nm_source": "ihp_sg13g2.deck LU.a, LU.b missing_tie max_distance 20um. Read by cells::mosfet::taps_in_reach (CELL-13).",
    "licon_poly_enc": 70,
    "polycon_to_diff_spacing": 140,
    "resistors": {
        "default": "rsil",
        "recipes": {
            "rsil": {
                "model": "rsil",
                "layers": {
                    "rpoly": "polyres",
                    "rpoly_b": "res",
                    "rpm": "extblock"
                },
                "rpm_encloses_poly": 180,
                "rpm_min_width": 0
            },
            "rppd": {
                "model": "rppd",
                "layers": {
                    "rpoly": "polyres",
                    "res_block": "salblock",
                    "res_implant": "psd",
                    "rpm": "extblock"
                },
                "rpm_encloses_poly": 180,
                "rpm_min_width": 0
            }
        },
        "note": "GPurify ihp_sg13g2.deck: rsil_body = ((gatpoly and polyres) and res) not psd not salblock, enclosed by extblock 180 (Rsil.e); rppd_body = ((gatpoly and polyres) and psd and salblock) not nsd; res_mk = polyres or res cuts gatpoly_c."
    },
    "waivers": {
        "multiple_drivers": "Analog: a node driven by two drains (a differential pair's output and its load, a mirror's input) is the circuit, not a fault. The rule counts distinct gate nets driving a net (a digital fan-in check)."
    }
})
}
