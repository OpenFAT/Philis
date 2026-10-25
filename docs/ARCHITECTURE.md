# Architecture

Philis 2.0 transforms a SPICE netlist plus an optional constraint sidecar into
a signed-off GDS and a Rust generator program, by recognising the circuit's
analog constraints and realizing each one with a layout **recipe**. Free-form
place-and-route is a separately selected mode that shares the netlist, the
constraints, the device blocks, the router and the signoff.

Every line below cites the decision it comes from (`DECISIONS.md`, Q numbers).
Lines marked **open** depend on a Round 3 question.

## Module DAG

Crates are owners. An arrow means "depends on"; leaves first. The layout is the
owner's skeleton of 2026-10-25.

| crate | owns | depends on |
|---|---|---|
| `kernel/analog` | ids, tables, geometry (1.0 `pnr_core` folded in, **open Q36**); `Rule`, `RuleBatch`, requirements; `intent` with `Confidence` added; matching math (moments, mismatch, patterns); claim scoring | nothing |
| `kernel/verify` (**open Q36**) | decks (JSON sidecars per PDK), `Process`, GPurify DRC, ERC, LVS, PEX, `Checker`, coverage | analog |
| `kernel/substrate3` | the fork of Substrate2 with `atoll` and `sky130` (**open Q35**); typed PDKs for sky130, gf180mcu, ihp_sg13g2, asap7 generated from decks; capability traits (`TRAITS.md`); device blocks (1.0 `cells`) as tiles on all four PDKs (Q8, Q27); set recipes and their ladders (Q1, Q19); signoff adapter | analog, verify |
| `backend/placement` | floorplan recipes: symmetric row stack, array (Q21); glue row (Q20); targets solver (Q22); free-form placer (1.0 `gp`, `dp`) | analog, substrate3 |
| `backend/routing` | routing recipes: mirror pair, star common node, shield, keepout (Q23); negotiated router (1.0 `gr`, `dr`); EM width, antenna repair | analog, substrate3 |
| `backend/post-processing` | compaction (Q24, Q25; research first); later fill and density | analog, substrate3 |
| `frontend/annotator` | netlist to `intent`: identification, confidence (Q6, Q30), sidecar reader (Q7, Q31) | analog, substrate3 (SCIR) |
| `frontend/engine` | the two spines, the `recipe_calls` table (Q32), the printer (Q12, Q34), the CLI, modes (Q16, Q18) | all of the above |
| `frontend/visualizer` (**open Q36**) | GDS parse, SVG export, viewer, progress probe | analog, `gpurify-geom` only |

Enforced by: Cargo crates (a cycle is a build error) plus `frontend/engine/tests/deps.rs`
over `cargo tree` for the forbidden edges: the visualizer never links the GPurify
engine; `kernel/analog` never links Substrate.

## Spines

One function per pipeline, one line per transform. Branches that choose which
work happens live here; transforms are straight loops over tables.

- **Recipe mode** (`frontend/engine`, the hot path; Q1 to Q34):
  `parse → annotate → gate_confidence → plan_recipes → build_tiles → arrange →
  compact → route_recipes → route_rest → score_claims → signoff → report →
  print`.
  `plan_recipes` is the bounded pick against top-down targets (Q3, Q22) and the
  only place that degrades along a ladder (Q19) or refuses (Q18).
  `route_recipes` routes the constrained nets first by construction;
  `route_rest` negotiates the remaining nets around them (Q23).
- **Free-form mode** (`frontend/engine`, selected by the user; Q16, Q18):
  `parse → annotate → enumerate_blocks → gp → dp → gr → dr → signoff`, in
  epochs as in 1.0, with any recipe tiles entering as fixed multi-device
  macros (Q4).
- **Printer** (`frontend/engine`): `recipe_calls → rust_source` (Q12).
- **Conformance** (`kernel/substrate3` tests, Q28): for every (recipe, PDK) and
  the recipe's parameter grid: `build → drc_erc → lvs → score_claims →
  availability_row`.

## Tables

| table | owner (crate) | written by | read by | capacity | lifetime |
|---|---|---|---|---|---|
| `netlist` (SCIR library, **open Q37**) | engine (`parse`) | parse | annotator, plan, LVS reference | 65 535 devices and nets (u16 ids, as 1.0 asserts) | run |
| `intent`: sets, compounds, orders, net classes, guard-ring requirements, budgets, device facts; every row carries `origin` and `confidence` | annotator | annotate, sidecar | gate_confidence, plan_recipes, score_claims | one row per constraint, bounded by devices × families | run |
| `recipe_calls`: recipe kind, parameters, claimed constraint ids, parent, children, placement relation, ladder step taken (Q32) | engine | plan_recipes | build_tiles, arrange, printer, visualizer, sidecar | ≤ sets + compounds + 1 glue + 1 floorplan per block | run |
| `tiles`: outline, pins and abstract, unit table, device membership | substrate3 | build_tiles | arrange, route, score_claims | one per set-level call | run |
| `claims`: constraint id, rule kind, promised value, status (met, violated, unknown) | engine | recipes declare, score_claims fills status | report, conformance | one per (call, constraint) | run |
| `floorplan`: tile position and orientation, axes | placement | arrange | compact, route | one row per tile | run |
| `compacted`: positions after compaction | post-processing | compact | route | one row per tile | run |
| `routes`: wires, cuts, pads | routing | route_recipes, route_rest | signoff, score_claims | nets × layers | run |
| `signoff`: DRC, ERC, LVS rows, PEX caps, coverage | verify | signoff | report | findings | run |
| `availability`: recipe × PDK (Q28) | substrate3 | conformance tests | plan_recipes | recipes × PDKs | checked in, regenerated by tests |

## Seams

| seam | axis | rung | why the rung below fails | revisit when |
|---|---|---|---|---|
| S1 recipe kind | kind: Adjacent, Interdigitated, Cc1d, Cc2d, arrays, glue, floorplan families, routing recipes | 4: closed `RecipeKind` enum, exhaustive match in `plan_recipes` and the printer | rung 3 cannot hold a build function per kind | recipes loaded at run time (a plugin) → rung 9 |
| S2 PDK | backend: sky130, gf180mcu, ihp_sg13g2, asap7 | 5: `P: PhilisPdk` generic parameter, as Substrate does; capability trait impls generated from decks (Q26) | rung 4 would switch on the PDK inside every tile | a PDK whose rules cannot be expressed as deck keys |
| S3 claim kind | kind: every `Rule` in `kernel/analog` | 5: static dispatch per rule type through `RuleBatch`, as 1.0 | rung 4 would re-implement 1.0's batches | never; cold code, probe n/a |
| S4 degradation ladder | policy: per recipe family | 3: ordered rows of `RecipeKind` per family (Q19) | rung 2 would hide the ladder in code the report cannot quote | a ladder that depends on the PDK → add a column |
| S5 confidence gate | policy: confidence × class → allowed recipe tier (Q6, Q30) | 3: a table | rung 2 scatters the policy | the gate needs the spec → a transform |
| S6 compaction strategy | algorithm: constraint graph, stochastic on top (Q24) | 4: enum chosen in the spine | one case today; the second is the research spike | the spike's verdict |
| S7 mode | policy: recipe, free-form (Q16) | 1 and 4: one `Mode` enum matched once at the top of `engine::run` | it is one branch | never |
| S8 output | kind: GDS, Rust, SVG, report | 4: separate transforms, one spine line each | nothing varies inside them | never |
| S9 routing engine | backend | none: `dr` is the one router (Q23); concrete code | one case | atoll's router gains symmetry |

Every rung-5 seam here sits in cold code: layout generation runs once per
circuit, never per element in a hot loop, so the `asm_probe` rule of
`AGENTS.md` does not apply to them. The hot loops are inside `kernel/analog`
scoring and `backend/routing` negotiation, where 1.0's SIMD and branchless
rules continue to apply.

## Invariants

- **No engine fallback on the hot path.** A block without an available recipe
  is a coded refusal naming the block and the sidecar entry that would fix it
  (Q18). Free-form is selected, never entered by falling (Q16).
- **Every claim is checked, never trusted.** `score_claims` runs the same
  `RuleBatch` on recipe output as on free-form output, and GPurify signs off
  both (Q2).
- **Degradation only along a declared ladder, and always reported** (Q13, Q19).
  Precise or User-mandated constraints refuse instead of degrading.
- **Confidence gates recipe tiers.** Hint never changes geometry; a bare
  pattern match selects conservative recipes; Proven, Corroborated, User select
  precise ones; role defaults stop at Moderate (Q6, Q30).
- **One call tree.** `recipe_calls` is the only intermediate representation;
  executor, printer, visualizer and sidecar read the same rows (Q32).
- **Recipe tiles are atomic in free-form** (Q4).
- **Compaction preserves claims by construction:** mirrored tiles move as one
  about their axis; centroid groups move rigidly; matched-set environment
  distances never shrink below the recipe's value; every claim is re-scored
  after the pass (Q24).
- **Philis draws, Substrate types.** Device internals come from the device
  blocks on every PDK, sky130 included; upstream tiles are optional variants
  (Q8, Q27).
- **Static recipe types, dynamic top, no compilation inside the loop** (Q11).
- **Seeded reproducibility is a test property, not a product promise**
  (Q15, Q33).
- **Writes cross crates only through tables the owner drains.** Routing
  requests from recipes are rows in a request table that `backend/routing`
  consumes; no recipe calls the router.

## Adding a feature

- A new set recipe: one type in `substrate3` implementing `Recipe`, one
  `RecipeKind` variant (the compiler lists the match sites), one row in its
  family's ladder, one conformance test per proof PDK, one seeded-violation
  test per new claim kind.
- A new floorplan family: one type in `backend/placement`, one variant, one
  acceptance fixture that needs it.
- A new routing recipe: one type in `backend/routing` writing the request
  table, one variant, one claim kind in `kernel/analog` if it promises
  something new.
- A new PDK: one deck JSON (rules, layers, recipe-capability keys), the tile
  constructors in `substrate3/pdk`, the conformance run that fills its
  availability column.
- A new constraint kind: one `Rule` in `kernel/analog`, its emitter in
  `annotator`, its seeded-violation test; recipes that can realize it add it to
  their claims.
- A new sidecar entry: one reader arm in `annotator/sidecar`, keyed by names
  (Q31).
- Outside code needing to change a table: a request table its owner drains.

## Change drill (**open Q51**)

The three likely changes come from the owner. Candidates and their expected
edit lists; radius counts existing function bodies edited.

| ID | change | source | edits | radius | weight Δ | verdict |
|---|---|---|---|---|---|---|
| D1 | a new PDK (gf180mcu) through the hot path | candidate | `pdks/gf180mcu.json` keys; `substrate3/pdk`: tile constructors (new); conformance run (new rows) | 0 expected | +0 for callers (same `PhilisPdk` bound) | pending |
| D2 | a new set recipe: 2-D common centroid for resistors | candidate | `substrate3/recipes/set/res_cc2d.rs` (new); `RecipeKind` variant (compiler-listed match arms in `plan_recipes`, printer); ladder row; conformance tests | arms listed by the compiler | +1 kind | pending |
| D3 | a new routing recipe: shield tracks beside a victim | candidate | `routing/recipes/shield.rs` (new); request-table row kind; `Shield` claim already exists in 1.0's rules | 0 expected | +1 request kind | pending |

## Migration from Philis 1.0

Copies, one commit each, citing the 1.0 commit (**open Q38**). 1.0 remains the
history archive.

| 1.0 | 2.0 | how |
|---|---|---|
| `kernel/core` (`pnr_core`) | `kernel/analog` leaf module (**open Q36**) | copy; ids and tables unchanged |
| `kernel/analog` | `kernel/analog` | copy; add `Confidence` to `intent` |
| `kernel/cells` | `kernel/substrate3` device blocks | copy; each generator wrapped as a tile on all four PDKs (plan-09 P2 pattern) |
| `kernel/macroMaster` (`pdk`, `signoff`) | `kernel/substrate3` | copy into the fork; `legacy` not carried |
| `backend/verify` | `kernel/verify` (**open Q36**) | copy |
| `backend/annotator` | `frontend/annotator` | copy; then ID-01 to ID-18 for confidence (Q6) |
| `backend/gp`, `backend/dp` | `backend/placement` (free-form) | copy |
| `backend/gr`, `backend/dr` | `backend/routing` | copy; constrained-first ordering added (Q23) |
| `frontend/library::parse` | `frontend/engine` or the fork's `spice` (**open Q37**) | decided by the M0 corpus run |
| `frontend/library::elaborate` | `frontend/engine` recipe spine (`route_recipes`, `route_rest`) | the hot path's route step |
| `frontend/library::emit` | not carried (Q34) | the printer is new |
| `kernel/visualizer` | `frontend/visualizer` (**open Q36**) | copy |
| `benchmarks/fixtures`, `signoff_xcheck.py`, `progress.py` | `benchmarks/` | copy |
| `docs/ref`, textbook digests | `docs/ref` | done |
