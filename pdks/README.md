# pdks

Process decks, one directory per sign-off engine (`verify::SignoffEngine`).

| directory | engine | contents |
|---|---|---|
| `*.deck` (moves to `gpurify/` at merge) | `Gpurify` | GPurify's decks, Philis's edits as `crates/kernel/verify/src/overrides.rs` |
| `klayout/<pdk>/` | `Klayout` | the foundry's KLayout DRC and LVS decks |
| `magic/<pdk>/` | `Magic` | magic's tech file and netgen's LVS setup |

Each foundry directory has a `PROVENANCE.md`: source repository, commit,
license, the files kept and the command Philis runs. Only what a run reads is
vendored: no regression layouts, cell libraries or binaries.

| PDK | KLayout DRC | KLayout LVS | magic + netgen |
|---|---|---|---|
| sky130 | yes | yes | yes |
| gf180mcu | yes (variant E) | yes | yes (tech variant D) |
| ihp_sg13g2 | yes | yes | yes |
| asap7 | yes (OpenROAD-flow-scripts) | none exists: refused | none exists: refused |

An engine with no deck for a PDK refuses with `foundry/no-deck`; it never
falls back to GPurify. The decks are found at this path in the checkout
(`verify::foundry::decks_root`), and the tools on `PATH` (the flake's dev
shell provides `klayout`, `magic` and `netgen`).
