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

## Known gaps (kept current)

- Carried failures from Philis 1.0 (`5b4fe98`), failing identically there:
  `routing::dr::tests::a_shielded_victim_gets_both_side_tracks`,
  `routing::dr::tests::foreign_nets_route_around_a_matched_cell`, and the
  annotator's `twelve_thousand_devices` wall-time budget (2 s, a debug-build
  timing assertion; Q53 makes performance a tracked number, so it moves to the
  benchmark tier). Each is fixed in its own commit, never inside a move.

- Compaction is research (M6); until its verdict, the hot path's area is the
  atoll-snapped area.
- `Confidence` does not exist until M4; before it, the hot path runs on
  Pattern-origin constraints with the conservative recipe tier only.
- SCIR is the netlist IR (Q37); which parser fills it is decided by the M0 corpus run.
- The analog-only PDK numbers split three ways (Q59); each family in the second group is a GPurify issue until its statement lands.
- PEX-level claims are signoff-only (Q29); in-loop extraction returns when a
  fixture shows the estimate lying.
