# API design

How every crate's boundary is shaped, written usage-code first after Muratori's
*Designing and Evaluating Reusable Components*. Decisions: Q53 (performance is
secondary to functionality), Q54 and Q58 (every variant family is a closed enum
plus a shared trait), Q55 (GPurify is the PDK input). Sketches, not compiled
code; names settle with the first implementation of each crate.

## 1. The usage code, written first

The caller is the CLI, a test, or a designer's own program. This is what it
writes against the API that should exist. The coarse tier is one call; the fine
tier is the same work as separate calls over tables the caller holds.

```rust
use engine::{Inputs, Mode, Outcome};

// Coarse tier: the whole recipe mode in one call. Most callers stop here.
let inputs = Inputs { spice: &spice_text, sidecar: Some(&sidecar_json), interface: None, op_point: None };
let outcome: Outcome = engine::run(Mode::Recipe, &inputs, &pdk, &cfg)?;
std::fs::write("out/ota.gds", outcome.gds)?;
println!("{}", outcome.report);

// Fine tier: the same work, one table per step, so the caller can stop,
// inspect, print or visualize between any two of them. `run` is these lines.
let netlist   = engine::parse(&inputs.spice, &pdk)?;                       // SCIR library
let intent    = annotator::annotate(&netlist, &cfg.annotation, &evidence); // rows with origin + confidence
let intent    = annotator::sidecar::apply(intent, &inputs.sidecar, &netlist)?;
let admitted  = engine::gate_confidence(&intent, &cfg.gate);               // which rows may select which tier
let calls     = engine::plan_recipes(&admitted, &pdk, &availability, seed)?; // the call tree, or a Refusal
let tiles     = substrate3::build_tiles(&calls, &intent, &pdk)?;           // one tile per set-level call
let floorplan = placement::arrange(&calls, &tiles, &intent)?;              // positions and axes
let compacted = post_processing::compact(&floorplan, &tiles, &intent, &pdk);
let routes    = routing::route(&compacted, &tiles, &tiles.routing_requests, &intent, &pdk)?;
let claims    = analog::score_claims(&calls, &tiles, &compacted, &routes, &intent);
let signoff   = verify::signoff(&compacted, &tiles, &routes, &netlist, &pdk);
let report    = engine::report(&calls, &claims, &signoff);
let rust_src  = engine::print(&calls, &pdk);                               // the Rust dump (Q12)

// Between any two steps the caller may do what it likes: no callbacks.
visualizer::svg(&compacted, &tiles, &routes, &pdk.layer_map(), &mut out)?;

// Free-form mode is a sibling spine over the same tables.
let outcome = engine::run(Mode::FreeForm, &inputs, &pdk, &cfg)?;
```

Three things this code fixes about the API before any type exists:

- every step takes the tables it reads and returns the one table it writes;
  nothing is retained inside a crate between calls;
- the PDK is a value (`pdk`) that proves the deck loaded, passed everywhere,
  never a global or an init call;
- a refusal is a value in the `Result`, not a fallback to another spine.

## 2. The variant rule (Q54, Q58)

A family of variants is a closed `enum` holding plain structs, one trait with
the shared operations implemented by every struct and by the enum through an
exhaustive match, and a `Kind` discriminant for tables that store only the tag.
`dyn Trait` is reserved for sets that live outside the workspace (Substrate's
own `Block` machinery, GPurify's engine) and is wrapped once in an adapter.

Applied to the largest family, the placement rules of `kernel/analog`, with
1.0's exact variants:

```rust
/// One placement rule. A caller matches over it; the scorer calls the trait.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlacementRule {
    MatchedSet(MatchedSet),
    Symmetry(Symmetry),
    SymmetryIsland(SymmetryIsland),
    Proximity(Proximity),
    Isolation(Isolation),
    SubstrateBalance(SubstrateBalance),
    DtiBand(DtiBand),
    HeatSeparation(HeatSeparation),
    Environment(Surroundings),
    OrientationSet(OrientationSet),
    Utilization(Utilization),
    PlacePerf(PlacePerf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PlacementRuleKind { MatchedSet, Symmetry, SymmetryIsland, Proximity, Isolation, SubstrateBalance, DtiBand, HeatSeparation, Environment, OrientationSet, Utilization, PlacePerf }

impl PlacementRule {
    pub const fn kind(self) -> PlacementRuleKind { match self { /* one arm per variant */ } }
}

/// The shared operations every rule has (1.0's `Rule`, kept). Implemented by
/// each struct, and by the enum through one exhaustive match per method.
pub trait Rule: Copy {
    type On;
    fn cost(self, state: &Self::On) -> f32;
    fn satisfied(self, state: &Self::On) -> bool;
    fn residual(self, state: &Self::On) -> f32;
    fn headroom(self, state: &Self::On) -> f32;
    fn applicable(self, state: &Self::On) -> bool;
    fn known(self, state: &Self::On) -> bool;
    fn project(self, state: &mut Self::On, grid: i32);
    fn touches(self, out: &mut Vec<u32>);
    fn retarget(self, cell_of: &[u16]) -> Self;
    fn mirror_pair(self) -> Option<(u32, u32, u16)>;
    fn matched_pair(self) -> Option<(u32, u32)>;
    fn repair(self) -> RepairKind;
}

impl Rule for PlacementRule {
    type On = Layout;
    fn cost(self, s: &Layout) -> f32 { match self { Self::MatchedSet(r) => r.cost(s), Self::Symmetry(r) => r.cost(s), /* ... every arm */ } }
    // every other method: the same exhaustive match
}

/// The applied rule set: SoA, one row per rule, the tier as a column (1.0 kept
/// a `Vec<Box<dyn RuleBatch>>` per tier; the enum replaces the box).
pub struct Requirements<R: Rule> {
    pub rules: Vec<R>,
    pub tier: Vec<Tier>,          // Hard, Budget, Cost
    pub meta: Vec<BatchMeta>,     // constraint id and origin, one per rule
}
```

The same shape, with the same four parts, for every other family:

| family | enum | trait | kind | variants from 1.0 |
|---|---|---|---|---|
| routing rules | `RoutingRule` | `Rule<On = Routes>` | `RoutingRuleKind` | Antenna, CommonNode, CouplingBudget, CrosstalkExclusion, Differential, Electromigration, EsdWidth, IrDrop, MetalOverGate, ParasiticBudget, PerformanceBudget, PlateRatio, Shield |
| cell directives | `CellDirective` | `Directive` | `DirectiveKind` | Unitization, GuardRingRequirement |
| intent rows | `IntentRow` | `Inferred` (id, origin, confidence, devices, nets) | `IntentKind` | MatchSpec, Compound, Order, GroupNode, DeviceFacts, NetClassification, budgets |
| claims | `Promise` | `Scorable` (which rule scores it, from what state) | `RuleKind` | moments to order n, mirror about axis, equal environment, equal lead C, equal path R, star R balance, shield present, keepout respected |
| recipe kinds | `RecipeParams` | `Recipe` (`TRAITS.md`) | `RecipeKind` | Adjacent, Interdigitated, Cc1d, Cc2d, resistor and capacitor arrays, BJT array, glue row, row stack, array floorplan, mirror pair, star, shield, keepout |
| diagnostics | `Diagnostic` | `Coded` (code, severity, devices, nets, message) | `DiagnosticKind` | refusal, degraded, unknown-rule, sidecar-unknown-name, parse error, coverage gap |
| routing requests | `RoutingRequest` | `Requested` (nets, priority) | `RequestKind` | MirrorPair, Star, Shield, Keepout, Separation |
| capabilities | `Capability` | none (a tag) | itself | one per capability-trait method |
| signoff rows | `Finding` | `Located` (layer, position, measured, limit) | `Check` | drc, erc, lvs, pex, engine, coverage |

Why this beats `dyn`: the caller sees the whole family in one place and can
match exhaustively, so adding a variant makes the compiler list every site; the
per-variant code sits together, so batching by kind is a sort rather than a
redesign; there is no vtable, so it is also the negative-overhead abstraction
Q54 asks for. Scoring keeps 1.0's per-kind monomorphised loops by sorting the
table by `kind()` once and slicing.

## 3. Crate by crate

Each entry: the surface, then the five characteristics (granularity,
redundancy, coupling, retention, flow control) and the named discontinuity.

### kernel/analog

Surface: the enums and traits above; `score_claims(calls, tiles, layout,
routes, intent) -> ClaimReport`; `matching::{moments, pattern, mismatch,
sizing}` as free functions over slices; `intent` row types.

- Granularity: `score_claims` is the coarse call; under it `score(&rules, &state) -> Scores` per tier and `Rule::*` per rule. A caller can score one rule on one state.
- Redundancy: moment and pattern functions accept plain slices (`&[(i32, i32)]` unit positions, `&[u16]` counts) as well as `&[Unit]`, so a recipe author or a test needs no table to ask a question.
- Coupling: rules read `Layout` and `Routes` tables by handle; no rule reads another's output; no hidden `set_time`-style state.
- Retention: none. `Requirements` is built by the annotator and passed in; `analog` keeps nothing between calls.
- Flow control: pure functions; nothing calls back.
- Discontinuity: a rule that needs the thermal field reads a derived table (`rise_at_point`), computed by the caller before scoring, so a new field kind is a new table, not a callback. Named risk: a rule over a state neither `Layout` nor `Routes` (a 3-D stack) would need a third `On`.

### kernel/verify (over GPurify, Q55)

Surface, to be fixed by the GPurify inventory (Q57): `Pdk::load(deck_text) ->
Pdk` as the proof the deck parsed; `pdk.rules()` as GPurify's own tables,
re-exported, not re-derived; `pdk.analog()` for the numbers GPurify's deck does
not carry; `signoff(shapes, labels, reference, &pdk) -> Signoff`;
`drc`, `erc`, `lvs`, `pex` as the fine tier taking a `GeometryStore` the
caller built. Everything in memory; no file paths cross the boundary.

- Granularity: `signoff` is `drc + erc + lvs + pex + coverage`; each is callable alone on the same store.
- Redundancy: the store builds from Philis shapes, from Substrate's `layir` cells, or from GDS bytes; three doors, one engine.
- Coupling: the one place `dyn` may appear, wrapping GPurify's engine once; nothing above `verify` sees it.
- Retention: the `GeometryStore` is the caller's; `verify` does not cache layouts between runs.
- Flow control: the caller runs each check when it wants, including between recipe steps for in-loop DRC.
- Discontinuity: a rule kind GPurify cannot express is a coverage row, never a pass (1.0's rule), and the request for it goes upstream to GPurify rather than into a Philis parser. *Section completed after the inventory.*

### kernel/substrate3

Surface: the fork's own API (`Block`, `Io`, `Layout`, `Tile`, `TileBuilder`,
`atoll`), plus `PhilisPdk` and the capability traits (`TRAITS.md`), the device
blocks as tiles, `RecipeKind` and `Recipe`, `build_tiles(calls, intent, pdk)`,
`conformance::run(kind, pdk) -> AvailabilityRow`.

- Granularity: `build_tiles` over a call tree; `Recipe::build` for one call; the device block's `draw` under it; a caller can draw one device on one PDK.
- Redundancy: a recipe's `Params` are plain data, so a test builds a tile from a literal without an `Intent`; the typed `Io` door exists for Substrate-native callers and the `Array<Signal>` door for the dynamic top (Q11).
- Coupling: a recipe reads capability traits and `Spec` rows, never a layer name or a rule number; the PDK is a type parameter, not a global `Context` lookup. Substrate's `Context` stays inside the fork's adapter.
- Retention: Substrate's cell cache is internal and invisible; Philis mirrors nothing into it.
- Flow control: `Recipe::build` is called by the executor; recipes never call the router or the checker, they write request rows and claim rows.
- Discontinuity: a recipe whose geometry cannot live on the construction grid (an off-grid serpentine) needs a raw `Layout` door beside `Tile`; keep both doors open from M1.

### backend/placement

Surface: `arrange(calls, tiles, intent) -> Floorplan`; the floorplan recipes
behind it; `targets(calls, intent, bounds) -> Targets` (Q22); free-form
`gp::place`, `dp::place` over the same `Layout` tables.

- Granularity: `arrange` is `targets + rows + axes + legalize`; each callable.
- Redundancy: `Floorplan` converts to and from 1.0's `Layout` columns so free-form and recipe mode share scoring.
- Coupling: none hidden; `arrange` takes the tiles' outlines, not the tiles.
- Retention: none.
- Flow control: no callbacks; the owner of `Floorplan` is the caller after return.
- Discontinuity: a third floorplan family that is not rows or an array (a ring, a radial bandgap) is a new variant, not a new engine.

### backend/routing

Surface: `route(placed, tiles, requests, intent, pdk) -> Routes`, which is
`route_requests` (mirror pairs, stars, shields, keepouts first, by
construction) then `route_rest` (negotiated `gr`, `dr`); `em::widths`,
`antenna::repair` as fine-tier calls.

- Granularity: the two halves are separate calls; a caller can route the constrained nets and inspect before the rest.
- Redundancy: requests arrive as a table from recipes or from the sidecar; the same `RoutingRequest` enum either way.
- Coupling: the router reads pins and outlines, never a tile's internals; the request table is the only cross-crate write (as a request table its owner drains).
- Retention: none across runs; `Negotiation` state lives inside one `route_rest` call.
- Flow control: no callbacks into recipes; repair is a function over the routes table.
- Discontinuity: a router with a different lattice (atoll's grid) would be a new backend; Q23 chose one, and the request table is what any future router would consume.

### backend/post-processing

Surface: `compact(floorplan, tiles, intent, pdk) -> Compacted`, with
`invariants(floorplan, intent) -> Invariants` exposed so a test can check them
without compacting.

- Granularity: x pass and y pass as separate calls under `compact`.
- Redundancy: the stochastic layer is a separate call over the constraint-graph result.
- Coupling: reads outlines and axes only; writes positions only.
- Retention: none.
- Flow control: no callbacks; the random source is a seed the caller passes (Q33).

### frontend/annotator

Surface: `annotate(netlist, cfg, evidence) -> Intent`; `sidecar::parse(json,
netlist) -> (Overrides, Vec<Diagnostic>)`; `sidecar::apply(intent, overrides)`;
`gold::score(intent, expected) -> Scoreboard`.

- Granularity: identification passes are exposed in order (normalise, invariants, symmetry, sets, classes, evidence, name) for the scoreboard to run catalog on and off.
- Redundancy: evidence (operating point, sensitivities) is an optional table, not a mode.
- Coupling: the annotator never reads geometry; its only inputs are the netlist, the configuration, evidence and the sidecar.
- Retention: none; `Intent` is returned, not stored.
- Flow control: none; diagnostics are returned as a table.
- Discontinuity: a learned or LLM-produced hint enters as `Intent` rows at Hint confidence through the same table, never as a callback.

### frontend/engine

Surface: `run(mode, inputs, pdk, cfg) -> Result<Outcome, Refusal>` and every
step of §1 as a free function; `print(calls, pdk) -> String`; the CLI.

- Granularity: `run` is the §1 steps pasted; a caller can delete the call and paste its body.
- Redundancy: `Inputs` accepts SPICE text or a SCIR library; `Outcome` carries GDS bytes, report text, JSON tables and the Rust source so no caller re-derives one from another.
- Coupling: mode is a parameter matched once at the top; no step reads it.
- Retention: `recipe_calls` and every table are returned; the engine keeps nothing between runs.
- Flow control: the visualizer and progress display are the caller's, driven between steps; no progress callback exists.
- Discontinuity: a caller wanting to replace one step (their own floorplan) substitutes one line of §1; nothing else changes.

### frontend/visualizer

Surface: `svg(layout, tiles, routes, layer_map, &mut impl Write)`;
`show(...)` for the live viewer; `parse_gds(bytes) -> Shapes`.

- Granularity: render one layer, one tile, or everything.
- Redundancy: input is tables or GDS bytes; both doors draw the same picture.
- Coupling: links `gpurify-geom` only, never the engine.
- Retention: the viewer's state is the viewer's; the tables are the caller's.
- Flow control: the caller sends frames; the viewer never pulls.

## 4. Evaluation checklist, answered

| question | answer |
|---|---|
| integration in under ten minutes, written our way first | yes: §1 is the integration, 15 lines coarse, 15 fine |
| every high-level call decomposes into two to four | `run` → steps; `signoff` → four checks; `route` → two halves; `arrange` → targets, rows, axes, legalize; `compact` → x, y, stochastic |
| common case reachable diagonally | `engine::run` in one call; `verify::signoff` in one call |
| use A without initializing B | yes: `analog` scores without a PDK; `annotator` runs without geometry; `visualizer` draws GDS without a netlist |
| every retained construct has an immediate twin | no retained constructs; every step is tables in, one table out |
| caller decides when everything happens | yes: no callbacks, no inheritance; requests are tables |
| own types and own files | SCIR or SPICE in; plain slices for matching math; GDS bytes or tables into the visualizer; GPurify decks as they are |
| data with no reason to be opaque is transparent | every table is `pub` plain data with derived `Debug`, `Clone`, `PartialEq`; Substrate's and GPurify's internals stay behind their adapters |
| allocation separable from initialization | tables are returned owned; `_into(&mut Vec)` doors are added at the second caller that needs one, per semantic compression |
| source available to the runtime | yes, one workspace |
| the discontinuity | named per crate above; the shared one: a constraint neither `Layout` nor `Routes` can express |

## 5. Semantic compression, applied

Write the first recipe of each family concretely. Compress at the second
recipe of the family, not before; the `Recipe` trait's final shape is the
residue of Cc1d and Interdigitated for MOS. The floorplan recipe compresses at
the second fixture that needs it. Capability traits gain a method when the
second PDK disagrees with the first, never speculatively. Diagonal shortcuts
(`run`, `signoff`) are written only after their orthogonal steps work.
