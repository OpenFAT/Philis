# M6 compaction spike: verdict

Question (ROADMAP M6, DECISIONS Q9 refinement, Q24, Q25): should Philis add a
post-processing pass that ignores the atoll grid, keeps only the PDK
manufacturing grid, and shrinks the floorplan with a constraint-graph
compaction (x, then y) plus a stochastic layer on top, under the Q24
invariants?

**Verdict: deferred.** Today's engine output has nothing for tile-level
compaction to recover on the three acceptance fixtures, and on the multi-tile
fixtures a 1-D pass recovers at most 3.6–4.4 % of the area. The whitespace
that is there (40–45 % of the bounding box) comes from tiles of unequal height
in a one-row floorplan. That is a floorplan problem, owned by M2's symmetric
row stack, not a compaction problem. No crate was built.
`crates/backend/post-processing` stays a stub outside the workspace. §5 gives
the conditions that reopen the question and the design to build if they hold.

## 1. Literature

### 1.1 Constraint-graph compaction (the baseline)

- **1-D longest path.** Each layout edge coordinate is a vertex. Each
  minimum-distance rule `x_j − x_i ≥ d_ij` is an arc of weight `d_ij`. A
  source vertex sits at x = 0. A graph of minimum-distance constraints alone
  is a DAG. The longest path from the source gives each coordinate its
  smallest legal value in O(V + E) (Said et al., in Graeb (ed.) 2011, §5.4.1;
  surveys.md BAL2-30).
- **Known defects.** (a) *Long wires*: longest path pushes everything to the
  left boundary, so an objective that minimises wirelength or distance to the
  source layout is added (§5.4.1; BAL2-31). (b) *Edge-order lock*: a
  constraint fixes the order of its two edges, so constraints that no longer
  apply after a change block area recovery. The fix is to alternate x and y
  passes and regenerate the constraints each time. 2-D compaction is
  NP-complete, and most tools stay 1-D (§5.6 "Edge Order"; BAL2-34). On their
  VCO the constraints had to be deleted by hand (Fig. 5.16).
- **Symmetry and matching as equalities.** A symmetry constraint
  `x_b − x_a = x_d − x_c` has more than two variables, so a plain constraint
  graph cannot express it (§5.4.1, eq. 5.2). There are three ways to handle
  this:
  - Okuda et al. solve the distance graph, then a reduced LP for the symmetry
    rows with revised simplex, and turn the result back into distance arcs
    (§5.7.1).
  - Said et al. use a graph-based simplex. The longest-path solution is the
    first basic feasible solution, and big-M artificial variables cover any
    unmet equality (BAL2-32). It runs 2–4× faster per iteration and 10× or
    more faster overall than revised simplex: a Miller opamp in 0.375 s for
    521 variables and 1,831 constraints in x, and a 500-device oscillator in
    6,750 iterations for 71,197 distance and 2,857 symmetry/equal
    constraints (Tables 5.1, 5.2).
  - Plantage (Strasser et al., ch. 3) builds horizontal and vertical
    constraint graphs from a B*-tree. It then solves
    `min y_e s.t. M·y ≥ d, C·y = k` with the symmetry and common-centroid
    rows in `C`, using an LP, or a MIP for piecewise-linear DTI distances
    (eqs. 3.26–3.28; surveys.md §1.5, §1.7).
- **Analog-safe compaction.** The canonical analog reference is Felt,
  Malavasi, Charbon, Totaro, Sangiovanni-Vincentelli, "Performance-Driven
  Compaction for Analog Integrated Circuits", CICC 1993. It feeds
  sensitivity-derived parasitic bounds into the compactor (cited in Lampaert
  et al. 1999, §1.5.2; the paper itself is not in `docs/ref`). Said et al.
  warn that wells drawn deliberately larger than minimum (for WPE) must be
  detected and kept, because a compactor shrinks every polygon to its rule
  (§5.6 "Nanometer Process Effects"; BAL2-37). Contact and via arrays are
  merged before compaction and regenerated after (BAL2-35).
- **Where it is used.** Retargeting a layout to a new process is the main
  use. Designers accept compaction because it keeps the floorplan,
  placement and routing (BAL2 §5.1–5.2). In Said et al.'s two opamp
  examples, the area drop came from the process change (6,060 → 2,800 µm²
  and 10,270 → 4,445 µm²). It was not slack recovered inside one process
  (§5.8).

### 1.2 Stochastic improvement on top

- In analog placement, the stochastic layer is almost always simulated
  annealing (SA) over a topological representation, with a deterministic
  solve inside it. Koda et al. use a sequence pair plus LP inside SA. Plantage
  uses a B*-tree plus enhanced shape functions plus LP, deterministically,
  with no perturbation (Graeb (ed.) 2011, ch. 2 Table 2.2, ch. 1 §1.1).
  Reported area usage is 104.7–118.5 % of device area on the
  `biasynth`/`lnamixbias` benchmarks. Symmetry islands are best at 104.68 %,
  sequence pair plus LP reaches 106.38 %, and Plantage 104.96 % (ch. 3 results
  table).
- So "Monte-Carlo-like moves over a constraint-graph solve" is the same
  machinery as a topological placer: the moves change relative order, and
  the graph solve legalises the result. Literature does not treat it as a
  separate post-pass.
- Lampaert et al. measured a 10–15 % area penalty against manual layout. They
  blame the separation of place and route, which makes the placer
  overestimate routing area. Their proposed remedies are a global router
  inside the placement loop, or compaction after routing "capable of
  handling performance constraints and ... symmetry and matching" (§7,
  General Conclusions; LAMP-47).
- The 2024 performance-driven survey (Xu et al.) and the common-centroid
  review do not mention compaction at all. Recent work recovers area in
  placement, not in a post-pass.

## 2. Q24 invariants as constraints

The tiles are rigid (Q24 (a)). Each tile `t` has one variable per axis, its
lower-left corner `(x_t, y_t)`. Every row below is an arc or an equality over
those variables.

| Invariant | Constraint form | Solver cost |
|---|---|---|
| Spacing between tiles | `x_b − x_a ≥ w_a + g_ab`, with `g_ab` the gap the facing shapes need (per layer pair, on the cut lattice) and the routing halo from Q25 added | DAG arc |
| Mirrored tiles move as one about their axis | One axis variable `a_k` per group. Pair `(i, j)`: `x_i + w_i + x_j = 2a_k`, `y_i = y_j`. Self-symmetric `s`: `x_s + w_s/2 = a_k` (Balasa corner form, BAL1-01) | Equality, so LP or graph-simplex. **Free when both halves are in one recipe tile**, which is the case today: the axis is internal to a rigid body |
| Centroid groups rigid | `x_m = x_g + off_m` for every member: contract the group into one vertex | Node contraction, no LP |
| Matched-set environment distance never shrinks | For member `m` and any non-member `n` facing it: `x_n − (x_m + w_m) ≥ max(g_mn, d_env)`, where `d_env` is the recipe's value. Easier and stronger: inflate the matched tile's outline by `d_env` as a keep-out, which keeps both members' surroundings equal up to the halo, not only bounded below | DAG arc, or a wider outline |
| Deliberate non-minimum wells (WPE) | Inside the tile, so they cannot change while tiles are rigid | none |
| Every claim re-scored | A postcondition, not a constraint: `score_claims` runs on the compacted table. On any regression, fall back to the floorplan table for that run | One extra scoring pass |
| Manufacturing grid | Each `g_ab` rounded up to `cut_lattice = 2 × grid` (10 nm on sky130), as `arrange` already does | none |

While every symmetry group and centroid group sits inside one tile, which is
true of every recipe M1 draws, the problem is pure longest path. LP is only
needed once separate tiles share an axis. That happens when M2's row stack
places mirrored halves as their own tiles.

## 3. Measurement

`crates/frontend/engine/examples/whitespace.rs` runs `engine::run` with
GPurify signoff on sky130 at `ee66c3d`, and reports:
- the tile bounding boxes against their sum;
- the floorplan and routed bounding boxes;
- for each pair of neighbouring tiles, the drawn gap against the gap their
  facing same-layer shapes need.

To reproduce:
`cargo run --release -p engine --example whitespace -- pair quad mirror_ratio ota two_ota`.

| Fixture | Tiles | Σ tile area | Floorplan bbox | Utilisation | Hard (non-PEX) | Recoverable by 1-D x pass |
|---|---|---|---|---|---|---|
| pair | 1 | 8.688 µm² | 2140 × 4060 nm = 8.688 µm² | 100 % | 0 | 0 |
| quad | 1 | 24.214 µm² | 6580 × 3680 nm = 24.214 µm² | 100 % | 0 | 0 |
| mirror_ratio | 1 | 54.317 µm² | 20420 × 2660 nm = 54.317 µm² | 100 % | 0 | 0 |
| ota | 3 | 954.7 µm² | 42210 × 41640 nm = 1757.6 µm² | 54.3 % | 15 | 1850 nm × 41640 nm = 77.0 µm² (4.4 %) |
| two_ota | 4 | 1573.1 µm² | 75680 × 41640 nm = 3151.3 µm² | 49.9 % | 92 | 2750 nm × 41640 nm = 114.5 µm² (3.6 %) |

The routed bounding box equals the floorplan bounding box on every fixture.

Per neighbouring pair, `arrange` draws 1270 nm, the widest spacing of any
layer the tiles draw (sky130 nwell). The facing shapes need 320–370 nm.
Where the whitespace sits:

- **ota:** 802.9 µm² of whitespace, 45.7 % of the bbox. The gap strips are
  105.8 µm². The other 697.1 µm² (39.7 %) comes from heights of 21.7, 9.1 and
  41.6 µm sharing one row.
- **two_ota:** 1578.2 µm² of whitespace. The gap strips are 158.6 µm². Height
  mismatch accounts for 1419.5 µm² (45.0 %).

Caveats:
- The needed gap counts only same-layer `min_spacing`. Inter-layer rules
  (diff to nwell, tap enclosure) can only raise it, so the 3.6–4.4 % figures
  are an upper bound.
- `ota` and `two_ota` do not sign off yet (M2), so their tables will change.
- `dac4`, `rc_filter`, `rdiv` and `bgr_core` are refused at parse ("M0 reads
  MOS only").
- The Q9 premise is absent today. No tile in the spine is built on the
  atoll grid: `arrange` places tiles on the 10 nm cut lattice. The
  "atoll-snapped area" that the refinement wants to give back is therefore
  zero.

## 4. Verdict

**Deferred**, for three reasons:

1. **The acceptance fixtures have nothing to compact.** pair, quad and
   mirror_ratio are each one rigid tile, and Q24 (a) forbids moving tile
   internals. Area reduction: 0 %.
2. **Where tiles are separate, a 1-D pass recovers ≤ 4.4 %.** One function
   gets the same result with no new crate: compute `arrange`'s gap per pair
   of neighbours from the layers that face each other, instead of
   `widest_spacing`. That edit belongs to the floorplan owner (engine
   `arrange` / M2's placement), not to `post-processing`.
3. **The 40–45 % that is left is floorplan topology.** Recovering it means
   changing which tiles share a row. That is the stochastic layer acting as
   a placer (§1.2), and it duplicates M2's row stack and M7's free-form
   gp/dp. The ladder stops at rung 1: the crate does not need to exist yet.

## 5. When to reopen, and what to build then

Re-run `examples/whitespace.rs` when any of these holds:

- M2's row stack lands with atoll-snapped tiles. This is the case Q9's
  refinement was written for.
- Separate tiles share a symmetry axis.
- A signoff-clean multi-tile fixture shows more than **10 %** recoverable
  area at fixed topology. 10 % is the low end of Lampaert's place-and-route
  penalty.

If the numbers clear that bar, build `post_processing::compact(floorplan,
tiles, intent, pdk) -> Compacted` as `docs/API.md` describes:

- `invariants(floorplan, intent)` returns the §2 table as rows: arcs, groups
  to contract, axis equalities, halos.
- `compact_x` and `compact_y` each regenerate the arcs from the current
  positions, contract the centroid groups and solve longest path in
  topological order, O(N + E) with N being tens of tiles. When axis
  equalities exist, a small LP replaces longest path for those rows (Okuda's
  split). Alternate x and y until the area stops falling (BAL2-34).
- `stochastic(compacted, seed)` makes local swaps of adjacent tiles that
  respect the invariants. Each swap is legalised by the x/y solve and
  accepted on area plus HPWL. It is a separate call, seeded by the caller
  (Q33).
- Tests:
  - `invariants` holds on the output of `compact`. The axis, centroid offsets
    and halos are unchanged, checked without signoff.
  - pair, quad, mirror_ratio and ota keep signoff findings at 0 after
    compaction.
  - Area never grows.
  - The same seed gives the same table.

## Sources

- H. Said et al., "Analog Layout Retargeting", ch. 5 in H. Graeb (ed.),
  *Analog Layout Synthesis: A Survey of Topological Approaches*, Springer
  2011: §5.4–5.9, Tables 5.1–5.2.
- M. Strasser et al., ch. 3 (Plantage), and Lin and Chang, ch. 2, same book:
  eqs. 3.26–3.28, Table 2.2, and the ch. 3 results table.
- K. Lampaert, G. Gielen, W. Sansen, *Analog Layout Generation for
  Performance and Manufacturability*, Kluwer 1999: §1.5.2 and General
  Conclusions, citing [Felt 93].
- E. Felt et al., "Performance-Driven Compaction for Analog Integrated
  Circuits", Proc. IEEE CICC 1993, pp. 17.3.1–17.3.5. Cited, not read.
- Xu et al., "Performance-Driven Analog Layout Automation: Current Status and
  Future Directions", ASP-DAC 2024. Checked for compaction: no mentions.
- `docs/ref/research/surveys.md` §1.5, §1.7, §2.7 (the BAL1, BAL2 and LAMP
  item numbers above).
- Measurements: `crates/frontend/engine/examples/whitespace.rs` at branch
  `m6-compaction`.
