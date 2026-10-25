# PDK sidecar keys, classified (2026-10-25)

The basis for removing `pdks/*.json`, which the owner asked for: decks
become top-level `pdks/<pdk>.deck`. The survey ran over the sidecars at
`7ab866b`.

## Classes

- **A**: the deck already states the fact. Delete the key once its reader
  derives the value from the deck.
- **A′**: the deck language can state it, but the deck does not yet. Add it
  as a `# PHILIS:` deck edit.
- **B**: Philis data that no deck statement can carry.
- **Bm**: the deck states the rule, but the sidecar raises its number.
  Deleting the key changes geometry, so it needs an owner decision.
- **C**: read by nothing. Delete.

| PDK | data keys | A | A′ | B | Bm | C | `_source` |
|---|---|---|---|---|---|---|---|
| sky130 | 77 | 6 | 3 | 47 | 2 | 19 | 30 |
| gf180mcu | 57 | 10 | 3 | 32 | 3 | 9 | 22 |
| ihp_sg13g2 | 58 | 10 | 3 | 33 | 2 | 10 | 24 |
| asap7 | 49 | 3 | 2 | 31 | 0 | 13 | 14 |

## Findings that shape the migration

- `parse_roles` copies every integer or bool `cell` key into `Pdk.rules`
  after the deck rules, and `rule()` searches in reverse. So a sidecar scalar
  beats a deck rule of the same name: the Bm keys.
- The engine draws MOS only, plus the antenna diode. The resistor, capacitor
  and BJT recipe tables reach production only through `recipe_layers` →
  `drawn_from`. `recipe()` has test callers only, and it drops float fields,
  so `k_a_pct_um` is unread.
- `sidecar::validate` enforces `_source` provenance at load. Its readers,
  `provenance()` and `unverified()`, are called only from tests.
- The `sidecar.rs` registry is stale:
  - its reader column names 1.0 paths;
  - it lists five keys no sidecar has: `em_front_row_cuts`,
    `latent_merge_nm`, `metal_family`, `res_self_heat_dt_k`, `res_tox_nm`;
  - the code reads three keys it does not register: `dti_max_spacing`,
    `dti_width`, `ecgr_min_width_nm`.
- asap7 copies sky130 numbers for devices its deck does not have: well
  spacing, diode gap, BJT, resistor, MOM, plate and cap unit.
- `retrograde_pwell_source` says "Read by no code", but `post_cell.rs:431`
  reads it.
- Three build steps take the deck name from the JSON `deck` field:
  `substrate3/build.rs` (`Pdk::builtin`), the `decks.rs` SIDECARS table, and
  `Pdk::deck_text`.

## Where each class goes

1. **Class policy goes to `kernel/analog`, as `const` tables by
   `MatchClass`.** These are identical across all four PDKs and come from
   Hastings §13.3:
   - the seven tier arrays: `wpe_clearance_nm`, `lod_moat_ext_nm`,
     `dummy_reach_nm`, `gate_ext_extra_nm`, `res_dummy_span_nm`,
     `res_width_floor_permille`, `res_length_floor_x`;
   - `bjt_ka_pct_um`, `vbe_tc_uv_per_k`, `dummy_gates_per_end`,
     `res_value_tol_ppm`.

   `Process::tier` then goes away.
2. **Per-process data goes to `kernel/verify`, as one typed `const` table per
   PDK.** It replaces the SIDECARS table and the registry, and covers:
   - mismatch and temperature coefficients: avt, abeta, svt, vt_tc,
     lod_kvth0, gate_cap;
   - the role and routing-stack map;
   - the recipe tables;
   - generator dimensions;
   - `retrograde_pwell`, `npn_isolation`, `inapplicable_rules`, `waivers`.

   `_source` strings become doc comments, and `Option` replaces "sourced".
3. **A′ goes into the decks as `# PHILIS:` edits:**
   - `em_derating` becomes the `electromigration` parameters;
   - `substrate_kind` and `epi_thickness_nm` become `pex_substrate`;
   - the resistor tc becomes `pex tc1`.
4. **A keys are deleted.** Two readers need deck reads first:
   - `dim` must read difftap.2 for `min_finger_width`, or sky130 drops from
     420 to 150;
   - `tie_max_dist_nm` must read the tap_distance or missing_tie rule.
5. **C keys are deleted:**
   - `cap_density_ff_um2`, `res_corner_squares`, `res_serpentine_aspect`;
   - `bjt_max_emitter_stripe`, `bjt_stripe_gap`;
   - `p_epi_thickness`, `p_well_depth`, `n_well_depth`;
   - `well_enclosure`, `well_spacing`, `dti`;
   - `sheet_tolerance`, `linewidth_control_nm`, `placement_space`;
   - the `_note` strings and dead `_source` strings.
6. **Bm keys: decided, stricter.** The owner chose "I prefer stricter deck
   rules" (2026-10-25). A key whose every reader means the deck rule now
   raises that rule in the deck, as an override in `overrides.rs`
   `stricter_margins` mirrored by a `# PHILIS:` edit in `pdks/<pdk>.deck`
   (outcome a). A key that is one structure's construction margin stays in
   `pdk_data` under a name that says so, with a `_source` naming the rule it
   exceeds (outcome b). A key the deck already beats at its reader is deleted.

   | PDK | key | deck | sidecar | outcome |
   |---|---|---|---|---|
   | sky130 | `m1_enc` | m1.5 60 | 65 | a: m1.5 is 65 in the deck; the key is deleted |
   | sky130 | `via_enclosure` | 55/85 | 300 | b: `cap_strap_via_enclosure_nm` |
   | gf180mcu | `m1_enc` | V1.4a 10 | 40 | deleted: `dim` already draws V1.4b's 60 |
   | gf180mcu | `via_enclosure` | 10 | 400 | b: `cap_strap_via_enclosure_nm` |
   | gf180mcu | `li_encloses_licon` | CO.6 5 | 40 | a: CO.6 is 40 in the deck; the key is deleted |
   | ihp_sg13g2 | `via_enclosure` | 10/50 | 360 | b: `cap_strap_via_enclosure_nm` |
   | ihp_sg13g2 | `via_spacing` | V1.b 220 | 420 | b: `cap_strap_via_spacing_nm` |

   - **Outcome a.** `m1_enc` and `li_encloses_licon` were read only as
     "metal over a contact cut, all round". That is m1.5's meaning and CO.6's,
     so the readers now take the deck value. The router's `cut_enclosure_pair`
     and the MOS `m1_land` read the same rule, so they grow with it.
   - **Outcome b.** `via_enclosure` and `via_spacing` have one reader: the
     strap column of the three-metal sandwich capacitor in
     `capacitor.rs` `Geom::new`. Raising a global via rule to 300-420 nm
     would fail every routed via. Three of the numbers are another rule's:
     400 is gf180 MIMTM.2, 360 is ihp MIM.d and 420 is ihp TV1.b.
   - **Behaviour changes.**
     - The sky130 contacted MOS pitch at L = 150 nm goes from 430 to 440 nm.
       The met1 landing pitch is now mcon 170 + 2 × 65 + m1.2 140.
     - At that pitch, a top gate pad's li came 160 nm from a dummy's li
       riser. That is under li.3's 170 nm. The 170 nm had held only through
       lattice snapping, so `mosfet.rs` now sizes a dummied end region to
       keep li.3 between them.
     - The ihp spacing used to be dead, because the reader preferred the
       deck's 290 nm. It is now maxed with the deck, so ihp strap cuts are
       420 nm apart.
   - **asap7.** asap7's `via_enclosure` 5 went with the rename. `dim` now
     reads the deck's end-cap there too, which is also 5 nm.
