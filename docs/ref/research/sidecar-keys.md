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
6. **Bm keys: an owner decision.** Either keep each as a named Philis margin
   or drop to the deck minimum:

   | PDK | key | deck | sidecar |
   |---|---|---|---|
   | sky130 | `m1_enc` | m1.5 60 | 65 |
   | sky130 | `via_enclosure` | 55/85 | 300 |
   | gf180mcu | `m1_enc` | V1.4a 10 | 40 |
   | gf180mcu | `via_enclosure` | 10 | 400 |
   | gf180mcu | `li_encloses_licon` | CO.6 5 | 40 |
   | ihp_sg13g2 | `via_enclosure` | 10/50 | 360 |
   | ihp_sg13g2 | `via_spacing` | V1.b 220 | 420 |

   Until the owner decides, the migration keeps the current values, so
   geometry does not change.
