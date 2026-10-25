# The `# PHILIS:` deck edits as `DeckBuilder` overrides (Q59)

Inventory for the M0 item "the `# PHILIS:` deck edits as `DeckBuilder`
overrides". Read on 2026-10-25.

Sources. Upstream decks: GPurify at the pinned rev `687c8bc8bf9468095a4104a05075cf44e762532b`
(`Cargo.toml`), the `pdks/` of its cargo checkout
(`~/.cargo/git/checkouts/gpurify-92358cf428c48214/687c8bc/pdks/`), which the
dependency also exposes byte for byte as `gdsverify::pdks::{SKY130, GF180MCU,
IHP_SG13G2, GENERIC_FINFET}` (`src/pdks.rs`, `include_str!` of the same files).
Edited decks: `pdks/decks/*.deck` in this repo. GPurify's own HEAD
(`40ff66c`, the owner's checkout) carries none of the edits below yet.

## 1. The diff

`diff upstream edited` per deck. Line numbers are of the edited deck.

- `gf180mcu.deck`, `ihp_sg13g2.deck`: only line 1, the vendoring header. No edits.
- `sky130.deck`: 10 edits.
- `asap7.deck` (upstream `generic_finfet.deck`): 5 edits.

## 2. Every edit, classified

Classes: **(a)** expressible as a `DeckBuilder::from_deck` override applied
after compiling the upstream deck; **(b)** needs a GPurify statement or feature
that does not exist (a GPurify issue); **(c)** better left as is.

| # | Deck:line | What it changes | Why (its comment) | Class | Builder call |
|---|---|---|---|---|---|
| 1 | all:1 | Vendoring header naming the upstream commit | Provenance of a copy | (c) obsolete: no copy ships | none |
| 2 | sky130:95-96 | Adds drawn `layer npn = gds(82, 20)` | npn marker, `sky130.lvs:744`; needed by #9 | (a) | `layer("npn", (82, 20))` |
| 3 | sky130:304-306 | `lvtn.10` inner layer `lvtn` becomes `lvtn.interacting(nwell_novar)` | GPurify measures an inner that no host covers as enclosure 0 (a violation), so an LVT nfet's lvtn (off every such well) is selected out | (a), and a GPurify issue for the engine behaviour | `replace("lvtn.10", noncoincident_enclosure(lvtn.interacting(nwell_novar), nwell_novar, 380nm))` |
| 4 | sky130:594-595 | `ar.met3.1` sidewall 800 nm to 845 nm | = the deck's own `pex met3 thickness`, and `sky130A.tech:4890 'height allm3 2.7861 0.845'` | (a); an upstream deck-data fix | `replace("ar.met3.1", antenna_electrical(..))` |
| 5 | sky130:601-602 | `ar.met4.1` sidewall 800 nm to 845 nm | = `pex met4 thickness`, as #4 | (a); upstream fix | `replace` |
| 6 | sky130:608-609 | `ar.met5.1` sidewall 1200 nm to 1260 nm | = `pex met5 thickness`, as #4 | (a); upstream fix | `replace` |
| 7 | sky130:620-623 | `soft_connection` gains `psub` | GPurify keeps a global layer in the hard connectivity (its docs say it drops it), so a supply tied to the substrate through ptap read as two pieces | (a), and a GPurify issue (global layer vs docs) | `replace("soft_connection", soft_connection(&[ntap, ptap, nwell, pwell_iso, psub]))` |
| 8 | sky130:662-670 | Adds derived `lu_ndiff`, `lu_ndiff_dnw`, `lu_pdiff` and rules `LU.2`, `LU.2.1`, `LU.3` (`tap_distance <= 15um`) | Latch-up, `sky130A.tech:4163-4170`, operands as magic's `ptap_missing`, `dptap_missing`, `ntap_missing` | (a); Q59 item 1 says upstream | `derived(..)` x3, `add("LU.*", max_distance_to_tap(..))` x3 |
| 9 | sky130:740-746 | Adds `device bjt pnp` (W3p40L3p40, terminals ntap, psd, ptap) and `device bjt npn` (W1p00L1p00, ptap, nsd, ntap) | Vertical PNP/NPN as `sky130.lvs:744-745, 1063-1065, 1089-1091`; one size each because two rows on one marker, the earlier wins, and there is no emitter-area select | (a); the emitter-area select for the other sizes is (b) | `device(DeviceKind::Bjt, ..)` x2 |
| 10 | sky130:794-802 | Adds `pex` rows `rbody_po` (sheet 48.2), `rbody_high_po` (317.3885), `rbody_xhigh_po` (2000), geometry columns = `poly_c`'s | Sheet = the model cards' rsh; head and width bias stay in the model card | (a); Q59 item 1 says upstream | `pex(..)` x3 |
| 11 | sky130:906-909 | Comment: "the bipolars have no recogniser" becomes "only the W3p40 PNP and W1p00L1p00 NPN are recognised" | Keeps the coverage note true after #9 | (c): a comment; goes upstream with #9 | none |
| 12 | asap7:320-328 | Adds derived `SD = (ACTIVE not GATE).interacting(SDT)` and `GATE_C = (GATE_UNCUT or LIG).interacting(LIG)` | Front end connected so LVS extracts transistors; SD/GATE_C/LISD and the SDT, V0 cuts have no pex row, so their resistance is unknown, never zero | (a) | `derived(..)` x2 |
| 13 | asap7:329-333 | Adds derived `CHANNEL_C = CHANNEL.interacting(GATE_C)` | A channel under an uncontacted (PODE) gate is no transistor | (a) | `derived(..)` |
| 14 | asap7:334 | `connect conductors` gains `SD, GATE_C, LISD` (in front of `M1..M9`) | As #12 | (a); the builder appends, so the list order differs (nothing reads it as ordered) | `conductors(&[sd, gate_c, lisd])` |
| 15 | asap7:336-340 | Adds `connect via SDT [SD, LISD]`, `V0 [LISD, M1]`, `V0 [GATE_C, M1]` | The front-end cuts [DRM 2.2] | (a) | `via(..)` x3 |
| 16 | asap7:359-379 | Adds 8 derived MOS markers (`{n,p}mos_{rvt,lvt,slvt,sram}`) and 8 `device mos` rows, terminals `[GATE_C, SD, SD]` | MOS recognisers as the deck's own "needs: sourced MOL and gate resistance" block lists them; 3 terminals since well and substrate are not conductors | (a) | `derived(..)` x8, `device(DeviceKind::Mos, ..)` x8 |
| 17 | asap7:443 | Comment above the upstream "needs" note: "done above" | Annotation | (c): a comment | none |

Totals: 14 (a), 0 (b), 3 (c). No edit needs a GPurify statement that does not
exist; two (#3, #7) work around GPurify engine behaviour and one (#9) is
limited by a missing feature, so each still has a GPurify issue behind it.

### GPurify issues behind the (a) rows

1. `enclosure` measures an inner no outer covers as 0 (#3). Either skip it or
   say so in `docs/deck.md`; the deck then drops the `interacting` select.
2. A `connect global` layer stays in the hard connectivity although the docs
   say it is dropped (#7).
3. BJT recognition by emitter area, so one marker can carry every size (#9).
4. Upstream the deck data: #4-#6 (sidewalls = the deck's own pex thickness),
   #8 (LU rules), #9-#10 (bipolars, poly resistor sheets), #12-#16 (ASAP7
   front end). Each override then leaves `crates/kernel/verify/src/overrides.rs`.

## 3. The `DeckBuilder` API used

`gdsverify::check::deck` (GPurify `crates/check/src/deck/builder.rs`, `rules.rs`, `decompile.rs`):

- `compile(text, grid, strings) -> CompiledDeck`; `DeckBuilder::from_deck(&base, &mut strings)` starts from it.
- Layers: `layer(name, (l, d))`, `derived(name, expr)`, `get(name)`, `replace_layer`, `remove_layer`. Expressions are `LayerExpr` (`and`, `or`, `not`, `group` = parentheses, `interacting`, ...).
- Rules: `add(id, row)` (appended after the base rules, sorted by id), `replace(id, row)` (keeps place and id), `remove(id)`. A `RuleRow` comes from the typed constructors in `rules`, named by engine kind (`noncoincident_enclosure`, `antenna_electrical`, `soft_connection`, `max_distance_to_tap` = the deck's `tap_distance`).
- Connectivity and devices: `conductors`, `via`, `global`, `label`, `device(kind, marker, model, terminals, DeviceValue)`, `remove_device`.
- PEX: `pex(layer, StackSource)`, `replace_pex`, `remove_pex`.
- `finish() -> CompiledDeck`, the same lowering and validation as text.
- `print(&CompiledDeck, &strings) -> String` and `CompiledDeck::source(&strings) -> DeckSource` decompile it.

What the builder cannot do: rewrite a `connect conductors` list in place
(only append, #14), or place an added layer, rule or device at a chosen
position. Neither matters here: drawn layer ids are by name, the added derived
layers come after the base's (their ids shift, nothing in Philis reads a
derived id by number), and no added device shares a marker with a base one.
