# Placement and routing theory from the surveys

Sources are the four digests in `docs/plans/`:

- **BAL1-NN** — Graeb (ed.), *Analog Layout Synthesis: A Survey of Topological Approaches*, 2011, chapters 1–3 (Balasa; Lin and Chang; Strasser, Eick, Graeb, Schlichtmann). Digest `ref-balasa-graeb-survey-part1-ch01-03.md`.
- **BAL2-NN** — same book, chapters 4–7 (Dündar and Unutulmaz on routing; Said et al. on retargeting; Castro-López et al. on layout-aware sizing; Jerke, Lienig, Freuer on constraint-driven design). Digest `ref-balasa-graeb-survey-part2-ch04-07.md`.
- **LAMP-NN** — Lampaert, Gielen, Sansen, *Analog Layout Generation for Performance and Manufacturability*, 1999 (the LAYLA system). Digest `ref-lampaert-performance-driven-layout.md`.
- **SURV-NN** — Xu et al., "Performance-Driven Analog Layout Automation" (ASP-DAC 2024, cited [PDS]) and Li et al., "Performance-driven Wire Sizing for Analog Integrated Circuits" (TODAES, cited [WS]). Digest `ref-surveys-state-of-the-art.md`.

Where a claim comes from a digest's section summary rather than a numbered item, the citation names the digest and section, for example (BAL1 §1.1.4) or (BAL2 §4.5.6). Years appear only where a digest states them.

Two words are used with a fixed meaning throughout. **Constructive** means the result follows from a deterministic rule, a closed-form decode or an exhaustive bounded enumeration, with no stochastic search over topologies. **Search-based** means the result is selected by simulated annealing, genetic search, Bayesian optimisation or iterated rip-up and reroute.

---

## 1. Constraint taxonomy

### 1.1 How the sources classify constraints

The sources use three frames that are compatible with one another.

- **By what is constrained.** Jerke, Lienig and Freuer separate *simple* constraints (independent variables) from *complex* ones (variables coupled through shared quantities), and group constraints into four categories: technology, functional (electrical), design methodology (geometry) and commercial. Every constraint carries a type with a unit (an IR drop in volts, a delay in seconds). Constraints are *explicit* or *implicit*; an implicit one such as "the differential pair must be symmetric" is unusable by automation until written down. A must-fulfil objective is a constraint and a may-fulfil constraint is an objective (BAL2-45).
- **By placement requirement type.** Strasser et al. define the set T = {M_B, M_S, S, P_B, P_N}: matching inside a building block, matching between the members of a symmetric pair, symmetry, proximity inside a building block, and netlist proximity (BAL1-48).
- **By the LAYLA placement problem statement.** Lampaert groups placement constraints into symmetry, matching, performance (interconnect, mismatch, thermal) and geometry (aspect ratio, fixed height, terminal positions) (LAMP §4.2 digest; LAMP-26; LAMP-29).

The later survey lists the constraint families that current tools implement: symmetry, symmetry islands, common centroid, array regularity, proximity and boundary in placement; symmetric pair routing, no routing over active regions, power routing and shielding in routing (SURV-04, SURV-05, SURV-06, SURV-13, SURV-16, SURV-17, SURV-18). It states that these constraints and proxy objectives "do not guarantee post-layout circuit performance" (SURV-01).

### 1.2 Symmetry

**Definition.** A symmetry group is a set of device pairs and self-symmetric devices sharing one axis. The members of a pair have equal dimensions and orientation (BAL1-01).

**Formal statement.** Three equivalent forms appear:

- Corner form (Balasa): for pair (B_i, B_j) in group k, (x_i + w_i) + x_j = 2·x_axis,k and y_i = y_j; a self-symmetric device satisfies x_i + w_i/2 = x_axis,k. Axes are vertical by convention (BAL1-01).
- Centre form (Lin and Chang), vertical axis: x_j + x_j' = 2·x̂_i and y_j = y_j' for every pair, x_k^s = x̂_i for every self-symmetric module. Horizontal axis: x_j = x_j', y_j + y_j' = 2·ŷ_i, y_k^s = ŷ_i. The two axis orientations are called the two symmetry types (BAL1-01).
- Strasser et al.: ½[(x_i + w_i/2) + (x_i' + w_i'/2)] = x_sym and y_i + h_i/2 = y_i' + h_i'/2 (BAL1-01).

**Three kinds of symmetric device.** *Mirror* symmetry gives identical geometry with mirrored orientation and permits mirror-symmetric routing. *Perfect* symmetry keeps identical orientation; it is needed against anisotropic disturbances such as oblique implant, and its routing must then be parasitic-matched rather than geometrically mirrored. *Self-symmetric* devices sit on the axis, for example a bias device tied to both halves or a thermal centre (BAL1-02).

**Why it exists.** Symmetry matches the layout-induced parasitics between the two halves of a differential circuit; mismatch in those parasitics raises offset and degrades PSRR (BAL1 §1.1.4). Thermal symmetry places sensitive pairs symmetrically about radiating devices so both members see the same temperature (BAL1-03). Symmetric routing requires symmetric placement (BAL1 §3.1.2).

**Axis direction.** The direction is normally predefined by the power and ground lines or by the signal flow, which is why the symmetry-type conversion move is rarely used (BAL1-34).

**Extensions.** Hierarchical symmetry lets a group contain self-symmetric sub-groups and pairs of groups (BAL1-40). Several islands may be aligned to one axis (BAL1-38). Embedded groups are allowed when the outer group's cells surround the inner group's cells (BAL1-12). A symmetry compound is the set of pairs sharing one axis, which may span several recognised building blocks (BAL1-51). LAYLA states the same semantics: a couple is placed symmetric about an axis with identical variants and mirrored orientations, self-symmetric devices sit on the axis, and one axis is shared per group (LAMP-26).

**Verification.** Exact equality of the centre-form equations for every pair and self-symmetric device, on both axes (BAL1-01). On the routing side, the routed pair is checked by per-layer signatures of length, area and via count (SURV-14) and by the per-aggressor coupling difference (BAL2-22).

### 1.3 Symmetry island

**Definition.** A symmetry group forms a symmetry island when every module abuts at least one other module of the same group and the group's placement is connected (Def. 2.1) (BAL1-27).

**Why.** Pelgrom's model σ²(ΔP) = A_P²/(WL) + S_P²·D_x² has a distance term, so farther pairs match worse and the symmetric devices of a group should be adjacent (BAL1-26). The island also lets the group share one substrate or well region (BAL1-25).

**Verification.** Build a graph over group members with an edge when two members abut (edge-to-edge gap within the pair's minimum distance and overlapping facing edges), then count connected components; one component is the ideal (BAL1-27).

### 1.4 Matching

**Definition.** Two sub-types: M_B, matching inside a building block (e.g. the two devices of a current mirror), and M_S, matching between the two members of a symmetric pair (BAL1-48). A matching group has equal orientations; at a 1:1 ratio the variants are equal, at other ratios the members are built from equal unit devices (LAMP-26).

**Formal content.**
- Pelgrom: σ²(ΔP) = A_P²/(WL) + S_P²·D², specialised to V_T0, K and β (LAMP-10; BAL1-26). The short-channel V_T form adds 1/(WL²) and 1/(W²L) terms and the 1/√area law fails for short L (LAMP-10). These coefficients are "a minimum value of the achievable accuracy" that layout rules are needed to reach (LAMP-10).
- Variant constraint: matched modules use the same variant, e.g. the same number of gate fingers, "including same orientation" (BAL1-43).
- Alignment: a matching group yields same-variant plus alignment constraints; when the devices are split into sub-devices the alignment is replaced by common centroid (BAL1-53).
- Shared budget: the layout-induced offset is ΔP_j = Σ_k (|S_ΔVT0,k|·3σ(V_T0)_k + |S_Δβ,k|·3σ(β)_k) summed over matched pairs, split into an area term fixed after sizing and a distance term Σ_k S_Dk·D_k recomputed every move (LAMP-09).

**Layout rules for optimum matching (as checks).** Same structure; same temperature (isotherms); same shape and size (same finger count; for resistors the same L, W and number of bends, not merely the same square count); common-centroid or two-axis symmetry; same orientation with current flowing parallel and in the same direction; same surroundings (dummies). Under package stress, finger and interdigitated-finger β fluctuation reaches up to 5× the model, while quad and waffle layouts show no systematic mismatch (LAMP-11).

**Importance.** M_S outranks M_B: breaking M_S degrades offset, which is critical, while breaking M_B equally on both halves shifts both operating points alike and degrades gain, which is less critical (BAL1-48).

**Verification.** Identical variant and orientation, same source-to-drain direction, equal resistor bends, dummies on both ends, identical neighbour set within a halo (LAMP-11); per-pair distance against a Pelgrom allowance, or the sensitivity-weighted sum against the offset margin (LAMP-09); equal well-proximity and oxide-spacing distances for matched devices (SURV-09).

### 1.5 Common centroid

**Definition.** For groups A and B, x_COG(A) = x_COG(B) and likewise in y, where the group centre of gravity is area-weighted: x_COG(G) = Σ w_m h_m x_COG(m) / Σ w_m h_m (BAL1-44; BAL1-42). Example: a 16-transistor differential pair with a1–a8 and b1–b8 in parallel sharing one centroid (BAL1-44).

**Why.** Current mirrors and differential pairs against process mismatch (BAL1-25). Lampaert lists "common-centroid or 2-axis symmetry" among the rules needed to reach the Pelgrom floor (LAMP-11).

**Verification.** The equality is exact and enters a compaction LP as an equality row C·x = k (BAL1-44). Common-centroid and array-like regularity are listed as standard placement constraints in current tools (SURV-05).

### 1.6 Proximity

**Definition.** Devices sharing a model or a function are kept together so the subcircuit can share one substrate or well region or one guard ring; the outline may be rectilinear. This reduces area, wirelength and substrate coupling (BAL1-25). Two sub-types: P_B, proximity inside a building block (e.g. T1–T3 and T2–T4 in a cascode mirror), and P_N, netlist proximity, which initialises the requirement graph with a clique per net (BAL1-48; BAL1-49; BAL1-50).

**Hierarchical form.** A cluster contains at least two modules, or a module and a sub-cluster, or two sub-clusters; every super-cluster's modules and sub-clusters must be placed connected (BAL1-41). The hierarchy is either exact (the circuit hierarchy) or virtual (clusters by model, function or constraint) (BAL1-25). In Plantage, proximity groups get no explicit constraint because bottom-up construction implies proximity (BAL1-53).

**Distance definition.** d(m,n) = min(d_hor, d_vert) with d_hor = max(|x_COG(m) − x_COG(n)| − (w_m + w_n)/2, 0) and d_vert analogous (BAL1-42).

**Verification.** Each cluster's members form one connected component (BAL1-41), checked with the same union-find as the island check (BAL1-27).

### 1.7 Minimum distance (wells, guard rings, deep trench)

**Linear form.** For all m in group A and n outside A: d(m,n) ≥ d_min(m,n) ≥ 0, with d_max(m) ≥ d_min(m,n) for all n. Transistors in the same well may abut; modules outside the well keep the guard-ring width or the well spacing. With one well per symmetry group, a module keeps d_well from modules outside its well and 2·d_well from modules in other wells. The n/p distance dominated Plantage's area overhead (example 3 never below 121% area usage) (BAL1-45).

**Piecewise-linear form (deep trench isolation).** The allowed gap is d ≤ s_max (shared, stretched trench) or d ≥ d_DTI (separate trenches); the band between is forbidden. Per module, the trench stretch satisfies s_l + s_r ≤ s_max. The set is non-convex and needs a binary per edge in a MIP (BAL1-46).

**Verification.** Manhattan per-axis distance (BAL1-42); in the HB*-tree packer, white space is reserved when adjacent device types differ (BAL1-32).

### 1.8 Boundary

Certain modules must sit on the block boundary, typically I/O-connected cells (SURV-06). Verification is the distance from the cell edge to the nearest die edge; it is hard when the netlist declares a pin side (SURV-06).

### 1.9 Pin, terminal and geometry constraints

The circuit-level generator takes aspect-ratio and pin constraints from the floorplanner (LAMP §1.3.1). The floorplanner or user may impose a target aspect ratio, a fixed height and fixed terminal positions (LAMP-29). Hard geometry (maximum height, standard-cell rails) belongs in the move set; soft geometry (target aspect ratio) belongs in the cost (LAMP-27). Semi-custom styles are supported by constraining the move set (LAMP §1.4). Terminals are rectangles, never points (LAMP-30).

### 1.10 Aspect ratio

LAYLA's cost term is C_AR = |AR − AR_desired| (LAMP-29). Castro-López et al. filter the floorplan-sizing matrix to AR ∈ [AR − ΔAR, AR + ΔAR] and to max/min W and H, then pick minimum area (BAL2-39). Plantage returns a Pareto front of aspect ratios so the designer or top-level floorplanner chooses (BAL1-63). Balasa's tool handles soft-cell (capacitor) aspect ranges (BAL1 §1.5). Verification is the bounding box.

### 1.11 Current flow

Monotonic current-flow placement enforces a direction VDD → load → signal devices → tail → GND along DC current paths; a pole-aware variant also minimises wire crossings. The aim is fewer bends, crossings and vias on high-current paths (SURV-07). Verification: an ordering constraint y(dev_k) ≥ y(dev_{k+1}) per path, or x for horizontal flow (SURV-07).

### 1.12 Signal flow

System signal-flow-aware placement explicitly considers the critical signal flow and reports post-layout gains (SURV-08). The HB*-tree framework clusters by model, function or signal flow (BAL1 preface digest) and can host a signal-flow placer inside a node (BAL1-41). The symmetry axis direction is normally set by rails or signal flow (BAL1-34). Verification: stage centroids monotone along the chosen axis from inputs to outputs (SURV-08).

### 1.13 Net length matching and exact routing

Exact routing requires the pair's wire lengths to be the same on every layer so that the wiring parasitics match precisely (SURV-14). LEMAR detours wires to fix parasitic mismatch (SURV-14). Two nets under a symmetry constraint always get the same width (SURV-29). Symmetric nets can still receive unequal parasitics from coupling to other nets and from corners in bundles; PARSY adds extra paths and adjustment sections to balance them (BAL2-22). Verification: per-layer signature of length, area and via count (SURV-14); per aggressor, ΔC_a = |C(pos,a) − C(neg,a)| (BAL2-22).

### 1.14 Shielding

Reduce coupling by spacing first; if that is impossible, insert a shield between critically coupled nets. Shields must be tied to a DC potential or ground. Three geometries: same layer, different layer (a plate between), and bulk (substrate contacts). ROAD builds shields after all wires, as a last resort (BAL2-10). A shielding routing algorithm selects victims by net classification (SURV-18). Verification: the coupling budget still fails after spacing (BAL2-10).

### 1.15 Crosstalk and coupling spacing

Net classes drive spacing and ordering: ILAC uses sensitive, noisy, non-critical signal and power; ANAGRAM uses neutral (supply, bias), noisy (high swing: clocks, outputs) and sensitive (BAL2-11; LAMP-48). SLAM assigns each node a distance constraint with priority (highest priority → shortest wire) and each node pair a prioritised spacing, sensitive–noisy highest (BAL2-13). The coupling model used inside routers is C_2D = C_1D + l·C/d, where C_1D = A·C_α + S·C_β is the area plus fringe term (BAL2-09). Verification: the 2-D model within a region of influence (LAMP-39; BAL2-09).

### 1.16 Isolation and substrate coupling

On a low-impedance (epi on heavily doped bulk) substrate the bulk acts as one node and separation beyond 4× the effective epi thickness (about 40 µm) gives no improvement; on a high-impedance substrate coupling falls almost linearly with separation. Guard rings help only when very close to the sensitive circuit and biased from dedicated package pins. Injectors are junction and collector capacitances, capacitor bottom plates, wells at varying potential and impact ionisation; receivers are capacitive coupling and the body effect. A resistive substrate model is adequate to 2 GHz (LAMP-16). Verification: the distance rule for the actual substrate type, plus an ERC check that the ring net is a dedicated pin (LAMP-16).

### 1.17 Thermal

Thermally sensitive pairs go symmetrically about heat-radiating devices; otherwise temperature mismatch results and unbalanced thermal coupling can even cause oscillation (BAL1-03). Degradation comes from absolute shifts (reference drift) and from gradients (offset, drift) (LAMP §2.8.1). The failure rate doubles for roughly every 10 °C (LAMP-15). The budget form is ΔP_j = Σ_k S_ΔTk·ΔT_k with the placement temperature profile obtained by superposing precomputed per-variant profiles (LAMP-13). Thermal-driven symmetry that keeps matching is a current-tool constraint (SURV-11). Verification: the profile superposition and a per-pair breakdown table (LAMP-13; LAMP-14).

### 1.18 Current density, electromigration, IR drop and net splitting

Wire width W_i = I/I_max,i, via count N_via = I/I_max,ij, with the via-array width [⌈√N_via⌉ − 1]·sep + 2·max(ovl_i, ovl_j); width is a property of a segment, not of the whole net (LAMP-37). Inside a device, S/D straps and via counts are sized from current, and very wide devices are sliced so no part is farther than a specified distance from a bulk contact (LAMP-22). Current densities in one net may differ by an order of magnitude; a low-current branch tapped from a segment carrying I sees V_drop = R_wire·I with R_wire = l·R_sh/w, so the branch is routed separately or the shared segment widened (BAL2-06). Nets are modelled as subnets of similar current density, each a set of physically connected groups (BAL2-04). IR constraints are pin-to-pin on star-shaped nets (BAL2-46) and wire resistance rises with temperature as (1 + TK1·(T − T_ref)) (BAL2-52). Wire sizing for EM uses Black's equation (SURV-17). Verification: a current-density verifier after current-driven routing (BAL2 §4.7), pin-pair IR (BAL2-46).

### 1.19 Keep-out over active regions

Routing over transistor active regions is forbidden in several tools (SURV-16). ROAD allows over-the-device routing but lets sensitive modules be excluded (BAL2 §4.5.3.2).

### 1.20 Layout-dependent effects and intentional spacing

Placement reduces well-proximity, length-of-diffusion and oxide-spacing effects, and well-proximity ties into well-island generation during placement (SURV-09; SURV-10). STI stress and well proximity change device performance, so compaction must detect wells not at minimum size and treat them specially (BAL2-37). Verification: equal environment distances for matched devices (SURV-09).

### 1.21 Device merging permission

An analog placer must exploit geometry sharing (shared diffusion) and generator reshaping (BAL1-64). Terminal overlap is legal iff the terminals are on the same net, on the same layer and their devices have the same bulk potential; legal overlap lowers junction capacitance and illegal overlap must reach zero (LAMP-20).

### 1.22 Array regularity

Array-like module arrangements for regularity (SURV-05). Regular arrays (flash ADCs, current-steering DACs) route supply and bias by abutment with cells flipped to share lines; the quality metric is equal R, C and matching, not delay (BAL2-28).

### 1.23 Performance constraints

A spec is an interval P ∈ [P_min, P_max]; after the process spread [P_proc^min, P_proc^max] the layout may spend ΔP_lay ∈ [P_min − P_proc^min, P_max − P_proc^max] (LAMP-01). Degradation is linear in the parasitics, ΔP_j = Σ_i S_i^j·x_i (LAMP-02). The penalty is the part outside the margin (LAMP-06). The constraint-generation alternative turns specs into parasitic bounds Σ_j S⁺_ij·p_j ≤ ΔW⁺_i,max (BAL2-16). Verification: ΔP within margins at every tier (LAMP-06); normalised distance to a bound d_u(x) = exp(x − x_u) − 1 (BAL2-51); post-layout extraction and simulation, which the modern survey treats as the only judge (SURV-01; BAL2-42).

### 1.24 Importance orderings given by the sources

| Context | Ordering | Rationale | Source |
|---|---|---|---|
| Placement requirement types | M_S ≻ M_B ≻ P_B ≻ S ≻ P_N | Breaking a symmetric pair's matching degrades offset; in-block matching broken equally on both halves degrades gain; block proximity cannot harm symmetry because symmetry acts on whole blocks; netlist proximity is least important. The static order was correct in all experiments; a dynamic priority from simulated violation impact is possible. | BAL1-48; BAL1-52 |
| Hard vs soft in annealing | Hard constraints (symmetry, equal orientation and variant, hard height) in the move set; performance, soft geometry and overlap as penalties | A constraint in the cost is traded off and not guaranteed to reach zero; performance cannot be maintained by construction so it must be a penalty; overlap stays soft to find beneficial merges | LAMP-27 |
| Routing net order (ILAC) | Power, sensitive, non-critical, noisy last | Power and non-critical nets act as shields between critical and noisy nets | BAL2-12 |
| Routing net order (ROAD) | More constraints → higher priority; symmetric nets very high; shields after all wires | Critical nets take privileged locations and become obstacles for later nets | BAL2-12 |
| Routing pairwise spacing (SLAM) | Sensitive–noisy pairs highest priority and largest spacing | Critical-net analysis | BAL2-13 |
| Routing order (LAYLA) | Rip up and reroute in increasing impact factor F_k | The highest-impact nets keep their ideal pre-routed paths | LAMP-42 |
| Detailed-routing module order (ALADIN) | Symmetric and more central modules first | Constructive compaction-based wiring | BAL2-23 |
| Constraint vs objective | Must-fulfil objective = constraint; may-fulfil constraint = objective | Uniform constraint representation | BAL2-45 |

### 1.25 Verification methods across the sources

- **Explicit, typed, categorised records with provenance**, so that every constraint is checkable and traceable (BAL2-45).
- **Complex constraints as conjunctions** of structural, functional and arithmetic parts, e.g. starShaped(N) ∧ netPin(N,P1) ∧ netPin(N,P2) ∧ irDrop(P1,P2,V) ∧ V < 0.1 (BAL2-46).
- **Hierarchical assignment** top-down, bottom-up or both, propagating through series metal resistors so every physically connected segment inherits the constraint (BAL2-47).
- **Over-constraint detection**: mutually conflicting constraints form a constraint-satisfaction problem, resolved by propagation, relaxation or backtracking; weight-based resolution degrades when many weights are equal; an earlier over-constraint can block later optimisation and needs a root-cause report (BAL2-48).
- **Simplification and propagation** before solving: derive A ≤ C from A ≤ B ∧ B ≤ C, drop dominated bounds (BAL2-54).
- **Meta-verification**: decompose complex checks into independent sub-checks served by existing tools, order them cheap-first, stop at the first violation; static (sign-off) versus dynamic (what-if inside algorithms) verification; the named tools are Calibre PERC and Virtuoso IC 6.1 (limited) (BAL2-53; BAL2 §7.4.5).
- **Constraint sensitivity analysis**: normalised distance to bounds and weakly coupled groups solved independently; scales to a few thousand design variables (BAL2-51).
- **Inverse transformation**: a high-level constraint transformed into low-level ones needs an inverse so the sum of shares can be verified against the original (BAL2-50).
- **Degradation attribution**: per spec, the contributions per net, per coupling pair, per matched pair (distance) and per thermal pair, since a plain extracted netlist gives "no clue" which parasitic caused a violation (LAMP-35).
- **Pre-layout infeasibility**: a spec whose process spread or mismatch area term alone exceeds the margin is flagged before placement (LAMP-49).
- **Quality metrics**: area usage = bounding area / module-plus-well area, ideal 100% (BAL1-62); area loss = W·H − Σ W_i·H_i − routing area (BAL2-40).
- **Model validity**: a segment is lumped RC only while |γD| ≪ 1, G₀ ≪ ωC₀ and ωL₀ ≪ R₀; RF needs inductance (LAMP-17).

---

## 2. Composition techniques

### 2.1 Why the representation decides what can be guaranteed

The device-level analog placement problem is NP-hard even without analog constraints (BAL1 §1.1.2). An analog placer must additionally support symmetry and matching constraints, device merging and a library of reshapeable module generators (BAL1 §1.1.2). The central finding of the Graeb survey is that in analog circuits most topological codes are symmetry-infeasible, and rejecting infeasible codes during annealing was "extremely ineffective". A representation is useful only if feasibility can be recognised from the code without building the layout and the moves stay inside the feasible subspace; evaluation speed matters less than this (BAL1-04).

| Representation | Kind | What it guarantees by construction | Known failure modes | Source |
|---|---|---|---|---|
| Absolute (flat) coordinates with SA | Search | Nothing geometric; symmetry and matching are easy to express in the move set; beneficial overlap (merging) can be explored | Slow convergence; nonzero final overlap needs post-processing that degrades optimality; overlap weight fragile (too large blocks search, too small lets cells collapse); TimberWolf needed negative-feedback weight control | BAL1-05; LAMP-30 |
| Slicing (Polish expressions) | Search or template | Non-overlap; channel structure | Cannot reach many topologies; packs badly when cell sizes differ, which is typical in analog; needs virtual symmetry axes in the cost | BAL1-04; BAL1 §1.1.6 |
| Sequence pair (SP) | Search | Non-overlap; O(n²) or O(n log log n) evaluation | Symmetry not encoded; (n!)² codes | BAL1 §1.1.6 |
| Symmetric-feasible SP | Search with constructive decode | Exact symmetry, minimum height and minimum group span for the given code; non-overlap | Condition is sufficient not necessary (rare misses); non-S-F codes loop forever in the fixer; groups must not intermingle; no island guarantee | BAL1-06..13 |
| TCG | Search | Equivalent to SP | Θ(n²) conversion and quadratic evaluation; not worth adopting | BAL1-19 |
| O-tree / binary tree, S-F variant | Search | Fewer codes than SP for n ≥ 3; symmetry by condition (1.2) | Quality "seems to be poorer" than S-F SP; O-tree infeasibility test quadratic | BAL1-20..24 |
| B*-tree | Search | Non-overlap; linear-time packing | Symmetry not encoded; 57,657,600 trees for 8 modules | BAL1-28; BAL1-20 |
| ASF-B*-tree + HB*-tree | Search with constructive decode | Exact symmetry, symmetry island, no redundant codes, O(n) packing, well white space, rectilinear islands | Axis alignment across islands needs dummy blocks; search over trees still SA | BAL1-29..36 |
| CBL | Search | Mosaic floorplans with zero dead space | Not analog-aware | BAL1 §1.1.6 |
| B*-tree with enhanced shape functions (Plantage) | Constructive (bounded enumeration + LP/MIP) | Pareto front of area-optimal placements satisfying proximity, symmetry, CC, minimum distance and variant constraints; deterministic; no tuning parameter | Enumeration bounded by hierarchy; a too-large root group was split by hand; MIP for DTI is slow | BAL1-56..63 |

Empirically, with one cost and one schedule, every symmetric-feasible topological technique beat the absolute representation in CPU and sometimes in quality, and annealing was easier to tune (BAL1-24). On MCNC and two industrial circuits the HB*-tree was 4.09× faster than SP with dummy nodes and gave 7.1%, 6.6%, 1.6% and 10.3% smaller area than the SP variants, with 39.88× and 5.68× speedups over SP+LP and SP-dummy (BAL1-37). The complexity table ranks ASF-B*+HB* first with O(lg n) perturbation and O(n) packing against Ω(n²) packing for SP+LP and B*+ESF+LP (BAL1-36).

### 2.2 Symmetry groups by construction

#### 2.2.1 Symmetric-feasible sequence pairs (Balasa)

A sequence pair (α, β) places A left of B when A precedes B in both sequences, and A above B when A precedes B in α but follows it in β (BAL1-06). The pair is symmetric-feasible when, for all distinct x and y in a group, α_x⁻¹ < α_y⁻¹ ⇔ β_sym(y)⁻¹ < β_sym(x)⁻¹: each pair keeps the same order in α and β so its members sit side by side, and cells from different pairs appear in reversed order relative to their partners (BAL1-06). The condition is sufficient but not necessary; a symmetric placement whose unique SP violates it exists but such cases are rare (BAL1-06). The feasible space is at most (n!)²/∏_k (2p_k + s_k)! codes; for 7 cells with 2 pairs and 2 self-symmetric cells this is 35,280 of 25,401,600, a 99.86% reduction (BAL1-07).

Decoding is four sweeps, all O(n log log n) with Johnson's queue, and needs no LP solver:
1. Ordinates: a reverse longest-common-subsequence sweep that enforces equal y for partners and restarts when a raised y breaks a vertical constraint; termination is guaranteed by the condition and the height is minimal; at most 3 passes in practice (BAL1-08).
2. Abscissae initialisation by a forward LCS sweep (BAL1-09).
3. Axis selection: build a DAG of pairs from the right contour, set x_sym to the maximum over sink pairs of (x_j + x_k + w_k)/2, shift inner pairs so they hug the axis; the group's x-span is minimal relative to the topological constraints (BAL1-10).
4. Sweep right pushing the right member of each pair, then sweep left pulling the left member (BAL1-11).

Multiple groups are processed one at a time and frozen, inner before outer, in O(G·n log log n). Groups must not intermingle in α and β, except that embedded groups may be surrounded. A code that is not symmetric-feasible for every group may loop forever (BAL1-12). The initial code is α = a1…ap c1…cs bp…b1 and β = a1…ap cs…c1 bp…b1, pairs nesting like brackets around a vertical stack of self-symmetric cells; the move set is restricted to paired interchange, paired relocation, paired rotation and mirroring, with asymmetric cells free and whole-group moves at low probability (BAL1-13). The contour data structures (segment tree, red–black interval tree, deterministic skip list, Johnson's priority queue) matter only for speed; analog circuits seldom contain more than 100 cells per hierarchical level (BAL1-14..17). Results: 17–116 cells in 0.2–21 minutes, with the priority queue fastest and the segment tree slowest (BAL1-24).

**Inputs needed.** Pairs, self-symmetric cells, group nesting, cell dimensions (equal per pair), and pairwise minimum distances as edge weights (BAL1-09). **Guarantees.** Exact symmetry, minimum height, minimum group span for the given code. **Does not guarantee.** A symmetry island, or coverage of every symmetric placement. **Kind.** The decode is constructive; the topology selection is annealing (BAL1-13).

#### 2.2.2 ASF-B*-trees and symmetry islands (Lin and Chang)

A B*-tree is a binary tree whose root is the bottom-left module; a node's left child is the lowest adjacent module to its right and its right child is the first module above it at the same x; y comes from a contour and preorder packing is linear (BAL1-28). The automatically symmetric-feasible B*-tree keeps one representative per pair and, for each self-symmetric module, its half on one side of the axis; only representatives are packed and the rest are mirrored. A self-symmetric representative must abut the axis (Lemma 2.1); a pair representative off the axis is always feasible (Lemma 2.2); self-symmetric representatives lie on the rightmost branch for a vertical axis or the leftmost for a horizontal one (Property 2.1). Theorems: the tree is symmetric-feasible for either axis (2.1), its packing is a symmetry island (2.2), and it corresponds one-to-one with compacted symmetric placements, so there is no redundancy (2.3) (BAL1-29). Packing keeps both horizontal and vertical contours and derives the island's bottom contour from the convex points of the two vertical contours, in time linear in the group size (BAL1-30).

The HB*-tree places islands together with non-symmetric modules. A hierarchy node holds one ASF-B*-tree; contour nodes represent the island's top-contour segments left to right under six structural rules, so rectilinear islands are handled without slicing them into sub-blocks (BAL1-31). Packing finds the island's best y by matching its bottom contour against the current contour (minimising dead space), and when a module's device type differs from its packed neighbours it is snapped out to reserve well or guard-ring space; Theorem 2.4 gives O(n) (BAL1-32). Perturbations are rotate, move and swap on the HB*-tree, plus representative change (O(1), area-neutral) and symmetry-type conversion inside the ASF-B*-tree, with hierarchy nodes chosen less often (BAL1-34). When a perturbed island loses top segments, dangling nodes reattach to the nearest contour node in amortised O(1) (BAL1-35). The cost is Φ = α·area + β·HPWL with T0 = −Δ_avg/ln P, cooling 0.9, 20,000 iterations per temperature (BAL1-33).

Extensions: several islands align to one axis by inserting a zero-width dummy block as each island's parent and adjusting its width (BAL1-38); non-symmetric modules join a group as a self-symmetric cluster or as a pair of clusters (BAL1-39); a group may contain self-symmetric sub-groups and group pairs, packed as nested islands (BAL1-40); hierarchical clustering uses one HB*-tree per cluster, with #trees = #subcircuits + 1, and the framework can host CBL, a grid-based common-centroid placer or a signal-flow placer inside a node (BAL1-41). Simultaneous hierarchical optimisation is preferred to bottom-up integration because a subcircuit's local optimum may not be globally optimal (BAL1 §2.1).

**Inputs needed.** Pairs and self-symmetric modules per group, axis orientation per group, module dimensions, device types for well spacing, cluster hierarchy. **Guarantees.** Exact symmetry, island connectivity, non-overlap, no redundant codes, well white space where types differ, O(n) packing. **Does not guarantee.** Minimum area or wirelength for a given tree (needs the anneal), perfect-symmetry orientation (the construction mirrors), axis alignment across islands without dummy blocks. **Kind.** Decode constructive; topology selection annealing.

#### 2.2.3 Symmetry in the flat representation (LAYLA)

LAYLA keeps absolute coordinates because symmetry lives in the move set, absolute coordinates allow parasitic estimation, non-slicing topologies are denser, and overlap is exploitable for merging (LAMP-30). Hard constraints are enforced in the move set, which wastes no CPU on illegal states, while performance and overlap are penalties (LAMP-27). The symmetric move set is symmetric translation, symmetric swap along the axis, symmetric flip and axis shift; orientation groups rotate together with precomputed orientation variants, and shape groups change variants together (LAMP-28; LAMP-12). Devices are modelled as minimum overlapping covers with bulk net and thermal profile, and terminals as disjoint rectangle partitions (LAMP-30). **Kind.** Search-based; symmetry holds by construction only through the move set, and overlap is repaired by the weight schedule (LAMP-29).

### 2.3 Matched groups

- **Variant constraints**: matched devices use the same variant, including orientation; modules may have several variants such as finger counts and capacitor aspect ratios (BAL1-43). LAYLA's shape groups change identical devices' variants together and orientation groups rotate together (LAMP-12).
- **Alignment and common centroid from matching groups**: a matching group yields same-variant plus alignment (equal centre y of the pair), replaced by common centroid when devices are split into sub-devices; the equalities enter the compaction LP (BAL1-53; BAL1-44).
- **Ratioed matching**: at ratios other than 1:1 the members are built from equal unit devices (LAMP-26).
- **Distance by performance**: the distance between matched devices is traded against the other effects through the sensitivity-weighted offset budget, so insensitive pairs may sit far apart to make room (LAMP-09). In the comparator example, pair distances ranged from 38 to 119 µm with sensitivities from 0.2 to 12 µV/µm and a total degradation of 1.645 mV (LAMP-09).
- **Building-block mapping**: differential pair, level shifter and simple mirror match T1–T2; cascode, four-transistor and wide-swing mirrors match T1–T2 and T3–T4 with block proximity T1–T3 and T2–T4 (BAL1-50).
- **Generators**: dedicated generators for interdigitated cascodes and common-centroid pairs exist because merging cannot produce these structures (LAMP §3.3.4; LAMP-23).

**Kind.** The equalities are constructive (exact in a decoder or LP); the distance trade is search-based.

### 2.4 Hierarchical and constraint-generation approaches

Prior constraint extraction used sensitivity analysis, classification of nets by susceptibility, structural symmetry detection by graph labelling or recursive symmetric-pair detection, and a sizing-rules library of building blocks (BAL1-47). Plantage's flow is netlist → symmetry analysis and building-block recognition → SMP graph → HSMPG tree → constraints (BAL1 §3.2).

- The **SMP graph** is a typed multigraph over devices with one edge per shared requirement, initialised with a clique per net (P_N), then filled with M_B and P_B from building blocks found by subgraph isomorphism against the library, and M_S and S from symmetry analysis (BAL1-49; BAL1-50). The digest notes that the source does not say whether supply nets are excluded from the cliques (BAL1-49).
- **Symmetry analysis** finds symmetric device pairs (similar to Arsintescu's method); all pairs sharing one axis form a symmetry compound, each pair gets an M_S edge and the compound an S clique (BAL1-51).
- The **HSMPG tree** is built by Algorithm 3.1: for each type in importance order, filter the edges, take connected components with more than one node, create a group (MG for matching, SG for symmetry, PG otherwise) and contract it to a super node. It is agglomerative clustering with the requirement types as similarity. Groups had 2–14 members (average 2.2–2.9) across five circuits (BAL1-52).
- **Constraint generation** from the tree: matching groups give same-variant and alignment (or common centroid); symmetry groups give y-equalities per pair and x-centre alignment across the group; proximity groups give nothing explicit because bottom-up construction keeps members together (BAL1-53).

**Kind.** Constructive and deterministic; constraint generation took 0.3–1.2 s (BAL1-61). **Failure mode.** The static order was correct in all experiments but is not proven general; a dynamic priority is possible (BAL1-52).

### 2.5 Deterministic enumeration with enhanced shape functions (Plantage)

A shape function is a Pareto list of (w, h) shapes; horizontal addition gives (w1 + w2, max h), vertical gives (max w, h1 + h2), dominated shapes are pruned, and plain shape functions produce slicing structures only (BAL1-56). An enhanced shape stores (w, h, B*-tree) and is dominated by lower h at equal or lower w or lower netlength at equal (w, h) (BAL1-57). Horizontal addition attaches the second tree's root to the first tree's lowest-rightmost node; vertical addition splits the second tree into baseline segments and attaches each to the node that shadows it. Both preserve in-order and pre-order relations, hence constraint feasibility, and often give a smaller result than the plain sum (BAL1-58; BAL1-59). Addition is not commutative, so parents combine children in every sequence (BAL1-60).

Each B*-tree is placed by linear or mixed-integer programming: the vertical constraint graph comes from the tree, the horizontal one from a shadow-based construction that adds only the edges needed for minimum distances (including diagonal pairs that are too close vertically), and symmetry and common-centroid equalities enter as equality rows (BAL1-54; BAL1-55). Deep-trench disjunctions need a binary per edge with a big-M (BAL1-46).

Enumeration is bounded by the hierarchy: basic groups enumerate all B*-trees times all allowed variant combinations, independently and in parallel; parents combine children's enhanced shape functions; the output is a Pareto front of complete placements with different aspect ratios. It is deterministic with no tuning parameters (BAL1-60; BAL1-63). On five circuits with 9–42 devices the best area usage was 110–129% and runtime 1–691 s; with wells the CPU rose 72–76%; a 30-module DTI example took about 15 minutes because MIP is much slower than simplex; the root proximity group of the largest example was too large and was split by hand (BAL1-61; BAL1-46; BAL1-60). It was the first placer to handle proximity, symmetry, common centroid, minimum distance and variant constraints deterministically (BAL1 §3.7). Labelled B*-tree counts (336 for 4 modules, 57,657,600 for 8) bound the group size that can be enumerated (BAL1-20).

**Kind.** Constructive. **Inputs.** HSMPG tree, module variants with dimensions, pairwise minimum distances, symmetry and CC equalities. **Guarantees.** Every placement on the front satisfies all constraints exactly; the front is area-optimal within the enumerated space. **Does not guarantee.** Global optimality beyond the hierarchy bound; wirelength optimality (netlength is only a dominance tie-break).

### 2.6 Slicing templates with shape functions (layout-aware sizing)

Castro-López et al. use a binary slicing-tree template whose leaves are device generators (folded transistor, differential pair, cascode, mirror, capacitor array, folded resistor) with styles such as common-centroid and interdigitated. Each leaf has a Pareto shape list from its geometric parameters (fingers, unit capacitor side); the tree is combined bottom-up keeping Pareto pairs, with template slice separations added, in O(d·l_T). Top-down, rows violating the aspect-ratio window or W/H limits are dropped, minimum area is picked, and leaf parameters follow. Wire width self-adapts to current; symmetries live in the template (BAL2-39). Results: AR targets [0.91, 1.1] → 1.00, [1.95, 2.05] → 1.96, W < 150 µm → 143.2, with area loss 0.57–3.06% (BAL2-39; BAL2-40). The template costs 1.5–2× the time of manual layout of the block to write and runs in a few seconds (BAL2 §6.3.2). **Kind.** Constructive (Stockmeyer-style). **Does not guarantee.** Non-slicing density; the template is per circuit type.

### 2.7 Constraint-graph compaction and retargeting

One-dimensional compaction places a vertex per edge coordinate, an arc of weight d_ij per constraint x_j − x_i ≥ d_ij, and solves longest path from the source; a minimum-distance-only graph is a DAG and the solve is O(N + E) (BAL2-30). Pure longest path pushes everything left and stretches wires, so an objective minimising wirelength or closeness to the source layout is added (BAL2-31). Symmetry x_b − x_a = x_d − x_c, exact dimensions x_j − x_i = K and hierarchy constraints are multivariable and need an LP; the longest-path solution is the initial basic feasible solution with artificial variables under a big-M for unsatisfied equalities; every loop in the constraint graph contains at least one non-distance constraint (Theorem 5.1) (BAL2-32). The graph-based simplex tends to linear for distance-only problems and to O(v³) when multivariable constraints dominate; on a Miller and a folded-cascode opamp it was 2–4× faster per iteration and ≥10× overall than revised simplex (BAL2-33; BAL2 §5.8). Minimum-distance constraints freeze edge order, so after resizing obsolete constraints block area recovery; the remedy is to iterate horizontal and vertical passes regenerating constraints, since 2-D compaction is NP-complete; a VCO migration needed manual constraint deletion (BAL2-34). Contact and via arrays are merged into one cut before compaction and regenerated after (BAL2-35). Conditional and context design rules are converted to edge form by the sign-off DRC engine, so generation stays consistent with sign-off (BAL2-36). Deliberate non-minimum wells must be detected and preserved (BAL2-37). Changing device realisations (finger counts) under the same relative placement recovers area after resizing without 2-D compaction (BAL2-38). Designers distrust optimisation-based retargeting that re-invents the layout and accept compaction that preserves floorplan, placement and routing (BAL2 §5.2).

**Kind.** Constructive. **Inputs.** Edge coordinates, design rules, equalities, optional closeness objective. **Guarantees.** Legal, minimal coordinates satisfying every linear constraint. **Does not guarantee.** Topology change, 2-D optimality, parasitic preservation unless the closeness objective is used.

### 2.8 Module generation

Module generation fixes device-level parasitics, placement sets their minimum values and routing sets the final values (LAMP §2.10). Four strategies: a fixed library of procedural sub-circuit generators hard-codes sharing but loses opportunities and must be maintained per process; dynamic merging (KOAN) finds every sharing option during placement at the cost of burdening the placer and cannot build interdigitated cascodes; simultaneous placement and module optimisation (Charbon) enumerates alternative module sets and swaps between them; LAYLA combines dedicated generators with dynamic merging (LAMP §3.3.1–3.3.4). Transistor stacking by Euler trail on a diffusion graph (nets as vertices, transistors as S–D edges; split by type and bulk, fold, split by width, add a supervertex to odd-degree vertices) is linear and optimal for a class of circuits, whereas path enumeration plus clique search is exponential (LAMP-21). Generators have an interface mode returning bounding box or cover plus terminals for every variant, and a layout mode drawing the chosen variant, so variants and shapes are chosen during placement (LAMP-25). They are written in symbolic geometric and electrical parameters with generic-to-process layer maps, which allowed porting to 20+ processes from 5 foundries in about 1 hour per port (LAMP-24). The MOS generator derives fingers and slices from a target aspect ratio under bulk-contact and current rules, sizes S/D straps and vias from current, and routes the source in metal2 so the gate lead stays short (LAMP-22). The cascode pair generator omits the contact on the shared node so the gates sit at minimum distance (LAMP-23). Legal overlap (same net, layer, bulk) lowers junction capacitance and is promoted on sensitive nets (LAMP-19; LAMP-20).

**Kind.** Constructive (generators, Euler stacking); dynamic merging is search-based.

### 2.9 Constructive placement, templates and knowledge-based placement

Constructive placement from an expert base or schematic-driven is fast but depends on the order in which devices are selected (BAL1 §1.1.3). RACHANA assumes the drawn schematic has the best distribution and interleaves place-a-module with route-its-connections (BAL2 §4.5.4). Templates store relative positions as equations on element boundaries; the extended constraint graph adds an edge from an element's right boundary to every element with larger x and nothing in between, which guarantees no overlap for any resizing, and compaction removes slack (BAL2-24). LAYGEN adapts template routing to a new placement and then optimises (BAL2-25). Templates give reproducible incremental updates where probabilistic generators give a different layout per run (BAL2 §4.5.6). **Kind.** Constructive. **Failure modes.** Order dependence; per-circuit and per-technology template effort (LAMP §1.5.2.2); templates cannot handle unseen designs (SURV-21).

### 2.10 Integrated place-and-route and in-loop estimates

Analog has few nets, so global routing can run inside placement: from routing between placement iterations to wires as placement objects (KOAN/ANAGRAM II anneals devices and wires with DRC, crosstalk and net area in the cost); GELSA routes approximately at each annealing step on slicing structures; a sequence pair over blocks and wire rectangles with compaction graphs; GA global placement then annealing with min-Steiner global routing per state; tabu search over tiles (BAL2 §4.5.4; BAL2-21). ALADIN scores placements with a minimum-Steiner-tree estimate on a channel graph weighted by length plus congestion because Manhattan estimators were unreliable (BAL2-21). LAYLA measured a 10–15% area penalty against manual layout caused by the separation of place and route and overestimated routing area, and proposed a global router inside the placement loop or performance-aware compaction, since simultaneous P&R was judged too complex beyond small circuits (LAMP-47). **Kind.** Search-based.

### 2.11 Flow-, well-, thermal- and LDE-aware placement in current tools

Monotonic current-flow placement (SURV-07), signal-flow-aware placement (SURV-08), LDE-reducing placement and well-island generation during placement (SURV-09; SURV-10), and thermal-driven symmetry (SURV-11) are implemented as constraints or costs inside analytical, annealing, SMT or reinforcement-learning engines (SURV-12). **Kind.** Search-based in the cited tools; the ordering constraints themselves are constructive rules.

### 2.12 Summary

| Technique | Kind | Realises by construction | Needs | Fails when |
|---|---|---|---|---|
| S-F sequence-pair decode | Constructive decode in SA | Symmetry, min height, min group span | Pairs, dims, d_min | Groups intermingle; non-S-F code | 
| ASF-B*/HB*-tree decode | Constructive decode in SA | Symmetry, island, well white space, rectilinear islands | Groups, types, hierarchy | Axis alignment without dummies |
| SMP/HSMPG constraint generation | Constructive | Hierarchy, priorities, equalities | Netlist, block library | Static order not general |
| Plantage enumeration | Constructive | Pareto front satisfying all constraints | HSMPG, variants, d_min | Group too large; MIP slow |
| Shape functions / Stockmeyer | Constructive | AR window, min area over variants | Slicing template, leaf shapes | Non-slicing density lost |
| Constraint-graph / LP compaction | Constructive | Legal minimal coordinates with equalities | Edges, rules, equalities | Edge-order lock; long wires |
| Module generators, Euler stacking | Constructive | Sharing, EM straps, variants | Netlist, process rules | Interdigitated structures need dedicated generators |
| Template extraction / LAYGEN | Constructive | Topology preservation under resize | Prior layout | Unseen design |
| Expert/schematic-driven placement | Constructive | Fast first placement | Schematic | Order dependence |
| Flat SA with analog moves | Search | Symmetry via moves only | Everything | Overlap weight; residual overlap |
| Integrated P&R | Search | Routing-aware placement | Nets | Complexity |

---

## 3. History of template, procedural and knowledge-based layout systems

Jerke, Lienig and Freuer describe the evolution of analog layout as polygon pushing → schematic-driven layout → constraint-driven design → analog design automation, with "analysing" and "verifying" a precondition for "synthesising" (BAL2 §7.1). The 2011 book's preface quotes an EDA Weekly statistic from 2005: analog appears on 75% of chips, takes 40% of design effort and causes 50% of first-silicon errors, and it calls industrial analog layout synthesis "still in its infancy" (BAL1 front matter). Lampaert's 1999 account of the analog macro-cell style, module recognition → variants per module → placement with variant selection → routing → optional compaction, is the frame most systems share (LAMP §1.5.1).

### 3.1 Procedural module generators

- **Approach.** Precoded, parameterised layouts of circuits or sub-circuits (BAL1 §1.1.1). Fixed library generators for differential pairs, mirrors and cascodes hard-code diffusion sharing (LAMP §3.3.1).
- **Quality and cost.** High quality but "a lot of code per topology" (LAMP §1.5.2.1). Mapping a circuit onto fixed generators is non-trivial, sharing opportunities are lost, and each generator must be maintained per process (LAMP §3.3.1).
- **Where it survives.** At the device and small-structure level. LAYLA's generator library (MOS, capacitors, resistors, inductors, transformers, matched pairs) was 36 kLOC of its 115 kLOC and was ported to 20+ processes in about an hour each through symbolic rules (LAMP §3.7; LAMP §6.2.1; LAMP-24). Dedicated generators remain necessary for interdigitated cascodes and common-centroid pairs (LAMP §3.3.4). Manual layout effort splits as module generation 50%, placement 12.5%, routing 25%, verification 12.5%, which is why generators carry the largest productivity share (LAMP-50). Modern template generators BAG and LAYGO/LAYGO2 are the descendants (SURV-02).

### 3.2 Template-based ("design by example") systems

- **Approach.** A template fixes relative placement and routing; Philips' design-by-example used slicing trees with a river router (LAMP §1.5.2.2). Routing templates stretch or compact rather than search (BAL2 §4.5.6). BALLISTIC is a layout language of relative placement and wiring commands, a few hundred lines per medium cell; CAIRO is a C-function superset with relative routing to reference points; hierarchical parameterised templates support retargeting with interconnect parameterised by design rules and current density; LAYGEN adapts template routing to a new placement and then optimises with a genetic algorithm (BAL2 §4.5.6). The LDS script expresses positions as equations on element boundaries, extracted from a placement by extended constraint graphs and solved by LP (BAL2-24).
- **Why abandoned for new design.** Templates must be redone per circuit type and per technology (LAMP §1.5.2.2); writing one costs 1.5–2× the time of manually laying out the block (BAL2 §6.3.2); they need human effort for designing and adjusting (SURV-02); retargeting from manual layouts cannot handle unseen designs (SURV-21).
- **Where they survive.** Designers want control and incremental change, and templates with pre-extracted parasitics give reproducible updates (BAL2 §4.5.6); the 2011 routing chapter expected templates to spread (BAL2 §4.8). Inside layout-aware sizing a template produces a full layout in "a few seconds" with symmetries coded in, fast enough for the sizing loop where optimisation-based layout is too slow (BAL2 §6.3.2). Retargeting by compaction is accepted by designers more readily than optimisation that re-invents the layout (BAL2 §5.1–5.2). BAG and LAYGO/LAYGO2 have silicon demonstrations, including a 15 Gb/s SerDes front end at 1.96 pJ/bit in 16 nm (SURV-02; SURV §V digest).

### 3.3 Rule-based and knowledge-based systems

- **ALSYN** combined min-cut placement with Stockmeyer floorplan sizing under a rule set; it is only as good as its rule set (LAMP §1.5.2.3).
- **AI design plans** ordered routes by sensitivity, distance, knowledge-base constraints and straightness, kept option paths, and went from a virtual grid to a stick diagram; the digest calls this the precursor of templates (BAL2 §4.5.1).
- **Expert-base and schematic-driven constructive placement** is fast but order-dependent (BAL1 §1.1.3).
- **Knowledge-based sizing** is fast but inaccurate and hard to port, with worst-case deviations of "several hundred percent" (BAL2 §6.2.1; BAL2 §6.3.1).
- **Fate.** The rule set and the order dependence are the limits; the knowledge survives as building-block libraries (the sizing-rules library used for block recognition) and as the derivation rules of constraint-driven design (BAL1-47; BAL1-50; BAL2-49).

### 3.4 Heuristic optimisation systems

- **ILAC** (MOSAIC; simulated annealing for placement and routing) was an early optimisation-based generator (SURV §II). It classified nets as sensitive, noisy, non-critical and power, routed power first and noisy last so that power and non-critical nets shield critical from noisy nets, and used a global maze router plus a scan-line channel router with stretchable spacing (BAL2 §4.5.1; LAMP-48). The heuristic cannot guarantee specs, which motivated sensitivities (LAMP-48). Year not stated in the digests.
- **SALIM** derived channels from the slicing tree, took electrical constraints mostly as user rules, and routed symbolic zero-width tracks first (BAL2 §4.5.1).
- **SLAM** assigned prioritised per-node distance constraints and prioritised pairwise spacing (BAL2-13).
- **MIGHTY/OPASYN** ordered nets by function and used rip-up and reroute (BAL2 §4.5.1).
- **KOAN/ANAGRAM II** (Cohn 91/94): absolute-coordinate annealing with dynamic diffusion merging, over-the-device routing, crosstalk avoidance and symmetric wiring; ANAGRAM II routes arbitrary rectangles with arbitrary width and guaranteed symmetric differential wiring, with routability priced as the cost of ripping up neighbours; KOAN/ANAGRAM II also anneals devices and wires together (LAMP §1.5.2.4; BAL2 §4.5.2; BAL2 §4.5.4; LAMP bibliography digest). Dynamic merging "burdens the placer" (LAMP §3.3.2).
- **GELSA** ran annealing on slicing with in-loop global routing but was restricted to slicing (LAMP §2.3.2.3). **PUPPY-A** is listed among the SA/GA engines (BAL1 §1.1.3).
- **Fate.** The absolute-representation failure modes (BAL1-05) and the lack of a link between cost terms and specs (BAL2 §4.5.2) pushed the field toward performance-driven methods and topological representations.

### 3.5 Performance-driven systems

- **Berkeley constraint generation** (Choudhury 90b/93, Charbon 93, Malavasi): sensitivities plus a quadratic programme give parasitic bounds that maximise layout flexibility; applied to channel routing, area routing, placement and compaction; iterations are needed when a bound fails (LAMP §2.3.2.1; LAMP-05). **PARCAR** assigns flexibility values refined as the layout evolves (BAL2-17). **ROAD** and **ANAGRAM III** route to parasitic bounds with sensitivity-derived weights, escalating weights and regenerating placement when all weights saturate (BAL2 §4.5.3; BAL2-18; BAL2-19; BAL2-20). **Basaran's** bounded router prunes partial paths exceeding their bound (Basaran 93) (LAMP §2.3.2.2).
- **LAYLA** (Lampaert, Gielen, Sansen; book 1999): removed the constraint-generation step and priced placement and routing directly by sensitivity-weighted degradation, handling interconnect, mismatch and thermal effects together, then spent leftover margin on yield and testability (LAMP §1.5.3; LAMP-04; LAMP-43). It was 115 kLOC of C++, interfaced to Falcon, DFII and ROSE, used on 20+ chips with a 5–7× productivity gain, and showed a 10–15% area penalty against manual layout (LAMP §6; LAMP-50; LAMP-47).
- **Fate.** The direct method and sensitivity budgets persist in current tools as capacitance-sensitivity-weighted P&R (SURV-20), while the fixed-bound conversion is criticised as over-constraining (LAMP-04). The modern survey concludes that proxy constraints still do not guarantee post-layout performance and that "a general way to account for post-layout performance remains an open question" (SURV-01; SURV §I).

### 3.6 Topological representations and deterministic placement (2011 survey)

Symmetric-feasible sequence pairs (Balasa and Lampaert), ASF-B*-trees with symmetry islands (Lin and Chang) and Plantage's deterministic enumeration (Strasser, Eick, Graeb, Schlichtmann) are the three approaches surveyed, with the shared principle that symmetry must be a property of the code (BAL1-04; BAL1 preface digest). Plantage is the first placer to handle proximity, symmetry, common centroid, minimum distance and variants deterministically with no tuning parameter (BAL1 §3.7).

### 3.7 Retargeting, layout-aware sizing and constraint-driven methodology (2011 survey)

Analog IP migration was manual and redesigned per node; compaction-based retargeting preserves the source layout's knowledge and reuses the industrial DRC/LVS engine for rules (BAL2 §5.1; BAL2 §5.3). Layout-aware sizing brought templates back as the fast layout engine inside simulation-based sizing, after showing that a schematic-sized opamp failed its phase-margin spec after extraction (BAL2-42). Constraint-driven methodology formalises management, derivation, transformation, sensitivity analysis and verification of constraints using constraint logic programming with Constraint Handling Rules, a Constraint Engineering System as middleware, and meta-verification over existing tools (BAL2 §7.4–7.5; BAL2-53).

### 3.8 Current families (2024 survey)

Three generator families: template-based (BAG, LAYGO/LAYGO2), digital-P&R-based, and analog-P&R-based (MAGICAL, ALIGN), the last fully automated and treating physical design as optimisation (SURV-02). MAGICAL (ICCAD 2019; TCAS-II 2023) is silicon-verified with a 40-nm 1 GS/s ΔΣ ADC and 1 and 80 MS/s SAR ADCs; ALIGN (DAC 2019); a temperature-sensor generator was verified on 64 instances in SkyWater 130 nm (SURV §V digest; SURV-03). Learn-from-human methods (GeniusRoute, WellGAN, a VAE for primitive cells) imitate manual layouts but are limited by small datasets and lack direct performance optimisation (SURV-21). Black-box Bayesian optimisation over net weights finds high-quality layouts but suits small circuits (SURV-22). ML performance models need about 2000 layouts per schematic and transfer only within a circuit type (SURV-37; SURV-38).

### 3.9 Summary for the decision record

| Family | Approach | Why it receded | Where it survives | Source |
|---|---|---|---|---|
| Procedural generators | Code per topology | Code volume; mapping and lost sharing; per-process maintenance | Device and matched-structure generators; BAG/LAYGO | LAMP §1.5.2.1, §3.3.1; SURV-02 |
| Templates | Fixed relative placement and routing | Per circuit type and technology; 1.5–2× manual effort to author; unseen designs | Retargeting; layout-aware sizing; reproducible incremental edits | LAMP §1.5.2.2; BAL2 §4.5.6, §6.3.2; SURV-21 |
| Rule / knowledge based | Rules, expert base, order-dependent construction | Only as good as the rule set; order dependence; inaccuracy | Building-block libraries; constraint derivation rules | LAMP §1.5.2.3; BAL1 §1.1.3; BAL2-49 |
| Heuristic SA/GA (ILAC, KOAN/ANAGRAM II) | Net classes, cost terms | No link to specs; absolute-representation failure modes | Net classes as fallback; symmetric wiring and merging ideas | LAMP-48; BAL1-05; BAL2 §4.5.2 |
| Constraint generation (Berkeley, ROAD) | Specs → parasitic bounds | Over-constrains; iterations | Per-net caps where a number is structurally needed | LAMP-04; LAMP-05; BAL2-19 |
| Direct performance-driven (LAYLA) | Sensitivities price every move | 10–15% area penalty; simulator sensitivity support | Sensitivity-weighted P&R | LAMP-47; SURV-20 |
| Topological symmetric-feasible | Symmetry in the code | Still SA over codes | ASF-B*/HB* complexity lead | BAL1-36 |
| Deterministic enumeration (Plantage) | Bounded enumeration + LP | Group size bound; MIP speed | Pareto front output | BAL1-60; BAL1-63 |
| Analog P&R + ML (MAGICAL, ALIGN) | Optimisation, surrogates | Post-layout performance unsolved; dataset cost | Current state of the art | SURV-01; SURV-37 |

---

## 4. Routing techniques for analog

### 4.1 Models and basic algorithms

Analog routing uses the digital and PCB algorithms, tailored: fewer paths but more constraints per path, and performance depends critically on routing parasitics (BAL2 §4.1–4.2). Routing quality is bounded by earlier steps, since device merging reduces the wires needed, so routing should be integrated with device generation and placement as much as possible (BAL2 §4.1).

- **Maze (Lee)**: guaranteed shortest path if one exists, O(nm) on an n×m grid (BAL2 §4.2.1). **Line expansion**: far less time and memory, fewer bends, no shortest-path guarantee, and Hightower's variant may miss an existing solution (BAL2 §4.2.2). **Channel routing**: vertical and horizontal constraint graphs with left-edge algorithms; not preferred in analog because parasitics are poorly estimated (BAL2 §4.2.3). **Steiner**: optimal multi-terminal routing is NP-complete; the rectilinear MST is at worst 1.5× the minimum rectilinear Steiner tree; heuristics grow the tree by using the found path as the new wavefront source (BAL2 §4.2.4; BAL2-01). **A\***: f = g + h with h a lower bound; for analog, h is the resistive cost of the Manhattan path and g accumulates per-performance ΔG from capacitance and resistance increments; worst case O(n²), average much better (BAL2-02; LAMP-38).
- **Grids**: a fine uniform grid is dense but memory-hungry, a coarse one cannot represent wires comparable to the pitch; analog needs variable width, and "route on uniform grid then compact" is called unsatisfactory (BAL2 §4.3.1.1). Nonuniform grids extend component boundaries, are finer in congested regions, and fold minimum spacing in by inflating each component by its layer's minimum space except the terminals being routed (BAL2-03). ROAD refines its 3-D nonuniform grid locally around each new wire (BAL2-03). **Tiles** (corner stitching) suit line expansion and widely differing block sizes but may need escape points (BAL2 §4.3.1.2). LAYLA's gridless maze router uses two corner-stitched tile planes per layer, DRC-checks every expansion and keeps partial paths in a Fibonacci heap (LAMP §5.6). **Topological** representations (multi-SP, channelled BSG) express block and wire vicinity only partially and are poor for parasitic extraction (BAL2 §4.3.1.3).
- **Connectivity model**: circuit → nets → subnets (wires with similar current density) → groups (physically connected components), to serve rip-up and reroute and nearest-connectable queries (BAL2-04). **Rule representation**: a two-level hash keyed by layer pair returns electrical and design rules in O(1) (BAL2-05).
- **Area vs global/detailed**: for analog cells with 10–20 nets, single-phase area routing is preferred because global/detailed decomposition loses the global view needed for symmetry, matching and crosstalk; a reserved-layer model restricts the solution space (LAMP-36). ALADIN runs global routing inside placement with a min-Steiner estimate on a congestion-weighted channel graph (BAL2-21).

### 4.2 Symmetric and matched routing

- **Applicability.** Symmetric routing applies only when the terminals are symmetric about an axis (BAL2-07).
- **Route one, mirror.** Early tools ignored non-symmetric obstacles. ANAGRAM II grows both halves simultaneously so an obstacle on either side blocks both; ROAD mirrors non-symmetric obstacles onto the other side, routes one half, and mirrors the result (BAL2-07). LAYLA searches one net, DRC-checks each candidate cell on both sides of the axis, accepts only cells legal on both sides, and generates the partner by mirroring; this guarantees symmetric routes even for partially symmetric placements (LAMP-40). **Guarantee.** Identical geometry, hence identical self-parasitics (LAMP-40). **Needs.** Mirror-symmetric terminals and an axis.
- **Crossing pairs.** When paired terminals lie on opposite sides and the paths must cross, a symmetric "connector" crossing structure matches R and C of both nets; residual coupling to other nets may still differ (BAL2-08).
- **Exact per-layer matching.** Exact routing requires equal wirelength on every layer so parasitics match precisely; LEMAR detours for parasitic mismatch; symmetric nets always get the same width (SURV-14; SURV-29).
- **Bundles and aggressors.** Corners cause length differences and each aggressor affects bundle members differently; PARSY chooses the routing area and adds extra paths and adjustment sections (BAL2-22).
- **Symmetry without a constraint.** Chen et al. maximise the degree of symmetry even when none is declared (SURV-15).
- **Priority.** Symmetric nets get very high routing priority (BAL2-12).

### 4.3 Shielding

Add space first; if impossible, insert a shield tied to a DC potential or ground; same-layer, different-layer (plate) and bulk (substrate contact) shields; ROAD builds shields as a last resort after all wires (BAL2-10). A shielding algorithm picks victims and aggressors by net classification (SURV-18). ILAC uses power and non-critical nets as shields between critical and noisy nets through its ordering (BAL2-12). **Guarantee.** Removes the specific coupling path at the cost of area and added ground capacitance (BAL2-10). **Needs.** A reference net, net classes or a coupling budget that spacing cannot meet.

### 4.4 Current-aware width and via arrays

Width per segment W_i = I/I_max,i, via count N_via = I/I_max,ij, via-array width [⌈√N_via⌉ − 1]·sep + 2·max(ovl_i, ovl_j); width belongs to the segment (LAMP-37). Current-driven routing for EM obtains currents by simulation (user patterns or Monte Carlo), fixes a terminal-to-terminal sequence so KCL sets per-segment currents before detailed routing, builds Steiner points greedily from three terminals summing currents, and verifies current density afterwards; an area-first terminal tree joins the minimum-current terminal to its nearest neighbour and sets width per pair, followed by annealing (BAL2 §4.7). Reliability-driven power routing and EM/IR-aware routing of multiport terminals are standard constraints (SURV-17). Contact and via arrays are kept as one abstract cut until final geometry (BAL2-35). Wire resistance rises with temperature; near a heat source one chooses between moving the wire and widening it, and widening dominates at small width (BAL2-52). **Guarantee.** EM-correct width on every segment with no over-widening (LAMP-37). **Needs.** Branch currents from the operating point and layer current limits.

### 4.5 Net splitting

Current densities in one net differ by an order of magnitude; a low-current branch tapped from a high-current segment sees a time-varying ground shift V_drop = R_wire·I; fix by routing the branch separately back to the source point or widening the shared segment. ROAD splits nets by a heuristic that estimates low-current parts (BAL2-06). The subnet model carries this split (BAL2-04). Pin-to-pin IR on star-shaped nets gives the verification form (BAL2-46). **Guarantee.** Sense branches do not see force-current drops. **Needs.** Terminal currents and a root (pad or strap) terminal.

### 4.6 Parasitic-aware routing

Three historical strategies: digital-like cost heuristics fed forward without performance guarantee; cost minimisation; parasitic bounds; then integrated P&R (BAL2 §4.5).

- **Cost minimisation (ANAGRAM).** Path cost = accumulated cost + material (length, layer, via) + parasitic cost to nearby nets + routability + cost-to-target; partial paths in a heap; effective for distributed terminals such as capacitor perimeters; weakness: no link to performance specs, congestion relies on aggressive rip-up (BAL2-14; BAL2 §4.5.2). ANAGRAM II prices routability as the cost of ripping up neighbours (BAL2-15).
- **Parasitic bounds (ROAD, ANAGRAM III).** A constraint generator maps specs to bounds on line R, C, L, line-to-line C and L, plus matching constraints on matched node pairs; sensitivities S_ij = ∂W_i/∂p_j become A* weights so path cost is the performance impact; weight ∝ sensitivity / maximum allowed variation; parasitics that collectively cause negligible degradation are dropped as non-critical (BAL2-16; BAL2-17; BAL2-18). Bounds are generated so that all parasitics within bounds imply all performances within bounds, so a violated performance implies at least one parasitic over its bound (BAL2 §4.5.3.2). ANAGRAM III prunes parasitically non-viable partial paths, finding the cheapest path that meets hard bounds without trading wirelength against crosstalk (BAL2-20). ROAD's loop: route, extract, estimate degradation; raise the weights of the most sensitive parasitics and reroute; when all are at maximum, regenerate the placement with a wider variation range; naive weight updates oscillate (BAL2-19).
- **Direct performance cost (LAYLA).** Segment cost is the spec-summed ΔP from R = ρ□·l/w and from ground, lateral, fringe and overlap capacitance within an influence distance, with coupling counted half from each side; the A* predictor applies the same estimate to the minimum Manhattan path and is optimal iff it is a lower bound (LAMP-39; LAMP-38). Net scheduling pre-routes every net alone, computes each net's impact factor F_k = (Σ_j ΔP_j^k/ΔP_j)/N_s, and rips up and reroutes in increasing F_k so the highest-impact nets keep their ideal paths (LAMP-42).
- **Capacitance models inside routers.** C_1D = A·C_α + S·C_β; C_2D adds l·C/d for parallel runs and is the router standard; 2.5-D and 3-D are too slow inside search (BAL2-09). LAYLA uses area ε/d·A, lateral F(d) = C₀ + C₁/d + … + C₄/d⁴ and fringe C_F0·l·(e^{−x₂/x₀} − e^{−x₁/x₀}) fitted per technology (LAMP-18).
- **Wire-load and R-aware routing.** Planar routing, routing-aware incremental placement to cut vias, prioritising R-sensitive nets and balancing layer changes against wirelength (SURV-19); capacitance sensitivity in placement and coupling reduction in routing, none of which addresses performance directly (SURV-20).
- **Candidate-based (x–y routing).** Generate candidates per two-terminal edge by via count, build a compatibility graph, and select an independent set so all nets are considered together; supports widths, layer confinement, keep-outs, parallel distance and coupled length versus width; extended to 45° (BAL2-26). Probabilistic routing estimates channel usage with resistor arrays including symmetry (BAL2 §4.5.7).
- **Hard-bound pruning.** Carrying accumulated parasitics with the search state and discarding labels over budget makes budgets hold by construction (BAL2-20; LAMP-43).

### 4.7 Crosstalk avoidance

Net classes (sensitive, noisy, non-critical, power; or neutral, noisy, sensitive) drive ordering, spacing and shielding (BAL2-11; LAMP-48). Ordering: power, sensitive, non-critical, noisy last (ILAC); more constraints first and symmetric nets very high (ROAD) (BAL2-12). SLAM gives prioritised node distances and pairwise spacing with sensitive–noisy highest (BAL2-13). ANAGRAM penalises coupling against already-routed wires and uses rip-up to remove ordering-induced violations (BAL2 §4.5.2). Crosstalk-aware area routers use a four-term cost with proximity checks at each partial-path head (BAL2 §4.5.3.2). The 2-D model prices parallel runs; inter-layer crossings add area and fringe terms from a layer-pair table (BAL2-09; BAL2-05). Symmetric pairs need per-aggressor balance, not only equal self-RC (BAL2-22). Over-the-device routing is allowed in ROAD with sensitive modules excluded, and several tools forbid routing over active regions outright (BAL2 §4.5.3.2; SURV-16).

### 4.8 Substrate-aware routing and isolation

Bulk shields are rows of substrate contacts (BAL2-10). Separation rules depend on substrate type: beyond 4× the epi thickness no gain on low-impedance substrates, almost linear gain on high-impedance ones; guard rings must be close and biased from dedicated pins (LAMP-16). Mondriaan's arrays route ground, bias and supply by abutment with cells flipped to share lines (BAL2-28). Substrate noise was studied in LAYLA but not implemented (LAMP §1.3.3).

### 4.9 Power routing

ILAC's global router handles planarity of power nets and routes power first (BAL2 §4.5.1; BAL2-12). Reliability-driven P/G routing and EM/IR-aware routing of multiport terminals (SURV-17). Net splitting keeps sense branches off force paths (BAL2-06). IR constraints are pin-pair on star nets with the current known, and the budget splits into placement (distance) and routing (length, layer, width) shares (BAL2-46; BAL2-50). Wire resistance is temperature-dependent (BAL2-52). In generators, wide source straps go on metal2 so gate leads stay short and gate resistance low (LAMP-22).

### 4.10 Performance-driven wire sizing (current state of the art)

With minimum-width wires the design is wire-resistance-constrained: widening the diff-pair source nets of a 7-nm OTA from 1× to 3× raised UGF from 688.5 to 972.0 MHz and gain from 23 to 25 dB because series R degenerates the effective transconductance (SURV-27). Widths are discrete because of multiple patterning, sizing runs after global routing and before detailed routing, symmetric nets get equal width, and analog circuits are rarely congested (SURV-29). Width is realised as parallel routes; 99% of solutions were routable and an unroutable net reverts to its original width (SURV-31). Optimised widths spread about uniformly over 1×, 2× and 3×, so uniform fattening is wrong: 3× uniform scored below 1× on one OTA (SURV-32). Designer annotation into non-sensitive, R-sensitive and RC-sensitive nets was beaten by the automatic method because the RC trade-off is design-dependent (SURV-28). The optimisation relaxes to continuous widths, rounds by partial enumeration, and simulates the top 10 (SURV-33), with a surrogate-guided Bayesian optimisation giving the same quality as 110 simulations at 1/8.7 of the runtime (SURV-35). Wire sizing fixed offset (17.8 mV → 1.8 mV on a comparator) and gain, not only speed (SURV-39).

### 4.11 Templates, arrays, RF and yield in routing

- **Template routing**: routing only stretches or compacts the template; LAYGEN scales paths, moves endpoints to new pins and optimises (BAL2 §4.5.6; BAL2-25). LDS encodes wire paths with 3 or 4 wires, the latter letting the orientation flip, solved by LP (BAL2-24).
- **Arrays (Mondriaan)**: vertical wires per column, IO pins pick free vertical wires, horizontal wire count follows, bus and tree generators give equal R/C to every unit (BAL2-28). FPAA routing has fixed switches and a performance bound as a maximum switch and crossing count (BAL2 §4.6.2).
- **RF (CORAL)**: coupling modelled as Z0 and loss degradation with models fitted to field-solver data; phase 1 constraint-based A*, phase 2 simultaneous expansion of all nets to enforce fixed interconnect lengths without new violations (BAL2-27). RF interconnects are devices and need self and mutual inductance (BAL2 §4.8; LAMP-17).
- **Yield and testability**: after performance-driven routing with margin left, rip up and reroute to minimise expected bridging faults from dielectric pinholes (critical area = inter-level overlap) and photolithographic defects (same-level parallel conductors against a 1/x³ defect-size distribution), dropping partial paths that exceed the remaining margin (BAL2-29; LAMP-43; LAMP-44). Testability weights come from the Mahalanobis distance of each short's supply-current signature (LAMP-45). LAYLA reduced the test error rate from 0.11% to 0.01% on one opamp while keeping specs (LAMP-46).

### 4.12 Constructive detailed routing

ALADIN orders detailed wiring from symmetric and central modules, packs intra-module wiring around the module boundary with a ring router, compacts the module toward the reference, and maze-routes inter-module connections in a small scope, so no routing-area estimate is needed (BAL2-23). Multi-terminal nets grow Prim-like or Kruskal-like with similar quality (LAMP-41).

### 4.13 Summary

| Technique | Kind | Guarantees | Needs | Source |
|---|---|---|---|---|
| Route one, mirror with two-sided DRC | Constructive given a path search | Identical pair geometry and self-parasitics | Mirror-symmetric terminals, axis | LAMP-40; BAL2-07 |
| Crossover connector | Constructive | Equal R and C through the crossing | Swapped terminals across the axis | BAL2-08 |
| Equal width, per-layer equality, detours | Constructive check and repair | Matched per-layer length and area | Pair, layer stack | SURV-14; SURV-29 |
| Shield insertion | Constructive | Removes a coupling path | Reference net, failing budget | BAL2-10 |
| Current-driven width and vias | Constructive | EM-correct segments | Branch currents, limits | LAMP-37; BAL2 §4.7 |
| Net splitting | Constructive | Sense branch free of force drop | Terminal currents, root | BAL2-06 |
| Net-class ordering and spacing | Constructive ordering | Critical nets take privileged tracks | Net classes | BAL2-11..13 |
| Impact-factor ordering | Constructive ordering | Highest-impact nets keep ideal paths | Sensitivities, pre-route | LAMP-42 |
| Obstacle inflation, nonuniform grid | Constructive model | DRC-clean spacing by construction | Layer rules | BAL2-03 |
| Steiner growth from tree | Constructive tree building | Near-minimal trees | Pins | BAL2-01 |
| Ring router + compaction | Constructive | No routing-area estimate needed | Module order | BAL2-23 |
| Candidate routes + independent set | Enumerative | Joint selection, all nets together | Candidates, conflicts | BAL2-26 |
| Template adaptation | Constructive | Topology kept | Prior routes | BAL2-25 |
| Exact-length refinement | Constructive refinement | Fixed lengths without new violations | Length targets | BAL2-27 |
| Array column routing | Constructive | Equal R/C per unit | Array structure | BAL2-28 |
| Parasitic-weighted A* / cost search | Search | Spec-priced paths | Sensitivities | BAL2-18; LAMP-39 |
| Hard-bound pruning | Search modifier | Budgets by construction | Budgets, per-label parasitics | BAL2-20; LAMP-43 |
| Weight escalation loop | Search (outer) | Convergence or placement regeneration | Extraction in loop | BAL2-19 |
| Wire sizing | Search (surrogate) | Spec-driven widths | Sensitivities or model | SURV-33; SURV-35 |
| Yield rerouting | Search | Fewer bridging faults within margin | Defect data | BAL2-29; LAMP-44 |

---

## 5. Performance-driven methodology

### 5.1 From specification to layout margin

A specification is an interval P ∈ [P_min, P_max]. P depends on design parameters, process parameters and layout parasitics; process spread gives [P_proc^min, P_proc^max], and the layout may spend ΔP_lay ∈ [P_min − P_proc^min, P_max − P_proc^max]. The first bound is ≤ 0 (a floor spec), the second ≥ 0 (a ceiling spec), and either may be infinite. The worst process corner, not the nominal value, is the reference (LAMP-01). Margins come from the specs versus the sized values including statistical spread, e.g. PM ≥ 60° (LAMP §4.3). A direct tool either yields a correct layout or flags the specs as infeasible, and a spec whose process spread or mismatch area term alone exceeds the margin is flagged before placement (LAMP-49). Where no margin exists at the schematic point the tool stops; the modern survey states the goal as approaching schematic performance, with specs taken as the schematic's own values (SURV-03).

### 5.2 Sensitivities

Degradation is modelled linearly: ΔP_j,lay = Σ_i S_xi^Pj·x_i with S = ∂P_j/∂x_i, valid because layout keeps each performance near its nominal value (LAMP-02). The parasitic vector covers ground C per net, coupling C per net pair, series R per segment, ΔV_T and Δβ per matched pair, distance per matched pair, ΔT per matched pair, and junction area and perimeter (LAMP-02). The routing survey defines S_ij = ∂W_i/∂p_j at p_j = 0 and adds matching constraints on matched node pairs with worst-case sensitivities including mismatch (BAL2-16).

Three ways to compute sensitivities: perturbation (one solve per parameter, unstable for tiny steps, and in transient a difference of two close numbers), the direct method (reuse the Newton LU, one back-substitution per parameter), and the adjoint method (two solves per output function). LAYLA used perturbation because HSPICE offered only DC sensitivities and the SPICE3 direct method was not robust (LAMP-03). The routing survey agrees that perturbation is slow and error-prone and that direct or adjoint methods are preferred (BAL2-16). Thermal-offset sensitivities were obtained by simulating with different device temperatures (LAMP-14). Constraint sensitivity analysis adds the relative distance to each bound, d_l = exp(x_l − x) − 1 and d_u = exp(x − x_u) − 1, to prioritise sensitive or violating parameters and to split weakly coupled groups into independent subproblems; it scales to a few thousand variables (BAL2-51).

### 5.3 Turning sensitivities into constraints and budgets

Two schools appear, and the sources argue between them.

**Constraint generation (Berkeley, PARCAR, ROAD).** Linearise: Σ_j S⁺_ij·p_j ≤ ΔW⁺_i,max for ceiling-limited performances and Σ_j S⁻_ij·p_j ≤ ΔW⁻_i,max for floor-limited ones, with S⁺ = max(S, 0) and S⁻ = max(−S, 0), so no parasitic earns credit for helping (BAL2-16). Infinitely many bound sets satisfy these; simplify by thresholding or independent-component analysis, or use heuristics: sensitivity graphs, or PARCAR's flexibility values, which come from each parasitic's estimated minimum and maximum, are refined as the layout evolves, and feed a quadratic programme so less flexible parasitics get more effort (BAL2-17). The Berkeley flexibility function is f_j = a_j + b_j·x + c_j·x² with c_j = −1/(x_max − x_min)², zero at x_min and maximal at x_max, maximised subject to Σ S_j·x_j,bound ≤ ΔP_lay per spec; how x_min and x_max are estimated is not given (LAMP-05). Parasitics whose maximum estimate times sensitivity is collectively negligible are dropped; the rest are critical (BAL2-17). Router weights are then ∝ sensitivity / maximum allowed variation (BAL2-18). Bounds are built so that all parasitics within bounds imply all performances within spec (BAL2 §4.5.3.2). Lampaert's critique: one fixed set of bounds over-constrains the tools, rejects valid layouts and forces re-iteration, and the flexibility criterion cannot be quantified because it ignores terminal area, distance, obstacles and previously routed nets (LAMP §2.3.3; LAMP-04).

**Direct method (LAYLA).** Price every intermediate placement or route by ΔP computed from its geometry through the sensitivities, and penalise only the part outside the margin: C_perf,i = ΔP^min − ΔP_i when below the floor margin, ΔP_i − ΔP^max when above the ceiling, else 0, summed over specs (LAMP-06; LAMP-04). The placement cost is C = α·area + β·|AR − AR_desired| + γ·overlap + δ·C_perf, with the performance and aspect-ratio weights decreasing linearly and the overlap weight increasing after each inner loop, so configuration is settled first and legality last (LAMP-29). Mismatch becomes one shared budget over all matched pairs, ΔP_j = Σ_k (|S_ΔVT0,k|·3σ_k(V_T0) + |S_Δβ,k|·3σ_k(β)), split into an area term fixed after sizing and a distance term Σ_k S_Dk·D_k that the placer trades (LAMP-09). Thermal becomes ΔP_j = Σ_k S_ΔTk·ΔT_k over a superposed profile (LAMP-13). Interconnect enters as ΔP_j = Σ_k [S_Ck·C_k + Σ_i S_Rk,i·R_k,i + Σ_{i≠k} ½·S_Cki·C_ki] (LAMP-08). Evidence: on a comparator, performance-driven placement gave offset 3.7 mV and delay 2.8 ns against specs of 5 mV and 5 ns, while the same tool with the mechanism off gave 6.9 mV and 5.4 ns, at 106 s versus 93 s CPU (LAMP-33).

**Constraint transformation (constraint-driven design).** Higher-level constraints become lower-level ones by transformation rules; several may apply and the choice removes global freedom; an inverse transformation must exist for verification. Example: a pad-to-terminal IR budget with known current becomes placement constraints (distance) plus routing constraints (length, layer, width), and with placement fixed only routing constraints remain (BAL2-50). Derivation uses fixed rules, logic deduction over device classes, or expert knowledge (BAL2-49). Over-constraint must be detected because an earlier over-constraint can block later optimisation (BAL2-48).

### 5.4 Estimating layout parasitics before routing

Extraction is too slow for inner loops and needs finished layouts, so fast geometric models are applied to partial layouts (LAMP §2.4.3).

- **Net topology.** Estimator options are semi-perimeter O(n), MST O(n²), centre of mass O(n²), Steiner (NP-complete), source-to-sink O(n) and complete graph O(n²); LAYLA uses the MST with edge-to-edge Manhattan distance between terminal rectangles because its geometry is needed for coupling and it is "the most accurate for analog layouts" (LAMP-07). ALADIN found HPWL, centre-of-mass, complete-graph and MST estimators unreliable and used a min-Steiner estimate on a channel graph with congestion (BAL2-21). Wire-based performance modelling later used star-model wirelength and found it inaccurate without real layout data (SURV-23).
- **Placement-time extraction.** Ground C = segment lengths × width × layer-averaged capacitance per area, plus terminal (junction) capacitance; coupling from MST segment pairs that run in parallel (L_p·C_c(D_p)) or cross (C_ov(W_i·W_j)); series R = ρ□·L/W with W from terminal current (LAMP-08). Junction capacitance C_j = A·C_j/(1−V/φ)^m + P·C_jsw/(1−V/φ)^m_sw is reducible by folding, merging and abutment, while gate capacitance is fixed by sizing (LAMP-19). Analytical diffusion area and perimeter per finger count and style feed the same estimate (BAL2-43).
- **Capacitance models.** Area ε/d·A; lateral F(d) polynomial in 1/d up to fourth order; inter-layer fringe with exponential decay; resistance by decomposition along equipotentials (LAMP-18). Routers use C_2D = C_1D + l·C/d (BAL2-09) and a layer-pair rule table for crossings (BAL2-05).
- **Routing space.** Each device side is expanded by a static part equal to the sum of the widths of its terminals' wires (sized from current) and a dynamic part that decreases with distance to each net's pin-bounding-box centre; the per-net form was "consistently and significantly better" than one relative to the placement centre; the constants are not given (LAMP-31). Overestimated routing space is one cause of LAYLA's 10–15% area penalty (LAMP-47).
- **Thermal profile.** Precompute a unit-source profile by DCT, fracture each variant's active area into unit sources, and superpose per placement; direct evaluation of the Fourier model took about 30 s (LAMP-13).
- **Template-based estimation (layout-aware sizing).** Approach A: geometric MOS extraction plus 3-D analytical-geometric interconnect extraction every iteration; Approach B: analytical diffusion equations plus a lookup table sampled per template. Two-step sampling simulates about 100 instances to find the critical parasitics, then samples densely only the parameters driving them (BAL2 §6.5.1; BAL2-44). Approach A was fast enough for cells of a few tens of devices (BAL2-42).
- **Generator-level models.** Each generator has a simulation model with its layout parasitics, and back-annotation lets designers re-simulate within minutes (LAMP-50).

### 5.5 The feedback loops the methodologies assume

| Methodology | Loop | Source |
|---|---|---|
| Traditional | Layout → extract → verify → fix by hand; the extracted netlist gives "no clue" which parasitic caused a violation | LAMP §2.1 |
| Berkeley constraint generation | Specs → bounds (QP) → P&R; iterate when a bound fails | LAMP §2.3.2.1 |
| ROAD | Route → extract → estimate degradation → raise weights of the most sensitive parasitics → reroute; when all weights saturate, regenerate placement with wider variation ranges; naive updates oscillate | BAL2-19 |
| LAYLA | No iteration: sensitivities once, direct penalty at every move, infeasibility flagged up front; placement leaves margin for routing because some parasitics differ from the placement estimate; routing then spends leftover margin on yield and testability with spec-violating partial paths pruned | LAMP-04; LAMP-34; LAMP-43 |
| Layout-aware sizing | Sizing SA → template layout + geometric sizing → extraction → simulation → cost, every iteration; a schematic-sized design failed PM after extraction while the layout-aware one held all specs | BAL2 §6.3.3; BAL2-42 |
| Constraint-driven design | Constraints available at every stage; all tools read the representation or convert it; constraint verification complements DRC/LVS; results back-annotated to limit iterations; degrees of freedom removed gradually across overlapping steps | BAL2 §7.6.1; BAL2-55 |
| Modern P&R | Netlist + spec → placement → routing → PEX → simulation → performance; post-layout simulation is the judge and proxies do not guarantee it; labels cost about 3–4 P&R iterations and post-layout evaluation is about 10× slower than schematic | SURV-01; SURV-24 |
| Black-box BO | Layout generation plus simulation as a black box; Bayesian optimisation tunes net weights, which set net priority and so shape the layout; suits small circuits | SURV-22 |
| Surrogate-guided | Surrogate (GNN or linear) scores candidates, simulation only for the top η; same quality as 110 simulations at 1/8.7 runtime; surrogates lose 3–22% accuracy on transfer and need out-of-distribution checks | SURV-35; SURV-26 |
| Multi-objective | A weighted FOM cannot express trade-offs; Pareto archives (MOBO) were top-1 on 3–5 metrics per OTA; Plantage likewise returns a Pareto front of aspect ratios | SURV-25; BAL1-63 |

Reserved margin: after placement the remaining margins "are needed for the subsequent routing phase" (LAMP-34); on Opamp1 GBW went 228 → 226 MHz against a 225 MHz spec and PM 61 → 60.2° against 60° (LAMP-34). Routing then kept specs while cutting test error rate 0.11% → 0.01% (LAMP-46). CPU split roughly 33% placement, 38–40% routing, 27–29% yield (LAMP-46).

Attribution: the placer outputs the final degradation and "an identification of the most important contributions", per net, per coupling pair, per matched pair and per thermal pair (LAMP-35; LAMP-09; LAMP-14).

Validity limits: the linear model holds only near the operating point, so sensitivities should be re-derived when the layout's simulated metrics differ from the prediction (SURV-26); RC models hold only while inductive and conductive terms are negligible (LAMP-17); transfer of learned models across topologies is "much more difficult" (SURV-38).

---

## 6. Technique cards

Cards cover every constructive technique named in sections 2 and 4. P-cards are placement and composition, R-cards are routing. The field `guarantees` / `does not guarantee` is one field with two parts.

### P-01
- `id`: P-01
- `technique`: ASF-B*-tree symmetry-island packing (decode of one symmetry group from a representative tree)
- `realizes`: Mirror symmetry about a vertical or horizontal axis, self-symmetric devices on the axis, symmetry island (connected, abutting), non-overlap, compaction of the group.
- `applicability`: One symmetry group with p pairs and q self-symmetric modules; pair members have equal dimensions; any module sizes; island outline may be rectilinear.
- `parameters`: Axis orientation; which member of each pair is the representative; the representative B*-tree shape (a left-skewed tree yields a row).
- `guarantees` / `does not guarantee`: Guarantees symmetric feasibility for either axis (Thm 2.1), that the packing is a symmetry island (Thm 2.2), one-to-one correspondence with compacted symmetric placements so no redundant codes (Thm 2.3), packing linear in group size. Does not guarantee minimum area or wirelength for a given tree, perfect (same-orientation) symmetry (the construction mirrors), or well spacing between differing device types.
- `inputs needed`: Pairs, self-symmetric modules, module widths and heights, axis orientation.
- `verification`: Centre-form equalities per pair and self-symmetric module; island connectivity by union-find over abutment.
- `prefer over / degrade to`: Prefer over S-F sequence pair (quadratic packing, no island guarantee) and absolute SA with projection. Degrade to P-03 when a group cannot be an island, or to P-11 LP compaction with symmetry equalities.
- `source`: BAL1-29; BAL1-30; BAL1-36; BAL1-27.

### P-02
- `id`: P-02
- `technique`: HB*-tree hierarchical packing with hierarchy nodes, contour nodes and well white space
- `realizes`: Several islands plus asymmetric modules; rectilinear islands without slicing them; hierarchical proximity (clusters placed connected); well or guard-ring spacing where device types differ.
- `applicability`: Blocks with multiple symmetry groups, clusters and mixed NMOS/PMOS or ringed devices.
- `parameters`: Cluster hierarchy (one tree per cluster); device class per module; class-pair minimum distance matrix; six structural rules for contour nodes.
- `guarantees` / `does not guarantee`: Guarantees O(n) packing (Thm 2.4), islands intact, non-overlap, white space reserved exactly where classes differ, every cluster connected. Does not guarantee axis alignment across islands (needs P-04) or optimality of the topology (that is the anneal's job).
- `inputs needed`: One ASF-B*-tree per group, module dimensions and device classes, cluster membership, well spacing rules.
- `verification`: Same as P-01 plus cluster connectivity and per-class spacing checks.
- `prefer over / degrade to`: Prefer over slicing rectilinear modules into sub-blocks and over SP+LP. Degrade to P-01 per group followed by P-10 compaction of islands as rigid blocks.
- `source`: BAL1-31; BAL1-32; BAL1-35; BAL1-41; BAL1-25.

### P-03
- `id`: P-03
- `technique`: Symmetric-feasible sequence-pair decode with Balasa's sweeps (ordinate pass with restart, abscissa pass, embedding-DAG axis selection, right and left symmetry sweeps)
- `realizes`: Exact symmetry with a computed axis, minimum height, minimum group x-span relative to the topology, multiple and nested groups, without an LP solver.
- `applicability`: Groups that do not intermingle in the two sequences, or that nest with the outer group surrounding the inner; vertical axis convention.
- `parameters`: Initial code α = a1…ap c1…cs bp…b1, β = a1…ap cs…c1 bp…b1; group processing order inner before outer; iteration cap on the fixing loop.
- `guarantees` / `does not guarantee`: Guarantees termination and exact symmetry for a symmetric-feasible code, minimal height, minimal group span, O(n log log n) per sweep. Does not guarantee coverage of every symmetric placement (condition sufficient, not necessary), termination on a non-feasible code, or a symmetry island.
- `inputs needed`: Pairs, self-symmetric cells, group nesting, cell dimensions, pairwise minimum distances as edge weights.
- `verification`: Symmetry equalities; iteration count under the cap; height and width equal to the longest paths.
- `prefer over / degrade to`: Prefer over TCG (quadratic) and absolute coordinates. Degrade to P-11 when common-centroid or other equalities beyond symmetry are required.
- `source`: BAL1-06; BAL1-08; BAL1-09; BAL1-10; BAL1-11; BAL1-12; BAL1-13; BAL1-19.

### P-04
- `id`: P-04
- `technique`: Multi-group axis alignment by zero-width dummy blocks
- `realizes`: Several symmetry islands sharing one vertical (or horizontal) axis.
- `applicability`: Symmetry compounds spanning several blocks, e.g. an input pair and a folded-cascode load on one axis.
- `parameters`: Dummy width (height) per island, set to align the axes.
- `guarantees` / `does not guarantee`: Guarantees collinear axes once the dummies are sized. Does not guarantee minimal area (dummies add white space).
- `inputs needed`: Islands with their axis positions, the target common axis.
- `verification`: All island axes equal.
- `prefer over / degrade to`: Prefer over separate axes when symmetric routes must be long straight mirrors. Degrade to independent axes per block.
- `source`: BAL1-38; BAL1-51.

### P-05
- `id`: P-05
- `technique`: Folding asymmetric modules into a symmetry group as clusters
- `realizes`: Non-island placements when needed; bias devices connected to both halves sit on the axis as a self-symmetric cluster; two clusters form a pair with the larger dimensions as representative.
- `applicability`: Glue devices that must live inside a symmetric region.
- `parameters`: Cluster membership; cluster packing tree.
- `guarantees` / `does not guarantee`: Guarantees the group stays symmetric with the cluster's bounding box as a member. Does not guarantee internal symmetry of the cluster.
- `inputs needed`: Asymmetric module list, group definition.
- `verification`: Symmetry equalities on the cluster bounding box.
- `prefer over / degrade to`: Prefer over leaving glue outside the island when wirelength to both halves matters. Degrade to placing glue outside the group.
- `source`: BAL1-39; BAL1 §1.1.4 (self-symmetric bias device).

### P-06
- `id`: P-06
- `technique`: Hierarchical symmetry (group pairs and self-symmetric sub-groups as nested islands)
- `realizes`: Fully differential structures where each half contains its own symmetric sub-structure, e.g. replicated halves each containing a mirror.
- `applicability`: Nested symmetry from the netlist hierarchy or symmetry compounds.
- `parameters`: Nesting tree; per-level axis.
- `guarantees` / `does not guarantee`: Guarantees symmetry at every level simultaneously. Does not guarantee area optimality.
- `inputs needed`: Group-of-groups definition.
- `verification`: Equalities at each level.
- `prefer over / degrade to`: Prefer over flattening to one axis. Degrade to one axis per top-level block.
- `source`: BAL1-40; BAL1-12.

### P-07
- `id`: P-07
- `technique`: SMP graph and HSMPG tree constraint generation with importance order M_S ≻ M_B ≻ P_B ≻ S ≻ P_N
- `realizes`: The placement hierarchy, group types (matching, symmetry, proximity) and the priority used to resolve conflicts.
- `applicability`: Any netlist with recognisable building blocks; groups observed at 2–14 members.
- `parameters`: Building-block library; the requirement order (static was correct in all experiments; a dynamic priority is possible); whether supply nets are excluded from the per-net cliques (the source does not say).
- `guarantees` / `does not guarantee`: Guarantees a deterministic hierarchy from netlist and priorities in 0.3–1.2 s. Does not guarantee the order is right for every circuit, or that the root proximity group is small enough to enumerate.
- `inputs needed`: Netlist, library of blocks (differential pair, level shifter, simple, cascode, four-transistor and wide-swing mirrors with their matching and proximity edges), symmetry pairs.
- `verification`: Each device in exactly one group per level; group count and sizes reported.
- `prefer over / degrade to`: Prefer over catalog-only two-level blocks. Degrade to the recognised blocks as flat groups.
- `source`: BAL1-48; BAL1-49; BAL1-50; BAL1-52; BAL1-47; BAL1-61.

### P-08
- `id`: P-08
- `technique`: Symmetry compound detection by recursive symmetric-pair propagation
- `realizes`: One shared axis for all pairs reachable from a seed through the signal path; M_S edges per pair, an S clique per compound.
- `applicability`: Differential signal paths, including glue that no block pattern matches.
- `parameters`: Seed pairs (recognised differential pairs); equivalence test for propagated devices.
- `guarantees` / `does not guarantee`: Guarantees that every pair sharing an axis is in one compound. Does not guarantee detection of symmetry that is not structural.
- `inputs needed`: Netlist connectivity, device types and sizes, pin roles.
- `verification`: Compound size and axis count reported; pairs have equal (type, W, L).
- `prefer over / degrade to`: Prefer over one axis per recognised block. Degrade to block-local pairs.
- `source`: BAL1-51; BAL1-47.

### P-09
- `id`: P-09
- `technique`: Constraint derivation from HSMPG groups (same variant plus alignment, common centroid for split devices, symmetry-pair y-equalities and symmetry-group x-centre alignment)
- `realizes`: The equality rows that a decoder or LP enforces.
- `applicability`: Every matching and symmetry group.
- `parameters`: Whether devices are split into sub-devices (selects common centroid over alignment).
- `guarantees` / `does not guarantee`: Guarantees exact equalities once solved. Does not guarantee proximity (left implicit in bottom-up construction).
- `inputs needed`: HSMPG tree, module variants.
- `verification`: Equalities hold to the grid.
- `prefer over / degrade to`: Prefer over budgets with tolerance for matched groups. Degrade to tolerance budgets when the solver is absent.
- `source`: BAL1-53; BAL1-44; BAL1-43.

### P-10
- `id`: P-10
- `technique`: One-dimensional constraint-graph compaction by longest path, alternating axes and regenerating constraints
- `realizes`: Legal, minimal coordinates under minimum-distance constraints; dead-space recovery after resizing.
- `applicability`: Any placement with pairwise separation constraints; cells or polygon edges.
- `parameters`: Distance per pair d_ij; closeness or wirelength objective; stop criterion for alternating passes (area gain threshold).
- `guarantees` / `does not guarantee`: Guarantees O(N + E) solve on a DAG and minimal coordinates. Does not guarantee preservation of wire lengths (long-wire problem) unless an objective is added, nor optimal 2-D compaction (NP-complete), nor release of edge-order locks without regeneration.
- `inputs needed`: Edge coordinates, design rules (conditional rules via the sign-off DRC engine), fixed cells, intentional non-minimum wells marked as lower bounds.
- `verification`: All d_ij satisfied; area per pass; wire lengths before and after.
- `prefer over / degrade to`: Prefer over pairwise push-apart legalisation. Degrade to P-11 when equalities are present.
- `source`: BAL2-30; BAL2-31; BAL2-34; BAL2-36; BAL2-37.

### P-11
- `id`: P-11
- `technique`: LP or graph-based simplex compaction with multivariable constraints (symmetry, common centroid, exact dimensions, hierarchy)
- `realizes`: Exact symmetry x_b − x_a = x_d − x_c, exact device dimensions x_j − x_i = K, hierarchical edge constraints, minimum area, optional wirelength objective.
- `applicability`: Retargeting and legalisation of blocks up to a few thousand variables (Miller opamp 521 variables, folded cascode 2,431; a 500-device oscillator 14,495 edges).
- `parameters`: Objective (area, wirelength, closeness to source); big-M for artificial variables.
- `guarantees` / `does not guarantee`: Guarantees exact equalities and minimal objective; starts from the longest-path basic solution. Does not guarantee speed for equality-dominated problems (tends to O(v³)).
- `inputs needed`: Constraint graph from P-10 or P-12, equality rows, fixed coordinates.
- `verification`: Residual of every equality is zero; objective reported.
- `prefer over / degrade to`: Prefer over Okuda's split into a reduced LP plus distance constraints. Degrade to P-03 sweeps when only symmetry is needed.
- `source`: BAL2-32; BAL2-33; BAL1-54; BAL1-55; BAL2 §5.7.1.

### P-12
- `id`: P-12
- `technique`: Shadow-based horizontal constraint graph with pairwise minimum distances
- `realizes`: Only the edges needed for minimum distance, including diagonal pairs that are too close vertically.
- `applicability`: Any B*-tree or placement being compacted with well and guard-ring distances.
- `parameters`: Core shadow [y, y + h]; partial shadow extended by d_max(m) above and below.
- `guarantees` / `does not guarantee`: Guarantees that every pair needing a horizontal separation has an edge. Does not guarantee a sparse graph when d_max is large.
- `inputs needed`: Module coordinates in y, heights, d_min matrix.
- `verification`: No pair closer than d_min in both axes after the solve.
- `prefer over / degrade to`: Prefer over y-overlap-only graphs, which miss diagonal violations. Degrade to full pairwise edges.
- `source`: BAL1-55; BAL1-45.

### P-13
- `id`: P-13
- `technique`: Piecewise-linear deep-trench distance by MIP with one binary per edge
- `realizes`: Either a shared stretched trench (gap ≤ s_max) or separate trenches (gap ≥ d_DTI), never the forbidden band; per-module stretch budget s_l + s_r ≤ s_max.
- `applicability`: Processes with deep trench isolation.
- `parameters`: s_max, d_DTI, big-M β > Σ widths + worst-case distances × (#symmetry constraints + 1), λ < w_min/(N·d_DTI).
- `guarantees` / `does not guarantee`: Guarantees a legal gap for every DTI pair. Does not guarantee speed: 30 modules took about 15 minutes.
- `inputs needed`: Module list with trench stretch limits.
- `verification`: Gap in one of the two allowed ranges; stretch sum per module.
- `prefer over / degrade to`: Prefer over ignoring the band. Degrade to a branch variable per pair inside annealing.
- `source`: BAL1-46.

### P-14
- `id`: P-14
- `technique`: Shape functions and Stockmeyer floorplan sizing over a slicing template
- `realizes`: Minimum area within an aspect-ratio window and W/H limits, with leaf parameters (fingers, unit capacitor side, style) chosen jointly.
- `applicability`: Slicing-tree templates; any block with reshapeable leaves.
- `parameters`: AR window [AR − ΔAR, AR + ΔAR]; max/min W, H; slice separations; area-loss objective.
- `guarantees` / `does not guarantee`: Guarantees the Pareto-optimal (w, h) set per node and the minimum-area feasible choice; complexity O(d·l_T). Does not guarantee non-slicing density; symmetries live in the template, not in the sizing.
- `inputs needed`: Slicing tree, per-leaf shape lists from generators, separations.
- `verification`: Area, area loss (0.57–3.06% reported), AR in window.
- `prefer over / degrade to`: Prefer over fixed finger counts. Degrade to a single default variant per leaf.
- `source`: BAL2-39; BAL2-40; BAL1-56.

### P-15
- `id`: P-15
- `technique`: Enhanced shape functions with hierarchically bounded B*-tree enumeration (Plantage)
- `realizes`: A Pareto front of complete placements over aspect ratio satisfying proximity, symmetry, common centroid, minimum distance and variant constraints, deterministically.
- `applicability`: Basic groups small enough to enumerate (labelled B*-trees number 336 for 4 modules, 57,657,600 for 8); circuits of 9–42 devices demonstrated.
- `parameters`: None for tuning; group-size cap; parallelism over basic groups.
- `guarantees` / `does not guarantee`: Guarantees every front member satisfies all constraints exactly and that horizontal and vertical additions preserve feasibility; no tuning parameter. Does not guarantee global optimality beyond the hierarchy bound, wirelength optimality, or runtime (1–691 s; wells add 72–76%).
- `inputs needed`: HSMPG tree (P-07), module variants with dimensions, d_min matrix, equalities (P-09).
- `verification`: Area usage (110–129% reported), constraint equalities, front size.
- `prefer over / degrade to`: Prefer over annealing for small groups. Degrade to annealing (P-01/P-02 decode) for groups above the cap.
- `source`: BAL1-57; BAL1-58; BAL1-59; BAL1-60; BAL1-61; BAL1-62; BAL1-63; BAL1-20.

### P-16
- `id`: P-16
- `technique`: Common centroid as an exact centroid equality or as a unit-array pattern
- `realizes`: Equal area-weighted centroids of two groups; mismatch gradients cancelled.
- `applicability`: Current mirrors and differential pairs built from unit devices; ratioed groups unitised to integer units.
- `parameters`: Unit size and count; pattern.
- `guarantees` / `does not guarantee`: Guarantees first-order gradient cancellation when the equality holds exactly. Does not guarantee stress immunity (finger structures reached 5× the model under package stress; quad and waffle showed none).
- `inputs needed`: Matched groups and ratios.
- `verification`: Centroid equality to the grid.
- `prefer over / degrade to`: Prefer over alignment when devices are split into sub-devices. Degrade to alignment plus adjacency.
- `source`: BAL1-44; BAL1-53; BAL1-42; LAMP-11; LAMP-26; SURV-05.

### P-17
- `id`: P-17
- `technique`: Pairwise well-class minimum distance (class matrix) instead of a global clearance
- `realizes`: Abutment inside a well, well or guard-ring spacing only where classes differ, 2·d_well between wells.
- `applicability`: All placements; the n/p distance dominated Plantage's area overhead.
- `parameters`: Class per cell (well net, substrate, trench tub, ringed); d_min per class pair.
- `guarantees` / `does not guarantee`: Guarantees exact spacing per pair. Does not guarantee minimal total area if classes are assigned coarsely.
- `inputs needed`: Deck spacing rules, device classes.
- `verification`: Per-pair Manhattan distance ≥ d_min.
- `prefer over / degrade to`: Prefer over one maximum spacing everywhere. Degrade to the global maximum.
- `source`: BAL1-45; BAL1-32; BAL1-42.

### P-18
- `id`: P-18
- `technique`: Transistor stacking by Euler trail on the diffusion graph
- `realizes`: Minimum-gap transistor chains with shared diffusion, respecting symmetry and matching.
- `applicability`: Same type and bulk subgraphs with equal channel width after folding.
- `parameters`: Edge cost favouring sensitive nets; supervertex for odd-degree vertices.
- `guarantees` / `does not guarantee`: Guarantees linear time and the minimum-cost stack set for a class of circuits. Does not guarantee interdigitated cascode or common-centroid structures (need P-19).
- `inputs needed`: Netlist with device types, bulks, widths; sensitivities optional.
- `verification`: Each added edge is a diffusion gap; count of gaps.
- `prefer over / degrade to`: Prefer over path enumeration plus clique search (exponential). Degrade to grouped-member chains only.
- `source`: LAMP-21; LAMP §3.4.

### P-19
- `id`: P-19
- `technique`: Dedicated generators for interdigitated cascodes and common-centroid pairs (contactless shared node)
- `realizes`: Minimum-capacitance internal node (gates at minimum poly spacing over diffusion), structures merging cannot produce.
- `applicability`: Series devices whose shared node has no external connection; matched pairs.
- `parameters`: Finger count, interdigitation pattern.
- `guarantees` / `does not guarantee`: Guarantees the contactless node and the pattern. Does not guarantee optimality against other sharing options.
- `inputs needed`: Recognised cascode or pair.
- `verification`: No contact on the shared node; pattern check.
- `prefer over / degrade to`: Prefer over dynamic merging for these structures. Degrade to P-18 chains with a contacted node.
- `source`: LAMP-23; LAMP §3.3.4.

### P-20
- `id`: P-20
- `technique`: MOS module generator with fingers, slices, current-sized straps and source-in-metal2
- `realizes`: Target aspect ratio under the bulk-contact distance rule; EM-correct internal S/D straps and via counts (W₂ = I/I_max,2; W₁ = I/(n_s·I_max,1); N_via = I/(n_s·I_max,via12)); short gate leads.
- `applicability`: Every MOS device; very wide devices sliced with bulk-contact rows.
- `parameters`: W, L, fingers, slices, current, aspect ratio, type, technology file, mode.
- `guarantees` / `does not guarantee`: Guarantees tap distance and current rules by construction. Does not guarantee the variant is optimal for parasitics (placement chooses).
- `inputs needed`: Device sizes, terminal currents, deck current densities and tap distance.
- `verification`: Max distance to tap; strap widths versus current.
- `prefer over / degrade to`: Prefer over routing-time fattening inside devices. Degrade to minimum straps plus a poly-resistance check.
- `source`: LAMP-22; LAMP §3.7.1.

### P-21
- `id`: P-21
- `technique`: Diffusion abutment legality and merging (same net, same layer, same bulk)
- `realizes`: Lower junction capacitance on the shared node, less area and fewer wires.
- `applicability`: Series devices share one node, parallel devices share two.
- `parameters`: Abutment permission table; sensitivity credit for the merged net.
- `guarantees` / `does not guarantee`: Guarantees electrical legality of the overlap. Does not guarantee that all beneficial merges are found when merging is a cost in annealing.
- `inputs needed`: Nets, layers, bulk potentials per terminal; junction area and perimeter per variant.
- `verification`: Overlap only between legal terminal pairs; illegal overlap zero.
- `prefer over / degrade to`: Prefer over separate devices on sensitive nets. Degrade to minimum spacing.
- `source`: LAMP-20; LAMP-19; BAL1-64; LAMP §3.2.

### P-22
- `id`: P-22
- `technique`: Generator interface mode with precomputed variant and orientation library over symbolic technology rules
- `realizes`: Variant and orientation choice during placement as a pointer switch; portability across processes.
- `applicability`: All module generators.
- `parameters`: Symbolic geometric parameters (width, spacing, enclosure), electrical parameters (sheet R, capacitance, current density), generic-to-process layer map with a void layer.
- `guarantees` / `does not guarantee`: Guarantees that placement sees bounding box or cover plus terminals for every variant before any geometry is drawn; 20+ processes ported in about 1 hour each. Does not guarantee electrical side data per variant unless added.
- `inputs needed`: Technology file, device definitions.
- `verification`: Every variant builds on every deck.
- `prefer over / degrade to`: Prefer over fixed-geometry cells. Degrade to one variant per device.
- `source`: LAMP-25; LAMP-24; LAMP-12.

### P-23
- `id`: P-23
- `technique`: Per-side routing-space halos (static from terminal wire widths, dynamic by distance to each net's pin-box centre)
- `realizes`: Routable placements without blanket spacing.
- `applicability`: Placement before routing.
- `parameters`: Static part Σ W_i over terminals on the side; dynamic k₁,i − k₂,i·distance per net (constants not given; clamp at zero).
- `guarantees` / `does not guarantee`: Guarantees space for the terminal wires of each side. Does not guarantee routability or density; the per-net form was consistently better than the centre form; overestimation causes area loss.
- `inputs needed`: Terminal currents, net pin boxes.
- `verification`: Routing overflow after routing; area against manual.
- `prefer over / degrade to`: Prefer over one uniform clearance. Degrade to a utilisation floor.
- `source`: LAMP-31; LAMP-47.

### P-24
- `id`: P-24
- `technique`: Relative-order template extraction with the extended constraint graph (LDS)
- `realizes`: A resizable template that cannot overlap for any new sizes; positions as equations on element boundaries; compaction removes slack.
- `applicability`: Retargeting, re-sizing, keeping a proven topology across variants.
- `parameters`: Spacing constraints only between same-layer shapes of different nets; 3-wire fixed or 4-wire flexible paths.
- `guarantees` / `does not guarantee`: Guarantees non-overlap under resizing (edge from each right boundary to every element with larger x and nothing between). Does not guarantee minimal area beyond compaction, nor topology change.
- `inputs needed`: A placement and routing to extract from.
- `verification`: No overlap or min-width/spacing violation after resynthesis (demonstrated on a two-stage opamp).
- `prefer over / degrade to`: Prefer over re-annealing when only sizes change. Degrade to re-placement.
- `source`: BAL2-24; BAL2 §4.5.6.1.

### P-25
- `id`: P-25
- `technique`: Hierarchical compaction (intracell constraints once per cell, edge coordinates relative to cell origins)
- `realizes`: Reuse of cell constraints across instances; fewer variables.
- `applicability`: Hierarchical layouts with repeated cells.
- `parameters`: Variable trimming.
- `guarantees` / `does not guarantee`: Guarantees consistency of each instance with its cell. Does not guarantee instance-specific optimisation.
- `inputs needed`: Hierarchy, cell constraint sets.
- `verification`: All constraints satisfied per instance.
- `prefer over / degrade to`: Prefer over flattening. Degrade to flat compaction.
- `source`: BAL2-32 (eq. 5.49); BAL2 §5.7.3.

### P-26
- `id`: P-26
- `technique`: Contact and via array merge before compaction, regenerated afterwards
- `realizes`: Far fewer constraints during compaction; cut count preserved.
- `applicability`: Any compaction over polygon edges.
- `parameters`: Merged cut size ≥ source cut count.
- `guarantees` / `does not guarantee`: Guarantees at least the source cut count. Does not guarantee EM adequacy unless the count is derived from current.
- `inputs needed`: Via arrays.
- `verification`: Cut count after regeneration.
- `prefer over / degrade to`: Prefer over per-cut constraints. Degrade to per-cut.
- `source`: BAL2-35.

### P-27
- `id`: P-27
- `technique`: Array macro generation by abutment (Mondriaan placement side)
- `realizes`: Regular arrays (flash ADC, DAC current sources, cellular networks) with supply, ground and bias by abutment and flipped cells sharing lines.
- `applicability`: Many identical unit cells; metric is equal R, C and matching, not delay.
- `parameters`: Row/column structure; flip pattern.
- `guarantees` / `does not guarantee`: Guarantees regularity and shared rails. Does not guarantee optimality for irregular connectivity.
- `inputs needed`: Unit cell, array dimensions, connectivity.
- `verification`: Equal wire R/C per unit.
- `prefer over / degrade to`: Prefer over general placement for arrays. Degrade to P&R of units.
- `source`: BAL2-28.

### P-28
- `id`: P-28
- `technique`: Thermal placement rule: sensitive pairs symmetric about heat-radiating devices, matched devices on isotherms
- `realizes`: Equal temperature on both members; prevents thermally induced offset and unbalanced thermal coupling.
- `applicability`: Class-AB and power stages; bipolars in particular.
- `parameters`: Which devices are hot (power threshold); whether the hot device is placed self-symmetric on the pair's axis.
- `guarantees` / `does not guarantee`: Guarantees equal distance to the source by construction. Does not guarantee the residual gradient from other sources; the budgeted form needs the profile and sensitivities.
- `inputs needed`: Device power, matched pairs.
- `verification`: Per-pair ΔT and degradation table (Opamp3 total 1.012 mV).
- `prefer over / degrade to`: Prefer over fixed per-pair ΔT caps. Degrade to a distance-from-hot-device rule.
- `source`: BAL1-03; LAMP-14; LAMP-11; LAMP-13.

### P-29
- `id`: P-29
- `technique`: Expert-base or schematic-driven constructive placement
- `realizes`: A fast first placement following the schematic's arrangement.
- `applicability`: Seeds and templates; RACHANA assumes the drawn schematic has the best distribution.
- `parameters`: Device selection order.
- `guarantees` / `does not guarantee`: Guarantees speed. Does not guarantee quality; the result depends on the selection order.
- `inputs needed`: Schematic positions or expert rules.
- `verification`: Constraint checks afterwards.
- `prefer over / degrade to`: Prefer as a seed over random starts. Degrade to random initial placement.
- `source`: BAL1 §1.1.3; BAL2 §4.5.4.

### P-30
- `id`: P-30
- `technique`: Slicing-tree template layout (design by example; BALLISTIC and CAIRO relative-placement languages)
- `realizes`: Reproducible full layout in seconds with symmetries coded in; river-routed channels.
- `applicability`: A known circuit type whose template exists; layout-aware sizing loops.
- `parameters`: Template tree, geometric parameters per leaf, separations.
- `guarantees` / `does not guarantee`: Guarantees the template's constraints and speed. Does not guarantee transfer to a new circuit type or technology (template rewritten); authoring costs 1.5–2× manual layout of the block.
- `inputs needed`: Template, sized netlist.
- `verification`: DRC/LVS; area loss.
- `prefer over / degrade to`: Prefer when the same topology is laid out repeatedly. Degrade to P&R.
- `source`: LAMP §1.5.2.2; BAL2 §6.3.2; BAL2 §4.5.6; BAL2-39.

### P-31
- `id`: P-31
- `technique`: Device realisation swap (finger count) at fixed relative placement
- `realizes`: Area recovery after size changes without 2-D compaction.
- `applicability`: Retargeting and variant changes.
- `parameters`: Allowed realisations per device.
- `guarantees` / `does not guarantee`: Guarantees the relative placement is kept. Does not guarantee matched variants unless the swap is applied to the whole matching group.
- `inputs needed`: Variants, relative-order graph (P-24).
- `verification`: Area; matched-variant equality.
- `prefer over / degrade to`: Prefer over keeping stale finger counts. Degrade to compaction only.
- `source`: BAL2-38; BAL1-43.

### R-01
- `id`: R-01
- `technique`: Mirror routing: search one net with two-sided legality, mirror the path (LAYLA, ROAD mirrored obstacles; ANAGRAM II joint growth)
- `realizes`: Geometrically identical pair routes, hence identical self-parasitics, even for partially symmetric placements.
- `applicability`: Pairs whose terminals are mirror images about an axis.
- `parameters`: Axis; obstacle set B ∪ mirror(B); whether the partner's own shapes are exempt.
- `guarantees` / `does not guarantee`: Guarantees legality on both sides and equal geometry. Does not guarantee equal coupling to third nets (aggressors may differ per side) or feasibility when terminals are not mirrored.
- `inputs needed`: Pair, axis, obstacles.
- `verification`: Per-layer signature equality; two-sided DRC.
- `prefer over / degrade to`: Prefer over independent routing then repair. Degrade to R-03 matched routing when terminals are not mirrored.
- `source`: LAMP-40; BAL2-07; SURV-13.

### R-02
- `id`: R-02
- `technique`: Crossover connector for axis-swapped symmetric pairs
- `realizes`: Both nets cross the axis through a symmetric crossing structure with matched R and C.
- `applicability`: Cross-coupled latches, chopper swaps, any pair with mirror-swapped pins.
- `parameters`: Crossing layer pair; position on the axis.
- `guarantees` / `does not guarantee`: Guarantees equal R and C of the crossing. Does not guarantee equal residual coupling to other nets.
- `inputs needed`: Pair with swapped terminals, axis.
- `verification`: Signature equality including via counts.
- `prefer over / degrade to`: Prefer over asymmetric crossing. Degrade to R-03 detours.
- `source`: BAL2-08.

### R-03
- `id`: R-03
- `technique`: Exact matched routing: equal width, equal per-layer length and area, detours and adjustment sections for bundles
- `realizes`: Precisely matched wiring parasitics for pairs that cannot be mirrored geometrically (e.g. perfect symmetry).
- `applicability`: Any matched pair or bundle.
- `parameters`: Which side detours; corner handling in bundles.
- `guarantees` / `does not guarantee`: Guarantees per-layer equality after repair and equal width. Does not guarantee geometric symmetry; aggressor coupling still needs balancing per aggressor.
- `inputs needed`: Pair, layer stack, aggressor list.
- `verification`: Σ_layer |a − b| over the mean; per-aggressor ΔC.
- `prefer over / degrade to`: Prefer over own-RC-only checks. Degrade to length-only matching.
- `source`: SURV-14; SURV-29; BAL2-22; BAL1-02.

### R-04
- `id`: R-04
- `technique`: Shield insertion policy (space first, shield last, tie to DC; same-layer, plate or bulk shield)
- `realizes`: Removal of a specific coupling path between a victim and an aggressor.
- `applicability`: Victim–aggressor pairs whose coupling budget cannot be met by spacing.
- `parameters`: Shield geometry; reference net (ground over supply, never a signal); order (after all wires).
- `guarantees` / `does not guarantee`: Guarantees the shielded path is removed. Does not guarantee no added ground capacitance or area.
- `inputs needed`: Net classes or coupling budgets, a quiet low-impedance reference net.
- `verification`: Coupling budget met after insertion.
- `prefer over / degrade to`: Prefer spacing first. Degrade to spacing only.
- `source`: BAL2-10; SURV-18; BAL2-12.

### R-05
- `id`: R-05
- `technique`: Current-driven wire width and via arrays per segment, with KCL-fixed currents from a terminal sequence and a current-density verifier
- `realizes`: Electromigration-safe widths W_i = I/I_max,i and via counts N_via = I/I_max,ij on every segment.
- `applicability`: Supply, bias and any current-carrying net; devices internally (P-20).
- `parameters`: Layer and via current limits; temperature derating; terminal-to-terminal sequence for KCL; greedy Steiner from three terminals.
- `guarantees` / `does not guarantee`: Guarantees per-segment compliance, no over-widening. Does not guarantee minimal capacitance on signal nets (width adds C), nor correctness if currents are not from simulation.
- `inputs needed`: Branch currents (simulation, user patterns or Monte Carlo), deck limits.
- `verification`: Current-density check per segment; via-array extent ⌈√N⌉ columns at cut pitch plus enclosures.
- `prefer over / degrade to`: Prefer over uniform fat supply wires. Degrade to a class-based width.
- `source`: LAMP-37; LAMP-22; BAL2 §4.7; SURV-17.

### R-06
- `id`: R-06
- `technique`: Net splitting into subnets by current density (force/sense separation)
- `realizes`: Low-current branches routed separately back to the source point so they do not see the time-varying drop of the high-current segment.
- `applicability`: Supply, ground and bias nets with order-of-magnitude current spread; Kelvin taps.
- `parameters`: Current threshold for force vs sense; root terminal; alternative of widening the shared segment.
- `guarantees` / `does not guarantee`: Guarantees that sense terminals see only their own drop. Does not guarantee minimal wire; the split adds a tree.
- `inputs needed`: Terminal currents, net root.
- `verification`: Pin-pair IR on star nets.
- `prefer over / degrade to`: Prefer over widening when the branch is sensitive. Degrade to widening the shared segment.
- `source`: BAL2-06; BAL2-04; BAL2-46.

### R-07
- `id`: R-07
- `technique`: Net-class routing order and class-pair spacing (ILAC, ANAGRAM, SLAM)
- `realizes`: Critical nets take privileged tracks; power and non-critical nets separate critical from noisy; sensitive–noisy pairs get the largest spacing.
- `applicability`: Any analog block; fallback when sensitivities are unavailable.
- `parameters`: Class per net (sensitive, noisy, non-critical, power; or neutral/noisy/sensitive); order power → sensitive → non-critical → noisy; spacing per class pair.
- `guarantees` / `does not guarantee`: Guarantees a consistent order and spacing. Does not guarantee specs (the heuristic has no quantitative link to performance).
- `inputs needed`: Net classes from names, topology or operating point.
- `verification`: Spacing per class pair; coupling estimate.
- `prefer over / degrade to`: Prefer over arbitrary order. Degrade to pin-count order.
- `source`: BAL2-11; BAL2-12; BAL2-13; LAMP-48.

### R-08
- `id`: R-08
- `technique`: Pre-route impact factor F_k scheduling
- `realizes`: The nets with the largest share of spec degradation keep their ideal paths; low-impact nets detour for DRC and performance.
- `applicability`: Routing with per-spec sensitivities.
- `parameters`: F_k = (Σ_j ΔP_j^k/ΔP_j)/N_s; rip-up order increasing F_k.
- `guarantees` / `does not guarantee`: Guarantees a spec-derived order. Does not guarantee convergence without rip-up.
- `inputs needed`: Sensitivities, a pre-route of every net alone ignoring DRC.
- `verification`: ΔP per net after routing versus pre-route.
- `prefer over / degrade to`: Prefer over class order when sensitivities exist. Degrade to R-07.
- `source`: LAMP-42.

### R-09
- `id`: R-09
- `technique`: Obstacle inflation by layer minimum space (terminals exempt) on a nonuniform grid refined around new wires
- `realizes`: DRC-clean spacing by construction; finer resolution where congested; fast neighbourhood lookup for parasitic evaluation.
- `applicability`: Any area router.
- `parameters`: Per-layer and width-dependent spacing; grid-cell size and aspect-ratio limits.
- `guarantees` / `does not guarantee`: Guarantees legal spacing to foreign shapes. Does not guarantee variable-width support unless spacing is width-dependent at search time.
- `inputs needed`: Layer rules, obstacles, terminals.
- `verification`: DRC on the result.
- `prefer over / degrade to`: Prefer over uniform grid plus compaction ("unsatisfactory"). Degrade to uniform pitch.
- `source`: BAL2-03; BAL2 §4.3.1.1.

### R-10
- `id`: R-10
- `technique`: Steiner tree growth by multi-source expansion from the partial tree (Prim-like or Kruskal-like)
- `realizes`: Near-minimal multi-terminal trees with Steiner points.
- `applicability`: All multi-terminal nets.
- `parameters`: Target order (nearest first).
- `guarantees` / `does not guarantee`: Guarantees a connected tree no worse than the rectilinear MST bound of 1.5× the optimum. Does not guarantee the optimum (NP-complete).
- `inputs needed`: Pins, routing graph.
- `verification`: Tree length versus MST.
- `prefer over / degrade to`: Prefer over pin-to-pin MST routing. Degrade to MST edges.
- `source`: BAL2-01; LAMP-41; BAL2 §4.2.4.

### R-11
- `id`: R-11
- `technique`: Compaction-based constructive detailed routing (ring router around module boundary, compaction toward reference, small-scope inter-module maze)
- `realizes`: Dense intra-module wiring without a routing-area estimate.
- `applicability`: Module-structured layouts; order from symmetric and central modules.
- `parameters`: Module order; reference position.
- `guarantees` / `does not guarantee`: Guarantees no routing-area estimate is needed. Does not guarantee global optimality.
- `inputs needed`: Modules with pins, order.
- `verification`: DRC, area.
- `prefer over / degrade to`: Prefer over blanket routing halos. Degrade to global/detailed routing.
- `source`: BAL2-23.

### R-12
- `id`: R-12
- `technique`: Candidate-route enumeration per two-terminal edge with a compatibility graph and independent-set selection (x–y routing)
- `realizes`: Joint selection of routes for all nets, honouring widths, layer confinement, keep-outs, parallel distance and coupled length versus width.
- `applicability`: Small sets of critical or symmetric nets; two adjacent layers, horizontals on one and verticals on the other; extended to 45°.
- `parameters`: Candidates by via count mostly within the net bounding box; MST edges per net.
- `guarantees` / `does not guarantee`: Guarantees that the chosen set is conflict-free and that nets are considered together rather than in order. Does not guarantee scalability beyond a few dozen candidates.
- `inputs needed`: Candidate shapes, conflict rules.
- `verification`: No conflicts in the chosen set.
- `prefer over / degrade to`: Prefer over sequential routing for the critical few. Degrade to ordered maze routing.
- `source`: BAL2-26.

### R-13
- `id`: R-13
- `technique`: Template routing adaptation (stretch or compact; LAYGEN scale and re-pin)
- `realizes`: Reuse of proven routes under a new placement or sizing.
- `applicability`: Retargeting; incremental epochs.
- `parameters`: Nets split into 2-pin wires; path scaling; endpoint moves.
- `guarantees` / `does not guarantee`: Guarantees topology preservation. Does not guarantee legality without a follow-up optimisation (LAYGEN used a GA with DRC, connectivity, wirelength and inter-net distance costs).
- `inputs needed`: Prior routes, new pin positions.
- `verification`: DRC and connectivity.
- `prefer over / degrade to`: Prefer over re-routing from scratch when pins moved little. Degrade to full re-route.
- `source`: BAL2-25; BAL2 §4.5.6; BAL2-24.

### R-14
- `id`: R-14
- `technique`: Exact-length refinement for RF interconnect (simultaneous expansion of all nets after constraint-based routing)
- `realizes`: Fixed interconnect lengths without new parasitic violations.
- `applicability`: RF nets with length targets; coupling modelled as Z0 and loss degradation.
- `parameters`: Target lengths; coupling model fitted to field-solver data.
- `guarantees` / `does not guarantee`: Guarantees the lengths once converged. Does not guarantee inductance accuracy without extraction.
- `inputs needed`: Length targets, coupling budgets.
- `verification`: Length per net; coupling check after each insertion.
- `prefer over / degrade to`: Prefer over post-hoc meanders. Degrade to shortest routes.
- `source`: BAL2-27; LAMP-17.

### R-15
- `id`: R-15
- `technique`: Array routing by column wires with bus and tree generators (Mondriaan)
- `realizes`: Equal R, C and matching to every unit; rails by abutment.
- `applicability`: Regular arrays.
- `parameters`: Vertical wires per column; horizontal wire count from vertical wires needing connection.
- `guarantees` / `does not guarantee`: Guarantees equal-length wiring per unit. Does not guarantee general-net routing.
- `inputs needed`: Array connectivity, edge pins.
- `verification`: Per-unit R/C equality.
- `prefer over / degrade to`: Prefer over general routing for arrays. Degrade to general routing.
- `source`: BAL2-28.

### R-16
- `id`: R-16
- `technique`: Keep-out over active regions (hard for matched and sensitive devices)
- `realizes`: No foreign metal crossing transistor active areas.
- `applicability`: All devices; ROAD allows over-device routing but excludes sensitive modules.
- `parameters`: Layers affected; hard versus cost.
- `guarantees` / `does not guarantee`: Guarantees no crossover coupling into the channel. Does not guarantee routability in dense blocks.
- `inputs needed`: Device active and gate geometry.
- `verification`: No foreign shapes over active.
- `prefer over / degrade to`: Prefer hard for matched devices. Degrade to a cost.
- `source`: SURV-16; BAL2 §4.5.3.2.

### R-17
- `id`: R-17
- `technique`: Width realisation by parallel tracks with a routability fallback
- `realizes`: Discrete widths 1×, 2×, 3× as parallel routes between the same pins on a gridded router.
- `applicability`: Performance-driven wire sizing; analog circuits are rarely congested.
- `parameters`: Width options; fallback to original width on failure.
- `guarantees` / `does not guarantee`: Guarantees design-rule compliance and 99% routability in the reported experiments. Does not guarantee equal path lengths of the parallel tracks (negligible effect reported).
- `inputs needed`: Per-net width choice; symmetric partners tied.
- `verification`: Routability; wire area.
- `prefer over / degrade to`: Prefer over trunk fattening when width-dependent spacing is unavailable. Degrade to 1×.
- `source`: SURV-31; SURV-29; SURV-32.

### R-18
- `id`: R-18
- `technique`: Substrate isolation distance by substrate type and guard-ring placement rules
- `realizes`: Noise isolation appropriate to the process: threshold rule on epi substrates (no gain beyond 4× epi thickness), linear separation reward on high-impedance substrates; guard rings close to the victim on dedicated pins.
- `applicability`: Mixed-signal blocks with noisy and sensitive devices.
- `parameters`: Substrate kind and epi thickness; ring-to-device gap; ring net.
- `guarantees` / `does not guarantee`: Guarantees the correct rule for the actual substrate. Does not guarantee quantitative isolation without a substrate network model (resistive to 2 GHz).
- `inputs needed`: Deck substrate data, noisy and sensitive device lists.
- `verification`: Distance rule; ERC that the ring net is a dedicated pin.
- `prefer over / degrade to`: Prefer over a fixed rule of thumb (wrong for one substrate type). Degrade to a fixed distance.
- `source`: LAMP-16; BAL2-10.

### R-19
- `id`: R-19
- `technique`: Hard parasitic-bound pruning of partial paths (listed as a hard-constraint enforcement inside a path search; it makes a budget hold by construction rather than by repair)
- `realizes`: Routes that never exceed a net's remaining parasitic or spec budget.
- `applicability`: Parasitic-bounded and manufacturability routing phases.
- `parameters`: Per-label accumulated R, C and coupling; remaining budget per row.
- `guarantees` / `does not guarantee`: Guarantees budgets are met by every accepted path. Does not guarantee a path exists; may need rip-up or placement change.
- `inputs needed`: Budgets or sensitivities and margins.
- `verification`: Budget usage ≤ 1 per row after routing.
- `prefer over / degrade to`: Prefer over cost-only pricing when a bound is hard. Degrade to weighted cost plus post-route repair.
- `source`: BAL2-20; LAMP-43; BAL2 §4.5.3.2.
