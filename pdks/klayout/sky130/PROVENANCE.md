# sky130 KLayout decks

- **Source**: open_pdks build of sky130A by volare, version
  `1341f54f5ce0c4955326297f235e4ace1eb6d419` (the volare/ciel version is the
  open_pdks commit it built; its `.config/nodeinfo.json` records open_pdks
  `10d637bc671d3859099970970ec059caccd83cab` and `klayout_sky130`
  `5e6e2b29d2b9f8d0f2de1b5d5f649f33017dac75` as references). Copied from
  `sky130A/libs.tech/klayout/` of that build (Philis 1.0's `.pdk/sky130A`).
- **Upstream repos**: https://github.com/RTimothyEdwards/open_pdks (packaging),
  https://github.com/efabless/mpw_precheck (sky130A_mr.drc),
  https://github.com/efabless/sky130_klayout_pdk (sky130.lvs).

| file | from | license |
|---|---|---|
| `sky130A_mr.drc` | `libs.tech/klayout/drc/sky130A_mr.drc`, release 2023.6.14_01.08 | GNU GPLv3 (its header). Run as data by KLayout, never linked; the notice stays in the file. |
| `sky130.lvs` | `libs.tech/klayout/lvs/sky130.lvs` (self-contained: no includes) | Apache-2.0, Efabless 2022 (its header) |

What Philis runs (`crates/kernel/verify/src/foundry/klayout.rs`):

```
klayout -b -r sky130A_mr.drc -rd input=<gds> -rd top_cell=PHILIS -rd report=<lyrdb> -rd feol=1 -rd beol=1 -rd offgrid=1 -rd thr=<n>
klayout -b -r sky130.lvs -rd input=<gds> -rd report=<lvsdb> -rd schematic=<ref.cir> -rd target_netlist=<ext.cir> -rd run_mode=deep -rd schematic_simplify=true -rd thr=<n> [-rd lvs_sub=<nmos bulk net>]
```

`feol`/`beol` default to false in the DRC deck: without both it checks
nothing. The LVS deck's substrate is an empty layer named globally
(`lvs_sub`, default `sky130_gnd`), so a tap is never what connects an NMOS
bulk; Philis names it after the reference's one NMOS bulk net. This deck
therefore cannot see a missing p-tap; GPurify and magic can.
