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
