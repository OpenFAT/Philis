# Philis 2.0

Analog layout from a SPICE netlist: constraints are recognised, then realized
by recipes, then signed off. This glossary fixes the words the code, the docs
and the decision log use.

## Language

**Recipe**:
A known layout technique that realizes one or more analog constraints by
construction, written as a static Rust generator type.
_Avoid_: template, generator (for this meaning), cell variant

**Set recipe**:
A recipe that arranges one matched set or symmetry pair into one tile.
_Avoid_: primitive, leaf recipe

**Floorplan recipe**:
A recipe that arranges finished tiles relative to each other around shared
axes, deterministically.
_Avoid_: placer, template, floorplanner (for the hot path)

**Routing recipe**:
A recipe that routes a constrained net or net group by construction before
general routing: a mirrored pair, a star common node, a shield.
_Avoid_: repair, reroute

**Glue recipe**:
The declared recipe for devices in no recognised block: a plain row by signal
flow at the floorplan's edge.
_Avoid_: fallback, leftovers

**Device block**:
A generator of one device's geometry, from Philis 1.0's `cells`, exposed as a
Substrate tile on every PDK. The unit a recipe is built from.
_Avoid_: cellgen, cell, primitive tile

**Claim**:
A recipe's declared promise about one constraint, scored afterwards by the
analog rules independently of the recipe.
_Avoid_: guarantee, assertion

**Ladder**:
The ordered list of recipes a family degrades through when the preferred one
is unavailable on a PDK; every step is reported.
_Avoid_: fallback chain, cascade

**Refusal**:
The hot path's coded diagnostic when no available recipe can realize a block;
it names the block and the sidecar entry that would fix it.
_Avoid_: fallback, error (unqualified)

**Capability**:
One thing a PDK can provide to recipes, named by a trait method and proven by
conformance; the availability matrix records which recipe has it on which PDK.
_Avoid_: feature, support

**Conformance**:
The per-recipe, per-PDK self-check that draws the recipe's parameter grid and
proves it DRC- and ERC-clean, LVS-correct and claim-meeting.
_Avoid_: self-test, smoke test

**Proof PDK**:
A PDK every recipe must pass conformance on before merge: sky130 and asap7.
_Avoid_: reference PDK, primary PDK

**Call tree**:
The one table of recipe invocations that the executor runs, the printer prints,
the visualizer shows and the sidecar names rows in.
_Avoid_: IR, plan, program

**Recipe mode**:
The default flow: constraints realized by recipes, no search, no fallback.
_Avoid_: hot path (in code and docs; fine in conversation), auto mode

**Free-form mode**:
The user-selected flow that places and routes by search over the device
blocks' whole variant space, with any recipe tiles as fixed macros.
_Avoid_: cold path, fallback, P&R, legacy flow

**Compaction**:
The post-processing pass that shrinks a constructed layout on the PDK
manufacturing grid, ignoring the construction grid, while preserving every
claim.
_Avoid_: legalization, optimization pass

**Construction grid**:
The atoll track grid tiles are built on: outlines in multiples of the routing
pitches' LCM, alternating metal directions.
_Avoid_: atoll grid (in docs), routing grid

**Intent**:
Everything the annotator infers about a circuit, each row with an origin and a
confidence: matched sets, compounds, orders, net classes, guard-ring
requirements, budgets, device facts.
_Avoid_: constraints file, problem

**Confidence**:
How far a constraint is trusted: User, Proven, Corroborated, Inferred, Hint.
It decides which recipe tier a constraint may select.
_Avoid_: score, weight, probability

**Origin**:
Where a constraint came from: a pattern, a net class, symmetry propagation,
shared bias, a passive set, a sensitivity, the user, a derivation.
_Avoid_: source, provenance

**Precision class**:
Minimal, Moderate or Precise: how closely a matched set must match, from a
spec, the user, or a role default.
_Avoid_: tier, grade

**Matched set**:
Devices whose parameters must track each other, with a ratio, a unit and a
precision class.
_Avoid_: match group, pair (when more than two)

**Compound**:
Devices and nets mirror-symmetric about one axis.
_Avoid_: symmetry group, island

**Unit**:
The repeated geometric element a matched set is built from: a finger, a unit
capacitor, a resistor segment, an emitter.
_Avoid_: finger (for non-MOS), segment (for non-R)

**Sidecar**:
The user's ALIGN-style JSON constraint file, keyed by instance and net names;
it adds, confirms, rejects and steers.
_Avoid_: constraints JSON, override file

**Acceptance set**:
The fixtures the recipe mode must take to clean signoff on both proof PDKs
before a version counts as done; it only grows.
_Avoid_: benchmark set, test circuits

**Fixture**:
One benchmark circuit: its SPICE, optional sidecar and interface file, and its
expected results.
_Avoid_: example, test case, sample
