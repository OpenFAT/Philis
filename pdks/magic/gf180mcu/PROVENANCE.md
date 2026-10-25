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
