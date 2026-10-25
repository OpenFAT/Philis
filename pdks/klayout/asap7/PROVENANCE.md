# asap7 KLayout DRC deck

- **Source**: https://github.com/The-OpenROAD-Project/OpenROAD-flow-scripts at
  `64939f3717ad7c694199ba04cb98924b98022868` (2026-10-25),
  `flow/platforms/asap7/drc/asap7.lydrc`.
- **License**: BSD-2-Clause, Copyright (c) 2021 laurentc2 (its header), based
  on https://github.com/laurentc2/ASAP7_for_KLayout after the ASAP7 DRM
  `asap7_drm_201207a.pdf`.

What Philis runs: `klayout -b -r asap7.lydrc -rd in_gds=<gds> -rd
report_file=<lyrdb>`, through `verify::foundry::drc` only.

No open LVS deck exists for asap7 (ORFS ships DRC only; the ASAP7 PDK's
own decks are Calibre), and no magic tech file or netgen setup: the
KLayout and magic sign-off engines refuse asap7 with `foundry/no-deck`.
