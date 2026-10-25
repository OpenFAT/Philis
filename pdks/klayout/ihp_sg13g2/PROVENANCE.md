# ihp_sg13g2 KLayout decks

- **Source**: https://github.com/IHP-GmbH/IHP-Open-PDK at
  `5e6d592e4002946a4616f798c357f0f3c06cf3b6` (2026-09-01, `git clone --depth 1`),
  `ihp-sg13g2/libs.tech/klayout/`.
- **License**: Apache-2.0 (repository `LICENSE`; file headers: IHP PDK Authors 2025).

| file | from |
|---|---|
| `drc/ihp-sg13g2.drc`, `drc/rule_decks/**` | `libs.tech/klayout/tech/drc/` (no `testing/`, `docs/`, `images/`) |
| `lvs/sg13g2.lvs`, `lvs/rule_decks/*.lvs` | `libs.tech/klayout/tech/lvs/` (no `testing/`, `images/`) |
| `sg13g2_tech_mod.json` | `libs.tech/klayout/python/sg13g2_pycell_lib/sg13g2_tech_mod.json`: the DRC rule values the deck reads |

What Philis runs:

```
klayout -b -r drc/ihp-sg13g2.drc -rd input=<gds> -rd topcell=PHILIS -rd report=<lyrdb> -rd run_mode=deep -rd threads=<n> -rd drc_json=sg13g2_tech_mod.json
klayout -b -r lvs/sg13g2.lvs -rd input=<gds> -rd report=<lvsdb> -rd schematic=<ref.cir> -rd target_netlist=<ext.cir> -rd topcell=PHILIS -rd run_mode=deep -rd disable_tap_extraction=true
```

`drc_json` is passed because the deck otherwise looks for it at
`../../python/sg13g2_pycell_lib/`; without it the deck raises and leaves an
empty report (Philis refuses a non-zero exit, so that never reads as
clean). Taps extract as `ntap1`/`ptap1` devices unless disabled, and the
reference has none. The reader upper-cases subcircuit names, hence the
upper-case top cell.
