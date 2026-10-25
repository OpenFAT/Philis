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

## Known gaps (kept current)

- Found by the GPurify re-pin to `687c8bc` (M0). Fixed: the asap7 FinFET
  block (`6ca7be3`), end-caps on `adjacent` enclosures (`59a9fe8`), the
  high_po npc (`83e34ed`), sky130 lvtn.10 and soft_connection (`2f3ad69`).
  Open, each a user decision:
  - asap7 PEX refuses: LISD (and SD, GATE_C, the SDT and V0 cuts) have no
    sourced resistance; GPurify refuses unknown parasitics rather than
    reading 0. `pdks::asap7` and `pdks::flavoured_models_extract_as_themselves`
    fail on this alone. Fix: a sourced number (the ASAP7 Calibre RCX deck from
    asap.asu.edu), or the front end leaves PEX's conductors.
  - Fixed on branch `pnp`: sky130 x.25 (the `pnp` marker only in the
    foundry's named cells). A fixed-geometry PNP recipe draws its units in a
    cell named after its open-PDK model (`Macro::cells`), verify loads named
    cells through an in-memory GDS (GPurify tags cells only when flattening),
    and x.25 allows the renamed cells. A PNP of the netlist's own size is
    refused on sky130 (`pnp_cells_only`). The engine draws no BJT yet; when
    it does, it must pick the recipe overlay and carry `Macro::cells` into
    signoff (`ExtractOptions::cells`).
  - Upstream GPurify: `noncoincident_enclosure` scores an unhosted inner 0,
    and `soft_connection` keeps global layers in its hard connectivity, both
    against its own docs. The deck works around each (`PHILIS:` lines).
  - verify's two field-solve tests take ~110 s in release: nightly tier
    (`#[ignore]`).

- Carried failures from Philis 1.0 (`5b4fe98`), both diagnosed as design
  calls, not router bugs (each a user decision):
  - `routing::dr::tests::foreign_nets_route_around_a_matched_cell`: a cost
    tie. Crossing the matched cell's soft blockage costs 27 (11 nodes ×
    `KEEPOUT_COST` 2.0 + 2 jogs × 2.5, the jog forced by net 1's pin on that
    row since RTE-28); going around costs 28 (4 vias × 4 + 12 rows). The
    search is exact (threshold ≈ 2.09). Fix: raise `KEEPOUT_COST` to ≥ 2.2
    (retunes every route), or decide foreign nets may cross a small matched
    cell and change the test.
  - `routing::dr::tests::a_shielded_victim_gets_both_side_tracks`: 0.95 is
    unreachable in this fixture. Net 2's pins sit on the victim's upper side
    track, so its end jogs and pads can never be shielded on both sides; even
    a fully covered trunk bounds coverage at 0.768 (measured 0.727, since
    `Shield::coverage` also counts via pads, overlaps included). Fix: move
    net 2's pins off the side track, or redefine the 0.95.
  The annotator's `twelve_thousand_devices` budget is asserted in release
  only (`4896cb5`, 349 ms); `cap_array::extracted_bit_ratios_follow_the_weights`
  passes since `59a9fe8`.

- Compaction is research (M6); until its verdict, the hot path's area is the
  atoll-snapped area.
- `Confidence` does not exist until M4; before it, the hot path runs on
  Pattern-origin constraints with the conservative recipe tier only.
- SCIR is the netlist IR (Q37); which parser fills it is decided by the M0 corpus run.
- The analog-only PDK numbers split three ways (Q59); each family in the second group is a GPurify issue until its statement lands.
- PEX-level claims are signoff-only (Q29); in-loop extraction returns when a
  fixture shows the estimate lying.
