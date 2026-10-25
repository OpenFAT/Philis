# Philis constraint model: inventory

Repo root: `/home/omare/Documents/Projects/EDA-Tools/Layout/Philis`. Paths below are relative to it. `kernel/analog` is abbreviated `analog/`, `backend/annotator/src` is `ann/`, `frontend/library/src/lib.rs` is `lib/lib.rs`, `backend/dr/src/lib.rs` is `dr/lib.rs`. Nothing was executed; every statement is from reading the code and docs.

## 0. The enforcement model in one paragraph

A constraint is a `Copy` value implementing `Rule` (`analog/src/rule.rs:36-182`) scored against `Layout` (placement) or `Routes` (routing), or a hand-written `RuleBatch` (`rule.rs:187-323`) when one condition spans many devices. Batches sit in one of three arms of `Requirements<On>` (`analog/src/requirements.rs:21-24`): `hard` counts violations and is repaired by `project`, `budget` sums `residual` (overshoot as a fraction of the spec, floored at `FAIL_FLOOR = 1e-3` for a boolean fail, `rule.rs:533-544`) into Θ priced by `gp::Prices`, `cost` sums `cost`. A kind may be in `hard` or `budget` and also in `cost` (the gradient toward the feasible set), never in both `hard` and `budget` (`requirements.rs:5-20`). `known == false` means inputs are missing: search treats it as satisfied, a report must not certify it (`rule.rs:80-88`). `applicable == false` is a justified nothing-to-check (`rule.rs:74-79`). Repair dispatches on `RepairKind` (`rule.rs:9-31`): `None, Reroute, Mirror, Balance, KeepAway, Antenna, Shield, Em, Ir, Budget`. Every batch the annotator emits is wrapped in `Tagged { meta: BatchMeta { id, origin } }` (`rule.rs:327-330`, `ann/lib.rs:547-584`). The normaliser `over(excess, budget)` is `rule.rs:551-556`; `rule_criticality` turns `headroom` and `margin` into a weight in `[0, 1]` (`rule.rs:519-527`).

Cell-tier directives are not scored: `Constraints { unitization, guard_rings }` (`analog/src/constraints.rs:8-10`) is read cold by `kernel/cells`.

## 1. Constraint inventory

Tier legend: H = hard arm, B = budget arm, C = cost arm. "Emitter" names the code that constructs and pushes the batch; several kernel rules are built by `frontend/library` or `backend/dr`, not by the annotator.

### 1a. Placement tier (`Rule::On = Layout`)

| Name | Module | Tier and how decided | Fields (units) | Scores / checks | State read | `Rule`/`RuleBatch` overrides | Emitter |
|---|---|---|---|---|---|---|---|
| `Symmetry` | `analog/src/placement/symmetry.rs:30-35` | H + C, fixed (`ann/emit.rs:170-172`); exact integer equality, repaired by `project` not weight | `a, b: Target`; `axis: AxisId`; `mode: SymMode` (`Perfect` same orient, `Mirror` b = a·Mx180, `symmetry.rs:19-23`) | `x_a + x_b = 2·axis_x`, `y_a = y_b`; distinct partners share `hw, hh, variant`, orient up to `mode` (`:46-48, :106-118`). `cost = ((|ex|+|ey|)/L_ref)²` (`:41-45`); `residual` = mirror error in µm (`:51-54`) | `Layout.x/y/axis/hw/hh/variant/orient` | `cost, satisfied, residual, touches, retarget, mirror_pair (:68-73), matched_pair (:75-80), project` (lone pair about its midpoint, on-grid, `:85-93`) | `ann/emit.rs:163-172`: per `Compound`, one `Symmetry` per couple that is an equal couple of one set (same unit counts, or same drawn geometry when not unitized, `:144-161`) plus one self-pair per `selfs`; mode from `Compound.kind` (`:166`) |
| `SymmetryGroup(Vec<Symmetry>)` | `symmetry.rs:153` | H + C (same push) | the pairs of one compound, one shared axis | `project` puts every pair on one axis at the mean of their midpoints (`:207-221`); `criticality` = 1 iff violated (`:172-174`); `mirrored_pairs` lists `Mirror` pairs, `demote_mirrors` turns disallowed ones `Perfect` (`:191-200`, PLC-21) | as above | batch: `cost, violations, residual, kind "Symmetry", count, worst_cost, criticality, violating_ids, touched, retarget, mirror_pairs, matched_pairs, mirrored_pairs, demote_mirrors, project` | same |
| `DtiBand` | `analog/src/placement/dti.rs:18-29` | H + C (`ann/emit.rs:233-236`); either-or legality, side committed by `Layout::branch` | `a, b: Target`; `s_max_nm` (max gap that still shares one trench); `d_dti_nm` (min gap that is separated); `branch: BranchId`; `seed_isolate: bool` | legal iff `gap ≤ s_max ∨ gap ≥ d_dti` (`:66-69`, branch-agnostic); `cost = (miss/band)²` toward the committed side (`:58-61`); `residual = over(miss, d_dti − s_max)` (`:71-76`) | `Layout::edge_gap`, `Layout.branch` | `cost, satisfied, residual, touches, retarget, branch (:87-89)` | `ann/emit.rs:176-180`: one per compound couple when `ProcessNumbers.dti = Some((s_max, d_dti))`; `BranchId` = dense emission index, `seed_isolate = false` |
| `MatchedSet` | `analog/src/placement/matched_set.rs:26-47` | B + C (`ann/emit.rs:190-192`); one batch per set | `members: Vec<DeviceId>` (slot 0 = reference); `kind: MatchKind`; `family: Family`; `class: MatchClass`; `coeffs: Coeffs` (deck numbers, see 1d); `budget: Budget`; `gate_um2: Vec<f32>` (µm², netlist areas, read without units); `tol_nm: f32` (coincidence tolerance = half the cut lattice); `cell_of: Vec<u16>`; `gm_over_id: Option<f32>` (1/V, puts a mirror's ledger in %); `sigma_rand_override: Option<f32>` (MC σ) | per pair `(0, i)` a `Ledger` (`:100-214`): `sigma_rand` Pelgrom pair σ; `sigma_grad = ‖(S_x·Δx, S_y·Δy)‖`; `mu_thermal = TC·|ΔT̄|` (unit-weighted rise, plus mobility term on a % mirror ledger); `mu_lod = KVTH0·|⟨lod⟩_a − ⟨lod⟩_b|`; `coincidence = Δm/tol` when the unit counts admit a diffusion-legal CC row (MOS) or any CC row (R/C/BJT); `second_order_nm` second-moment residue as centroid offset. Violation iff `known ∧ usage > 1` (`:264-266`); `residual = Σ over(spent − allowance, allowance) ∨ (coincidence − 1)` (`mismatch.rs:280-284`); `cost = (Δm² + so²)/L_ref² + (μ_thermal/allowance)²` (`:254-263`) | `Layout.units` (per-unit x, y, weight, φ, lod), cell centres as fallback, `Layout::rise_at_point_mc`, `power_uw` | batch: `cost, violations, residual, unknown, worst_usage, kind "MatchedSet", count, retarget (sets cell_of), touched, reads_field = true, matched_pairs, matched_class, violating_ids, violating_residuals, ledger_rows, sizing_notes, offset_allowances` | `ann/emit.rs:183-192`: per `Intent.sets` entry; reference swapped to slot 0; `coeffs` by polarity (`:84-101`; R/C sets get `Coeffs::default()` so their ledger reads unknown); `budget = set_budget` (`:50-60`: an allocated MOS allowance, else `mismatch::choose(offset_sigma_mv, None, class limit if class source ≠ Role, kind)`) |
| `OrientationSet` | `analog/src/placement/orientation.rs:30-35` | `Axis` H; `Phi` by class via `phi_arm` (`analog/src/matching/class.rs:125-131`): Exceptional H, Moderate B, Minimal none (`ann/emit.rs:193-202`) | `members: Vec<DeviceId>`; `check: OrientCheck` (`Axis`: all members' current on one axis, none `Mixed`; `Phi`: every member's mean signed current equals member 0's, `:17-25`); `cell_of` | `Axis` residual 0/1; `Phi` residual `max |ΔΦx|+|ΔΦy| ≤ 4` (`:43-62`); unknown when any member has no placed units | `Layout.units` φ per unit | batch: `cost, violations, residual, unknown, kind "Orientation", count, retarget, touched, matched_pairs, violating_ids` | `ann/emit.rs:193-202`: MOS/Bipolar sets with ≥ 2 members |
| `Proximity` | `analog/src/placement/proximity.rs:11-15` | B + C always (`ann/emit.rs:203-232`); never hard | `a, b: Target`; `max_distance_nm` (edge-to-edge) | `gap ≤ max` (`:36-38`); `cost = (excess/max)²` (`:31-35`); `residual = over(gap − max, max)`; `usage = gap/max` | `Layout::centre/extent` | `cost, satisfied, residual, usage, touches, retarget` | `ann/emit.rs:203-207` Minimal-class sets (reference → each member); `:210-214` 2-device `Stack` leaves; `:216-223` each stage self (tail, shared bias) → both DiffPair devices; `:227-232` sidecar `GroupBlocks` (first → each, `Origin::User`). Distance = `Policy.proximity_nm` 5000 (`ann/policy.rs:64`) |
| `Isolation` | `analog/src/placement/isolation.rs:12-18` | B + C when substrate `EpiOnLowRes` with `epi_nm`; else C only and a `missing` entry (`ann/emit.rs:278-304`) | `a` aggressor, `b` victim: `Target`; `min_distance_nm` = `4·t_epi` or `4·2500` nm uncalibrated (`emit.rs:242-266`) | `gap ≥ min` (`:28-30`); `cost = (shortfall/min)²`; `residual = over(min − gap, min)` | `Layout::edge_gap` | `cost, satisfied, residual, touches, retarget` | `ann/lib.rs:542-544` → `emit::isolation`: every `Switching`/`Capacitive` aggressor × every victim not `related` (same block, set or compound; `lib.rs:523-530`); a sidecar `GroupBlocks` pull on the pair drops the Isolation and reports a conflict (`lib.rs:534-541, 588-597`) |
| `SubstrateBalance` | `analog/src/placement/isolation.rs:51-55` | C only (no threshold in Charbon §8.3.1) | `aggressor, a, b: Target` | `cost = ((d_ca − d_cb)/d_ab)²` on centres, 0 on the bisector (`:62-73`); default `satisfied`/`residual` | `Layout::centre` | `cost, touches, retarget` | `ann/emit.rs:309-324`: per 2-device `DiffPair` leaf × aggressor outside its block; `ann/lib.rs:545` (env `LOCAL_NO_SB` disables) |
| `HeatSeparation` | `analog/src/placement/heat.rs:12-16` | B + C (`lib/lib.rs:910-913`) | `victim, source: Target`; `min_gap_nm` = source power in µW (1 µm/mW, Hastings rule 14) | `gap ≥ min` (`:26-28`); `cost = ((min − gap)⁺/min)²`; `residual = over(min − gap, min)` | `Layout::edge_gap` | `cost, satisfied, residual, touches, retarget` | `heat::separations` (`:51-68`): sets of class ≠ Minimal × every cell with `power_uw ≥ source_uw` outside the set. Built by the library, not the annotator |
| `Environment` / `LiveEnvironment` | `analog/src/placement/environment.rs:54, 92-98` | B (`lib/lib.rs:896`); `Environment` is the report-only snapshot (`lib/lib.rs:2285`) | `Surroundings { wpe_nm: [f32; 2], ose_nm: [f32; 2], wpe_min_nm, ose_range_nm }` (`:20-25`; nm, `INFINITY` = none); `LiveEnvironment { pairs: (a, b, cell_a, cell_b), geo: Arc<EnvGeo>, wpe_min_nm, ose_range_nm }` | usage = worst of: a member's well edge nearer than `wpe_min`; WPE distances (capped at 10·min) differing by more than `ENV_TOL = 0.2`; OSE distances (capped at range) likewise (`:34-48`). Violation `usage > 1`; `residual Σ(u − 1)⁺`; unknown when the deck gives neither range | placed nwell and diffusion rects of every cell (`:106-170`) | batch: `cost, violations, residual, unknown, kind "Environment", count, worst_usage` | `lib/lib.rs:2770` builds it per matched pair from cell geometry. Library, not annotator |
| `SymmetryIsland` | `analog/src/placement/island.rs:14-17` | B + C (`lib/lib.rs:936-937`) | `members: Vec<Target>`; `touch_nm` (largest gap that counts as touching) | connected components under shared-edge adjacency (`:22-45`); `cost = residual = components − 1`; violation iff > 1 (`:53-63`) | `Layout::bbox` | batch: `cost, violations, residual, kind "SymmetryIsland", count, touched, retarget` | `lib/lib.rs:1752` per symmetry group. Library |
| `PlacePerf` | `analog/src/placement/perf.rs:19-29` | B (`lib/lib.rs:943`); `local = true` | `metric: String`; `nets`; `weights` (1/aF); `af_per_nm`; `limit` (1 headroom, 0 do-not-worsen); `reserve` (`RESERVE = 0.2`, `:13`); `items` (cell, per-variant pin box) | `used = Σ w·MST_len·af_per_nm` (rectilinear MST over pin boxes, `:79-97`); `residual = (used/(1 − reserve) − limit)⁺` (`:105-107`); `criticality`, `worst_usage` | `Layout.x/y/orient/variant` | batch: `cost, violations, residual, kind "PlacePerf", count, local, touched, criticality, worst_usage` | `PlacePerf::new(row, variants, RESERVE)` from a `PerformanceBudget` row. Library |
| `Utilization` | `analog/src/placement/utilization.rs:13-15` | B (`lib/lib.rs:932`) | `u_min: f32` (declared fill floor) | `used = footprint·u_min/Σ cell area`; `residual = (used − 1)⁺` (`:19-37`) | `Layout::footprint_nm2`, `hw, hh` | batch: `cost, violations, residual, kind "Utilization", count, criticality, worst_usage` | Library (policy, not physics; `:10-12`) |

### 1b. Routing tier (`Rule::On = Routes`)

| Name | Module | Tier and how decided | Fields (units) | Scores / checks | State read | `Rule`/`RuleBatch` overrides | Emitter |
|---|---|---|---|---|---|---|---|
| `Antenna` | `analog/src/routing/antenna.rs:14-25` | H (`ann/extract.rs:98`) | `net`; `max_ratio_x100: i32` (deck's tightest ratio, fallback); `gate_area_nm2: i64` (total gate area the net drives); `margin_pct: u8`; `stack: Option<&Stack>` | with a stack: worst etch stage of `Stack::antenna` (per connected piece, charged only the gates it reaches, latent merge, diode credit; `stack.rs:341-391`); without: all routed metal over the gate area against the tightest ratio (`:63-79`). `cost = (ratio − limit)·100`; `residual = over(ratio − limit, limit)`; `headroom`, `usage`, `margin` | `Routes.wires`, `Routes.cell` (cell metal), gate pins | `cost, satisfied, known (net has shapes), touches, headroom, usage, margin, residual`; `REPAIR = Antenna`, `LOCAL = true` | `ann/extract.rs:74-98`: one per net with FET gate area when `ProcessNumbers.antenna_max_ratio` is `Some`; margin `Policy.antenna_margin_pct` 20 |
| `Differential` | `analog/src/routing/differential.rs:32-43` | B (`ann/extract.rs:142`) | `pos, neg: NetId`; `max_len_delta_pct10: i32` (0.1 %); `same_layer_required: bool`; `stack`; `aggressor_weight: Option<&[f32]>` | with a stack: worst of ground-C delta, terminal R (sorted, star model), via count per cut layer, weighted coupling asymmetry, all in % of the pair mean (`:60-123`); without: per-layer signature delta (length, area, square shapes) or plain length delta (`:124-148`). `cost = excess %`; `residual = over(mismatch − budget, budget)` | `Routes.wires/terms` | `touches, cost, satisfied, residual, known (both sides routed)`; `REPAIR = Mirror`; no `project` (dr mirrors by rip-up) | `ann/extract.rs:111-119`: one per compound `net_pairs` entry with both nets non-rail, `diff_pct10 = Policy.diff_pct10` 50, `same_layer_required = true` |
| `CrosstalkExclusion` | `analog/src/routing/crosstalk.rs:11-17` | B (`ann/extract.rs:143`) | `a, b: NetId`; `min_spacing_nm`; `margin_pct` | min same-layer edge gap ≥ floor (`:22-32, :53-55`); nets on no shared layer pass with headroom 1; `cost` = shortfall nm; `residual = over(floor − d, floor)`; `usage = floor/d` | `Routes.wires` | `cost, satisfied, known (both routed), touches, keepaway (:65-67), separation (:69-71, same-layer only), headroom, usage, margin, residual`; `REPAIR = KeepAway` | `ann/extract.rs:121-141`: (a) every Voltage set's gate nets × its drain nets; (b) every routed victim net (Sensitive/Bias/Reference) × every routed aggressor net (Clock/DigitalSwitching/Noisy). Spacing = `route_space_nm · max(spacing_multiple[class])` with multiples `[8, 7, 3, 1]` (`policy.rs:65`), margin 25 |
| `ParasiticBudget` | `analog/src/routing/parasitic.rs:17-27` | B (`ann/extract.rs:158`) | `net`; `max_len_nm: i64` (fallback length cap); `max_c_af: i64` (ground-C budget, 0 = check length); `margin_pct`; `stack` | spent = `Stack::ground_af` (area + fringe per layer union) when stack and C budget, else drawn length (`:31-36`); `cost = (spent/budget)²`; `residual = over(spent − budget, budget)`; `headroom`, `usage` | `Routes.wires` | `cost, satisfied, known (length > 0), touches, headroom, usage, margin, residual, extract (placeholder 1 mm / 100 fF, :81-94, unused by the annotator)`; `REPAIR = Budget` | `ann/extract.rs:146-158`: every net touching a device with `c_budget_af` and a deck `wire_af_per_um`; budgets from `classify::budgets` (`ann/classify.rs:25-35`: Sensitive/Bias/Reference 1× load, Signal 2×, aggressor classes 2× wire and no coupling budget, rails none); margin by class `[35, 30, 25, 20]` (`policy.rs:66`) |
| `CouplingBudget` | `analog/src/routing/coupling.rs:141-155` | B (`ann/extract.rs:191`) | `net` (victim); `max_coupling_af: i64`; `margin_pct`; `stack`; `exclude: Option<NetId>` (own shield); `aggressor_weight: Option<&'static [f32]>` | `total_af = Σ_other w·net_pair_af(victim, other, screens)` (`:180-197`); `net_pair_af` (`:70-131`) merges runs, screens nearer wires, adds end-on and cross-layer overlap terms; `cost = residual = over(total − budget, budget)`; `headroom`, `usage` | all `Routes.wires` | `cost, satisfied, headroom, usage, margin, residual, touches`; `REPAIR = KeepAway` | `ann/extract.rs:179-191`: every routed net with `max_coupling_af`; `exclude` = its shield reference; weights `CouplingBudget::default_weights` zero the rails (`coupling.rs:162-172`) |
| `Shield` | `analog/src/routing/shield.rs:21-27` | B (`ann/extract.rs:203-205`) | `victim, reference: NetId`; `min_coverage_pct: u8`; `max_gap_nm` | coverage = fraction of the victim's run with `reference` metal on both sides, same layer, within gap (`:32-64`); `residual = over(want − coverage, want)`; `usage = coverage` | `Routes.wires` | `cost, satisfied, known, residual, usage, touches, shield (:112-114)`; `REPAIR = Shield` | `ann/extract.rs:193-205`: every routed victim net, reference = `quiet_ground` (`:215-220`: analog-ground name, else untouched Ground, else lowest Ground flagged shared), only when an aggressor net exists; coverage 80 %, gap `2·route_space_nm` (`policy.rs:67-68`) |
| `CommonNode` / `CommonNodes` | `analog/src/routing/common_node.rs:20-34, 38-46` | B; `repair_kind = Balance`, `local = true` | `CommonNode { net, groups: Vec<Vec<Rect>>` (terminal groups), `feeds: Vec<Rect>` (where current enters), `max_delta_ohm: f32` (≤ 0 unknown or no balance), `star: bool }`; `CommonNodes { nodes, stack: &Stack, halo_nm (root halo), joins: Vec<Join> }` | `ΔR = max − min` of the groups' parallel R from the feeds (`:51-70`); `usage = ΔR/max_delta_ohm`; `star_broken` when branches meet outside the root halo (`:82-114`); violations = star breaks + overshoots (`:141-143`) | `Routes.wires`, pins | batch: `cost, violations, residual, kind "CommonNode", repair_kind, local, count, worst_usage, unknown, touched, violating_ids` | requests written by `ann/extract.rs:233-308` into `Intent.common_nodes/stars/kelvins`; rules built by `lib/lib.rs:2243-2273` (`star: false`) and `dr/lib.rs:1151-1158` |
| `PlateRatio` / `PlateRatios` | `analog/src/routing/plate_ratio.rs:17-39, 86` | B; `repair_kind = None` (dr equalises with stubs) | `PlateSet { top: NetId, bits: Vec<(NetId, u32 units)>, c_unit_af (NAN unknown), array: Rect }`; `PlateRatio { set, tol_pct10, stack: &Stack, space_nm }` | per bit lead C = ground C + screened coupling of its metal outside the array (`:44-57`); spread of per-unit C over `c_unit` in % vs `tol` (`:68-80`); `separations` top × every bit at `space_nm`, `no_cross = true` (`:123-127`) | `Routes.wires` | batch: `cost, violations, residual, kind "PlateRatio", repair_kind, count, unknown, touched, violating_ids, separations` | `dr/lib.rs:380-384` from a capacitor `Unitization`. Not the annotator |
| `Electromigration` | `analog/src/routing/em.rs:102-116` | H (`lib/lib.rs:2646-2648`) | `net`; `limits: [(layer, Limit); 16]` with `Limit { ua_per_um (µA/µm), ua_per_cut (µA), blech ((µA/µm)·µm) }` (`:27-36`); `stack`; `front_row: bool` | per routed metal shape `width ≥ I/J` or Blech-immortal (`Limit::width_nm`, `:52-62`); per via group `cuts ≥ ⌈I/I_cut⌉` with front-row counting (`:69-74, :165-242`); currents from `net_flow` (`current.rs:34-138`, KCL over a least-R tree, chord bound); `residual` = worst `(need − have)/need`; `usage` = worst need/have; unknown without stack, terminal current, reached terminal or limited layer | `Routes.wires/cell/terms` | `cost, satisfied, known, residual, headroom, usage, touches`; `REPAIR = Em` (width is not a search resource, `rule.rs:24-26`) | `lib/lib.rs:2646-2648`, `dr/lib.rs:1646`. Derating `derate` (`em.rs:19-24`). Not the annotator |
| `EsdWidth` | `analog/src/routing/em.rs:267-271` | H (`lib/lib.rs:988-993`) | `net`; `area_um2` (`esd_area_um2(hbm_v, ρ, C_V)`, `:247-250`; `metal_family`, `:255-260`); `stack` | every routed metal shape of known thickness: short side ≥ `area·1e6/thickness` (`:276-287`) | `Routes.wires` | `cost, satisfied, known, residual, usage, touches`; `REPAIR = Em` | Library, pad nets. Not the annotator |
| `IrDrop` | `analog/src/routing/ir.rs:22-31` | B (`lib/lib.rs:999-1004`) | `net`; `current_ua: i32` (negative unknown); `max_drop_uv: i64`; `margin_pct`; `stack` | drop = terminal-resolved `net_flow.drop_uv` else `I·path_resistance_ohm` (`:35-43`); `residual = over(d − budget, budget)`; `headroom`, `usage` | `Routes.wires/cell/terms` | `cost, satisfied, known, residual, headroom, usage, margin, touches`; `REPAIR = Ir`, `LOCAL = true` | budgets `ann/ir.rs:21-47` (`(net, |I|, ΔV)`: rails and signal nets carrying ≥ `ir_high_current_share` of the max current; `ΔV = ir_headroom_share·headroom` or `ir_rail_share·V_DD`); rules built by `lib/lib.rs:999-1004` |
| `MetalOverGate` | `analog/src/routing/metal_over_gate.rs:17-23` | H in intent; evaluated directly by dr (`dr/lib.rs:1815`), not pushed into `Requirements` | `rect: Rect` (placed matched gate, Moderate+); `cell: u32`; `metals: [u16; 8]` (routed metal layer ids) | Σ overlap of every routed metal shape of any net with `rect`, µm² (`:28-43`); satisfied iff 0 | `Routes.wires` | `cost, satisfied, residual`; `REPAIR = None` | dr. Not the annotator |
| `PerformanceBudget` | `analog/src/routing/performance.rs:28-49` | B (`lib/lib.rs:884`); `repair_kind = Budget` | `metric: String`; `nets`, `weights` (1/aF = share of headroom per aF); `af_per_nm`; `limit` (1 headroom / 0 do-not-worsen); `r_nets`, `r_weights` (1/Ω); `diff_pairs`, `diff_weights`; `coupling: Vec<(a, b, w)>` | `used = Σ w·length·af_per_nm` (ground C only, `:60-62`); `residual = (used − limit)⁺`; a row carrying R, diff or coupling terms reads `unknown` until RTE-21 measures them (`:77-79`) | `Routes::length` | batch: `cost, violations, residual, unknown, kind "PerformanceBudget", repair_kind, count, criticality, worst_usage, violating_ids, touched` | rows from `ann/budget.rs:28-73` (one per finite spec bound, weights `sign·∂f/∂x / margin`, margins from `ann/allocate.rs:27-36`); pushed by `lib/lib.rs:884` |

### 1c. Cell tier (`analog::Constraints`, not scored)

| Name | Module | Fields (units) | Meaning | Emitter |
|---|---|---|---|---|
| `Unitization` | `analog/src/cell.rs:52-76` | `devices`; `device_type: DeviceKind`; `dev_nf: Vec<u16>` (units per device); `target_ratio: Vec<u16>`; `unit_w, unit_l` (nm); `series_parallel: SeriesParallel` (`:80-83`); `dummy_required`; `route_matching_required`; `class: Option<MatchClass>`; `kind: Option<MatchKind>`; `series: Vec<u16>`; `style: Option<ArrayStyle>` | decompose the devices into identical unit fingers so ratios are exact integers; the pattern is the cell generator's | `ann/constraints.rs:66-175`: one per matched set with one finger W/L (`:71-113`; drawn in schematic fingers, `dummy_required` = MOS/R/C and class ≥ Moderate `:48-50`, `route_matching_required` = kind ≠ Ratio or class ≥ Moderate, split-DAC bridge excluded `:54-63`); one per (kind, model, bulk, W, L) class of every recognised block (`:119-146`, dummies and route matching on); identical parallel FETs still uncovered (`:150-173`, neither) |
| `GuardRingRequirement` | `analog/src/cell.rs:8-21` | `device`; `ring_type: GuardRingType` (`Tap { in_well }`, `Ecgr`, `Tub { id }`, `Hcgr`, `:33-46`); `shareable`; `min_width_nm`; `max_ring_resistance_mohm` (literal 100 000, `ann/rings.rs:72`); `connection_net`; `role: RingRole` (`Injector`, `Aggressor`, `Victim`, `:26-30`) | a latch-up / collection ring around one device, tied to a net | `ann/rings.rs:60-137`, first matching row: user `IsolatedTub` → `Tub` victim ring on its tie when drawable; electron injector → supply-tied `Ecgr`; hole injector → ground-tied `Hcgr`; undrawable collector → majority tap on its bulk; aggressor → shareable tap on its bulk; victim → shareable tap on `quiet_ring_net` only once some device got a ring; rows 5–6 skipped on epi (`:48-58`) |
| `NetClassification` | `analog/src/metadata.rs:8-15` | `net`; `class: NetClass` (`:17-36`: `Signal, Clock, Supply, Ground, Sensitive, Substrate, Bias, Reference, DigitalStatic, DigitalSwitching, Noisy`); `c_budget_af`; `max_coupling_af` (aF, `None` unbudgeted) | the net's functional class and the two routing budgets it implies | `ann/classify.rs:49-94` pre-pass (name role, structure: bulk-only → Substrate, matched gate or gates-only → Sensitive), `refine` (`:188-317`) once sets exist, `set_class` (`:320-323`) |

### 1d. Matching math the rules read (not rules themselves)

| Item | Module | What it gives |
|---|---|---|
| `Family` (`Mos, Bipolar, Diode, Resistor, Capacitor`), `Family::of` (inductor → `None`) | `analog/src/matching/class.rs:12-32` | the class-limit table a set belongs to |
| `GateStrap` (`PolyBar, PolyBarFar, MetalIsolated`), `MosEnv { dummy_reach_nm, moat_nm, wpe_nm, gate_ext_extra_nm, gate_strap }`, `mos_env(class, process)`; `PassiveEnv { min_dummies, dummy_span_nm, width_floor_permille, length_floor_x }`, `resistor_env`; `TIERS`, `missing_tiers` | `class.rs:39-119` | what a MOS or resistor set of one class asks of its surroundings, from the deck's `[MIN, MOD, EXC]` tier keys (`dummy_reach_nm, gate_ext_extra_nm, lod_moat_ext_nm, res_dummy_span_nm, res_length_floor_x, res_width_floor_permille, wpe_clearance_nm`). Read by `kernel/cells`, not by rules |
| `phi_arm(class)` | `class.rs:125-131` | Φ check armed: Exceptional hard, Moderate budget, Minimal off |
| `ClassLimit { Mv, Pct }`, `limit(family, kind, class)` | `class.rs:135-156` | Hastings 6σ class limits: MOS Voltage 10/3/1 mV, MOS Current 10/3/1 %, Bipolar Voltage 2/0.5/0.1 mV, Bipolar Current 8/2/0.5 %, R or C Ratio 1/0.1/0.01 %; `None` for a diode or a meaningless pair |
| `sigma_pair`, `sigma_current_pct`, `sigma_voltage_mv` | `analog/src/matching/mismatch.rs:10-15, 51-59` | Pelgrom pair σ for unequal areas; current-domain and voltage-domain σ with the β share |
| `MatchKind` (`Voltage, Current, Ratio`), `LedgerUnit` (`Mv, Pct`) | `mismatch.rs:21-25, 32-47` | which quantity a set matches and what its ledger is in |
| `GRADIENT_SHARE = 0.3`, `Budget` (`Eta, Sigma1Mv, Allowance, Sigma1Pct`), `from_class`, `allowance`, `to_pct`, `choose` | `mismatch.rs:67-135` | how much systematic mismatch a pair may spend; first source given wins: circuit 1σ, explicit allowance, class limit (User/Spec only), else `η·σ_rand` |
| `Coeffs { avt_mv_um, svt_uv_per_um, svt_xy, kvth0_mv_um, tc_uv_per_k, abeta_pct_um, ka_pct_um, tc_ppm_per_k, vbe_tc_uv_per_k, sd_pct_per_mm, mobility_exp, die_temp_k }` | `mismatch.rs:140-166` | deck numbers per set; `None` = that term is unknown. `svt_xy`: no deck key sets it (`:145`) |
| `svt_of_l`, `sigma_grad_mv`, `mobility_pct`, `bjt_sigma_vbe_mv`, `ratio_thermal_pct` | `mismatch.rs:171-204` | S(L) fit, gradient σ, mobility thermal term, ΔV_BE from ΔI_S, R/C thermal ratio error |
| `Ledger { sigma_rand, sigma_grad, mu_thermal, mu_lod, allowance, coincidence, second_order_nm, delta_m_nm, known, unit }`, `spent`, `usage`, `residual`; `LedgerRow` | `mismatch.rs:222-284, 246-261` | one pair's mismatch ledger; the report row (MAT-13) |
| `Pt`, `Axis`, `Sums`, `sums`, `centroid`, `second`, `phi`, `phi_equal`, `phi_equal_all`, `mirror_allowed`, `mirror_allowed_units`, `cancelled_order` | `analog/src/matching/moments.rs:9-207` | unit moments, Hastings' Φ test, whether an `Mx180` mirror keeps every member's current direction, how many leading moment orders a figure cancels |
| `Fill` (`Compact, Dispersed, Balanced`), `grids`, `cc_feasible`, `spiral`, `centro_assign`, `segment_row`, `scale2`, `Grid` (`refl, free_pair, put_pair, rect, rc, chessboard, spread, corridor`), `Outer`, `diffusion_legal`, `nth_order_rows`, `segmentation`, `partial_segments`, `diffusion_cc_row` | `analog/src/matching/pattern.rs:12-485` | common-centroid cell assignment on a grid, resistor segment rows (Hastings Table 8.5), diffusion-legal CC finger rows for ratioed MOS mirrors, Nth-order (Dai) row stacking, Hastings resistor segmentation sweeps |
| `gradient_caps`, `inl_dnl`, `msys`, `second_um2`, `attenuation_cap`, `nonunit_dims`, `split_dac_assign` | `analog/src/matching/dac.rs:13-147` | capacitor-array figures under an oxide-thickness gradient; split-DAC bridge value, non-unit plate dimensions, DACP §IV-C slot order |
| `SizingNote`, `area_need_um2`, `dstar_um`, `notes` | `analog/src/matching/sizing.rs:15-72` | report-only sizing-reach diagnostics: area for a class limit, budget below σ_rand, Pelgrom D* |
| `Stack`, `Layer`, `DiodeCredit`, `ground_af`, `resistance_ohm`, `path_resistance_ohm`, `terminal_resistance_ohm`, `fed_resistance_ohm`, `port_graph`, `lateral_af`, `lateral_run_af`, `cross_af_um2`, `connected`, `antenna`, `parallel` | `analog/src/routing/stack.rs:10-463` | the deck's per-layer parasitics, resistances and antenna stages that every routing rule measures drawn metal with |
| `NetFlow`, `net_flow` | `analog/src/routing/current.rs:9-138` | DC current per routed shape and the largest drop, from terminal currents (PWR-01) |

## 2. Intent model (`analog/src/intent.rs`)

Re-exported into the module: `Family`, `MatchClass`, `MatchKind` from `matching::class` and `AxisDir` from `placement::symmetry` (`intent.rs:7-8`). `MatchClass` is `pnr_core::MatchClass { Minimal = 0, Moderate = 1 (default), Exceptional = 2 }` (`kernel/core/src/process.rs:107-112`), ordered. There is no `Confidence` type anywhere in the crate; the nearest things are `EvidenceLevel` (categorical provenance), `MatchSpec.weight`, `Victim.weight` and `Order.weight` (numeric shares), and `BatchMeta.origin`.

| Type | Lines | Fields | Meaning |
|---|---|---|---|
| `ConstraintId(pub u32)` | 14 | | dense id within one `Problem`, assigned in emission order (permutation-invariant) |
| `Origin` | 19-34 | `Pattern { template }`, `NetClass`, `Symmetry { seed: ConstraintId }`, `SharedBias`, `PassiveSet { rule }`, `Sensitivity`, `User { index }`, `Derived { parent }` | where a batch or inferred constraint came from: a recognised template, net classes / substrate / cross-block, symmetry propagation from a seed, a shared-bias group, a passive or bipolar set rule (`"divider"`, `"symmetric_r"`, `"bandgap_r"`, `"dac_bank"`, `"cap_ratio"`, `"split_dac"`, `"degeneration"`, `"diode_set"`), sensitivities, a sidecar entry index, a parent constraint. Assigned at `ann/lib.rs:566-572` for placement (pre-tagged keeps its origin; first touched device in a block → that block's template; Isolation/SubstrateBalance → NetClass) and `:581` for routing (all NetClass); sets at `ann/sets.rs:301-309` |
| `BatchMeta` | 37-40 | `id: ConstraintId`, `origin: Origin` | identity of a `Tagged` batch |
| `ClassSource` | 44-48 | `User`, `Spec`, `Role` | where a set's `MatchClass` came from; User and Spec count as explicit (the class limit becomes a budget only then, `ann/emit.rs:56`) |
| `Half` | 51-54 | `A`, `B` | which side of a symmetry couple a member is |
| `Term` | 58-67 | `G, D, S, B, C, E, P, N` | terminal names, the pin columns of the pattern matcher |
| `ArrayStyle` | 70-76 | `Adjacent`, `Interdigitated`, `CommonCentroid1d`, `CommonCentroid2d`, `Any` | the layout pattern a set asks for; chosen by `ann/class.rs:91-98` from class and kind (Minimal adjacent; Moderate interdigitated for a source-sharing Current set, else 1-D CC; Exceptional 2-D CC) |
| `Member` | 80-85 | `device`, `parallel: u16`, `series: u16`, `half: Option<Half>` | one device of a set in units of the set's `UnitGeom` |
| `UnitGeom` | 89-93 | `w_nm`, `l_nm`, `model: u16` | the unit every member is built from (`ann/sets.rs:136-209`) |
| `MatchSpec` | 98-117 | `id`, `origin`, `members`, `reference: Option<usize>` (mirror diode or bank termination), `family: Family`, `kind: MatchKind`, `class: MatchClass`, `class_source`, `unit: Option<UnitGeom>`, `allowance: Option<f32>` (1σ systematic, mV or %), `weight: Option<f32>` (share of spec variance), `style: ArrayStyle`, `compound: Option<u16>` | what extraction knows about one matched set; `MatchedSet` is built from it (`ann/emit.rs:183-192`) |
| `SymKind` | 120-123 | `Mirror`, `Perfect` | how a compound's partners are drawn: reflected, or identical orient. Perfect when the compound holds an Exceptional Voltage set (`ann/lib.rs:384-390`) |
| `Compound` | 127-142 | `id`, `axis: AxisId`, `dir: AxisDir` (`V` default, `H`), `kind: SymKind`, `pairs: Vec<(DeviceId, DeviceId)>` (half A first), `selfs`, `net_pairs: Vec<(NetId, NetId)>`, `self_nets`, `set_pairs: Vec<(u16, u16)>` | devices symmetric about one axis (EXT-14), from `ann/symmetry.rs:317-391` |
| `ReqType` | 146-152 | `MatchSym < MatchBlock < ProxBlock < Sym < ProxNet` | requirement-edge types in survey §3.2.1 importance order (`ann/graph.rs:38-120`) |
| `GroupKind` | 155-160 | `Matching`, `Symmetry`, `Proximity`, `Root` | node kinds of the HSMPG tree (`ann/graph.rs:130-187`) |
| `GroupNode` | 165-169 | `kind`, `devices`, `children: Vec<u32>` | one tree node; `Root` last |
| `Order` | 175-182 | `steps: Vec<Vec<DeviceId>>`, `dir: AxisDir`, `reversible: bool`, `weight: f32` | a placement order: user `Order` (irreversible), current paths (V, weight = path current / max), signal stages (H), instance arrays (H) (`ann/lib.rs:434-457`, `ann/flow.rs`) |
| `Region` | 185-191 | `Unknown, Off, Subthreshold, Triode, Saturation` | operating region from the op point (`ann/evidence.rs:80-94`) |
| `DeviceRole` | 194-203 | `Unknown, Amplifier, CurrentSource, Cascode, Load, Switch, Diode, Passive` | role per device (`ann/evidence.rs:103-156`): Switch = triode with a digital gate; Diode = D = G; Cascode = saturated, Bias gate, non-rail source; CurrentSource = saturated, Bias/Reference gate or shared-bias member; Amplifier = saturated, Signal/Sensitive gate; Load = in a Load leaf; Passive = R/C/L |
| `DeviceFacts` | 206-209 | `region`, `role` | per-device facts |
| `RcClass` | 212-218 | `Unknown, None, R, C, Rc` | whether a net's series R or ground C spends a spec margin (`ann/budget.rs:60-71`); DAC top plate `C`, bottom plates `R` (`ann/extract.rs:309-324`) |
| `EvidenceLevel` | 221-229 | `User, Port, Name, Testbench, Structure, OpPoint, Default` | how a net's class was decided (`ann/classify.rs:289-304`) |
| `NetFacts` | 232-239 | `evidence`, `port: bool`, `dc_mv: Option<(i32, i32)>`, `rc: RcClass`, `z_ohm: Option<f32>`, `shield_ref: Option<NetId>` | per-net facts |
| `Inject` | 243-250 | `Switching`, `Capacitive`, `MinorityElectron`, `MinorityHole` | how an aggressor injects into the substrate |
| `Aggressor` | 253-257 | `device`, `inject`, `reason: &'static str` | `ann/substrate.rs:23-40` (terminal on an aggressor net; capacitor-coupled) and `:76-143` (pin diffusion within `inj_series_ohm`; forward-biased bulk) |
| `Victim` | 260-264 | `device`, `weight: f32`, `reason` | Moderate+ set members (weight = set weight, else 3 Moderate / 10 Exceptional) and Bias/Reference-gated FETs (weight 1) (`ann/substrate.rs:45-62`) |
| `CommonNodeReq` | 267-273 | `net`, `set: u16`, `a`, `b: Vec<DeviceId>`, `term: Term` | two terminal groups on one shared net whose routed R must agree |
| `StarReq` | 276-280 | `net`, `root: Option<(DeviceId, Term)>`, `branches` | a shared node other devices feed: branches meet only at the root |
| `KelvinReq` | 283-287 | `device`, `term`, `sense: Vec<(DeviceId, Term)>` | force/sense split of a resistor end sensed by a Voltage set's gates (`ann/extract.rs:294-308`) or a sidecar entry |
| `Diagnostic` | 291-295 | `kind: &'static str`, `devices`, `message` | a finding reported instead of guessed (kinds seen: `ambiguous_symmetry`, `conflict`, `asymmetric_net_pair`, `mixed_kind_set`, `unit_deck_incomplete`, `non_integer_ratio`, `resistor_widths_differ`, `unknown_size`, `series_units_exceptional`, `beyond_exceptional_trim`, `spec_infeasible_at_schematic`, `no_layout_margin`, `bridge_cap_value`, `degeneration_ratio`, `shared_shield_return`, `probe_bias`, `sidecar_unknown_name`, `sidecar_unsupported`, and the audit kinds `vgst_low, clm_mismatch, cascode_ratio, cascode_bulk, bjt_ratio, vce_unequal, audit_not_checked`, `ann/audit.rs:12`) |
| `Intent` | 298-313 | `sets`, `compounds`, `tree`, `order`, `nets`, `devices`, `aggressors`, `victims`, `common_nodes`, `stars`, `kelvins`, `diagnostics` | extraction's whole contract (EXT-12) |

## 3. Topology catalog (`ann/catalog.rs`)

`PATTERNS` (`:2338-2447`) holds 97 patterns: 91 MOS plus 6 bipolar. Priority tiers (`:16-21`): 2-device 5–13, 3-device 14–19, 4-device 20–29, 5-device 30–39, 6-device 40–49, 8-device 50–59. The matcher (`ann/pattern.rs:268-384`) returns every match of every pattern (`recognize_all`, `:391-404`); `select_disjoint` (`:455-475`) keeps a greedy disjoint set by `(priority desc, declares pairs, sorted canonical labels, sorted names)` for `Problem.blocks`; the overlapping matches still feed `graph::requirements` (`ann/lib.rs:291`).

Roles (`Roles { pairs: (slot, slot, BlockKind), selfs, prox }`, `:2456-2463`) come from the `ROLES` table (`:2472-2504`) or, for a 2-slot pattern absent from it, from `BlockKind::from_template` (`ann/block.rs:50-64`: name contains `diff_pair`/`diff_switch` or starts with `cross_coupled` → DiffPair; contains `mirror` → CurrentMirror; contains `load` → Load; contains `cascode` or equals `source_follower`/`series_stack` → Stack prox; else none). A composite with ≥ 3 slots and no `ROLES` entry yields no constraint (`:2469-2471, :2514-2516`).

What each leaf kind emits (`ann/emit.rs:4-12`, `ann/lib.rs`):
- **DiffPair leaf**: symmetry seed (`lib.rs:220-225`) → `Compound` → `SymmetryGroup` H+C, `DtiBand` H+C, `Differential` B on the mirrored nets, `Sensitive` gate nets (`classify.rs:225-227`), `CrosstalkExclusion` G×D; `MatchBlock` edge → `MatchedSet` B+C with `kind = Voltage` (`class.rs:37-46`), role `InputPair` → class Moderate by default, `OrientationSet` Axis H + Phi B; `Unitization` with dummies and route matching; `is_sensitive` (`block.rs:69-71`) → victims; `SubstrateBalance` against outside aggressors; common node on the shared source and a star when others feed it (`extract.rs:243-292`); degeneration resistor set (`passive.rs:186-213`).
- **Load leaf**: symmetry seed; `MatchedSet` Current, role `LoadOfPair` when its compound also holds an input pair → Moderate; `OrientationSet`.
- **CascodePair leaf**: symmetry seed; `MatchedSet` Current; `LoadOfPair` role under the same condition, else `Other` → Minimal → `Proximity` pull.
- **CurrentMirror leaf**: no symmetry seed; `MatchedSet` Current with `reference` = the diode slot (`sets.rs:317-323`), role `BiasMirror` → Minimal → `Proximity` reference→member, Orientation Axis H only; gate nets Sensitive in the pre-pass (`lib.rs:189-198`), demoted to `Bias` by `bias_lines` or a CurrentSource gate (`classify.rs:284-286`, `lib.rs:468-475`).
- **Stack prox**: `Proximity` B+C (`emit.rs:210-214`), `ProxBlock` edge.
- **selfs**: on-axis `Symmetry` self-pair once `symmetry::analyze` confirms the device self-symmetric; `Proximity` to both input devices (`emit.rs:216-223`); `ProxBlock` edge to the first pair (`graph.rs:81-83`).

### 3a. Differential pairs and inputs

| Template | Lines | Slots | Roles | Emits |
|---|---|---|---|---|
| `diff_pair` | 124-143 | 2: same type, exact W+L, signal gates; shared **signal** source, distinct G and D, not cross-coupled | (0,1) DiffPair (fallback) | DiffPair set |
| `diff_switch` | 1981-2000 | 2: exact, no diode, gates unchecked; shared signal source | (0,1) DiffPair | DiffPair set |
| `cross_coupled` | 190-202 | 2: exact; G_a = D_b, D_a = G_b, shared S | (0,1) DiffPair | DiffPair set |
| `cross_coupled_split_source` | 207-219 | 2: exact; cross-coupled, distinct S | (0,1) DiffPair | DiffPair set |
| `diff_pair_with_tail` | 496-515 | 3: pair + tail (D = pair S) | ROLES: (0,1) DiffPair, self 2 | DiffPair set + tail on axis, tail Proximity |
| `diff_pair_diode_load` | 519-543 | 3: pair + complementary diode load on one drain | none (not in ROLES) | nothing from the composite |
| `diff_pair_with_reference` | 2260-2284 | 3: pair, slot 1 gate driven by a complementary diode | none | nothing |
| `diff_pair_with_active_load` | 784-805 | 4: pair + complementary exact load pair sharing gate | ROLES: (0,1) DiffPair, (2,3) Load | DiffPair + Load sets |
| `diff_pair_with_mirror_load` | 810-837 | 4: pair + diode/mirror load (shared G and S) | ROLES: (0,1) DiffPair, (2,3) Load | DiffPair + Load sets |
| `diff_pair_with_cascodes` | 1497-1519 | 4: pair + same-type cascodes sharing gate | none | nothing |
| `diff_pair_with_split_cascodes` | 2134-2156 | 4: pair (signal source) + cascodes with distinct gates | none | nothing |
| `folded_load_pair` | 2161-2182 | 4: pair drains into complementary device sources | none | nothing |
| `complementary_diff_pair` | 1773-1806 | 4: NMOS pair + PMOS pair, shared inputs, signal sources | ROLES: (0,1) DiffPair, (2,3) DiffPair | two DiffPair sets |
| `cmfb_sense_pair` | 1811-1826 | 3: two sensing FETs + diode reference, shared D and S | none | nothing |
| `common_mode_sense_pair` | 1358-1375 | 2: same gate, tied drains, distinct sources, no diode | none (Group) | nothing |
| `degeneration_pair` | 407-419 | 2: shared D, distinct G and S | none (Group) | nothing |
| `diff_pair_with_degen` (not in catalog) | `ann/lib.rs:165-169` | 2: `passive::degenerated_pairs` (`passive.rs:220-250`): same signature, distinct gates, sources each through one resistor to a common node | DiffPair block | DiffPair set |
| `bjt_diff_pair_npn` / `_pnp` | 2314-2322 (macro), 2335-2336 | 2: `Kind`, exact emitter; shared signal emitter, distinct B and C | ROLES: (0,1) DiffPair | DiffPair set, kind Voltage (`class.rs:38-43`) |

### 3b. Current mirrors and bias

| Template | Lines | Slots | Roles | Emits |
|---|---|---|---|---|
| `current_mirror` | 150-161 | 2: diode ref + same L; shared G and S | (0,1) CurrentMirror | mirror set, reference slot 0 |
| `current_mirror_output_pair` | 166-183 | 2: no diodes, same L; shared G and S, distinct D | (0,1) CurrentMirror | mirror set (no reference) |
| `current_mirror_3` | 473-490 | 3: diode + 2 outputs, same L | ROLES: (0,1), (0,2) CurrentMirror | one 3-member set |
| `current_mirror_4` | 956-976 | 4: diode + 3 outputs | ROLES: (0,1), (0,2), (0,3) | one 4-member set |
| `current_mirror_1_to_2` | 2029-2045 | 3: diode + 2 outputs, no diode on outputs, same L | ROLES: (0,1), (0,2) | 3-member set |
| `mirror_with_dual_output` | 1524-1543 | 4: diode + 2 outputs + cascode on one | none | nothing (the inner `current_mirror` match still gives a set via the overlap graph) |
| `mirror_with_cascode_output` | 1402-1415 | 3: diode + output + cascode | none | nothing |
| `wilson_mirror` | 430-444 | 3: diode ref, input, output with feedback | none | nothing |
| `improved_wilson_mirror` | 450-469 | 3 | none | nothing |
| `wilson_mirror_4` | 740-756 | 4 | none | nothing |
| `improved_wilson_mirror_4` | 761-778 | 4 | none | nothing |
| `cascode_mirror` | 669-685 | 4: bottom diode mirror + top diode cascode pair | ROLES: (0,1) CurrentMirror, (2,3) CascodePair, prox (0,2), (1,3) | mirror set, cascode set, two Proximity pulls; `audit` cascode checks (`ann/audit.rs:98-111`) |
| `wide_swing_cascode_mirror` | 693-710 | 4 | ROLES: same as cascode_mirror | same |
| `low_voltage_cascode_mirror` | 717-733 | 4: top pair not diodes | ROLES: same | same |
| `wide_swing_cascode_mirror_5` | 1072-1091 | 5: + bias diode | none | nothing |
| `regulated_cascode_mirror` | 1548-1564 | 4 | none | nothing |
| `sooch_mirror` | 1440-1455 | 3 | none | nothing |
| `cascoded_reference` | 2067-2080 | 3: diode ref + output + diode cascode on ref | none; `classify.rs:262-264` makes slot 1's drain `Reference` | net class only |
| `cascode_with_mirror_bias` | 1384-1397 | 3 | none | nothing |
| `self_biased_cascode` | 1420-1434 | 3 | none | nothing |
| `beta_multiplier_core` | 1315-1332 | 2: diode + wider same-L, distinct S and D | none (Group) | nothing |
| `bias_chain_3` | 1681-1703 | 3 diodes in series | none | nothing |
| `bandgap_mirror_pair` | 1708-1730 | 4: mirror + complementary diode pair | none | nothing |
| `cascode_current_source_pair` | 2207-2234 | 4: no diodes, bottom shares G and S, top shares G | none | nothing |
| `bjt_mirror_npn` / `_pnp` | 2324-2332, 2335-2336 | 2: diode ref (C = B), shared B and E | ROLES: (0,1) CurrentMirror | mirror set |
| `bjt_ratioed_pair_npn` / `_pnp` | 2304-2312 | 2: shared base, distinct emitters, exact unit | ROLES: (0,1) CurrentMirror; `kind_of` → Voltage (ΔV_BE), role `BandgapCore` → Moderate (`lib.rs:342-344`) | set; emitter resistors → `bandgap_r` set (`passive.rs:96-107`); shared base → `Reference` net (`classify.rs:248-261`) |
| shared-bias groups (no template) | `ann/sets.rs:55-76` | FET/BJT of one kind, model, L sharing gate (base) and source (emitter) net, with a diode member or a non-Signal gate, on ≥ 2 drains | `MatchBlock` star from the diode | `Origin::SharedBias` set, role `BiasMirror` → Minimal; CurrentSource role (`evidence.rs:144`) |

### 3c. Cascodes and stacks

| Template | Lines | Slots | Roles | Emits |
|---|---|---|---|---|
| `cascode` | 226-237 | 2: same type, D_0 = S_1, distinct gates | prox (0,1) (fallback Stack) | Proximity |
| `cascode_matched` | 241-252 | 2: + same L | Stack | Proximity |
| `cascode_diode_top` | 1943-1958 | 2: bottom no diode, top diode same L | Stack | Proximity |
| `series_stack` | 374-385 | 2: exact, shared gate, D_0 = S_1 | Stack | Proximity |
| `series_stack_4` | 390-402 | 4 chained, one gate, exact | none | nothing |
| `triple_cascode` | 2009-2023 | 3 | none | nothing |
| `regulated_cascode` | 550-563 | 3 | none | nothing |
| `cascode_with_degeneration` | 568-582 | 3 | none | nothing |
| `cascode_pair` | 870-887 | 4: two stacks, bottom shares G+S, top shares G | none (not in ROLES) | nothing |
| `cascode_pair_symmetric` | 892-908 | 4 | none | nothing |
| `source_follower` | 281-297 | 2: current source + follower (signal gate) | Stack | Proximity |
| `source_follower_with_mirror` | 589-607 | 3 | none | nothing |
| `super_source_follower` | 2050-2063 | 3 | none | nothing |
| `flipped_voltage_follower` | 612-625 | 3 | none | nothing |
| `complementary_source_follower` | 1336-1353 | 2: N + P followers sharing source output | none (Group) | nothing |
| `common_gate_pair` | 1280-1292 | 2: shared G and D, distinct S | none (Group) | nothing |

### 3d. Loads

| Template | Lines | Slots | Roles | Emits |
|---|---|---|---|---|
| `diode_load_pair` | 258-274 | 2: both diodes, shared S, distinct D | (0,1) Load (fallback) | Load set |
| `active_load_3` | 631-658 | 3: shared G and S, no diodes, exact | none | nothing |
| `level_shifter` | 915-929 | 4 | none | nothing |

### 3e. Latches, logic and switches

| Template | Lines | Slots | Roles | Emits |
|---|---|---|---|---|
| `latch_half` | 354-366 | 2: diode drives the other's gate, distinct S and D | none (Group) | nothing |
| `cross_coupled_inverters` | 845-862 | 4: two inverters cross-coupled | ROLES: (0,1) DiffPair, (2,3) DiffPair, prox (0,2), (1,3) | two DiffPair sets, two Proximity |
| `nand_cross_coupled` | 1835-1851 | 4 | none | nothing |
| `cmos_inverter` | 303-314 | 2: complementary, shared G and D | ROLES: prox (0,1) | Proximity; `LOGIC` template → DigitalStatic/DigitalSwitching nets (`classify.rs:153, 211-235`) |
| `inverter_buffer` | 2186-2202 | 4 | none | nothing |
| `inverter_with_tail` | 1459-1472 | 3 | none | nothing |
| `active_loaded_inverter` | 2243-2256 | 3 | none | nothing |
| `nand_gate`, `nor_gate` | 2089-2106, 2111-2128 | 4 each | none | net classes only (`LOGIC`) |
| `schmitt_trigger` | 1638-1653 | 4 | none | nothing |
| `transmission_gate` | 319-331 | 2: complementary, shared D and S, distinct G | none (Group) | DigitalStatic on G, D, S (`classify.rs:229-235`) |
| `transmission_gate_pair` | 1568-1585 | 4 | none | nothing |
| `switch_pair` | 1297-1309 | 2: exact, shared D and S, distinct G | none (Group) | nothing |
| `anti_parallel_switch` | 1962-1974 | 2: exact, D_0 = S_1, S_0 = D_1 | none (Group) | nothing |
| `bootstrapped_switch` | 1855-1868 | 3 | none | nothing |
| `push_pull_pair` | 335-347 | 2: complementary, shared G, distinct D and S | none (Group) | nothing |
| `push_pull_with_bias` | 1476-1489 | 3 | none | nothing |
| `charge_pump_cell` | 936-951 | 4 | none; slot 0 drain → `Noisy` (`classify.rs:237-239`) | net class only |
| `dac_current_cell` | 1590-1611 | 4: diff switch + source + cascode | none | nothing |
| `esd_diode_clamp` | 1662-1677 | 2 series diodes | none (Group) | nothing |

### 3f. OTAs and multi-stage composites

| Template | Lines | Slots | Roles | Emits |
|---|---|---|---|---|
| `five_transistor_ota` | 985-1009 | 5: pair, complementary exact load pair (shared G and S), tail | ROLES: (0,1) DiffPair, (2,3) Load, self 4 | DiffPair + Load sets, tail on axis and pulled to the pair |
| `ota_self_biased_load` | 1738-1768 | 5: same-type diode/mirror loads, complementary tail | none | nothing |
| `diff_pair_cascode_load` | 1015-1038 | 5: pair, complementary cascode loads, tail | ROLES: (0,1) DiffPair, (2,3) CascodePair, self 4, prox (0,2), (1,3) | two sets, tail, two Proximity |
| `cascoded_diff_pair_with_tail` | 1877-1901 | 5: pair, same-type cascodes, tail | ROLES: same shape as above | same |
| `diff_pair_cross_coupled_load` | 1042-1067 | 5 | none | nothing |
| `diff_output_stage` | 1906-1935 | 5 | none | nothing |
| `telescopic_ota_core` | 1100-1127 | 6: pair, input cascodes, load cascodes | ROLES: (0,1) DiffPair, (2,3) CascodePair, (4,5) Load, prox (0,2), (1,3), (2,4), (3,5) | three sets, four Proximity |
| `telescopic_ota_full` | 1233-1269 | 8: core + tail + load bias | ROLES: as core, self 6; slot 7 no role | three sets, tail, four Proximity |
| `folded_cascode_core` | 1133-1160 | 6: pair, folding cascodes, fold current sources | ROLES: (0,1) DiffPair, (2,3) CascodePair, (4,5) Load, prox (4,2), (5,3) | three sets, two Proximity |
| `gilbert_cell` | 1166-1195 | 6: RF pair + LO quad | ROLES: (0,1), (2,5), (3,4) DiffPair | three DiffPair sets (quad as two 2-member sets) |
| `vco_core_with_tails` | 1201-1224 | 6: N and P cross-coupled pairs + two tails | ROLES: (0,1), (2,3) DiffPair, selfs 4, 5 | two sets, two selfs |
| `feedback_pair` | 1616-1633 | 4 | none | nothing |

### 3g. Passive and bipolar sets (procedural, `ann/passive.rs`)

| Rule (`Origin::PassiveSet { rule }`) | Lines | Members | Role → default class | Emits |
|---|---|---|---|---|
| `divider` | 50-81 | resistors of one model and width joined through internal nodes reaching only resistors and FET gates | `FeedbackRatio` → Moderate | `MatchedSet` (Ratio kind, ledger unknown without `k_A`), `Unitization` with series units (`sets.rs:169-181`) |
| `symmetric_r` | 82-89 | each compound couple of resistors of one material | `Other` → Minimal | set + Proximity |
| `bandgap_r` | 96-107 | resistors on a BJT ratioed pair's emitters | `BandgapCore` → Moderate | set |
| `dac_bank` / `cap_ratio` | 125-179 | capacitors sharing a top plate with one W, L, model; terminating unit first; binary counts → `dac_bank` (`DacBank` → Exceptional), else `cap_ratio` (`FeedbackRatio`) | | set; plate nets `Reference`/`Sensitive` and `RcClass` C/R (`classify.rs:265-274`, `extract.rs:309-324`); `PlateRatio` built by dr from the `Unitization` |
| `split_dac` | 141-171 | two banks joined by exactly one bridge capacitor; `bridge_cap_value` diagnostic when its area is off `(C_T^LSB/C_T^MSB)·C_u` by > 1 % | `DacBank` | set holding the bridge (unit `(1, 1)`), bridge left out of the `Unitization` (`constraints.rs:76-77`) |
| `degeneration` | 186-213 | the two resistors through which a pair's sources (emitters) reach one node; `degeneration_ratio` when lengths are not the inverse of the pair's units | `InputPair` → Moderate | set |
| `diode_set` | 255-260 | diodes of one model and area on a shared anode or cathode | `Other` → Minimal | set + Proximity |

### 3h. Hierarchy (`ann/hier.rs`)

Exactly two identical instances of one subckt under one parent (`same_template`, `:64-66`) whose ports pair up (`ports_pair`, `:93-98`) seed device couples (`ann/lib.rs:231-248`) and `MatchBlock` edges (`graph.rs:93-97`); ≥ 3 identical instances sharing a bias port net are an array (`arrays`, `:73-89`): one `ProxBlock` star and one horizontal reversible `Order` (`lib.rs:271-290, 457`).

## 4. Requirement families (`docs/ref/Notes/constraints.html`)

84 families. Role letters: L legality, E electrical acceptance, C exact construction invariant, B derived budget or proxy, P preference.

**INT — Electrical intent and physical identity**
- INT-01 Terminal connectivity (L/E): every terminal on its net, nets distinct; LVS.
- INT-02 Device electrical identity (E/C): extracted class, model, dimensions, multiplicity match the design.
- INT-03 Required device ratios (E/C): ratios by valid unit counts, not area alone.
- INT-04 Bias and body domains (L/E): every body, well, tap on its permitted bias domain; no forward bias or domain short.
- INT-05 Allowed transformations (C/E): rotation, reflection, terminal exchange, diffusion sharing only where proven valid.
- INT-06 Fixed objects and interfaces (C/E): pads, macros, pins, boundaries stay fixed.
- INT-07 Realization-map preservation (C): splits and merges keep units, weights, terminals, constraints.
- INT-08 Realizable terminal access (L/C/B): legal connected access per terminal, not rectangle-centre reachability.

**GEO — Manufacturing geometry and placement legality**
- GEO-01 Units and legal grids (L/C).
- GEO-02 Width and length legality (L).
- GEO-03 Contextual separation (L): spacing dependent on layer, width, run, voltage, end geometry.
- GEO-04 Enclosure and overlap (L).
- GEO-05 Shape area and topology (L): min area, notches, after merging.
- GEO-06 Occupancy and reserved regions (L/C): keepouts, exclusive wells, rings, reservations.
- GEO-07 Disjunctive spacing (L/C): sharing interval or isolation interval, no gap between.
- GEO-08 Routing-resource feasibility (B/C): capacities and pin corridors suffice.

**MAT — Matching, orientation and device environment**
- MAT-01 Random mismatch allowance (E/B): propagated local variation meets the offset/ratio/noise spec; area is a lever.
- MAT-02 Equivalent matched units (C/B): compatible type, geometry, segmentation, environment.
- MAT-03 Weighted first moments (C/B): electrically weighted centroids coincide within tolerance.
- MAT-04 Higher-order spatial moments (B/P): second and higher moments within allocation.
- MAT-05 Physical orientation equivalence (C/B): compatible orientation and S/D environment; reflection is not identity.
- MAT-06 Selected symmetry relations (C/B): shared-axis equations and pin mapping hold.
- MAT-07 Spatial proximity budget (B/P): separation within a gradient/correlation allowance.
- MAT-08 Edge and dummy equivalence (C): edge units see equivalent surroundings via dummies.
- MAT-09 Well-proximity environment (B/E): WPE distances keep shifts within allocation.
- MAT-10 Isolation/stress environment (B/E): LOD, STI, OSE constraints satisfied.
- MAT-11 Matched electrical interconnect (E/B/C): route R, C, coupling, access differences within the differential allocation.
- MAT-12 Matched temperature response (E/B): ΔT × TC within the offset/ratio error.

**NET — Signal routing and parasitic performance**
- NET-01 Series resistance (E/B): path R including contacts and vias within allowance.
- NET-02 Node loading capacitance (E/B): per sensitive node.
- NET-03 Aggressor coupling (E/B): transfer below noise limits under actual waveforms.
- NET-04 Shared performance budget (E/B): coupled inequalities, no double spending.
- NET-05 Width/layer choice domain (L/C/B): legal width, layer, via combination per segment.
- NET-06 Shield connectivity and reference (E/C): shield tied to the intended reference with adequate impedance.
- NET-07 Sensitive-node route environment (B/P): avoid aggressive regions and crossings.
- NET-08 Shared-impedance coupling (E/B): no correlated error through common return segments.
- NET-09 Distributed-network adequacy (E/C): the lumped model resolves the spectrum.
- NET-10 Complete route realization (L/C): every route and protective conductor in final geometry.

**PWR — Power, current and lifetime**
- PWR-01 Branch-current conservation (C/E): trunks carry the signed sum; meshes need a network solve.
- PWR-02 DC supply/reference drop (E/B).
- PWR-03 Dynamic supply/ground error (E/B): droop, bounce, settling.
- PWR-04 Conductor EM lifetime (E/L/B): the foundry's metric under temperature and current history.
- PWR-05 Contact/via current reliability (E/L/B): arrays with derating and sharing assumptions.
- PWR-06 Local current crowding (E/B): bends, bottlenecks, unequal parallel paths.
- PWR-07 Return-path integrity (E/C/B).
- PWR-08 Device voltage stress (E/L): terminal voltages and junction biases inside ratings.

**ENV — Thermal, substrate and isolation environment**
- ENV-01 Dissipation model completeness (C/E): every heat source has a justified power estimate.
- ENV-02 Junction-temperature bound (E/B).
- ENV-03 Thermal settling and drift (E/B).
- ENV-04 Substrate transfer (E/B): injected disturbance below the permitted error at victims; spacing is a proxy.
- ENV-05 Guard-ring realization (L/C/B): type, continuity, contacts, connection implement the function.
- ENV-06 Guard-return impedance (E/B): collected disturbance returns through a low, appropriate impedance.
- ENV-07 Latch-up prevention (L/E).
- ENV-08 Isolation-domain compatibility (L/C/E): deep wells, trenches, taps valid per device and domain.

**DEV — Passive, bipolar and RF device behavior**
- DEV-01 Resistor value and ratio (E/B/C): sheet, end, contact, bend contributions.
- DEV-02 Resistor parasitic/environment error (E/B): voltage coefficient, thermal, capacitive loading.
- DEV-03 Capacitor value and terminal roles (E/C): value, polarity, top/bottom plate assignment.
- DEV-04 Capacitor voltage/frequency domain (E/B).
- DEV-05 Bipolar/diode construction (L/E/C).
- DEV-06 Bipolar matching and current density (E/B).
- DEV-07 Inductor/RF passive performance (E/B).
- DEV-08 RF interaction and distributed routing (E/B).

**ARR — DAC and precision-array realization**
- ARR-01 Exact array weights (C/E): unit counts and terminals implement every code weight.
- ARR-02 Array spatial error (B/C): centroids, dispersion, environment per code group.
- ARR-03 Code-dependent parasitics (E/B): plate and inter-element coupling.
- ARR-04 INL/DNL and monotonicity (E).
- ARR-05 Settling and switching transients (E/B).
- ARR-06 Array routing realizability (C/B/P).

**MFG — Manufacturability and protection beyond ordinary geometry**
- MFG-01 Density and fill (L/E/B).
- MFG-02 Defect critical area (B/P/E).
- MFG-03 Process topology robustness (L/B/P): litho, etch/CMP, redundant contacts.
- MFG-04 Antenna exposure (L/E): stage-aware ratios.
- MFG-05 ESD discharge path (L/E/C).
- MFG-06 Test accessibility and discrimination (E/B/P).

**EVD — Circuit acceptance, robustness and evidence completeness**
- EVD-01 Electrical specification vector (E): every metric meets its bound on the extracted circuit.
- EVD-02 Operating-domain coverage (E/C).
- EVD-03 Valid variation model (C/E).
- EVD-04 Joint parametric yield (E/B).
- EVD-05 Statistical confidence (C/E).
- EVD-06 Proxy/model validity (C/B): sensitivities and surrogates within their validated domain.
- EVD-07 Required checker coverage (C/E): pass/fail/unknown/not-applicable reported separately.
- EVD-08 Fresh evidence identity (C): results match the delivered geometry and models.
- EVD-09 Search-domain and proof scope (C).
- EVD-10 Independent outcome validation (C/E).

## 5. Status table (`docs/LAYOUT-FUNDAMENTALS.md`)

Legend (`:5-10`): BC by construction, S scored (a Rule/Θ term search acts on), M measured only, — missing, K an already-known gap. Row numbers are the document's.

| # | Technique | Status | Where (as the document cites it) |
|---|---|---|---|
| 1 | Identical unit sections per matched set | BC | `Unitization` unit_w/unit_l |
| 2 | Size for the matching class (area vs length) | M | η from A_VT and spec; no resize |
| 3 | V_ov ≥ 100 mV for current-matched devices | — (now `vgst_low` diagnostic, `ann/audit.rs:49-63`) | op point |
| 4 | Same channel orientation and signed current Φ | BC/S | units carry φ; `OrientationSet` now checks cross-cell |
| 5 | LOD/STI: equal SA/SB per matched set | K8 + S | `Unit.sa/sb/lod`; `MatchedSet.mu_lod`; perf deck `sa/sb` |
| 6 | OD-to-OD spacing (OSE) equal | — (later `Environment` OSE, §2 item 5) | |
| 7 | Poly spacing / gate density (PSE) | K2 | dummies at pitch, no neighbour check |
| 8 | WPE distance ≥ class and equal across set | BC (partial) (later `Environment` WPE) | PMOS halo; NMOS vs foreign nwell |
| 9 | End dummies, moat past the last gate | K2/K3 | dummies tied to bulk |
| 10 | S/D fully contacted along W | — (new) (later fixed: full contact column, §2 item 3) | |
| 11 | Metal gate straps, two-ended gate contact | — | poly strap, one cut |
| 12 | Equal gate poly extension beyond rule | BC (partial) | rule minimum only |
| 13 | No contacts/metal over matched gates | K5 (later `MetalOverGate`, gr `KEEPOUT_COST`) | |
| 14 | Block fill over matched arrays | BC | `library/fill.rs` |
| 15 | Diffusion sharing, drains inner | BC | |
| 16 | Series-stack diffusion sharing | — | Stack → Proximity only |
| 17 | Body tap per device, symmetric | BC (partial) | one tap strip per cell |
| 18 | Thin-oxide for matching, no pocket at long L | — | sizing choice |
| 19 | Matched resistors: same material, segment W/L, orientation | BC | |
| 20 | Interdigitated / CC resistor arrays | — (new) (later interleaved variants, §2 item 9) | |
| 21 | Dummy segments on both array ends | — | |
| 22 | Minimum segment length by class | BC (partial) | `res_min_segment` |
| 23 | Thermoelectric cancellation (even segments, alternating) | BC | |
| 24 | Interconnect R scales with the ratio; Kelvin taps | — | |
| 25 | No unconnected leads over matched resistors; field plate | — | |
| 26 | Matched resistor power; distance from poly caps | M | thermal field, not self-heating |
| 27 | Capacitors: unit caps, CC, dummy ring, shield | BC for binary DACs; — general (later generic `cap_array`, §2 item 8) | |
| 28 | Non-integer ratios: equal A/P of the odd unit | — (`dac::nonunit_dims` exists in the kernel) | |
| 29 | Lead capacitance per unit matched | BC for `cap_array`; — elsewhere (now `PlateRatio` B) | |
| 30 | Bottom plate to low-Z node; well shield under cap | — | |
| 31 | BJT identical emitters, CC with unit centred, dummy ring | BC (partial) (later centre-out fill, §2 item 10) | |
| 32 | BJT equal V_CE, contact geometry, injection | — (now `vce_unequal`, `bjt_ratio` diagnostics) | |
| 33 | Coincidence of weighted centroids | S | `MatchedSet.coincidence` |
| 34 | Symmetry about both axes, 2nd moments | — (now `second_order_nm` cost/report; `cancelled_order` report) | |
| 35 | Dispersion: many small CC sub-arrays | — | |
| 36 | Compactness: aspect ratio by class | — | |
| 37 | 2-D common centroid for MOS | — (later 2-row variants, §2 item 6) | |
| 38 | Equal environment signature per member | K3 + gaps | |
| 39 | Proximity: gradient term within budget | S/M | `MatchedSet.sigma_grad` on a proxy `S_VT` |
| 40 | Matched route R/C per terminal | S | `Differential` |
| 41 | Common-node resistance matched: star or Kelvin | — (now `CommonNode` measured budget, §2 item 4) | |
| 42 | Mirror symmetry about one shared axis, tail on axis | S (hard) | `SymmetryGroup` |
| 43 | Matched devices isothermal, 1 µm/mW | S | `MatchedSet.mu_thermal`, `HeatSeparation` |
| 44 | Low-stress die region | — | block-level engine |
| 45 | Substrate isolation, saturates past 4× epi | S | `Isolation` |
| 46 | DTI either-or banding | S (hard) | `DtiBand` |
| 47 | Shared well for same-body PMOS | BC (partial) (later same-bulk cells share a well) | |
| 48 | Route-aware placement reservations | M | gr overflow only |
| 49 | Symmetric routing of differential nets | S | `Differential` |
| 50 | Crosstalk exclusion and total coupling budget | S | `CrosstalkExclusion`, `CouplingBudget` |
| 51 | Shielding by a quiet ground | S/BC | `Shield`, dr draws shields; same layer, tie not measured |
| 52 | Per-net C budgets and shared performance budgets | S | `ParasiticBudget`, `PerformanceBudget` |
| 53 | Series R in the performance loop | S (routed part) | `Rpex_*`; cell-internal and gate R not included |
| 54 | Separate analog/digital supplies, no shared returns | — | |
| 55 | Kelvin sense connections | — (now `KelvinReq` → `CommonNode` star) | |
| 56 | Noisy nets not over matched devices | — (now `MetalOverGate`, gr keep-out) | |
| 57 | Width sizing by criticality | BC (partial) | supply fatten, EM floor |
| 58 | Guard rings by role | BC (then every MOS, bulk-tied; now `rings.rs` by role) | |
| 59 | Ring return impedance | M | `max_ring_resistance_mohm` sizes contacts only |
| 60 | Latch-up tap distance | BC (local); — global | |
| 61 | Well/substrate shields under sensitive routes and caps | — | |
| 62 | DC EM per segment and via array, derated | S | `Electromigration` |
| 63 | RMS/peak EM, Blech only where valid | — | DC only |
| 64 | EM inside cells (li, licon) | — | |
| 65 | Antenna per stage, diode or jumper repair | S | `Antenna`; diode insertion; no layer-jump repair |
| 66 | Redundant vias on critical nets | BC (partial) | arrays on fattened trunks |
| 67 | HCI/NBTI equal stress | — | |
| 68 | Metal density/fill without disturbing sensitive nodes | BC | `library/fill.rs` |

The document's "top 10 missing" (`:154-183`) records a second pass: LOD, routed R, MOS contacting, common-node R, neighbour environment, 2-D arrays, metal over matched devices, general matched capacitors, resistor interleaving, BJT centre-out fill are marked fixed or partly fixed; still open there: metal gate straps, cell-internal EM, star routing, PSE, resistor end dummies, RMS/peak EM, quiet ring return, series-stack sharing, measured `S_VT`, global latch-up, layer-jump antenna repair (`:234-240`).

## 6. Known defects of the annotator (`docs/plans/audit-01-annotator.md:498-536`)

The audit predates the current tree (it cites `pattern.rs:94` as FET-only and a 166-line `lib.rs`). The second column is the audit's finding; the third says what the current code shows.

| ID | Finding (audit) | Visible in the current tree |
|---|---|---|
| AA-01 | Greedy disjoint selection: a device belongs to one structure only; the survey's overlapping requirement graph cannot be expressed | `recognize_all` + `graph::requirements` over all matches + `hsmpg` tree (`ann/graph.rs`); disjoint `blocks` kept for cells |
| AA-02 | One symmetry axis per block; no circuit-level symmetry | `ann/symmetry.rs` propagation to compounds, one axis each |
| AA-03 | `cmos_inverter` beats `cross_coupled` inside latches; no role declarations | `ROLES` table (`catalog.rs:2472-2504`) with `cross_coupled_inverters` and `vco_core_with_tails` |
| AA-04 | Only FETs recognised; R, C, BJT, diode are glue; cellgen's hidden recognizers | `SlotKind::Kind`, bipolar patterns, `ann/passive.rs` sets |
| AA-05 | Loose primitives create false pairs (rail as shared source, any shared-gate pair a mirror) | `PinRel::SameSignal` on diff-pair sources; `diff_pair_split_source` and `mirror_pair_split_source` no longer in the catalog |
| AA-06 | Generic 3–4 device shapes outrank primitives and steal devices (`miller_comp_fets`, `push_pull_with_bias`, `level_shifter`, `charge_pump_cell`, `schmitt_trigger`, `nand`/`nor`, `bandgap_mirror_pair`, `feedback_pair`) | `miller_comp_fets` gone; the others remain with no `ROLES` entry, so a device they claim gets no constraint from them (see §7) |
| AA-07 | Results depend on netlist order (id tie-breaks) | `canonical_labels` (`pattern.rs:418-449`), names only on exact label ties |
| AA-08 | Composite slot roles discarded; two-level hierarchy; no SameTemplate or arrays | `Roles.selfs/prox`, `Block.sub_blocks`, `ann/hier.rs` |
| AA-09 | No ratio inference; unitization keyed on exact (kind, W, L) | `sets::unitize` gcd units (`sets.rs:136-209`); `Unitization` still drawn in schematic fingers (`constraints.rs:3-9`) |
| AA-10 | No precision class and no voltage/current distinction; one η everywhere | `MatchClass`, `MatchKind`, `class::class_of`, `phi_arm`, class-scaled budgets |
| AA-11 | Net roles from names only; two rail lists; nothing sets `AnnotationConfig` | `rail_of` single source, bulk inference, testbench rails, `EvidenceLevel`, sidecar |
| AA-12 | No op point, sizing intent or sensitivity | `Evidence { op, sens, switching_nets, dc_sources }`, `allocate`, `budget::rows` |
| AA-13 | Proximity ≤ 5 µm and Isolation ≥ 10 µm emitted on the same pair | `related` filter and GAP-09 conflict report (`lib.rs:523-544, 588-597`) |
| AA-14 | `Differential`/`CrosstalkExclusion` re-derived by `analog`'s own `is_diff_pair`; `Differential` hard | both from compounds and sets in `extract.rs`; `Differential` in the budget arm |
| AA-15 | Wrong topologies: `wilson_mirror` links swapped, `gilbert_cell` ties 2G = 3G, `triode_load_pair` G = S; eight dead patterns | `wilson_mirror` now `1D = 2G, 0D = 2S` (`:438-443`) and gilbert `2G = 5G, 3G = 4G, 2G ≠ 3G, 2D = 4D, 3D = 5D` (`:1189-1194`), both the audit's fix; `triode_load_pair` gone; shadowing is not tested |
| AA-16 | Terminal roles positional; BJT collectors count as gates | `ann/terms.rs` by name and kind |
| AA-17 | Stack devices count as sensitive | `is_sensitive` excludes Stack and CascodePair (`block.rs:69-71`) |
| AA-18 | Rings and dummies keyed on "was recognised", not role; literals | `rings::plan` by role; dummies by class; `max_ring_resistance_mohm` still the literal 100 000 (`rings.rs:72`) |
| AA-19 | Abutment permission whole-group or first member | unchanged shape (`lib.rs:175-185`): mixed-polarity groups cut to one member |
| AA-20 | Model and bulk ignored by matching and unitization | `size::exact_as` includes model and bulk (`size.rs:59-71`); unitization keyed on model and bulk (`constraints.rs:114-117`) |
| AA-21 | Missing W/L reads as 0 | `Drawn` fields `Option` (`size.rs:14-22`) |
| AA-22 | Matcher O(n³), permutations, O(M²) dedupe | `on_pin` candidate lists, `slot_order`, hashed `seen` (`pattern.rs:136-149, 230-266, 327-342`) |
| AA-23 | Every unpaired member of a DP stage forced onto the axis | selfs from declared roles and symmetry analysis |
| AA-24 | W-ratioed mirror pairs get couple Symmetry | `equal` couple check (`emit.rs:155-168`) |
| AA-25 | Budget policy: drain-only nets against the smallest gate; Sensitive 1×; Clock like Signal | `net_load_af` (`classify.rs:102-131`), aggressor classes get no coupling budget; `missing` entry for drain-only nets (`lib.rs:400-415`) |
| AA-26 | 5 of 103 patterns tested | `every_pattern_matches_its_own_minimal_netlist` (`catalog.rs:2694-2718`) |
| AA-27 | `Block` drops the template; no coverage report | `Block.template`, `Problem.coverage` |
| AA-28 | Policy literals untied to deck, spec or class | `ann/policy.rs` table; still policy values |
| AA-29 | `missing` lists MatchingPair with no matched pair | `Needs` gating (`lib.rs:90-120`) |
| AA-30 | Stale doc references | not checked here |
| AA-31 | Shield reference = first Ground net; shields on every Sensitive net | `quiet_ground` tiers, shields only when an aggressor exists |
| AA-32 | Library re-implements gate area, A_VT from member a only | `gate_um2` shared; `coeffs` by slot 0 polarity (`emit.rs:84-101`) still reads one member |
| AA-33 | Isolation all-pairs, every clock-touching device an aggressor, no strength | still all-pairs over `Switching`/`Capacitive` tags (`emit.rs:290-295`); no strength |
| AA-34 | Catalog invariants unchecked | `catalog_is_well_formed` (`catalog.rs:2548-2576`) |
| AA-35 | `u16` ids wrap silently | assert at `lib.rs:139-141` |

## 7. Gaps

### 7a. Kernel rules the annotator does not emit

Built by `frontend/library` or `backend/dr` from the `Problem`, so a successor that re-implements extraction must also plan their inputs: `HeatSeparation` (`lib/lib.rs:910-913`), `LiveEnvironment` (`:896, :2770`), `SymmetryIsland` (`:936-937, :1752`), `PlacePerf` (`:943`), `Utilization` (`:932`), `PerformanceBudget` rows (`:884`, from `ann/budget::rows`), `IrDrop` (`:999-1004`, from `ann/ir::budgets`), `EsdWidth` (`:988-993`), `Electromigration` (`:2646-2648`, `dr/lib.rs:1646`), `CommonNodes` (`lib/lib.rs:2243-2273`, `dr/lib.rs:1151-1158`), `PlateRatios` (`dr/lib.rs:380-384`), `MetalOverGate` (`dr/lib.rs:1815`, evaluated outside `Requirements`).

### 7b. Kernel items nothing emits or that read unknown by construction

- Resistor and capacitor `MatchedSet`s get `Coeffs::default()` (`ann/emit.rs:99`): no `k_A`, `S_D` or `TC`, so their ledger has `sigma_rand = 0`, is not `budgeted`, and only pulls by `cost`. `ProcessNumbers` has no slot for them (`emit.rs:82-83`).
- `Coeffs.svt_xy` (anisotropic gradient) has no deck key (`mismatch.rs:145`); `svt_uv_per_um` itself is a flagged proxy on sky130 (`LAYOUT-FUNDAMENTALS.md:30, row 39`).
- `MatchedSet::sizing_notes` passes `None` for the class limit (`matched_set.rs:346-347`), so the `"area"` sizing note is never produced.
- `Ledger.second_order_nm` is cost and report only; no deck has a second-order coefficient (`mismatch.rs:218-219`).
- `OrientCheck::PhiZero` is named in the doc comment (`orientation.rs:15-16`) and as "MAT-12 step 3" (`emit.rs:196`) but no such variant exists; a self-cancelling resistor string has no Φ rule.
- `Rule::extract` (`rule.rs:175-181`) and `ParasiticBudget::extract` (`parasitic.rs:81-94`) are placeholders the annotator never calls; `extract.rs:4-6` still labels `Antenna` "self-extracted" though it is built at `extract.rs:86-98`.
- `PerformanceBudget.r_nets`, `coupling` and `diff_pairs`: EXT-25 fills R and coupling, leaves `diff_pairs` empty, and the batch reads `unknown` whenever any is present (`performance.rs:23-26, 77-79`). `PlacePerf` prices ground C only (`perf.rs:17`).
- `Differential` has no `project`; dr repairs by mirrored rip-up (`differential.rs:30`).
- `Shield` is same-layer, one reference, tie impedance unmeasured (`shield.rs:17-19`); `CouplingBudget` crossing term omits fringe (`coupling.rs:66-68`).
- `Electromigration` is DC only, Blech dormant (no shipped deck has `blech_limit`), via grouping transitive (`em.rs:91-100, 196-200`).
- `IrDrop` falls back to the worst-path bound when any terminal current is unknown (`ir.rs:19-20`).
- `nth_order_rows`, `segmentation`, `partial_segments`, `dac::*` are generators for `kernel/cells`, not constraints; nothing in the annotator or kernel scores an Nth-order figure beyond `cancelled_order` in the report (`matched_set.rs:336`).

### 7c. Catalog composites that recognise but constrain nothing

Any ≥ 3-slot pattern absent from `ROLES` (`catalog.rs:2469-2471`): `diff_pair_diode_load`, `diff_pair_with_reference`, `cmfb_sense_pair`, `diff_pair_with_cascodes`, `diff_pair_with_split_cascodes`, `folded_load_pair`, `wide_swing_cascode_mirror_5`, `regulated_cascode_mirror`, `sooch_mirror`, `cascoded_reference`, `cascode_with_mirror_bias`, `self_biased_cascode`, `bias_chain_3`, `bandgap_mirror_pair`, `cascode_current_source_pair`, `wilson_mirror`, `improved_wilson_mirror`, `wilson_mirror_4`, `improved_wilson_mirror_4`, `mirror_with_dual_output`, `mirror_with_cascode_output`, `series_stack_4`, `triple_cascode`, `regulated_cascode`, `cascode_with_degeneration`, `cascode_pair`, `cascode_pair_symmetric`, `source_follower_with_mirror`, `super_source_follower`, `flipped_voltage_follower`, `active_load_3`, `level_shifter`, `nand_cross_coupled`, `inverter_buffer`, `inverter_with_tail`, `active_loaded_inverter`, `nand_gate`, `nor_gate`, `schmitt_trigger`, `transmission_gate_pair`, `bootstrapped_switch`, `push_pull_with_bias`, `charge_pump_cell`, `dac_current_cell`, `ota_self_biased_load`, `diff_pair_cross_coupled_load`, `diff_output_stage`, `feedback_pair`. Since priority beats everything in `select_disjoint` (`pattern.rs:456-462`), such a composite wins the block over the primitives inside it. Consequences as read: no symmetry seed (`lib.rs:220-225` seeds only DiffPair/Load/CascodePair leaves), no tail self, no Proximity; the inner primitive's overlapping match still gives a `MatchBlock` edge and a set (`graph.rs:69-77`, `sets.rs:246`), but `kind_of` sees no DiffPair leaf and returns `Current` (`lib.rs:299-310`, `class.rs:37-46`), the role falls to `Other` → Minimal (`lib.rs:330-349`), so a diff pair inside `diff_pair_diode_load`, `cmfb_sense_pair` or `dac_current_cell` gets a Minimal Current set: Proximity only, no Sensitive gates, no G×D crosstalk, no symmetry. Two-slot patterns with no role (`transmission_gate`, `push_pull_pair`, `complementary_source_follower`, `anti_parallel_switch`, `beta_multiplier_core`, `esd_diode_clamp`, `latch_half`, `common_mode_sense_pair`, `common_gate_pair`, `degeneration_pair`, `switch_pair`) are the same except that they rarely outrank a constraint-bearing primitive.

### 7d. Families with no code anywhere in the two crates

Checked against §4; "verify/cells/dr" means another crate owns it and was not read here.

- INT-04 bias/body domains: only the `cascode_bulk` diagnostic (`audit.rs:106-110`); no domain graph. INT-06 fixed objects: a `Block.injected` flag only. INT-07, INT-08: not in these crates.
- GEO-01..06, GEO-08: DRC and gr, not in these crates. GEO-07: `DtiBand` only.
- MAT-02: `Unitization` plus `mixed_kind_set`; variant equality only through `Symmetry.same_shape`. MAT-04: cost/report only. MAT-08: `dummy_required` flag, drawn by cells; no equivalence check of the drawn edge environment. MAT-09/MAT-10: `Environment` measures WPE and OSE; PSE never measured. MAT-11: `Differential` lacks terminal-resolved RC and access differences. MAT-12: thermal from the op-point power map only; no transient.
- NET-01: no series-R budget per net (`parasitic.rs:14-15`); only `IrDrop` on current-carrying nets and `PerformanceBudget.r_nets` (unknown). NET-05: dr's. NET-06: tie impedance unmeasured. NET-08: a `shared_shield_return` diagnostic (`extract.rs:165-171`) and `quiet_ground`; no shared-impedance rule. NET-09: none. NET-10: LVS.
- PWR-03 dynamic supply error, PWR-07 return-path integrity: none. PWR-06: the via `front_row` heuristic only. PWR-08: not in these crates (README `:70` places it in verify).
- ENV-01: `power_uw` from the op point, no activity model. ENV-02, ENV-03: none. ENV-04: spacing proxy (`Isolation`) and bisector pull; no transfer estimate (row 45). ENV-06: a literal ring resistance (`rings.rs:72`) and a quiet-return name rule. ENV-07: not in these crates. ENV-08: user-declared `IsolatedTub` only; no automatic tub decision (`netrole.rs:160-162`).
- DEV-02, DEV-04, DEV-05, DEV-07, DEV-08: none (inductors are never matched, `class.rs:21-32`). DEV-06: `bjt_ratio`, `vce_unequal` diagnostics and the bipolar ledger thermal term only.
- ARR-04 INL/DNL and ARR-02 moments exist as kernel math (`dac.rs`) for cells, not as rules; ARR-05 settling: only a `missing` note (`lib.rs:213-215`); ARR-06: none.
- MFG-01: library fill. MFG-02, MFG-03, MFG-06: none. MFG-05: `EsdWidth` on pad nets; the `esd_diode_clamp` pattern emits nothing.
- EVD-07 is served by `known`/`unknown`/`inapplicable`/`missing`; EVD-08 partly by `ConstraintId` emission order. EVD-01..06, 09, 10 belong to library/verify.

### 7e. Audit gaps G1–G14 (`audit-01-annotator.md:446-494`) still open as read

G6 well grouping by bulk net and well min-distance: no requirement edge or rule (abutment still group-wise, `lib.rs:175-185`). G7 per-pair abutment permission: not computed. G9: sensitivities set allowances and budgets but not requirement-graph order. G13: hierarchy is read, but subckt boundaries are not placement islands beyond the `ProxNet` instance rule (`graph.rs:33-35`). G12 scale: canonical labels and `on_pin` exist; no benchmark cited at 10⁴ devices in these crates.
