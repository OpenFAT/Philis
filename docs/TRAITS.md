# The trait system

What the extended Substrate2 (`kernel/substrate3`) adds so that recipes have
full degrees of freedom while a recipe compiles against any PDK that fulfils
the capabilities it declares. Sketches, not compiled code: field widths and
names settle when the first recipe lands, under the `data-oriented-design` six
questions. Decisions cited from `DECISIONS.md`.

## Layering

| layer | PDK-agnostic? | lives in | examples |
|---|---|---|---|
| arrangement (data) | yes | `intent`, `recipe_calls` | ABBA order, axis, rows, unit counts, ratio |
| realization (tiles) | no: per PDK | `substrate3` device blocks and PDK tile constructors | a finger of W×L with its contacts on sky130 |
| legalization and routing | grid plus rules | `atoll` grid for construction, `dr` on the track lattice, compaction on the PDK grid (Q9, Q23, Q24) | |

A recipe reads arrangement data and capability traits, builds a tile through
PDK tile constructors, and declares claims. It never reads a layer name or a
rule number directly.

## The recipe trait

```rust
/// A known layout technique that realizes constraints by construction (Q1).
/// Static type, runtime parameters (Q11). One `RecipeKind` variant per type (S1).
pub trait Recipe: Sized {
    /// The parameters a call carries; printed verbatim by the printer (Q12).
    type Params: Copy + Debug;
    /// Capabilities the PDK must have; checked against the availability matrix
    /// before any candidate is drawn (Q13, Q28).
    const REQUIRES: &'static [Capability];
    /// This family's ladder: which kinds to degrade to, in order (Q19).
    const LADDER: &'static [RecipeKind];

    /// Bounded enumeration against the targets the floorplan handed down
    /// (Q3, Q22). At most a dozen rows; deterministic given the seed (Q33).
    fn candidates<P: PhilisPdk>(spec: &Spec, targets: Targets, pdk: &P, out: &mut Vec<Self::Params>);

    /// Draw one candidate. Returns the tile and the claims it makes (Q2).
    fn build<P: PhilisPdk>(params: Self::Params, spec: &Spec, cell: &mut TileBuilder<'_, P>) -> Result<Built>;
}

pub struct Built {
    pub tile: TileData,            // atoll outline, abstract, ports
    pub units: Vec<Unit>,          // one row per unit: position, member, finger, sa/sb
    pub claims: Vec<Claim>,        // scored later by `kernel/analog`, never trusted
    pub routing_requests: Vec<RoutingRequest>, // mirror, star, shield, keepout (Q23)
}
```

`Spec` is the recipe's slice of `intent`: a `MatchSpec` for a set recipe, a
`Compound` plus stage order for a floorplan recipe, a net pair or common node
for a routing recipe. `Targets` is a width or height bound plus the axis the
floorplan imposes (Q22).

## Claims

```rust
/// What a recipe promises about one constraint. `kernel/analog` scores it
/// with the same `Rule` that scores free-form output (Q2).
pub struct Claim {
    pub constraint: ConstraintId,
    pub rule: RuleKind,            // MatchedSet, Symmetry, CentroidGroup, Environment, Differential, CommonNode, ...
    pub promised: Promise,         // e.g. moments cancelled to order n, equal sa/sb, mirror about axis k
}
```

Status after scoring: met, violated, unknown (inputs missing, as 1.0's
`Rule::known`). Unknown is reported, never certified.

## Capability traits (Q10)

A handful, shared by every recipe. Implemented per PDK; the rule values come
from GPurify's `CompiledDeck` typed queries through `kernel/verify` (Q55),
generated into the typed PDK by `build.rs` (Q26); the analog-only numbers
follow Q59 (deck statement where one exists, `AnalogNumbers` sidecar until a
statement lands, `kernel/analog` tables for policy); the geometry constructors
are hand-written per PDK.

| capability method | GPurify source today | until then |
|---|---|---|
| `InterconnectModel::sheet_r`, `cap_per_um` | `pex` rows: `sheet`, `area_cap`, `fringe_cap`, `tc1`/`tc2` with `temperature` | decks must state `temperature` |
| `InterconnectModel::em_idc`, `via_idc` | `em_current_density`, `electromigration` tables; `em_limit(layer)` | `em_derating` sidecar for T_ref, Ea, n |
| `MatchablePrimitive::unit`, min sizes | `lvs range "model" w l` | `min_finger_width`, `sd_width` sidecar |
| `Lde::wpe_reach`, `lod_reach` | `lvs lde "model" od .. max .. well .. scref ..` | `lod_kvth0_*` sidecar; class clearances are policy tables |
| tap reach, latch-up | `tap_distance`, `missing_tie`, `conditional_tie`, `latchup_victim` | sky130 `LU.*` rows as `DeckBuilder` overrides |
| `GuardRing::ring` | `guard_ring`, `guard_ring_bias` check a ring; construction from `min_width` and `cut_spacing` of the ring layers | `min_guard_ring_width`, `guard_licon_pitch` sidecar |
| `ArrayablePrimitive` caps | `device capacitor ... area_cap fringe_cap` | `c_dw_nm`, `cap_density_ff_um2` sidecar |
| matching coefficients | none | sidecar; a `mismatch` statement is a GPurify issue |
| layer roles | none; mos recognisers imply gate and diffusion | `cell.layers` sidecar |

```rust
pub trait PhilisPdk: layout::schema::Schema + schematic::schema::Schema + Sized {
    const DECK: &'static str;
    type Model: MosModel<Pdk = Self>;
    fn deck() -> &'static verify::Pdk;
    // ... as plan-09, plus the capabilities below as supertraits or associated items
}

/// Device blocks a matched set can be built from.
pub trait MatchablePrimitive<P: PhilisPdk> {
    fn unit(w: i64, l: i64, model: P::Model) -> UnitGeom;          // the unit every member is built from
    fn can_split_fingers(&self) -> bool;                            // interleave members in one row
    fn abuttable_sides(&self) -> Sides<bool>;                       // diffusion sharing
    fn allowed_flips(&self) -> Flips;                               // FinFET S/D asymmetry forbids some
    fn dummy(&self) -> DummySpec;                                   // tie-off style, count per end
    fn environment_reach(&self) -> i64;                             // WPE, LOD reach in nm
    fn emit_units(&self, out: &mut Vec<Unit>);                      // for centroid moment checks
}

/// Resistor, capacitor and BJT arrays.
pub trait ArrayablePrimitive<P: PhilisPdk> {
    fn unit_cell(&self) -> UnitCell;                                // pitch, outline
    fn grid_aspect_limits(&self) -> (f32, f32);
    fn dummy_ring(&self) -> Option<RingSpec>;
    fn lead_points(&self) -> Leads;                                 // where jumpers and leads attach
    fn plate_rank(&self) -> PlateRank;                              // capacitor bottom plate to the quiet net
}

pub trait GuardRing<P: PhilisPdk> {
    fn ring(kind: RingKind, role: RingRole) -> RingSpec;           // tap ring, electron collector, hole collector, isolated tub
}

pub trait Lde<P: PhilisPdk> {
    fn wpe_reach(&self) -> i64;
    fn lod_reach(&self) -> i64;
    fn pse_reach(&self) -> i64;
    fn stress_params(&self, unit: &Unit) -> (i64, i64);            // sa, sb for the simulation card
}

pub trait InterconnectModel<P: PhilisPdk> {
    fn sheet_r(layer: Layer) -> f32;                                // ohm per square
    fn cap_per_um(layer: Layer) -> f32;                             // aF per um
    fn em_idc(layer: Layer) -> f32;                                 // uA per um
    fn via_idc(cut: Layer) -> f32;                                  // uA per cut
}
```

`Capability` is a closed enum naming each trait and each optional method; the
availability matrix (Q28) says which (recipe, PDK) pairs passed conformance.
The full list of numbers the recipes read, grouped by family and with the
ones no source gives, is `RECIPES.md` §6; the capability traits are shaped
from that list, not the other way round.
`plan_recipes` consults it before drawing anything; a missing capability is a
ladder step or a refusal, never a silent fallback (Q13).

Routing capabilities are **not** PDK traits. A recipe writes `RoutingRequest`
rows (mirror pair about axis k, star at centroid, shield net beside victim,
keepout over gates); `backend/routing` drains them first, then negotiates the
rest (Q23). This is the one request table that crosses from recipes to the
router.

## The recipe call tree (Q32)

The intermediate representation. One table, SoA, handles not pointers.

| column | width | meaning |
|---|---|---|
| `kind` | u8 (`RecipeKind`) | which recipe type |
| `params` | per kind, fixed-size | the `Params` of that kind, in a parallel column per kind |
| `parent` | u16 handle, `NONE` for roots | floorplan calls are parents of set calls |
| `first_child`, `child_count` | u16, u16 | children laid out contiguously |
| `relation` | u8 enum + i32 gap | placement relative to the parent or previous sibling (align mode, gap by rule name or nm) |
| `claims_start`, `claims_count` | u32, u16 | rows in `claims` |
| `ladder_step` | u8 | 0 = first choice; the report quotes it |
| `confidence_used` | u8 | the confidence that admitted this call (Q6) |
| `seed` | u64 | for replay (Q33) |

Six questions: produced by `plan_recipes`, consumed by `build_tiles`,
`arrange`, the printer, the visualizer and the sidecar; capacity bounded by
sets + compounds + glue + floorplans per block; lifetime one run; access is a
pre-order walk, so children are contiguous; widths as above; the hot access is
the walk, the cold access is the print.

The printer emits one Rust statement per row: a constructor of the recipe type
with its `Params`, nested by `parent`, placed by `relation`. The emitted
program instantiates the same static types, so running it rebuilds the same
table (Q12, Q47).

## Device blocks as tiles (Q8, Q27)

Each 1.0 generator (`mosfet`, `finfet`, `resistor`, `capacitor`, `cap_array`,
`bjt`, `diode`) becomes a tile on every PDK the way plan-09 P2 wrapped
`MosTile<M>`: the block draws the geometry through the `Process` behind the
typed PDK, the tile exposes one port entry per unjoined region (a two-finger
planar device's two sources), the composing recipe draws the joins, every port
shape is labelled, and `Unit` rows are emitted for moment checks. sky130 is no
exception; upstream's `NmosTile` and `PmosTile` remain as optional variants.

## The dynamic top (Q11)

Recipes are static Rust types. The circuit is not: the top cell is a
SCIR-bound cell whose ports are the netlist's `.subckt` ports and whose
instances are the recipe tiles, connected by net id. Where a recipe's port
count varies (members of a set, fingers of a row), the port is an
`Array<Signal>` sized at run time. No Rust is generated or compiled inside the
run.

## Open items

- Field widths and the exact `Params` per kind: settled with the first recipe
  of each family (`RECIPES.md`).
- Whether capability traits are supertraits of `PhilisPdk` or associated
  types: decided by the first conformance test's ergonomics.
- The fork's upstream sync cadence: the rev moves on purpose, recorded in the
  README (Q35); how often is decided when the first upstream fix is needed.
- Where the analog-only PDK numbers live (Q57).
