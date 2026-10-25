# Testing

Oracles first, inputs second. Every layer names the independent thing that says
an output is wrong; a bug found by luck means an oracle is missing, and the fix
is a new oracle or a new input generator, not one more example. Round 3 of
`DECISIONS.md` (Q39 to Q50) accepted this plan as written; Q53 made performance
a tracked number rather than a gate.

**In one sentence each.** Input: a SPICE netlist (plus an optional ALIGN-style
sidecar, an interface file, an operating point) and a PDK deck. Output: a GDS,
an LVS reference netlist, a signoff report, a claims report, and optionally a
Rust generator program. Correct: the GDS signs off (DRC 0, ERC 0, LVS MATCH)
on the real decks, every claim the recipes made is met by the drawn geometry
and the extracted parasitics, and the hot path either produced that or refused
with a coded diagnostic.

## Domains

| component | domain | bug classes that matter |
|---|---|---|
| parser, sidecar reader, deck loader | compiler frontend / parser | crash on malformed input; accepting invalid, rejecting valid; silent misread of a value or unit |
| annotator | compiler semantic analysis | false positive and false negative constraints; wrong confidence; non-permutation-invariant output |
| recipes, device blocks, floorplan, compaction | geometry / EDA | claim violated silently; DRC-dirty geometry; off-grid; wrong on a PDK not tested; degraded without a report row |
| router | geometry / EDA | opens, shorts, unlanded pins; asymmetric differential routes; EM under-width |
| claim scorer, mismatch math | numerical | wrong moment, wrong sigma; NaN and overflow |
| signoff | physical verification | missed violation (worst case); rule silently not run |
| printer | code generator | emitted Rust does not compile or rebuilds a different call tree |
| CLI, modes, report | stateful pipeline | fallback where a refusal was owed; report row missing |

## Bug class → oracle → layer

| bug class | oracle | layer | tier |
|---|---|---|---|
| geometry violates a claim (asymmetry, centroid off, unequal environment) | `RuleBatch` in `kernel/analog`, computed from drawn geometry independently of the recipe (Q2, Q40) | seeded-violation test per claim kind: clean tile, inject one defect, exactly that claim fails | merge |
| geometry DRC- or ERC-dirty | GPurify in-process on the real decks (certificate check) | conformance matrix: every recipe × proof PDK × parameter grid (Q28) | merge (sky130, asap7); nightly (gf180mcu, ihp) |
| GPurify deck gap | foundry KLayout, magic, netgen on the same GDS (differential), in process through `verify::SignoffEngine` | `engine/tests/foundry.rs`, `substrate3/tests/sky130_inverter.rs` (`signoff_xcheck.py` from 1.0 kept for its PEX calibration) | nightly |
| LVS mismatch | GPurify against the LVS reference built from what was drawn (merge); KLayout LVS and netgen against the same reference (nightly) | acceptance fixtures | merge, nightly |
| claim met in geometry but broken by parasitics (lead C, path R) | extracted network from GPurify PEX (Q29) | acceptance fixtures: differential pairs' lead C and path R equal within tolerance | merge |
| recipe wrong on an untested PDK | the same conformance test on the second proof PDK (Q14) | conformance matrix | merge |
| silent degradation or fallback | report rows (`ladder_step`, refusal diagnostics) asserted by the test | every acceptance fixture asserts its expected ladder steps and that no refusal occurred; a fixture with a missing recipe asserts the refusal | merge |
| annotator false positive / negative | gold corpus: 69 ALIGN `*.const.json` plus 1.0's `corpus.rs`, extended with expected confidence and recipe eligibility (Q46) | scoreboard test, catalog on and off | merge |
| annotator order dependence | metamorphic: permute netlist lines, rename instances → identical `intent` and `recipe_calls` | property test with a seeded shuffler | merge |
| symmetric circuit laid out asymmetrically | metamorphic: swap the two halves' names in the netlist → mirrored call tree and mirrored geometry | property test on symmetric fixtures | merge |
| wrong pattern math | exhaustive: every member-count vector up to 16 units → moments cancel to the promised order; spec examples from the papers (Hastings, Karmokar) | unit tests in `kernel/analog::matching` | local |
| compaction breaks an invariant | property: random tile sets with axes and centroid groups → after compaction, mirror pairs still mirror, groups rigid, environment distances not below the recipe's, DRC clean, every claim re-met (Q24) | property test with 1.0's `SplitMix64`, no `proptest` (Q44) | merge |
| parser or reader crash or misread | coverage-guided fuzz with `cargo-fuzz`: SPICE, sidecar JSON, deck text, GDS bytes into the visualizer (Q45); oracle: no panic, every rejection carries a diagnostic, value round-trips | fuzz targets with a checked-in corpus | merge replays the corpus; nightly fuzzes |
| printer drift | round-trip: `recipe_calls → Rust → cargo check → run → recipe_calls'` equal (Q47) | one generated crate per acceptance fixture | nightly |
| output not reproducible for a seed | run twice with the same seed, compare `recipe_calls` and the GDS (Q33) | acceptance fixtures | merge |
| hot path slower than its target | wall per fixture against the 10 s OTA target; +20 % alerts (Q48) | benchmark harness, `progress.py` | tracked on every push, reviewed at release, gates nothing until the acceptance set passes (Q53, Q56) |
| free-form and recipe modes disagree on signoff | both modes must sign off every acceptance fixture (Q43) | differential run | merge |
| overflow, NaN, out-of-range | `debug_assert!` contracts, `overflow-checks` on in test builds, saturating arithmetic where 1.0 found overflows | all debug tests | local |
| rule silently not run | coverage rows in the signoff report (1.0's `Coverage`); a test asserts the set of rules run equals the deck's set | acceptance fixtures | merge |

No oracle above recomputes the answer the way the code under test does: the
rules read geometry, the signoff reads GDS, the foundry tools read nothing of
ours.

## Definition of done for a feature (Q39)

A feature lands with all four, checked in the PR:

1. its self-check or unit test, naming its oracle from the table above;
2. a fixture in the acceptance set, or an existing one, that exercises it end
   to end through the spine;
3. a report row the user can see in `report.txt` (a claim status, a ladder
   step, a coverage row, a diagnostic);
4. a negative control: an input that must fail, and does (plan-09's resized
   schematic failing LVS is the model).

A feature without a negative control is untested: a test that cannot fail
proves nothing.

## Harnesses

- `kernel/substrate3/tests/conformance.rs`: for every `RecipeKind` and every
  proof PDK, draw the parameter grid, run GPurify DRC and ERC, LVS against the
  tile's devices, score claims, write `availability.toml`. The checked-in table
  must equal the generated one, so a recipe that stops passing on a PDK fails
  the build rather than quietly disappearing from the matrix.
- `kernel/analog/tests/seeded_violations.rs`: one test per `RuleKind`.
- `frontend/annotator/tests/gold.rs`: the scoreboard; the expected files live
  beside the fixtures.
- `benchmarks/`: fixtures (`*.spice`, `*.const.json`, `*.interface.json`), the
  bench binary that runs both modes and writes `signoff.json`, `claims.json`,
  `calls.json`, an SVG per fixture (Q49), and `progress.py` for the gate and
  the progress page.
- `fuzz/`: `cargo-fuzz` targets and corpus.
- `frontend/engine/tests/deps.rs`: forbidden dependency edges over `cargo metadata`.
- `frontend/engine/tests/roundtrip.rs`: the printer round-trip, nightly.

## Gates (Q42, Q50)

| tier | budget | contents |
|---|---|---|
| local | < 10 s | `cargo fmt --check`, `cargo clippy`, unit and small-iteration property tests |
| merge (required) | < 15 min | every unit, property and exhaustive test; conformance on sky130 and asap7; acceptance fixtures through both modes with in-process signoff; fuzz corpus replay; seeded reproducibility; `deps.rs` |
| nightly | hours | foundry cross-check; gf180mcu and ihp conformance; long fuzz; printer round-trip; high-iteration properties; perf trend |
| release | unbounded | exit criteria below on the release commit |

One entry point, `ci/run.sh <tier>`, over the Philis crates (`analog verify
annotator visualizer placement routing substrate3 engine`, never the vendored
Substrate2 members); every step runs and the tier fails if any did:

| tier | today |
|---|---|
| local | `cargo fmt --check` (reports a count, not gating yet), `cargo clippy --all-targets`, `cargo test --lib --no-fail-fast` on every crate but substrate3 (its lib tests DRC every device variant, ~160 s, and run at merge) |
| merge | `cargo test --no-fail-fast`: every lib and integration test, `engine`'s `pair`, `conformance` and `deps` included |
| nightly | merge, then `cargo test --no-fail-fast -- --ignored` (verify's field solves; the foundry engines' runs, which need `klayout`, `magic` and `netgen` on `PATH`: the dev shell has them) |
| release | nightly with `--release` |

`.github/workflows/ci.yml` runs `nix develop --command ci/run.sh merge` on push
and PR and `nightly` on a daily schedule. Outside the dev shell set `PROTOC`
(the script defaults to a nix store path). Not wired yet: fuzz replay, perf trend. `local` runs in ~10.5 s once built (placement's
lib tests ~4 s, cargo start-up the rest).

Main always passes. A flaky test is fixed or quarantined the same day, and a
quarantined test on a core crate blocks release. When the acceptance set
outgrows the merge budget, fixtures rotate to nightly by size and are never
dropped.

## Foundry engines (the differential oracle in process)

`verify::foundry::signoff(engine, …)` signs the same shapes, labels and
reference off with GPurify, the foundry's KLayout decks, or magic plus
netgen (`pdks/klayout/`, `pdks/magic/`, each with its `PROVENANCE.md`), and
returns the same `Signoff` table: `drc/<rule>` rows, `lvs/mismatch`, and
`erc`, `pex`, `em_ir` coverage rows the foundry engines do not check. A
missing tool or deck is a `foundry/…` refusal, never a fallback and never a
pass; a KLayout deck that exits non-zero without a verdict is refused even if
it left an empty report (IHP's DRC did, before its rule JSON was vendored).
The report parsers have unit tests on captured reports; the runs are
`#[ignore]`d and run nightly.

Results on `pair` (`engine/tests/foundry.rs`), 2026-10-25, KLayout 0.30.8,
magic 8.3.629, netgen 1.5.320:

| PDK | GPurify | KLayout | magic + netgen |
|---|---|---|---|
| sky130 | clean, match | clean, match | clean, match |
| gf180mcu | M2.1 ×2, match | M2.1 ×2, match | M2.1 ×2, CO.7 ×10, CO.3 ×1, match |
| ihp_sg13g2 | clean, match | clean, match | CntB.b ×10, CntB.h1 ×13 (engine difference), match |
| asap7 | clean, match | DRC only: ACTIVE.S.2A ×2 | refused (`foundry/no-deck`) |

What the differences say, open:

- gf180mcu M2.1: all three agree; a routing defect, not a deck gap.
- ihp_sg13g2, closed: the IHP deck excludes any gate under drawn nSD (7/0,
  the rhigh implant) from MOS extraction (`mos_exclude`), and SG13G2 dopes
  n+ wherever pSD is not drawn, so Philis's IHP cells draw no nSD (the role
  is gone from its `cell.layers`) and GPurify's deck excludes the gate too
  (`overrides::ihp_sg13g2`; `pair.rs` lays nSD over the gates and fails
  LVS). M1.b was the router: cell metal closed lattice nodes by half a wire,
  not half the 430 nm via pad, and not at all on a pin's stitch nodes.
- asap7 ACTIVE.S.2A (92 nm between the active regions of different
  transistors): GPurify's asap7 deck has no such rule; to be decided against
  the ASAP7 DRM.
- magic's gf180mcu CO.7/CO.3 rows: KLayout ran the same rules and passed
  them; magic's contact model, not confirmed defects.
- magic's IHP CntB rows: an engine difference. magic applies the ContBar
  rules to square cuts, but the DRM's ContBar is a non-square Cont
  (`pdks/magic/ihp_sg13g2/PROVENANCE.md`). The IHP magic gate names the two
  rows and fails on any other.
- sky130's KLayout LVS names the NMOS bulk globally (`lvs_sub`), so it
  cannot see a missing p-tap; GPurify and magic can.

The negative controls (a schematic twice as wide fails LVS; one routing
metal at half its minimum width adds a DRC finding against the unmodified
run) pass under both foundry engines on sky130, gf180mcu and ihp_sg13g2, and
under KLayout on asap7 (DRC).
When a PDK's `pair` signs off clean under an engine, its `pair_signs_off_…`
test is added beside sky130's.

## Exit criteria for v1

- Acceptance set (Q17): ota, bgr_core, dac4, strongarm, pair, mirror_ratio,
  rc_filter and every fixture added later sign off DRC 0, ERC 0, LVS MATCH on
  sky130 and asap7, in-process and by the foundry tools, through the recipe
  mode with no refusal and with every claim met.
- Every `RecipeKind` has a conformance row on both proof PDKs; the checked-in
  availability matrix equals the generated one.
- Every `RuleKind` has a seeded-violation test.
- Fuzz: each target ≥ 24 CPU-hours with no open crash and coverage plateaued.
- Printer round-trip passes for every acceptance fixture.
- OTA on sky130 wall recorded in the release note against the 10 s target; it does not block the release (Q53).
- Both modes sign off every acceptance fixture.
- Zero quarantined tests on `kernel/*`, `backend/*`, `frontend/annotator`,
  `frontend/engine`.
- A written list of known gaps in `ROADMAP.md`.
