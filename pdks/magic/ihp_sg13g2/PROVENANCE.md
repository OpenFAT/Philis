# ihp_sg13g2 magic tech and netgen setup

- **Source**: https://github.com/IHP-GmbH/IHP-Open-PDK at
  `5e6d592e4002946a4616f798c357f0f3c06cf3b6`, `ihp-sg13g2/libs.tech/{magic,netgen}/`.
- **License**: Apache-2.0 (file headers: IHP PDK Authors 2025).

| file | from |
|---|---|
| `ihp-sg13g2.tech` | `libs.tech/magic/ihp-sg13g2.tech` |
| `ihp-sg13g2-{cifin,cifout,drc,extract}.tech` | the files it `include`s, same directory |
| `ihp-sg13g2_setup.tcl` | `libs.tech/netgen/ihp-sg13g2_setup.tcl` |

Same magic and netgen run as sky130 (`pdks/magic/sky130/PROVENANCE.md`);
lengths carry a `u` suffix. Not vendored: the device generator Tcl, fill
and seal scripts.

## Engine difference: CntB rows on square contacts

magic applies the ContBar block (`ihp-sg13g2-drc.tech:402-407`, `exception
CONTBAR`: CntB.h1 metal enclosure 0.05 um, CntB.b spacing 0.28 um) to
square 0.16 um contacts. On `pair` (2026-10-25, magic 8.3.629) it reports
CntB.b on the tap row's cuts, 0.20 um apart, and CntB.h1 on S/D cuts whose
Metal1 is flush on one side. The DRM's ContBar is a non-square Cont
(KLayout `drc/ihp-sg13g2.drc:331`, `contbar = cont_nseal.non_squares`; the
cifin's `contbar` templayer is `not-square` too). The square-Cont rules that
do govern these cuts (Cnt.b 0.18 um spacing; M1.c 0, M1.c1 0.05 um endcap)
pass under KLayout and GPurify. Read with `gds maskhints yes`, magic stores
the hint as `MASKHINTS_contbar`, the templayer's lower-case name, while the
DRC block names `CONTBAR`. Philis does not patch the vendored tech:
`engine/tests/foundry.rs` names these two rows in the IHP magic gate and
fails on any other.
