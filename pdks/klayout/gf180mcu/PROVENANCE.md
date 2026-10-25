# gf180mcu KLayout decks

- **Source**: open_pdks build of gf180mcuD by ciel, version
  `1e3554706ddbb2529193bdabbc7128296f3cb44a` (`~/.ciel/gf180mcuD`); its
  `.config/nodeinfo.json` records gf180mcu_fd_pv
  `035e6d1f4bf0acfb60606b056e7d2159cba30fef`, gf180mcu_fd_pr
  `e11a8c98dfc905561d38813b1416e65a4b9280e9`, open_pdks
  `1a725397f00168b58e7087e423d339627a08329d`.
- **Upstream**: https://github.com/fossi-foundation/globalfoundries-pdk-libs-gf180mcu_fd_pv
  (`klayout/drc`, `klayout/lvs`), packaged by open_pdks into
  `libs.tech/klayout/tech/{drc,lvs}`.
- **License**: Apache-2.0, GlobalFoundries PDK Authors 2023 (file headers).

Files: `drc/gf180mcu.drc` and what it requires (`rule_framework.rb`,
`generic_layers.rb`, `options.rb`, `connectivity.rb`, `helper.rb`,
`rule_decks/*.rb`); `lvs/gf180mcu.lvs` and `lvs/rule_decks/*.lvs` it
includes. Not vendored: `testing/` (170 MB of regression layouts), the
Python runners, pcells.

What Philis runs:

```
klayout -b -r drc/gf180mcu.drc -rd input=<gds> -rd topcell=PHILIS -rd report=<lyrdb> -rd variant=E -rd run_mode=deep -rd threads=<n> -rd decks=all,-density
klayout -b -r lvs/gf180mcu.lvs -rd input=<gds> -rd report=<lvsdb> -rd schematic=<ref.cir> -rd target_netlist=<ext.cir> -rd topcell=PHILIS -rd run_mode=deep -rd metal_level=6LM -rd metal_top=9K -rd mim_option=B
```

Variant E (6 metals, 9 kÅ top metal, MIM option B) is the stack
`pdks/gpurify/gf180mcu.deck` draws. Density is a chip-level check, excluded as
GPurify defers it.
