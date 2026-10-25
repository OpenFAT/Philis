# Recipes: the initial catalog

The techniques a recipe can realize, drawn from the textbook and paper digests
in `docs/ref/textbooks/` and from what Philis 1.0 already draws, organised the
way the engine will use them: by the constraint each one realizes, with its
claims, its parameters, its ladder position and the PDK data it needs. Card
ids in brackets are the research cards this catalog condenses; their full
text, with every source item id, is in the research reports listed at the end.

Precision classes throughout are Hastings' MIN, MOD and EXC (1.0's Minimal,
Moderate, Exceptional): MOS ±10 / ±3 / ±1 mV or %; resistors and capacitors
±1–5 % / ±0.1–1 % / ±0.01–0.1 %, six sigma over ten years [MOS §4, MAT-01].

Status words: **v1** lands in M1 to M3 (`ROADMAP.md`); **v2** after the
acceptance set passes; **research** needs a spike before a recipe exists;
**1.0** names what Philis 1.0 already draws and the device block it comes from.

## 1. From constraint to recipe

What the annotator emits (1.0's `intent` and rule vocabulary) and which recipe
family answers it. The ladder runs left to right: the preferred recipe first,
then what the engine degrades to, with a report row at every step (Q19).

| constraint (intent row or rule) | class or kind | recipe ladder | claims scored by |
|---|---|---|---|
| `MatchSpec` MOS, kind Voltage (input pair) | EXC | MOS-Cc2d cross-quad or nth-order → MOS-Cc1d split-gate with dummies → refuse | MatchedSet, Symmetry, OrientationSet, Environment |
| | MOD | MOS-Cc1d with dummies → MOS-Interdigitated → MOS-Adjacent | same |
| | MIN | MOS-Adjacent → MOS-Interdigitated | MatchedSet, OrientationSet |
| `MatchSpec` MOS, kind Current (mirror, load) | EXC | MOS-Cascoded long-L units + Cc2d → Cc1d → refuse | MatchedSet, OrientationSet, Environment |
| | MOD | MOS-Ratioed CC row (diffusion-legal) → MOS-Interdigitated under one gate → Adjacent | same |
| | MIN | MOS-Adjacent (≤ 3–4 mirrors from the reference) | MatchedSet |
| `MatchSpec` MOS, series stack (cascode, chain) | any | MOS-Chain contact-free; cascode pair sized in ratio | MatchedSet (cascode set), Proximity |
| `MatchSpec` Resistor, kind Ratio | EXC | RES-Array dispersed CC sub-arrays + Kelvin + field plate → RES-Array CC → refuse | MatchedSet (ledger), CommonNode, MetalOverGate (body keepout) |
| | MOD | RES-Array CC with end dummies and ratiometric jumpers → RES-Array interdigitated | same |
| | MIN | RES-Array adjacent segments → RES-Serpentine | MatchedSet |
| `MatchSpec` Capacitor, kind Ratio, binary counts | EXC | CAP-Chessboard → CAP-BlockChessboard → CAP-Spiral | MatchedSet (moments), PlateRatio |
| | MOD | CAP-BlockChessboard → CAP-Spiral | same |
| `MatchSpec` Capacitor, kind Ratio, general counts | any ≥ MOD | CAP-Array CC (centro_assign) with dummy ring and top shield → unit array without shield | MatchedSet, PlateRatio |
| `MatchSpec` split DAC | any | CAP-SplitDac with attenuation residual | PlateRatio, bridge value |
| `MatchSpec` Bipolar, kind Voltage or Current | any | BJT-Array centre-out (unit centred, 1:N) → adjacent | MatchedSet |
| `MatchSpec` Diode | any | DIODE-Array CC → adjacent | MatchedSet |
| `Compound` (mirror symmetry about one axis) | any | FLOOR-RowStack places pairs astride the axis, selfs on it; set recipes inside are built mirror-symmetric | SymmetryGroup, SymmetryIsland |
| `Unitization` | any | realized inside every set recipe: integer unit counts, one unit geometry | Unitization honoured by construction, LVS |
| `OrientationSet` (Axis, Φ) | any | every set recipe locks orientation and balances signed Φ; FinFET balances gate shift | OrientationSet |
| `Environment` (WPE, OSE, LOD) | ≥ MOD | ENV-Dummies, ENV-WellClearance, ENV-MoatExtension applied by the set recipe at class reach | Environment, MatchedSet.mu_lod |
| `Proximity` | any | FLOOR-RowStack keeps a block's devices in one row group; glue row for the rest | Proximity |
| `Isolation`, `SubstrateBalance`, aggressor and victim rows | any | FLOOR-RowStack: aggressors on the far side, victim pair's axis through the aggressor; ISO-Ring by role and substrate kind; ISO-Taps | Isolation, SubstrateBalance, ring checks |
| `HeatSeparation`, thermal rule | ≥ MOD | FLOOR-RowStack: power devices at one end on the axis, matched sets at the class distance | HeatSeparation, MatchedSet.mu_thermal |
| `DtiBand` | any | FLOOR-RowStack commits a side per pair and places at ≤ s_max or ≥ d_dti | DtiBand |
| `Differential` net pair | any | ROUTE-MirrorPair → ROUTE-Crossover (swapped pins) → ROUTE-ExactMatched (perfect symmetry) | Differential |
| `CommonNodeReq`, `StarReq`, `KelvinReq` | any | ROUTE-Star from the centroid; ROUTE-Kelvin force and sense split | CommonNode |
| `CrosstalkExclusion`, `CouplingBudget`, Victim rows | any | ROUTE-Spacing by class pair (L²/S) → ROUTE-Shield | CrosstalkExclusion, CouplingBudget, Shield |
| `MetalOverGate`, resistor body and plate keepouts | ≥ MOD | ROUTE-Keepout hard; MIN may take a field plate | MetalOverGate |
| `Antenna` | any | ROUTE-AntennaRepair (1.0's jumper and diode insertion, unchanged) | Antenna |
| `Electromigration`, `IrDrop`, `EsdWidth` | any | ROUTE-EmWidth per segment and via array; ROUTE-NetSplit for sense branches | Electromigration, IrDrop, EsdWidth |
| `PlateRatio` | any | CAP routing recipes: mirrored branches, trunks, non-overlapped top and bottom plates | PlateRatio |
| `GuardRingRequirement` | any | ISO-Ring by role (tap, ECGR, HCGR, tub) with a quiet dedicated net | ring resistance, ring-net check |
| `Order` (signal flow, current flow, user) | any | FLOOR-RowStack row order | Order |
| no block, no set (glue) | | FLOOR-GlueRow | every routing and reliability rule |

Two constraints have no constructive recipe and stay checks: `Utilization`
(a floor the floorplan reports) and `PerformanceBudget` (a spec the routed
layout is simulated against).

## 2. Set recipes

One tile per matched set. Every MOS recipe applies the environment recipes of
§2.2 at its class; every recipe emits one `Unit` row per finger, segment or
plate so the moment checks are independent of the recipe (Q2).

### 2.1 MOS pairs, mirrors and stacks

**MOS-Adjacent** [MOS-24, PAT-02]. Members as compact blocks side by side,
same orientation, no interleave. Realizes MIN proximity and the smallest
parasitics; REV's silicon-era simulations rank it best for small devices
(StrongARM devices of 1.6–6.1 µm) where LOD and parasitic mismatch exceed the
gradient term. Claims: orientation, unit identity; centroid offset reported,
not promised. Params: order, orientation. Degrade: none. **v1**, 1.0 `Mosfet`
style Single.

**MOS-Interdigitated** [PAT-01, MOS-25]. ABAB fingers in one diffusion row
with a shared source; equal LOD and equal source and drain wire lengths by row
symmetry, so equal R_s and the best effective G_m of the three row styles; not
common-centroid (centroids half a pitch apart). Params: unit count, pitch,
pattern string with S/D subscripts (Hastings Table 13.3), end dummies.
Applicability: shared-source pairs and common-gate mirrors; 1.0 never offers it
because an ABAB boundary lands on a drain, so the recipe needs either a shared
source at the boundary or a gap finger. Claims: LOD equality, R_s equality,
Φ equality. **v1** (new pattern over 1.0's row generator).

**MOS-Cc1d** [PAT-03, MOS-25, MOS-16, MOS-13]. ABBA palindrome row, boundary
on a source, end dummies at the class reach, split gates (odd members' gate
bars above the diffusion, even below) so the two gates stay private, mirror
pins so device 1's pins are device 0's reflected about the cell centre. Cancels
every odd gradient order along the row; leaves the x² residue (outer device 9×
the inner's) and the LOD asymmetry that the two-row swap fixes. Params: nf per
member, dummies per end, split, mirror, double gate. Claims: coincidence,
cancelled order 1, Φ equal, equal environment. **v1**, 1.0 `Mosfet` Cc1d with
`split_gates`, `mirror_pins`, `dummies_per_edge`.

**MOS-Cc2d** [PAT-04, PAT-05, MOS-26]. Two rows sharing one tap strip: the
cross-quad (A on one diagonal, B on the other) for two units per device, or the
two-row swapped CC (ABBA over BAAB) for four or more, which cancels orders 1 and
2 and every even order, and equalizes LOD and S/D wire length. About 60 % of
the ABBA residual at equal elements. Params: rows, row order, tap sharing.
Claims: moments equal through order 2, Φ_H and Φ_V equal. **v1**, 1.0 `Mosfet`
`rows = 2`.

**MOS-NthOrder** [PAT-06]. The recursive central-symmetric pattern: P_n is
P_{n−1} beside its 180° copy, labels swapped when n is even; 2^n units per
device; 2×4, 4×4, 8×4, 8×8. Cancels every polynomial gradient through order n,
for two identical devices only. Params: n, P_1 (ABBA or cross-quad), stacking
axis. Claims: moments equal through order n. **v1 for n = 3** (1.0 `Mosfet`
`rows = 4`, `nth_order_rows(3)`), higher n **v2**.

**MOS-Ratioed** [PAT-15, PAT-16, MOS-02]. k devices with integer ratios in one
diffusion-legal CC row: half the multiplicities, an Euler path or ALIGN's
ratio-greedy fill, reflected about the centre; the odd leftover at the edge or
on the axis. For large ratios the small member as series units, the large as
parallel (20:1 = 4 series against 5 parallel), never series for EXC. Params:
ratio vector, row width, tie-break. Claims: coincidence per member, no
diffusion break, equal unit. **v1**, 1.0 `diffusion_cc_row` and `sets::unitize`.

**MOS-Chain** [MOS-05, P-19]. Series devices sharing the internal diffusion
without a contact; gates at minimum poly spacing; the recipe for cascode stacks
and the series member of a ratio. Claims: no contact on the internal node,
drain-left order. **v1**, 1.0 `Mosfet` Chain (odd nf per member; even nf is an
open gap, AC-24).

**MOS-Cascode** [MOS-30, MOS-28]. The cascode pair over a mirror sized in the
mirror's ratio so both members see equal V_DS; cascode backgates to their own
sources; laid out as two chains or one row of chains. Removes the
channel-length-modulation and hot-carrier systematic terms. Claims: ratio
identity on the netlist, equal |V_DS| from the operating point, cascode set
coincidence. **v2** (needs the op point for the V_DS claim).

**MOS-Sense** [MOS-33]. A sense finger inside a power array, never outermost,
with dummies; ratio by section count. **v2**.

**MOS-FinFET** [PAT-18, PAT-20, 1.0 `FinFet`]. On-grid unit fins with one
uncontacted edge gate per end, shared active for the CC variant, orientation
counts balanced per device against gate misalignment. 1.0 has the shared CC
row and SA/SB but no dummies as cards, no rows, no split gates. **v1** on asap7
for Adjacent and Cc1d, **v2** for the rest.

### 2.2 Environment recipes applied by every MOS set recipe

| recipe | what it draws | class rule | source |
|---|---|---|---|
| ENV-Dummies | full tied-off dummy gates at each end, identical pitch, W and extension; half dummies above 3–5 µm L | MIN optional; MOD one full dummy per end below ~3 µm L; EXC until the outer poly edge is ≥ 10 µm from any active gate | MOS-16, MOS-17 |
| ENV-MoatExtension | active extended past the last gate so end devices see inner STI stress; SA and SB recorded per unit | MOD ≥ 5 µm; EXC ≥ 10 µm | MOS-18 |
| ENV-WellClearance | well edges and deep diffusions kept from matched gates; PMOS kept inside their well | MIN ≥ 2 µm, MOD ≥ 3 µm in WPE-prone nodes; EXC ≥ 5–10 µm or 2× well depth | MOS-19 |
| ENV-GateStrap | gates contacted off the active area, extension +1–2 µm equal on all fingers and dummies, fingers joined in metal rather than a poly comb; both ends on wide switching fingers | MOD and EXC required; MIN exempt | MOS-13 |
| ENV-MetalOverGate | no metal over active gates; EXC also minimizes metal within 5–10 µm and mirrors adjacent leads; MIN may take one M1 plate | per class | MOS-22 |
| ENV-FillBlock | fill-block markers over the array plus halo, symmetric custom fill to restore density | MOD block; EXC block plus 5–10 µm halo on every metal | MOS-23 |
| ENV-PolyHalo | no foreign or generated dummy poly within the class halo; 150 µm from poly-0 | MOD 3–5 µm; EXC 5–10 µm | MOS-21 |
| ENV-Taps | butting tap when bulk is the source, else a strip within the deck's maximum tap distance; distributed plugs for power devices; every diffusion point within reach | always | MOS-06, MOS-07, SUBN-10 |
| ENV-Well | same-bulk cells share one well; class spacing between wells of different voltage; a matched PMOS with V_SB ≠ 0 gets a private source-tied well | always | MOS-10 |
| ENV-Flavour | the threshold or oxide marker drawn around every gate including dummies, clustered | always | MOS-14 |
| ENV-Orientation | parallel channels, equal signed Φ per member, allowed transforms {translate, mirror, R180}; translation only for non-self-aligned devices; no 90° turn in directional-halo decks | EXC exact; MOD near; MIN parallel | MOS-15, PAT-19 |

### 2.3 Resistor arrays

**RES-Array** [RES-02, MAT-02, MAT-03, MAT-04, MAT-07, MAT-09, MAT-11]. Every
member as n identical straight segments at one width, series-connected by
jumpers with current alternating segment to segment (even count where
possible), ordered along the row by the ratio pattern table (1:1 ABBA, 2:1
ABA, 3:1 AABAABAA, 3:2 ABABA, 4:1 AABAA, 4:3 ABABABA, 5:1 AAABAAAABAAA, 5:2
AABABAA, 5:3 ABAABABAABABAABA, 5:4 ABABABABA, 9:5 ABAABAABABAABA), end dummies
at the segment pitch, jumpers folded out and ratiometric (same layer, several
vias each). Length solved per segment so the head and contact terms stay
inside the value tolerance. Claims: value within tolerance, coincidence,
Σ direction = 0 per string, equal jumper construction. Params: segments, width,
pattern, dummies, jumper layer. **v1**, 1.0 `Resistor` (Single and Interdig
segment rows; no end dummies on single devices; no 2-D).

**RES-Array-2D** [MAT-05]. Dispersion into CC sub-arrays in a mirror-symmetric
bank grid when the quadratic residue dominates, each sub-array obeying the
four CC rules. **v2**.

**RES-NonInteger** [MAT-06]. Per-resistor segment length or one partial
segment per resistor, chosen by the sensitivity sweep, with embedded dummies
between differing lengths. **v2**.

**RES-Serpentine** [RES-03]. The MIN-only compact form with turn terms in the
value model. **v2**.

Resistor environment: field plate per segment tied to its tank for diffused
bodies above the threshold [RES-08]; tank per segment group tied to its
positive end [MAT-14]; voltage-aware gap between segments ΔV = 2V_d/N
[RES-11]; width from self-heating and current density, power density by class
[RES-17]; moat setback and nothing beneath [MAT-15]; one orientation per set,
H+V split for orientation-sensitive monocrystalline bodies [MAT-08]; material
ladder thin film > N-poly 500 Ω/□ > HSR [RES-19].

### 2.4 Capacitor arrays

**CAP-Array** [CAP-01, CAP-11, CAP-10, CAP-09, PAT-08]. Unit plates with equal
area-to-perimeter ratio, integer counts, a dummy ring at the active pitch with
both dummy electrodes tied, a top shield one level above the top electrode
tied with the bottom shield to a quiet low-impedance node, the bottom plate to
the lower-impedance net. Units square 20–50 µm for EXC, 2:1 to 3:1 rectangles
for MOD. Claims: equal A/P, coincidence, every edge unit has a same-pitch
neighbour on four sides, plate orientation by impedance. **v1**, 1.0
`cap_array` (MOM and MIM, dummies, met3 straps).

**CAP-Chessboard**, **CAP-Spiral**, **CAP-BlockChessboard** [PAT-10, PAT-11,
PAT-12]. The three unit-assignment patterns for binary banks: chessboard for
linearity (maximum dispersion, worst speed), spiral for speed (contiguous bits,
60 vias against 295 at 8 bits), block chessboard between them with its (k,
block size) family enumerated. The bounded pick of Q3 chooses by the INL, DNL
and f3dB figures the pattern math reports. **v1**, 1.0 `cap_array` Spiral,
Chessboard, BlockChessboard with `metrics`.

**CAP-SplitDac** [PAT-13, PAT-14]. Two banks with the attenuation capacitor
drawn as a non-unit plate whose edge sensitivity equals the unit's, odd counts
as PAT-09, LSB and MSB bits alternated along the spiral. **v1**, 1.0
`CapArray::split`.

**CAP-NonInteger** [CAP-02]. One rectangle, notch or tang whose A/P matches the
unit for a non-integer ratio. **v2** (1.0 has `dac::nonunit_dims`).

**CAP-Antiparallel** [CAP-08]. Two halves with swapped plates, common
centroid, for asymmetric-electrode types on linearity-critical nets. **v2**.

Capacitor routing recipes [PAT-22 to PAT-26]: same-bit groups joined by
mirrored branches, trunks in channel tracks with a one-track target, bridges
at the periphery, top and bottom plates on disjoint tracks and layers, parallel
wires for the critical bit on quantised-width layers, ratio-proportional lead
parasitics for non-binary banks. **v1** for branches and trunks (1.0 draws
them inside the macro), **v2** for parallel wires and ratio-proportional leads.

### 2.5 Bipolar and diode arrays

Pending the die-level reader: this section is appended when the Hastings
chapter 9 to 11 cards arrive. 1.0 draws **BJT-Array** (unit emitters on a
grid, `centro_assign` Balanced, 1:8 as the 3×3 with the unit centred, shared
PNP collector band, NPN isolation per unit) and **DIODE-Array** (the same
grid rule, cathode plates with anode taps). Both **v1** on sky130 for the
acceptance fixture `bgr_core`.

## 3. Floorplan recipes

**FLOOR-RowStack** [P-01, P-02, P-04, P-05, P-06, P-17, P-28, MOS-31, MOS-32,
SUBN-15, SUBN-17, BAL1-34]. The one composition recipe for OTA-like blocks:

1. One shared vertical axis from the circuit's compounds (P-04); rails and
   signal flow fix the direction (BAL1-34).
2. Rows ordered by stage along the signal flow or the current flow (`Order`):
   loads, then signal devices, then tails and bias, so current flows
   monotonically and crossings are few (SURV-07, SURV-08).
3. Each pair straddles the axis as one set-recipe tile; self-symmetric devices
   (tails, shared bias) sit on the axis; a bias device tied to both halves is
   a self-symmetric cluster (P-05).
4. Nested symmetry when a half contains its own mirror (P-06).
5. Well classes decide spacing between rows and tiles by a class matrix, never
   a global clearance; same-bulk tiles abut and share a well (P-17, ENV-Well).
6. Power devices at one end on the axis; matched sets at the class distance
   (≥ 1 µm per mW for MOD near small heaters; the far end for EXC); the
   dominant substrate aggressor on the victim pair's bisector (P-28, MOS-31,
   SUBN-17).
7. Row widths line up because the floorplan hands each set recipe a width
   target and takes back its feasible parameterisations (Q22; P-14 shape
   functions, atoll `ResizableInstance`).
8. The island check: every group member abuts another (P-01's island), checked
   by union-find after construction (`SymmetryIsland`).

Claims: every compound's equalities, island connectivity, class spacing, order
monotone, thermal distance. **v1** for `ota`, `tt_ota`, `ota_constrained`,
`strongarm`. Deterministic; the only choice is the bounded pick over the set
recipes' parameterisations against the row target.

**FLOOR-Array** [P-27, R-15, BAL2-28]. Unit cells abutted in rows and columns
with flipped cells sharing rails, bus and tree generators giving equal R and C
to every unit; for DAC switch banks under a CAP bank and for instance arrays
(`hier::arrays`). **v1** for `dac4`.

**FLOOR-GlueRow** (Q20) [P-05, P-29]. Devices in no block, as a row ordered by
signal flow at the floorplan's edge, every routing and reliability rule
applied, a report row listing them. **v1**.

**FLOOR-Isolation** [SUBN decision procedure §3.1, MOS-08, MOS-09]. Not a
separate placement; the rules the row stack applies when aggressors exist:
read the substrate kind first (bulk responds to distance and rings, epi to the
backplate ground, and a ring in epi is worth 7–10 dB only); unconditional
local rules (adjacent drain tap at every vertical PNP and strong injector,
every n-well AC-grounded, taps around sensitive devices, nothing added at ESD
cells, capacitor bottom plates to the reference); per-pair minimum distance
from the victim budget inverted through the kind's distance law, clamped to
4× epi thickness on epi; then rings by role on a dedicated quiet pin,
injector first, one shared thin ring around a differential pair, dual rings
when two pins exist, deep n-well around the smaller group last. **v1** for
taps and tap rings (1.0 `post_cell::guard_rings` by role), **v2** for the
per-pair budget, **research** for the two-port macromodel.

**Compaction** (Q24, Q25) [P-10, P-11, P-12, P-26]. The post-processing
baseline is the constraint-graph compaction the retargeting literature uses:
one vertex per tile edge, an arc per minimum distance, longest path in x then
y, alternating with constraint regeneration; symmetry and common-centroid
equalities as LP rows; via arrays merged to one cut before and regenerated
after. The stochastic layer the owner asked for sits on top of this result.
**research**, M6.

## 4. Routing recipes

Routed first, by construction, before the negotiated router takes the rest
(Q23). Each writes `RoutingRequest` rows; `backend/routing` drains them.

**ROUTE-MirrorPair** [R-01, LAMP-40, BAL2-07]. Search one net of the pair with
two-sided legality (a cell is legal only if its mirror is legal), then mirror
the path; obstacles on either side block both. Guarantees identical geometry
and self-parasitics even for partially symmetric placements. Needs
mirror-symmetric terminals and the axis. 1.0 mirrors as a reroute pass; the
recipe moves it first. **v1**.

**ROUTE-Crossover** [R-02]. The symmetric crossing structure for pairs whose
terminals swap across the axis (cross-coupled latches). **v1** for
`strongarm`.

**ROUTE-ExactMatched** [R-03, SURV-14]. For perfect-symmetry pairs that cannot
be mirrored: equal width, equal per-layer length and area by detours, bundles
balanced per aggressor. **v2**.

**ROUTE-Star** and **ROUTE-Kelvin** [PAT-28, R-06, BAL2-06, BAL2-46]. A shared
current-carrying node routed as a star from one tap at the members' centroid
with equal branches, or a force path plus a separate no-current sense branch;
low-current branches never tapped off a high-current segment. 1.0 has
`CommonNode` scoring and a `mirror_guide`; the recipe draws the star first.
**v1**.

**ROUTE-Spacing** [R-07, BAL2-13, SUBN-27]. Class-pair spacing and routing
order (power, sensitive, non-critical, noisy last) so power and non-critical
nets separate critical from noisy; spacing priced by parallel run length
squared over spacing, not a threshold. **v1** (1.0's crosstalk multiples by
class, made run-length aware).

**ROUTE-Shield** [R-04, BAL2-10, SUBN-26]. Space first; when the coupling
budget still fails, a shield beside the victim on the same layer or a plate
between layers, tied to the quiet reference, after all other wires. **v1**
(1.0 `Shield` and dr draws shields).

**ROUTE-Keepout** [R-16, ENV-MetalOverGate, MAT-10]. No foreign metal over
matched gates, resistor bodies or capacitor plates, hard for MOD and EXC;
heads are zero-cost corridors for the resistor's own net. **v1** (1.0
`MetalOverGate`, `Macro::keepouts`).

**ROUTE-EmWidth** [EMR-01 to EMR-05, EMR-13 to EMR-15, R-05]. Per segment:
width as the maximum of the equivalent, peak and minimum terms with process
margins, via cut count from the full segment current with the temperature
factor dividing, arrays in line with the current, chamfered corners on
critical segments, criticality by current never by role. 1.0 has
`Electromigration` (DC) and `em_limit` widths; the recipe adds the via-array
geometry and the three current types. **v1** DC, **v2** RMS and peak.

**ROUTE-NetSplit** [R-06, BAL2-04]. Subnets by current density so sense
branches return to the source point. **v2**.

**ROUTE-AntennaRepair**. 1.0's per-stage antenna check with jumpers and diode
insertion, unchanged. **v1**.

**ROUTE-Array** [PAT-22 to PAT-26, R-15]. The capacitor-array branch, trunk and
bridge routing and the unit-array column routing of §2.4. **v1** inside the
cap tile.

**ROUTE-CurrentSteiner** [EMR-16]. Topology by minimising Σ L_e·w(i_e) rather
than length for heavy nets. **v2**.

## 5. Ladders

Ordered rows the report quotes when a step is taken (Q19). A step is taken
only when the PDK lacks a capability or the fit fails; Precise or User-mandated
constraints refuse instead of stepping (Q13).

| family | ladder |
|---|---|
| MOS Voltage | MOS-NthOrder(3) → MOS-Cc2d → MOS-Cc1d split-gate mirror-pin → MOS-Cc1d → MOS-Interdigitated → MOS-Adjacent |
| MOS Current | MOS-Cascode+Cc2d → MOS-Ratioed CC row → MOS-Interdigitated (under one gate) → MOS-Adjacent |
| MOS dummies | full dummies at class reach → half dummies → moat extension only → none (MIN) |
| MOS metal | no metal and identical surroundings → no metal → M1 plate (MIN) → identical crossings (MIN) |
| MOS taps | sublayer plus distant large contacts → butting → distributed plugs → interdigitated strips → side strip |
| Resistor | RES-Array-2D dispersed → RES-Array CC → RES-Array interdigitated → RES-Array adjacent → RES-Serpentine (MIN) |
| Resistor non-integer | integer units → per-resistor lengths → partial segments |
| Capacitor binary | CAP-Chessboard → CAP-BlockChessboard → CAP-Spiral |
| Capacitor general | CAP-Array CC with shield → CAP-Array CC without shield (wider dummies) → merged plate (MIN singleton) |
| Capacitor type | MIM → poly-poly → MOS (several volts into the flat region) → never junction |
| Guard ring | dual rings on two pins → injector ring on a dedicated pin → victim ring → taps near the victim → none rather than a ring on a shared noisy net |
| Differential routing | ROUTE-MirrorPair → ROUTE-Crossover → ROUTE-ExactMatched → length-only match |
| Coupling | spacing by run length → ROUTE-Shield → reroute |
| EM width | per-segment three-type width → DC equivalent only → class width |

## 6. What the PDK must carry

The capability data the recipes read, grouped for `TRAITS.md` and Q57. 1.0
keeps most of these in a JSON sidecar beside the GPurify deck; the GPurify
inventory decides where they live in 2.0.

**Device blocks and matching** (MOS, MAT, PAT cards): c_Vt and c_k per polarity
(A_Vt, A_β), S_D distance coefficient per L, k_A and k_R for resistors and
capacitors, TC per recipe, minimum matched finger width, maximum finger width,
poly sheet R and t_ox for the gate-RC limit, dummy reach and count by class,
moat extension by class, WPE reach and clearance by class, well junction
depth, pocket flag and L_c, stringer flag, directional-halo flag, flavour
marker rules, contact-over-gate flag, tap maximum distance per well type,
sublayer sheet R, butting-contact rules, piezoresistance table, class limits
(Tables 13.4 and 13.5).

**Passives**: R_s, head sheet, W_b, L_b, L_h, contact ρ_c, grain size, film
thickness, silicide kind, voltage-dependent poly spacing, thick-field
threshold, body modulation coefficient, junction C, NBL shadow; capacitor
c_area, c_perim, dielectric thickness and E_max, TC and VC per recipe, unit
side default, over-circuit flag, etch penetration depth, litho w_min.

**Substrate and isolation** (noise-em §4.1): substrate kind, layer stack with
thickness and resistivity, backplate state, L_gnd per package option, L_gr,
deep n-well availability and depth, junction C per area and sidewall, impact
ionization constants, transfer-model fit constants, ring minimum width and
clearance, cut resistance and maximum ring resistance, d_tap for latch-up and
for sensitive devices, well tap resistance budget.

**Interconnect** (noise-em §4.2): per metal current limits for average, RMS
and peak at T_ref, per via current per cut, T_ref, Ea, n, h_nom and h_min,
Δw and w_etch, w_min and maximum width with slot rules, sheet R and via R,
capacitance per area and per length, Blech products for via-above and
via-below, σ_crit, reservoir credit, low-k flag, self-healing γ, self-heating
limit, crowding factors per via pattern.

What no source gives and the engine must own as a deck key with provenance:
the narrow-channel minimum finger width, the σ inflation for V_SB ≠ 0 beyond
one point, the halo tilt angle, a quadratic gradient coefficient, a measured
capacitor correlation distance, a WPE coefficient, a FinFET misalignment
coefficient, and the HCI ΔV_DS tolerance.

## 7. Claims and their scorers

Every claim a recipe declares is scored by a rule that reads geometry, never
by the recipe (Q2). The rules are 1.0's, kept.

| claim | scorer in `kernel/analog` | what it reads |
|---|---|---|
| coincidence, cancelled order n, second-moment residue | `MatchedSet` (coincidence, second_order_nm, cancelled_order), `matching::moments` | `Unit` rows |
| mirror about axis k, selfs on axis | `Symmetry`, `SymmetryGroup` | tile positions and axis |
| island connectivity | `SymmetryIsland` | tile bboxes |
| equal signed Φ, parallel channels | `OrientationSet` | `Unit.phi` |
| equal WPE and OSE distances, well clearance | `Environment`, `LiveEnvironment` | placed wells and diffusions |
| equal SA/SB | `MatchedSet.mu_lod` | `Unit.sa`, `Unit.sb` |
| isothermal, distance from heaters | `MatchedSet.mu_thermal`, `HeatSeparation` | power map |
| differential R, C, via count equal | `Differential` | routes |
| star R balance, Kelvin | `CommonNode` | routes and pins |
| shield coverage | `Shield` | routes |
| spacing and total coupling | `CrosstalkExclusion`, `CouplingBudget` | routes |
| no metal over matched gates or bodies | `MetalOverGate` | routes and keepouts |
| EM width and cuts, IR drop | `Electromigration`, `IrDrop`, `EsdWidth` | routes, currents |
| lead C per unit, plate separation | `PlateRatio` | routes and extracted caps |
| antenna ratio | `Antenna` | routes and gates |
| ring resistance, ring net quiet | ring checks (1.0 `ring_ohm`; ring-net rule new) | ring macro and nets |
| unit identity, integer ratio, value | LVS and the drawn cards; resistor value model | `Macro::drawn` |

Claims no 1.0 rule scores yet, to add with the recipe that promises them: PSE
(poly spacing) equality, metal coverage similarity within a halo, dummy
geometric equivalence to active gates, per-aggressor coupling difference for a
pair, ring useful-frequency check, Blech and via-array geometry.

## 8. What 1.0 already draws, and the gaps the recipes expose

From the device-block inventory (`codebase-cells` report):

- `Mosfet`: nf, Single, Cc1d, Chain, dummies per edge, split gates, mirror
  pins, rows 1, 2, 4, double gate; every variant DRC and ERC clean on sky130
  for one to four devices, ratioed, every class; pins on `li`. Gaps: no
  Interdigitated (drain boundary), even-nf chains, no 2-D beyond four rows,
  taps as one strip, dummies tied to bulk only.
- `FinFet`: shared CC row, SA/SB, edge gates; no dummies as cards, no split
  gates, mirror pins, rows or double gate; drain pins on M2 while others on M1.
- `Resistor`: segments, Single and Interdig CC rows, value model, folded
  jumpers on met1, end dummies on matched arrays, high-po recipe; no 2-D, no
  dummies on single devices, interleave disabled for multi-segment strings.
- `Capacitor`: single merged plate per device, three stacks; no recogniser for
  MOM, no keepout export: an LVS skip.
- `CapArray`: spiral, chessboard, block chessboard, split, MOM and MIM, dummy
  ring, internal routing, metrics; met1 to met3 hard-wired; MIM recipe on
  sky130 only.
- `Bjt`, `Diode`: unit grids by `centro_assign`; NPN needs `dnwell`; no sky130
  BJT recogniser.
- `post_cell`: guard rings by role, well bridges, implant bridges, tap rings.
- Portability blockers to remove in the embedding: the sky130 role vocabulary
  resolved by literal layer name, the `li` assumption, policy tiers that are
  not deck data, recipes only on sky130 for MIM and BJT, rect-only geometry.

## 9. Research items

- Compaction with claim preservation (Q24): constraint-graph baseline,
  stochastic shrink on top; verdict before M6.
- When common centroid beats interdigitation or clustering: depends on device
  size, correlation distance and parasitics; no closed rule in the sources
  (REV open problem). The bounded pick needs a cost that includes LOD and R_s
  mismatch, not moments alone.
- Quadratic gradient coefficient, capacitor correlation distance, WPE
  coefficient, FinFET misalignment coefficient: unmeasured anywhere; the
  recipes that need them report the first uncancelled order instead.
- Dummy effect: no source measures it; conformance tests compare extraction
  with and without dummies per PDK.
- Substrate two-port macromodel calibration per deck (SUBN-20, SUBN-22).

## 10. v1 scope

M1: MOS-Adjacent, MOS-Interdigitated, MOS-Cc1d, MOS-Cc2d, MOS-Ratioed,
MOS-Chain, ENV-* at class, on sky130 and asap7. M2: FLOOR-RowStack,
FLOOR-GlueRow, ROUTE-MirrorPair, ROUTE-Crossover, ROUTE-Star, ROUTE-Spacing,
ROUTE-Shield, ROUTE-Keepout, ROUTE-EmWidth DC, ROUTE-AntennaRepair. M3:
RES-Array, CAP-Array, CAP-Chessboard, CAP-Spiral, CAP-BlockChessboard,
CAP-SplitDac, BJT-Array, DIODE-Array, FLOOR-Array, ROUTE-Array. Everything
marked v2 follows the acceptance set; research items follow their spikes.

## Sources

The research reports this catalog condenses, each with full technique cards
and every source item id: `hastings-mos` (MOS-01 to MOS-35), `hastings-passives`
(RES, CAP, IND, MAT), `matching` (PAT-01 to PAT-28), `noise-em` (SUBN-01 to
SUBN-29, EMR-01 to EMR-24), `surveys` (P-01 to P-31, R-01 to R-19),
`codebase-constraints` (1.0's rule and intent inventory, the 97 templates, the
84 requirement families), `codebase-cells` (the device blocks). They are
copied into `docs/ref/research/` so the item ids resolve.
