# gf180mcu magic tech and netgen setup

- **Source**: the ciel build in `pdks/klayout/gf180mcu/PROVENANCE.md`
  (open_pdks `1e3554706ddbb2529193bdabbc7128296f3cb44a`; nodeinfo: magic
  `d157eea7f3c397271a0b6d1e5a98b06101284a13`).
- **Upstream**: https://github.com/RTimothyEdwards/open_pdks (`gf180mcu/magic/gf180mcu.tech`,
  `gf180mcu/netgen/gf180mcu_setup.tcl`, preprocessed for variant D).
- **License**: open_pdks, Apache-2.0; the tech file's header: Copyright (c)
  2022 Efabless, Inc.

| file | from |
|---|---|
| `gf180mcuD.tech` | `libs.tech/magic/gf180mcuD.tech` |
| `gf180mcuD_setup.tcl` | `libs.tech/netgen/gf180mcuD_setup.tcl` |

Variant D is the only one ciel builds next to E's 6-metal stack: it has 5
metals with an 11 kÅ top, so a layout using `metal5` and `metaltop` both is
outside this tech. `pair` uses metal1 and metal2 only. Same magic and
netgen run as sky130 (`pdks/magic/sky130/PROVENANCE.md`); this tech writes
lengths with a `u` suffix, so the reference carries one.

## Engine differences

- **CO.7 and CO.3 at the exact minimum.** The tech's `cifinput` grows each
  `CONT` by 5 nm (`grow 145`, `shrink 140`) to model CO.6's metal1 surround,
  then checks CO.7 at 145 nm and CO.3 at 65 nm. Import lands on a 10 nm grid
  and the 5 nm edge rounds half up, so only a contact's +x and +y edges move
  out. A contact drawn exactly 150 nm from a gate (CO.7) or exactly 70 nm
  inside poly (CO.3) reads 5 nm short on those edges and legal on the others.
  The same `pair` GDS shifted by 5 nm is clean. KLayout's deck runs both
  rules (`rule_decks/contact.rb`, DRM 0.15 um and 0.07 um) and passes them.
  These rows are magic's, not the layout's (`engine/tests/foundry.rs`).
- **CO.7's other layer.** magic spaces diffusion contacts from all poly
  (`allpoly`); KLayout and GPurify from the gate only (`tgate`, poly over
  COMP). magic is the stricter where field poly runs near a diffusion contact.
