# Roadmap

Build order for Philis 2.0. Each milestone is a vertical slice: a fixture goes
through the whole hot path and signs off before the next milestone starts.
Every feature inside a milestone meets the definition of done in `TESTING.md`
(test with a named oracle, fixture, report row, negative control). The order is
Q52 of `DECISIONS.md`, accepted. Performance is tracked, not gated (Q53).

| milestone | slice | gate |
|---|---|---|
| M0 skeleton | workspace, the moved crates (`analog`, `verify`, `substrate3` fork with `cells` as device blocks, `annotator`, `placement`, `routing`, `visualizer`), GPurify re-pinned at head with `verify` on `CompiledDeck` and the `# PHILIS:` deck edits as `DeckBuilder` overrides (Q59), CI with the four tiers wired, `pair` through the hot path with one set recipe (Adjacent) and a trivial floorplan | `pair` DRC 0, ERC 0, LVS MATCH on sky130 and asap7 in-process; the conformance harness runs and writes the availability matrix; the parser decision (Q37) made from the fixture-corpus run |
| M1 set recipes | MOS pair and mirror recipes: Adjacent, Interdigitated, Cc1d with edge dummies and split gates, two-row Cc2d; ladders; claims scored; bounded candidate pick against a width target | `pair`, `mirror_ratio`, `quad` clean on both PDKs; seeded-violation tests for MatchedSet, Symmetry, CentroidGroup, Environment; conformance matrix complete for M1 recipes |
| M2 floorplan and routing | symmetric row stack floorplan recipe; glue row; targets top-down; routing recipes mirror pair and star common node routed first, `dr` negotiates the rest; refusal diagnostics | `ota`, `tt_ota`, `ota_constrained` clean on both PDKs; `ota` wall recorded against the 10 s target; differential lead C and path R equal at signoff |
| M3 arrays | capacitor and resistor common-centroid recipes with dummy rings; BJT ratio arrays; array floorplan family | `dac4`, `dac4_mim`, `res_m2`, `rdiv`, `bgr_core` clean on both PDKs |
| M4 confidence and sidecar | `Confidence` in `intent`; the identification work (ID-01 to ID-18) that produces it; the confidence gate table; sidecar `Recipe`, `Confirm`, `Reject` keyed by names | gold scoreboard with confidence labels; `strongarm` clean; a Hint-only constraint provably changes no geometry |
| M5 printer | `recipe_calls → Rust`; round-trip harness | every acceptance fixture's emitted crate compiles and rebuilds an equal call tree |
| M6 compaction | research spike with a written verdict (constraint-graph baseline, stochastic on top, invariants of Q24); implementation only on a positive verdict | area reduction measured per fixture; every claim re-met; signoff clean after compaction |
| M7 free-form mode | 1.0's gp, dp, gr, dr flow as the selected mode, recipe tiles as fixed macros | both modes sign off every acceptance fixture; area and wall compared in the progress page |
| continuous | SVG per fixture in CI artifacts; progress page; `docs/RECIPES.md` grows with each recipe | |

## Acceptance set

From Q17, growing by owner decision, never shrinking:

`ota`, `bgr_core`, `dac4`, `strongarm`, `pair`, `mirror_ratio`, `rc_filter`.
Also carried from 1.0 as milestone gates: `tt_ota`, `ota_constrained`, `quad`,
`dac4_mim`, `res_m2`, `rdiv`. `tq_chain` stays a free-form case.

## M0 status (2026-10-25)

Gate met. Each item and where it lives:

- **pair:** DRC 0, ERC 0, LVS MATCH on sky130 and asap7, in process, with a
  resized-schematic negative control (`engine/tests/pair.rs`).
- **Conformance:** the harness writes `availability.toml`, where MosAdjacent
  holds on both proof PDKs (`engine/tests/conformance.rs`).
- **Parser decision (Q37):** recorded in DECISIONS.
- **GPurify:** pinned at 687c8bc; the `# PHILIS:` edits are DeckBuilder
  overrides (`verify/src/overrides.rs`).
- **CI:** four tiers (`ci/run.sh`, `.github/workflows/ci.yml`); the
  dependency DAG is enforced (`engine/tests/deps.rs`).
- **PDK input:** the decks only, with no sidecar JSON (Q59 addendum).

Left open in M0 scope: the remaining Q59 dedup of verify's re-derivations,
and typed fields for the `pdk_data` tables.

## M1/M2 progress (2026-10-27)

- **M1, mostly met.** Recipes MosAdjacent, MosCc1d, MosCc2d and
  MosInterdigitated (Hastings Table 13.3) on ladders as data (Q19), the first
  rung the deck draws taken; claims declared per recipe and scored by
  kernel/analog's rules (Q2, `engine/src/claims.rs`, seeded violations per
  rule kind); conformance over every kind, n 1–3, nf 1/2/4 (availability:
  all four on sky130; Adjacent and Cc1d on asap7, Cc2d/Interdigitated are
  planar). `pair`, `quad`, `mirror_ratio` clean on sky130 and asap7. A size
  the deck cannot draw exactly is a coded `device` refusal. Open: the Ratioed
  rung, the bounded candidate pick against a width target (Q3/Q22).
- **M2, started.** Floorplan library (targets, symmetric row stack, glue row
  with gaps from facing shapes and the fin-deck gate lattice); the engine
  takes the row stack from the annotator's mirror pairs on planar decks, else
  the glue row; fingers folded to the tap reach. Rows follow the annotator's
  vertical `Order`s (sidecar, then EXT-28 current chains; `steps[0]` at the
  bottom; H orders do not bind rows), each scored after placement as a
  floorplan claim by `analog::placement::Below`; a user order the rows break
  is a coded `floorplan` refusal, an extracted one a violated claim. `ota`
  stacks tail, pair, load from the bottom. `ota` clean on sky130. The
  block harness (`engine/src/interface.rs`, 1.0's interface format): boundary
  pins routed and labelled as LVS ports; with a die, each call takes its
  smallest variant that fits. `tt_ota` and `ota_constrained` clean on sky130
  (GPurify; KLayout and magic/netgen checked once). Open: `ota`, `tt_ota`,
  `ota_constrained` on asap7; dr lands pins only on its two lowest layers, so
  a harness pin above met2 carries a via stack down to met1 (dr should land
  from the pin's own layer); routing recipes (mirror pair,
  star) routed first; dr's off-lattice landing
  spacing in general (being made exhaustive and self-reporting).

## Known gaps (kept current)

- Found by the GPurify re-pin to `687c8bc` (M0). Fixed: the asap7 FinFET
  block (`6ca7be3`), end-caps on `adjacent` enclosures (`59a9fe8`), the
  high_po npc (`83e34ed`), sky130 lvtn.10 and soft_connection (`2f3ad69`).
  Open, each a user decision:
  - asap7 PEX refuses: LISD (and SD, GATE_C, the SDT and V0 cuts) have no
    sourced resistance; GPurify refuses unknown parasitics rather than
    reading 0. `pdks::asap7` and `pdks::flavoured_models_extract_as_themselves`
    fail on this alone. Searched and unsourced (docs/ref/research/asap7-mol-resistance.md):
    the MEJ paper, DRM, design-flows paper, Vashishtha's dissertation, BSIM-CMG
    cards, the encrypted asap7sc7p5t QRC tech, ORFS. Unblock: the owner
    registers for the ASAP7 Calibre PEX deck (asap.asu.edu; its MIPT carries
    the resistivities).
  - Fixed: sky130 x.25 (the `pnp` marker only in the foundry's named
    cells). A fixed-geometry PNP recipe draws its units in a cell named after
    its open-PDK model (`Macro::cells`), verify loads named cells through an
    in-memory GDS (GPurify tags cells only when flattening), and x.25 allows
    the renamed cells. A PNP of the netlist's own size is refused on sky130
    (`pnp_cells_only`). The engine draws no BJT yet; when it does, it must
    pick the recipe overlay and carry `Macro::cells` into signoff
    (`ExtractOptions::cells`).
  - Fixed upstream in GPurify `11b5578`: sky130 lvtn.10 (the deck measured
    what its note exempts) and `soft_connection` (it kept global layers in
    its hard connectivity); Philis's two workarounds are removed.
  - Fixed by the foundry cross-check (KLayout, magic): gf180mcu M2.1 (dr
    pin-access jogs below metal2's width), ihp_sg13g2 (nSD over nfet gates;
    GPurify's IHP deck gained the gate exclusion; dr blocked cell metal by
    half a wire, not half the via pad), asap7 ACTIVE.S.2A (92 nm between
    separate actives; GPurify's asap7 deck gained the rule), sky130 rpm.1b
    (a high_po body only at a deck-listed width; others refused).
  - Licence check, owner's call: `pdks/klayout/sky130/sky130A_mr.drc` is
    GPLv3 (run as data by KLayout, notice kept); everything else vendored is
    Apache-2.0 or BSD-2-Clause.
  - verify's two field-solve tests take ~110 s in release: nightly tier
    (`#[ignore]`).

- Carried failures from Philis 1.0 (`5b4fe98`), fixed by owner decision:
  `foreign_nets_route_around_a_matched_cell` (`KEEPOUT_COST` 2.0 → 4.0, the
  lattice via cost: crossing a soft blockage of ≥ 6 nodes now always loses
  to the four-via detour) and `a_shielded_victim_gets_both_side_tracks`
  (fixture pins off the victim's side tracks; `Shield::coverage` counts the
  union of the victim's runs, not via pads: 1.0).
  The annotator's `twelve_thousand_devices` budget is asserted in release
  only (`4896cb5`, 349 ms); `cap_array::extracted_bit_ratios_follow_the_weights`
  passes since `59a9fe8`.

- M6 compaction verdict: deferred (docs/ref/research/compaction-verdict.md).
  The clean fixtures are single rigid tiles (100 % utilisation); on multi-tile
  ota/two_ota a 1-D pass recovers only 3.6-4.4 %, all from `arrange` spacing
  every gap at the widest layer spacing (sky130 nwell 1270 nm vs 320-370 nm
  facing). That fix belongs to M2's floorplan (gap from facing layers); the
  rest of the whitespace is row assignment. Reopen when a clean fixture shows
  more than 10 % recoverable area without reordering tiles
  (`cargo run --release -p engine --example whitespace`).
- `Confidence` does not exist until M4; before it, the hot path runs on
  Pattern-origin constraints with the conservative recipe tier only.
- SCIR is the netlist IR (Q37); which parser fills it is decided by the M0 corpus run.
- The analog-only PDK numbers split three ways (Q59); each family in the second group is a GPurify issue until its statement lands.
- PEX-level claims are signoff-only (Q29); in-loop extraction returns when a
  fixture shows the estimate lying.
