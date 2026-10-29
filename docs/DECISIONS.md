# Decisions: the design tree of Philis 2.0

This is the decision log. It was produced by grilling the owner round by round:
every question is one decision, numbered once and never renumbered. A round is
the frontier of questions whose prerequisites were already settled. Each entry
keeps the question as asked, the options, the recommendation, and the owner's
answer with any refinement. Later decisions append as new numbers; an answer is
never edited in place, a reversal is a new entry that names the one it reverses.

Status words: **accepted**, **accepted with refinement**, **open**.

Related: `ARCHITECTURE.md` turns these into owners, spines and seams;
`TRAITS.md` sketches the trait system; `TESTING.md` the oracles and gates;
`ROADMAP.md` the build order; `RECIPES.md` the initial recipe catalog.

---

## Premise (the owner's statement, 2026-10-25)

The key to a good analog layout engine is not raw movement of shapes and
routing algorithms. It is good **recipes**: known layout techniques, built in a
generator style like Substrate2, by extending its trait system to support
building them. A recipe exists to fix a problem, and the problem is defined by
the **annotator**: it defines the analog constraints and gives information about
them. The layout is then dumped in Rust using an extended Substrate2 library
whose traits support those constraints. Open question at the time: which new
traits give full degrees of freedom while keeping the generator guarantee of
being PDK-agnostic.

If the recipes are insufficient, or the user wants to explore the design space,
the classic place-and-route trial and error exists as a **cold path** for power
users, off the hot path.

Clarifications the owner added after Round 0:

- A recipe is a technique such as a common-centroid layout. The `cells`
  generators of Philis 1.0 (`cellgen`) are **not** recipes: they are
  foundational device blocks that give the free-form optimizer full degrees of
  freedom, and they stay for free-form.
- Those device blocks link to the new Substrate2 extension and ideally derive
  from it rather than being created separately.
- P&R has issues; the trait system and the recipes handle the owner's concerns.
  P&R is for free-form only.

---

## Round 0: the devil's-advocate critique

Research done before answering: the Philis 1.0 tree at `5b4fe98`, Substrate2 at
the pinned rev `01d9600` (substrate 0.10.3) including `atoll`, the sky130 tiles
and the StrongARM example, plan-09, the placement and routing audits, and the
benchmark table.

**Verdict.** The idea is half right, and the right half already existed in
Philis 1.0: `kernel/cells` is a device-block library, `cellgen` enumerates its
variants, `dp` picks among them, and `elaborate.rs` is already "a composition
places, the router routes". The new half, recipes at the composition level as
the default path with search demoted to a power-user mode, is the architecture
template-based analog tools tried in the 1990s and that ALIGN and MAGICAL both
abandoned for the structure 1.0 has.

### Where the premise holds

- Leaf recipes are the right unit. Pair, mirror, cascode stack, capacitor and
  resistor arrays are a finite vocabulary with closed-form matching math.
- Recipes are explainable and fast. A designer trusts an ABBA row with edge
  dummies because they can read it.
- Recipes can feed sizing. A recipe knows its own parasitics in closed form.

### Where it breaks (ranked by damage)

1. **The hot path is mis-located.** 1.0's benchmarks fail in routing and LVS,
   not in moving shapes. Recipes do not route between recipes. Master plan row
   16: symmetric routing is not exact, pin landing is first-come, the lattice
   is not aligned to the axis.

   | circuit | wall | DRC | LVS |
   |---|---|---|---|
   | ota | 138 s | 0 | MISMATCH |
   | dac4 | 74 s | 6 | PARTIAL(16) |
   | tq_chain | 96 s | 4 | MISMATCH |

2. **Composition is the long tail, and templates are brittle.** The catalog
   holds 91 topologies and an 80 KB research document exists on going
   catalog-free because 91 is not enough. A floorplan recipe per topology is
   the template approach; it broke when device sizes moved by an order of
   magnitude or a pin side changed.
3. **Recipes contain search anyway.** Substrate2's StrongARM tile calls a
   `ResizableGrid` solver to pick fingers per tile against a height budget. A
   recipe is a structured, low-dimensional search space with analog priors.
   Framed as "recipes versus P&R" you build two engines.
4. **The annotator cannot carry the weight alone.** Topology yields candidate
   constraints, not weights. A comparator input pair and a buffer input pair
   are the same subgraph with opposite layout needs ("a template name is not
   the electrical role", `research-constraint-identification.md`). A wrong
   recipe applied with confidence is a wrong layout, not a slightly worse cost.
5. **Full degrees of freedom and PDK agnosticism pull apart, and Substrate2
   does not promise layout portability.** Its documented portability is SCIR
   and `FromSchema` for schematics and simulators. For layout, `atoll` gives a
   gridded abstraction; the sky130 atoll tiles are MOS and taps only; no
   symmetry, centroid, interdigitation, dummy or guard-ring helper exists in
   `substrate`, `atoll` or `sky130`. The StrongARM "PDK-agnostic" generator is
   generic over a per-generator trait `StrongArmImpl`:

   | file | lines |
   |---|---|
   | strongarm `lib.rs` | 758 |
   | strongarm `tech/sky130.rs` | 873 |
   | strongarm `tiles.rs` | 107 |

6. **Grids fight planar analog.** FinFET is naturally gridded, so asap7 fits
   atoll. Planar 130 nm analog wants off-grid finger pitch, serpentine
   resistors, MIM units at exact pitch and guard rings.
7. **Static types versus a netlist-driven flow.** Substrate's value is typed
   `Io` and LVS by construction for authored generators; Philis is netlist-in.
   Options: codegen and compile per design (slow); a dynamic layer (discards
   the typed benefit); finite static recipe types with runtime parameters
   under a dynamic SCIR-bound top (viable).
8. **"Guarantee" is the wrong word.** A generic generator guarantees it
   compiles against any PDK implementing the trait. plan-09's own finding:
   minimum-width tiles are dirty on gf180mcu, ihp and asap7 contact enclosure.
   Portability of intent, never of correctness.
9. **Recipes conflict and need a global arbiter.** The pair, the load and the
   output routing must share one axis; proximity fights isolation; aspect
   fights the centroid grid. Whatever resolves it is a placer with symmetry
   groups.
10. **The cold path's justification is weak.** Annealing does not find
    revolutionary layouts; designers do. The cold path's value is coverage and
    floorplan what-ifs.
11. **Foundation risk.** Substrate is 0.x; the orphan rule makes every added
    trait need adapters on upstream tiles unless the code is forked.

### The trait question (Round 0 answer)

Layering: arrangement is PDK-agnostic data; realization is PDK-specific tiles;
legalization and routing sit on the grid plus rules.

| trait | question it answers | existed in 1.0 |
|---|---|---|
| MatchablePrimitive | unit geometry, finger split and interleave, diffusion sharing sides, allowed flips, dummy construction, environment reach, `Unit` emission | partly in `cells::mosfet` and the deck keys |
| ArrayablePrimitive | unit pitch, allowed grid aspect, dummy ring, lead and jumper points, plate rank | `cap_array`, `resistor`, `bjt` |
| GuardRing | ring kind by role, width, spacing, tie net | `GuardRingRequirement` |
| Lde | WPE, LOD and PSE reach, `sa/sb` emission | measured only |
| InterconnectModel | per-layer sheet R, C per µm, EM limits per layer and via | `verify::Pdk`; absent from `AtollLayer` |
| SymmetricRouter, BalancedRouter, Shield, Keepout | mirror pair, star common node, reference tracks, no metal over gates | `RepairKind` in `dr`; absent from atoll's `Router` |
| Arrangement | rows, order, axis, abutment, pin side | `Compound`, `Order`, `abutment` |
| Resizable, Foldable | fit a strength to a width or height | atoll has both |
| ConstraintCheck | score every recipe output with the same rule batches | `Rule`, `RuleBatch` |

Plus capability negotiation: a recipe declares what it needs; a PDK lacking a
capability yields a coded diagnostic and a stated degrade path, never a silent
fallback.

### Round 0 recommendations

1. Do not build two engines; let recipes expand the variant space upward.
2. Fix routing first; it is where 1.0 fails.
3. Gate recipes on confidence; make sidecar Confirm and Reject real.
4. Keep Substrate2 as the primitive, PDK and signoff medium; composition
   recipes as data; emit Rust as an export.
5. Prove agnosticism on sky130 and asap7 from day one.
6. Measure the nine stages before naming the hot path.

The owner's reply reshaped point 1 (recipes are not `cellgen`; see the
clarifications above) and kept the rest for the grill.

---

## Round 1 (17 decisions, 2026-10-25)

Facts checked before the round: `ArrayStyle` carries five values (Adjacent,
Interdigitated, CommonCentroid1d, CommonCentroid2d, Any); `Member` has parallel
and series unit counts; FLOW-11 places a multi-device block as one fixed macro;
no `Confidence` type exists; upstream `MosTile` has four knobs (w, length,
fingers, gate direction) against seven on 1.0's `Mosfet` (nf, style,
dummies_per_edge, split_gates, mirror_pins, rows, double_gate).

Settled from the owner's messages, not asked: a recipe is a known technique for
a constrained group, written as a Rust generator on Substrate2 traits; the
annotator plus the sidecar is the only constraint source in the hot path;
free-form P&R is the cold path and keeps `cells`' whole variant space; the
device blocks link to Substrate2.

### Q1 Recipe scope — accepted

A set-level recipe arranges one matched set. An OTA has a pair, a load mirror,
a tail and a bias branch, and something must arrange those around one shared
axis. Options: (a) set-level only, so composition falls to free-form and the
hot path alone never finishes an OTA; (b) a composition recipe per topology,
StrongARM style, 758 lines of generator plus an 873-line sky130 binding per
circuit; (c) a small family of constructive floorplan recipes, such as a
symmetric row stack ordered by signal flow, parameterised by the annotator's
compounds and stage order, deterministic, no annealing.

Recommendation: (c). Recipes at both levels, composition as a handful of
constructive floorplan recipes rather than per-topology templates.

### Q2 What a recipe promises, and who checks it — accepted

Does a recipe output a tile plus a declared list of constraint ids it claims to
satisfy, with every claim scored by the same `RuleBatch` and signed off by
GPurify, never trusted by construction? The alternative trusts recipes and
skips scoring, which saves runtime and removes the only independent check.

Recommendation: yes. Recipes are producers; the checker is the contract, for
both paths.

### Q3 Choosing among recipes that all fit — accepted

For a Moderate pair, an ABBA row, a split-gate ABBA, a two-row centroid and an
interdigitated row all satisfy the constraint. Options: (a) one fixed table
from class and kind to recipe; (b) bounded enumeration, each recipe family
draws at most a dozen parameterisations and rule cost picks, deterministic, no
annealing; (c) anything goes.

Recommendation: (b). It is a search, but over a dozen candidates, and it is
what lets a floorplan recipe hit an aspect target.

### Q4 Fallback granularity — accepted, then reconciled by Q18

When one block has no recipe, does only that block go free-form, with finished
recipe tiles entering the free-form placer as fixed multi-device macros through
the FLOW-11 block hook, or does the whole design fall to free-form? Is a recipe
tile atomic inside free-form, never re-opened?

Recommendation: per block, glue devices included, and recipe tiles stay atomic.
(Q16's refinement and Q18 restrict the per-block mix to free-form mode.)

### Q5 No recipe mining — accepted

Rust recipes cannot be learned from a free-form result. v1 has no promotion
path from a free-form layout to a recipe; a better arrangement becomes a recipe
only when a human writes it.

### Q6 Confidence gating — accepted

`Origin` exists, `Confidence` does not. A recipe applied to a false-positive
match is a wrong layout. Does recipe selection require a confidence level, with
Hint-level constraints never changing geometry, which makes the identification
work ID-01 to ID-18 a prerequisite of the hot path? Alternative: apply recipes
on every pattern match as 1.0 does.

Recommendation: yes. A bare pattern match selects only conservative recipes;
Proven, Corroborated or User select precise ones.

### Q7 Sidecar steering — accepted

The ALIGN-style sidecar gains entries that name a recipe and its parameters
for a constraint, plus Confirm and Reject, so a designer steers the hot path
without touching Rust. It is the cheapest degree of freedom and the one
designers will use.

### Q8 Which way the device blocks derive — accepted

Upstream's `MosTile` knows width, length, fingers and gate direction. 1.0's
`Mosfet` adds dummies, split gates, mirrored pins, rows and double gates.
Options: (a) Philis draws device internals through `cells` and wraps them as
Substrate `Tile` and `AtollPrimitive`, as plan-09 P2 already does, with
upstream tiles as extra variants; (b) upstream tiles are the source, and the
knobs go upstream or into a fork.

Recommendation: (a). "Derived from Substrate2" means typed on Substrate's
traits, not drawn by Substrate's code.

### Q9 Grid model — accepted with refinement

Atoll needs every tile outline to be a multiple of the routing pitches' LCM,
alternating metal directions, and no routing above the tile's top layer.
Device internals inside a tile are free. Do recipes live as atoll tiles with
snapped outlines, accepting the snapping area cost, so atoll's routing, straps
and abstracts apply? Alternative: raw `Layout` cells routed only by Philis's
router.

Recommendation: atoll tiles; the area cost is small at block level and keeps
FinFET and planar on one model.

**Refinement (owner):** the atoll grid is for construction only. A
post-processing step then ignores the atoll grid, respects only the PDK
manufacturing grid, and shrinks the layout with a Monte-Carlo-like action.
This is a new feature and needs research. (Detailed by Q24 and Q25.)

### Q10 Trait granularity — accepted

StrongARM carries its own tech trait per generator. Options: (a) a tech trait
per recipe; (b) a handful of shared capability traits, roughly matchable
primitive, arrayable primitive, guard ring, layout-dependent effects,
interconnect model and routing requests, with recipes generic over them and a
declared capability list.

Recommendation: (b). Bindings cost once per PDK, not once per recipe per PDK.

### Q11 The typing bridge — accepted

Substrate wants static blocks with typed `Io`; Philis is netlist-in. Recipes
stay static Rust types with runtime parameters and array-sized ports where
counts vary, under a dynamic SCIR-bound top cell, with no code generation and
compilation inside the loop. Alternative rejected: emit Rust per design and
compile it to get a GDS.

### Q12 What "dump the layout in Rust" means — accepted

The emitted Rust is an export: a pretty-printed recipe call tree a designer can
edit and re-run, while Philis executes the same call tree in-process. One
recipe call tree, two consumers: the in-process executor and the printer.

### Q13 Degradation when a PDK lacks a capability — accepted

Options: refuse with a diagnostic; degrade to the next recipe down (two-row
centroid to ABBA with dummies) with a report row; fall back silently.

Recommendation: degrade with a report row by default; refuse only when the
class is Precise or the constraint is User-mandated; never silent.

### Q14 Proof PDKs — accepted

Every recipe passes on sky130 and asap7 (planar and FinFET, both decked) from
day one. gf180mcu and ihp_sg13g2 follow. A trait shaped on sky130 alone will
not survive asap7.

### Q15 Hot-path targets — accepted with refinement

Asked: what runtime and reproducibility does the hot path owe? Recommended:
byte-identical output for identical inputs, and under 10 s wall for the OTA
fixture on sky130 against 138 s in 1.0.

**Refinement (owner):** no byte-identical requirement. Not every output is
correct, so correctness, not reproducibility, is the bar. The 10 s OTA target
stands. (Q33 keeps seeded replay as a test property only.)

### Q16 The free-form engine's fate — accepted with refinement

Asked: does gp, dp, gr and dr stay as it is, as the cold path, with recipe
tiles injected as fixed macros, and no rewrite onto Substrate? Recommended:
yes; it is the fallback and the test oracle.

**Refinement (owner):** free-form is **not a fallback**. It is a mode the user
selects. There are **no fallbacks on the Rust dump path**. (Q18 to Q20 draw the
consequences.)

### Q17 Acceptance set — accepted with refinement

Asked: which fixtures must the hot path alone take to clean signoff before v1
counts as done? Recommended: ota, bgr_core and dac4 on sky130 and asap7, plus
strongarm; pair, mirror_ratio and rc_filter come for free; tq_chain stays a
free-form case.

**Refinement (owner):** more fixtures will be added; every one listed must
pass.

---

## Round 2 (17 decisions, 2026-10-25; the owner: "Yes this is all good")

Facts checked: `dr` mirrors a differential pair as a reroute pass after plain
routing, not first; `elaborate.rs` runs build, bind nets, gr, dr, signoff with
no placer; plan-09 says sky130 keeps upstream's tiles.

### Q18 No engine fallback on the hot path — accepted

On the hot path, a block without a recipe is a coded refusal naming the block
and the sidecar entry that would fix it. The per-block mix (recipe tiles as
fixed macros plus free-form for the rest) exists only when the user selects
free-form mode. Reconciles Q4 with Q16.

### Q19 Degradation is not fallback — accepted

A recipe may degrade along its family's ladder on the hot path, because that is
a declared step inside one recipe family rather than an engine switch. Every
family's ladder is an ordered list the report quotes. Ladders are data; engine
switches are forbidden.

### Q20 Glue devices on the hot path — accepted

Devices in no recognised block exist in every real circuit. Options: (a)
refuse until coverage is total; (b) a glue recipe, a plain row ordered by
signal flow at the floorplan's edge, every routing and reliability rule still
applied, and a report row listing the glued devices.

Recommendation: (b). A glue row is a declared recipe, not a fallback.

### Q21 Floorplan recipe families for v1 — accepted

Options: (a) one family, stacked symmetric rows ordered by stage along the
signal flow, pairs straddling the shared axis, selfs on it, rails as straps,
bias at an edge; (b) that plus an array family for DAC and capacitor banks
with pitch-matched unit cells; (c) more. Inputs: compounds, the stage order
from `flow.rs`, sidecar `Order`, proximity and isolation requirements,
abutment.

Recommendation: (b). Two families cover the acceptance set.

### Q22 Targets flow top-down — accepted

The floorplan recipe hands each set recipe a width or height target; the set
recipe answers with its feasible parameterisations, the way atoll's
`ResizableInstance` folds fingers to a bound, so Q3's bounded pick happens
against a target. Without it row widths never line up.

### Q23 The router on the hot path — accepted

Options: (a) atoll's `GreedyRouter`, which has no symmetry, star, shield or EM
notion; (b) Philis's `dr` through the existing elaborate path, with
constrained nets routed first by construction as routing recipes (mirror pair,
star common node, shield) and the remaining nets negotiated around them as
fixed obstacles; (c) a new router.

Recommendation: (b). It reuses the five thousand lines that know EM, antenna
and mirroring, and an off-grid compaction pass needs a track-lattice router.

### Q24 Compaction scope and invariants — accepted

What may the compaction pass move? (a) recipe tiles as rigid bodies only;
(b) tile internals too; (c) wires too. Invariants held by construction:
mirrored tiles move as one about the axis, centroid groups move rigidly,
environment distances of matched sets never shrink below their recipe's value,
and every claim from Q2 is re-scored afterwards.

Recommendation: (a) with those invariants. A constraint-graph compaction, x
then y, is the known baseline; the stochastic moves sit on top for the slack
it leaves. A research spike with a written verdict precedes any commitment.

### Q25 Where compaction sits — accepted

(a) between the floorplan recipe and detailed routing, with routing halos from
the global router's estimate, then route; (b) after routing, stretching or
re-routing wires. Recommendation: (a). Compact, then route once. Re-routing
after compaction is the simple variant of (b) if (a) leaves too much slack.

### Q26 Where capability bindings come from — accepted

`build.rs` already generates layer and model enums from each deck. The
capability trait impls of Q10 come from deck keys too: the sidecar JSON gains
recipe-capability keys, and a new PDK is one JSON file plus hand-written tile
constructors, owned by the PDK module.

### Q27 sky130 gets Philis-drawn tiles — accepted

Reverses plan-09's "sky130 keeps upstream's tiles": Philis draws on all four
PDKs, and upstream's tiles become an optional variant. Otherwise the
acceptance set on sky130 would run on four-knob device blocks.

### Q28 Conformance matrix — accepted

Every recipe carries a self-check per proof PDK (DRC and ERC clean, every
claim satisfied, LVS on the tile's devices), and the table of which recipe is
available on which PDK is generated from those tests, never declared by hand.

### Q29 PEX-level claims — accepted

Equal lead capacitance and equal path resistance of a differential pair are
checked at signoff from GPurify's extracted network; the in-loop estimate from
`stack.rs` path resistance is the steering signal. In-loop extraction returns
when a fixture shows the estimate lying.

### Q30 Classes from role defaults — accepted

When no spec or sensitivity exists, `ClassSource::Role` picks the precision
class. A role default may select up to a Moderate recipe; Precise is reserved
for a Spec or User source; a report row names the source.

### Q31 Sidecar keys — accepted

Constraint ids are dense emission-order numbers and shift whenever the netlist
changes. `Confirm`, `Reject` and `Recipe` entries are keyed by instance and net
names, resolved to ids at parse time as the existing entries are.

### Q32 The recipe call tree is the IR — accepted

One table of recipe invocations, each row a recipe id, its parameters, the
constraint ids it claims, its children and its placement relation. The
executor runs it, the printer emits the Rust, the visualizer shows it, the
sidecar names rows in it. No second representation. It is also the only shape
that satisfies the data-layout rule in `AGENTS.md`.

### Q33 Seeded reproducibility — accepted

The same seed gives the same output, as a debugging property for tests and
goldens, never as a product promise (Q15).

### Q34 Rust dump is recipe-only — accepted

The new printer serves the recipe path only. 1.0's `emit.rs`, which decompiles
a free-form solution into Composition source, is left untouched until it is in
the way.

---

## Round 3 (18 decisions, 2026-10-25; the owner: "Otherwise I agree with all your suggestions")

Asked 2026-10-25 after the owner created the `Philis2_0` skeleton
(`crates/kernel/{substrate3,analog}`, `crates/backend/{placement,routing,
post-processing}`, `crates/frontend/{annotator,engine}`; `substrate3/README.md`
says "Fork of substrate2") and asked for testability: each feature built and
tested to be functional as we go. Every recommendation below was accepted as
written; the owner's same message added the three decisions of Round 4.

### Q35 substrate3 is a fork, not a re-export — accepted

plan-09 chose "re-export substrate2, send fixes upstream". The skeleton says
fork. A fork lets capability traits be implemented directly on upstream tiles
(no orphan rule), lets sky130 get Philis-drawn tiles in-tree, and lets atoll's
router and grid be patched; it costs tracking upstream.

Recommendation: fork, vendored at `crates/kernel/substrate3`, the upstream rev
recorded in its README, BSD-3 notice kept, resynced on purpose and never
silently.

### Q36 Crate map — accepted

Where do the owners live? Settled by the skeleton: set recipes and the device
blocks (`cells`) in `substrate3`; floorplan recipes in `backend/placement`;
routing recipes and the router in `backend/routing`; compaction in
`backend/post-processing`; identification in `frontend/annotator`; the spine,
the call tree, the printer and the CLI in `frontend/engine`. Not yet placed:
1.0's `pnr_core` (ids, tables, geometry), `verify` (decks, GPurify checker),
`visualizer` (GPU viewer).

Recommendation: `pnr_core` folds into `kernel/analog` as its leaf module;
`verify` becomes `kernel/verify` because GPurify is a heavy dependency the
viewer must not link; the visualizer becomes `frontend/visualizer` because of
its GPU dependency.

### Q37 Netlist frontend — accepted

Substrate's `spice` parser produces SCIR; 1.0's `library::parse` (1170 lines)
handles dialects, `.param` expressions, hierarchy and TinyTapeout
preprocessing. Recommendation: SCIR is the netlist IR, so parsed netlists and
recipe-built schematics share one representation and the dynamic top gets LVS
by construction. Which parser fills it is a fact to establish in M0 by running
both over the fixture corpus; 1.0's dialect handling is ported into the fork's
parser where it falls short.

**M0 corpus run (2026-10-25), outcome: the fork's parser fills SCIR.**
`substrate3/examples/parser_corpus.rs` against 1.0's `library::parse` over
1241 netlists: the 18 in `benchmarks/fixtures` and `field`, and 1223 `.sp`
and `.cir` files in the ALIGN and MAGICAL submodules.

| corpus | fork OK | 1.0 OK | both OK | device counts differ |
|---|---|---|---|---|
| fixtures and field (18) | 18 | 18 | 18 | 0 |
| competition (1223) | 60 | 68 | 32 | 0 |

Neither parser dominates the competition files, and most of those are ALIGN
templates whose parameters a testbench sets (1.0: "undefined parameter",
fork: `.PARAM`). The fork fails only on directives it does not know
(`.param`/`.PARAM` 1059, `.option` 14, `.model` 11, `.GLOBAL` 9, `.TEMP` 8,
`.title` 4, `.END` 3) and on `S`, `E`, `G` cards; 1.0 fails where a
parameter is undefined or several top-level sub-circuits exist. So the port
is 1.0's directive handling and `.param` evaluation into the fork's parser,
first where an acceptance fixture needs it.

### Q38 How code moves — accepted

Plain copy from Philis 1.0 with a commit message citing the 1.0 commit, versus
`git subtree` or `filter-repo` to keep history. Recommendation: plain copy; 1.0
stays as the history archive; every move is one commit.

### Q39 Definition of done per feature — accepted

A feature lands with all four: (1) its self-check or unit test with a named
oracle; (2) a fixture in the acceptance set, or an existing one, that
exercises it end to end; (3) a report row visible in `report.txt`; (4) a
negative control that fails when the feature is broken (plan-09's resized
schematic failing LVS is the model). Recommendation: yes, checked by CI.

### Q40 Oracle for claims — accepted

The independent `RuleBatch` scoring in `kernel/analog` is the oracle for every
claim, plus one seeded-violation test per claim kind: inject one asymmetry into
a clean tile and exactly that claim fails. Recommendation: yes.

### Q41 Goldens — accepted

Per fixture: the call-tree table (seeded), the signoff verdict and an area
bound. Never GDS bytes (Q15). Recommendation: yes.

### Q42 Signoff oracles and tiers — accepted

GPurify in-process on every push for sky130 and asap7; the foundry KLayout,
magic and netgen cross-check (1.0's `signoff_xcheck.py`) nightly; gf180mcu and
ihp_sg13g2 nightly. Recommendation: yes.

### Q43 Differential against free-form — accepted

Every acceptance fixture must sign off in both modes; hot-path area and wall
are tracked against free-form; the gate in v1 is signoff plus the 10 s OTA
target only. Recommendation: yes.

### Q44 Property and exhaustive tests — accepted

Exhaustive for small pattern spaces (every member count up to 16 for centroid
assignment: moments cancel); property tests for compaction invariants and
floorplan symmetry with a hand-rolled seeded generator (1.0's `SplitMix64`),
no `proptest` dependency. Recommendation: yes.

### Q45 Fuzzing at trust boundaries — accepted

SPICE netlist, sidecar JSON, deck JSON and GDS input to the visualizer, with
`cargo-fuzz` nightly; corpus checked in; every crash becomes a regression
input. Recommendation: yes.

### Q46 Annotator gold — accepted

Extend the 69 ALIGN `*.const.json` corpus with expected confidence per
constraint and expected recipe eligibility; the scoreboard gates every
identification step. Recommendation: yes.

### Q47 Printer round-trip — accepted

For every acceptance fixture the emitted Rust compiles (`cargo check` of a
generated crate) and rebuilds an equal call tree; nightly tier because of
compile time. Recommendation: yes.

### Q48 Performance regression — accepted

Wall per fixture recorded per push (1.0's `progress.py`); gate only on the OTA
target; nightly alert at +20 %. Recommendation: yes.

### Q49 Visual artifacts — accepted

One SVG per fixture from the visualizer attached to every CI run for human
review; the progress page carries them. Recommendation: yes.

### Q50 CI budget — accepted

Merge tier under 15 min: unit, property, conformance on two PDKs, acceptance
fixtures in-process. Nightly unbounded. When the acceptance set outgrows the
budget, fixtures rotate to nightly by size, never dropped. Recommendation: yes.

### Q51 Three change drills — accepted

The drill table in `ARCHITECTURE.md` needs three likely changes from the owner.
Candidates: (D1) a new PDK, gf180mcu, through the hot path; (D2) a new set
recipe, 2-D common centroid for resistors; (D3) a new routing recipe, shield
tracks beside a victim net. Recommendation: take the three unless the owner has
others.

### Q52 Roadmap order — accepted

M0 skeleton with `pair` through the hot path; M1 set recipes; M2 floorplan and
routing to `ota`; M3 arrays to `dac4` and `bgr_core`; M4 confidence and sidecar
to `strongarm`; M5 printer; M6 compaction spike; M7 free-form mode.
Recommendation: yes, routing inside M2 because that is where 1.0 fails.

---

## Round 4 (2026-10-25; Q53 to Q55 decided by the owner with the `/api-design` request, Q56 to Q58 accepted: "I agree with your answers")

### Q53 Performance is secondary to functionality, for now — decided

The owner: "Performance is secondary to functionality right now." Consequences
recorded in `TESTING.md` and `ROADMAP.md`: the 10 s OTA wall target is tracked
and reported on every push, and is not a merge gate until the acceptance set
passes functionally. The data-layout rules of `AGENTS.md` still shape tables
(layout is cheap to get right once and expensive to change), but no feature
waits on an `asm_probe` or a benchmark. Follow-up: Q56.

### Q54 Every exposed struct family shares a trait and a closed enum — decided

The owner: every crate's API must be easy to use; abstraction is good when it
has negative overhead; the structs a crate exposes should share traits so a
caller can match over them, especially where there are many variants, as in
`kernel/analog`. Recorded as the API rule in `API.md`: a family of variants is
one closed `enum` (a tagged union a caller can `match` exhaustively) plus one
trait every variant implements for the shared operations, plus a `kind()`
discriminant; `dyn Trait` is reserved for sets of implementations that live
outside the workspace. Follow-up: Q58.

### Q55 GPurify is the PDK input and the rule source — decided

The owner: GPurify reads the PDK and sets up all the DRC, LVS, PEX and ERC
rules; Philis consumes it to read the same rules and use them, and to read the
PDK input without a special parser. Recorded in `ARCHITECTURE.md` and
`TRAITS.md`: `kernel/verify` is a thin adapter over GPurify's deck tables; the
capability traits of Q10 read rule values from those tables; Philis writes no
deck parser. Open: where the analog-only numbers live (mismatch coefficients,
layout-dependent-effect reach, EM derating) once GPurify's extension mechanism
is known. Follow-up: Q57.

### Q56 The OTA target becomes a tracked number — accepted

Given Q53: the 10 s OTA wall is recorded per push and shown on the progress
page, reviewed at each release, and gates nothing until the acceptance set is
functionally complete. Recommendation: yes.

### Q57 Where the analog-only PDK numbers live — accepted as a conditional (resolved by the GPurify inventory)

1.0 kept them in a JSON sidecar beside the GPurify deck. Options: (a) extend
GPurify's deck grammar upstream so one file carries DRC, LVS, ERC, PEX and the
analog numbers; (b) keep a sidecar read through GPurify's own reader if its
grammar admits unknown statements; (c) keep 1.0's JSON sidecar. Recommendation, accepted: (a) if GPurify's
grammar is extensible without forking it, since the owner owns both projects
and one file per PDK is the point of Q55; otherwise (c). The inventory of
GPurify's deck reader decides which branch applies; the result is recorded as
Q59.

### Q58 The shape of the shared-trait rule — accepted

For every variant family (constraints and rules, intent rows, claims, recipe
kinds, diagnostics, routing requests, capabilities): one closed enum holding the
variant structs, one trait with the shared operations implemented by every
struct and by the enum through an exhaustive match, and a `Kind` discriminant
enum for tables that store only the tag. `dyn Trait` only for sets that live
outside the workspace. Recommendation: yes, and `API.md` applies it crate by
crate with usage code first.

### Q59 Where the analog-only PDK numbers live, by kind — accepted (2026-10-25, from the GPurify inventory)

The inventory (`docs/ref/research/gpurify-api.md`) resolves Q57's condition:
GPurify's deck text admits no unknown statement, keyword, unit or parameter;
its extension mechanism is Rust (`DeckBuilder`, `CustomRule`), and the design
intent file is `deny_unknown_fields`. So branch (b) is impossible and branches
(a) and (c) both apply, by the kind of number:

1. **Process facts GPurify's language already carries but the shipped decks do
   not state**: `temperature` with `tc1`/`tc2`, `device ... sheet`,
   `area_cap`, `fringe_cap`, `lvs range` for minimum device sizes, `lvs lde`
   for LOD and WPE layers and reach, `electromigration` with Black parameters,
   the sky130 `LU.*` tap-distance rules, `pex_substrate`. These go into the
   decks upstream, in GPurify, and Philis reads them through `CompiledDeck`'s
   typed queries. The `# PHILIS:` edits in 1.0's vendored decks are expressed
   as `DeckBuilder::from_deck` overrides until each is upstreamed.
2. **Process facts with no GPurify statement**: matching coefficients (A_VT,
   A_β, A_R, S_VT gradient, BJT k_A), LOD `KVTH0`, V_T and V_BE temperature
   coefficients, the resistor head and bias value model, capacitor plate bias,
   guard-ring construction, routing-stack order, layer roles. These need new
   statements and new dimensions (mV·µm, %·µm, µV/µm) upstream; the owner owns
   GPurify, so that is a GPurify issue per family. Until a statement lands,
   1.0's JSON sidecar carries them, read by `kernel/verify` as `AnalogNumbers`
   with the registry that refuses unknown keys, and a sidecar value never
   shadows a deck value.
3. **Policy, not process**: the per-class tables from Hastings (dummy reach,
   moat extension, WPE clearance, gate extension, resistor width and length
   floors, dummy spans), identical across 1.0's four sidecars. These are
   `kernel/analog` class tables with provenance comments, not PDK input at all.

Also from the inventory: Philis re-pins GPurify at head (1.0 is 733 commits
behind and its `Checker` does not compile), reads the shipped decks through
`gpurify::pdks::*`, holds a `CompiledDeck`, and deletes every rule
re-derivation in 1.0's `pdk.rs` that a typed query now answers (the inventory's
§4.6 table). Recommendation: yes to the three-way split and the re-pin.

**Owner, 2026-10-25: no PDK JSON.** The decks are top-level
(`pdks/gpurify/<pdk>.deck`); `Pdk::builtin` builds each from GPurify's upstream deck
plus the `# PHILIS:` edits as DeckBuilder overrides (`kernel/verify/src/overrides.rs`),
with the hand-edited decks kept as its equivalence fixtures. The sidecar
`.json` files are gone: Hastings' class tiers are `kernel/analog` constants,
dead keys are deleted, and the rest is `kernel/verify/src/pdk_data/<pdk>.rs`,
one table per PDK (classification: `docs/ref/research/sidecar-keys.md`).
Typed fields replace the keyed table reader by reader; the Bm keys (a
sidecar number above the deck's) wait on the owner.

### Q60 Chip assembly is a later milestone; the die-context table is defined now — accepted (2026-10-25). The design tree is closed; M0 begins.

The die-level Hastings cards (`docs/ref/research/hastings-die.md`) cover chip
assembly: area estimation, seal and scribe, corner exclusion, pads, the ESD
network by pin type, periphery ECGR, single-level crossings, fill, hierarchy,
and a 23-step floorplan checklist. Philis 2.0's acceptance set is block level.
Proposal: chip assembly, ESD devices and the periphery are a milestone after
M7 with the checklist as its specification; the die-context input table
(outline, origin, thickness, package and bumps, θ, heaters, pin classes, pad
order) is defined in v1 so the block floorplan's stress, thermal and pad rules
read it when present and report unknown when absent. Recommendation: yes.

---

## After M0

### Q61 Simulation evidence is ESPice's files, read by name — accepted (UW-ASIC/Philis#77, 2026-10-26)

Philis 1.0 got its operating point by spawning ngspice (`--op-lib`,
`--testbench`). The analog flow runs one simulator, ESPice, and Philis 2.0
spawns none. ESPice writes each instance terminal's average, EM-average, RMS
and peak current (`--emir`, `espice-emir/1`, OmarSiwy/ESPice#237) and each
device's operating-point variables into a raw file (OmarSiwy/ESPice#236).
The same current file feeds GPurify's EM sign-off (UW-ASIC/GPurify#82), so
routing and sign-off size and check wires from the same numbers.

Decided by the issue: Philis reads both files (`--evidence-currents`,
`--evidence-op`), keyed by instance names resolved at parse time as Q31 keys
the sidecar; a name the netlist lacks is a diagnostic, never dropped in
silence. The facts enter `intent` with origin Evidence, and M4's
`Confidence` rates them Proven. Their consumers keep their milestones:
ROUTE-EmWidth DC in M2 (v2 adds RMS and peak) and the MOS-cascode V_DS claim
in v2. Recorded with the implementation: the engine decodes the raw file
(the fork's `nutlex`), as it parses the SPICE, and the annotator resolves
the names; ngspice's device letter (`m.x1.mn1`) and the inner device of a
model subcircuit (sky130's `xm1.msky130_fd_pr__nfet_01v8`) name the
netlist's device; and the engine's flat device names are the netlist's own,
since the fork's SCIR drops each SPICE letter (`XM1` is instance `M1`).
