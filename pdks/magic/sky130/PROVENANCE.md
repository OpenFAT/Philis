# sky130 magic tech and netgen setup

- **Source**: the same volare build as `pdks/klayout/sky130/` (open_pdks
  `1341f54f5ce0c4955326297f235e4ace1eb6d419`; nodeinfo references open_pdks
  `10d637bc671d3859099970970ec059caccd83cab`, magic
  `22fbb2865682485b9c4a1a5803af65d44c3585c6`).
- **Upstream**: https://github.com/RTimothyEdwards/open_pdks (`sky130/magic/sky130.tech`,
  `sky130/netgen/sky130_setup.tcl`, preprocessed for sky130A).

| file | from | license |
|---|---|---|
| `sky130A.tech` | `libs.tech/magic/sky130A.tech` | Apache-2.0, R. Timothy Edwards 2020 (its header; open_pdks) |
| `sky130A_setup.tcl` | `libs.tech/netgen/sky130A_setup.tcl` | Apache-2.0 (open_pdks) |

What Philis runs (`crates/kernel/verify/src/foundry/magic.rs`): `magic
-dnull -noconsole -T sky130A.tech` with a script on stdin (`gds read`,
`drc style drc(full)`, `drc check`, `drc listall why`, `drc count total`,
`port makeall`, `extract all`, `ext2spice lvs`, `ext2spice -o`), then
`netgen -batch lvs "<ext.spice> PHILIS" "<ref.spice> PHILIS"
sky130A_setup.tcl comp.out`. The reference is `Pdk::reference_spice` with
bare µm lengths, the unit this tech's `ext2spice lvs` writes.
