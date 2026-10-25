# Common-centroid construction and mismatch theory

How sources are cited. Digest item ids: `CC-nn` = docs/plans/ref-common-centroid-papers.md, `MM-nn` = docs/plans/ref-mismatch-papers.md, `GRAEB-nn` = docs/plans/ref-graeb-design-centering-sizing.md. Papers: REV = Karmokar et al., "Common-Centroid Layout for Active and Passive Devices: A Review and the Road Ahead", ASP-DAC 2022; DACP = Karmokar et al., "Constructive Placement and Routing for Common-Centroid Capacitor Arrays in Binary-Weighted and Split DACs", TCAD 2023; NTH = Dai et al., "An Nth Order Central Symmetrical Layout Pattern for Nonlinear Gradients Cancellation", ISCAS 2005 (read from the PDF); PEL = Pelgrom 1988; DRE = Drennan and McAndrew 2003; LDM = Schaper et al. 2001 (long-distance mismatch); DVP = Schaper and Linnenbank (distance vs pair matching, 130 nm). "L" numbers are the digests' reftext line numbers. "[derived]" marks my algebra on source statements, not a source claim. Card ids are `PAT-nn` so they do not collide with digest ids.

Two caveats that apply throughout:

- Every pattern ranking in these papers is simulated, not measured on silicon: NTH Table I, DACP Tables I–VI (12 nm models, Python evaluation, DACP L884–911), REV §V (14 nm, two correlation distances). The silicon data (PEL, DRE, LDM, DVP) measures the mismatch model: area, distance, orientation, bias, geometry. It does not compare CC patterns.
- No source gives a quadratic-gradient coefficient, a measured capacitor correlation (DACP L1014–1019), a WPE coefficient (CC-36), or a FinFET misalignment coefficient (CC-42). Cards that need one say so.

## 1. Technique cards

### PAT-01
- `id`: PAT-01
- `technique`: Interdigitated row (ABAB…), two devices alternating along one diffusion-shared row. Not common-centroid.
- `realizes`: Equal LOD for both devices (outer A's SA/SB equals outer B's SB/SA) and equal source/drain wire lengths, so equal R_s (REV §V-A L428–447). Proximity.
- `applicability`: MOS pairs (diff pair, 1:1 mirror); caps and resistors by analogy. Two devices, equal counts, any even count, one row.
- `parameters`: Units per device, pitch, shared-source orientation.
- `guarantees`: LOD equality and routing-parasitic equality by symmetry of the row ends (REV L428–447). Best effective G_m of the three row styles: ranking ID > CC > clustered under G_m = g_m(v_in − i_ac·R_s)/v_in (REV eq. 25, L454–503).
- `does not guarantee`: First-order gradient cancellation. A's centroid sits half a pitch left of B's (REV §I L73–78). Residual under Pelgrom's distance term is S_P times that offset (MM-01).
- `inputs needed`: Unit count, pitch; S_P (or S(L), MM-24/MM-25) to price the residual.
- `verification`: Moment check: per-device mean x differs by pitch/2 (CC-01). Extraction check: R_s and via count equal per device (CC-31).
- `prefer over / degrade to`: Prefer over single-row CC for small devices where LOD and parasitic mismatch exceed the gradient term (REV §V-A L448–453). Degrade to clustered (PAT-02) for very small devices. Upgrade to two-row swapped CC (PAT-05) when the device is large and the gradient term binds.
- `source`: REV §I L73–78, §V-A L428–503; CC-35, CC-40, CC-41.

### PAT-02
- `id`: PAT-02
- `technique`: Clustered (compact, non-CC) placement: each device's units contiguous, devices side by side.
- `realizes`: Minimum parasitics, matched LOD at device level, smallest area.
- `applicability`: Small devices. REV's examples: StrongARM latch with DP 6.1 µm, NMOS cross-coupled pair 3.1 µm, PMOS 1.6 µm at 14 nm; PMOS mirror 2.3 µm/14 nm in the 5T-OTA (REV L481–498, L520–531). Any device count.
- `parameters`: Device order, orientation.
- `guarantees`: LOD matched (REV L428–437). When the correlation distance R_L is long (1000 µm) the uncorrelated term dominates and the pattern barely changes σ (REV L476–480). Best for StrongARM because CC creates X/Y capacitance mismatch and ID adds parasitics at X/Y (REV L481–498).
- `does not guarantee`: Gradient cancellation. Highest current through R_s, lowest G_m (REV L454–503). At R_L = 10 µm "clustered is clearly worse" (REV L476–480).
- `inputs needed`: R_L and σ_s per device type (CC-07); device size; sensitive-node capacitance budget.
- `verification`: Moment check reports centroid offset equal to one device width. Extraction check: parasitic C at sensitive nodes per side, R_s per side.
- `prefer over / degrade to`: Prefer when the devices are small enough that LOD plus parasitic mismatch exceed the gradient term (REV §V-B). Upgrade to ID (PAT-01) or multi-row CC (PAT-05) as device size grows.
- `source`: REV §V-A, §V-B; CC-07, CC-40.

### PAT-03
- `id`: PAT-03
- `technique`: One-dimensional common-centroid row: palindrome of units about the row centre (ABBA; ABBAABBA for more units) (REV §I L58; NTH Fig. 1(a)).
- `realizes`: Coincident centroids along the row, so any linear gradient along the row cancels. For k ratioed devices the condition is equal per-device unit means (REV eq. 1).
- `applicability`: Two devices, equal or ratioed unit counts, even count per device (minimum 2). MOS fingers with diffusion sharing, capacitor rows, resistor rows.
- `parameters`: Units per device; palindrome order (plain ABBA vs the higher-order ABBABAAB, PAT-06); row boundary on a source diffusion.
- `guarantees`: Order 1 exactly for any linear gradient (NTH eqs. 13–16). Every odd order when each device is centrally symmetric by itself, which ABBA is (NTH §III corollary) [derived from the corollary].
- `does not guarantee`: Even orders. In ABBA the per-unit x² moment of the outer device is 9 times that of the inner device (positions ±1.5p against ±0.5p) [derived from NTH eqs. 3–6; CC-02], so a quadratic gradient along the row leaves a residual. LOD: inner cells see different SA/SB from outer cells (REV §V-A L428–437). Source/drain wire lengths differ between A and B (REV L438–447).
- `inputs needed`: Unit counts; S_P or S(L) to price the linear residual of the odd member if any; a quadratic coefficient (not given by any source, MM-26); KVTH0 for LOD (CC-35).
- `verification`: Moment check: per-device means of x and x² (CC-02). LOD sum per device (CC-35). Extraction check: R_s and via count per device (CC-31).
- `prefer over / degrade to`: Prefer over ABAB when the gradient term can bind (D* small, MM-06). Degrade to ABAB or clustered for small devices (REV L448–453). Upgrade to two-row swapped (PAT-05) to fix LOD and R_s, or to nth order (PAT-06) when 2^n units per device are available.
- `source`: REV §I eq. 1, §V-A; NTH §III Fig. 1(a), eqs. 13–16, corollary L216–222; CC-01, CC-02, CC-03, CC-35.

### PAT-04
- `id`: PAT-04
- `technique`: Two-dimensional cross-quad: 2×2 grid, A on one diagonal, B on the other (NTH Fig. 1(b)).
- `realizes`: Two-dimensional centroid coincidence with two units per device in the smallest area; a near-square aspect minimises the maximum distance from the origin (REV §I L65–73).
- `applicability`: Two devices, two units each (tile for multiples). MOS in two rows, caps, resistors, BJTs.
- `parameters`: Unit size, row and column pitch, which diagonal holds A.
- `guarantees`: Order 1 (NTH eqs. 13–16). All odd orders (each device self-symmetric, NTH corollary). Of the three second-order terms, x² and y² cancel; only the xy term survives [derived from NTH eqs. 3–6: the device difference under a second-order field equals the xy coefficient times the unit pitch squared]. NTH Table I row (a): 0 at order 1, 2.77 % at cumulative order 2.
- `does not guarantee`: The xy (diagonal) curvature; even orders beyond. LOD equality needs identical rows with end dummies. Diagonal connections must be routed symmetrically.
- `inputs needed`: Unit counts; same process coefficients as PAT-03.
- `verification`: Moment check: means of x, y, x², xy, y² per device (CC-02). Extraction check: diagonal lead lengths and via counts equal (CC-31).
- `prefer over / degrade to`: Prefer over 1-D ABBA for two-dimensional gradients at equal unit count. Upgrade to 2×4 (PAT-06, n = 2) to cancel xy. Degrade to ABBA when single-row diffusion sharing matters more.
- `source`: NTH §III Fig. 1(b), Table I row (a); REV §I eqs. 2–3; CC-02, CC-03.

### PAT-05
- `id`: PAT-05
- `technique`: Two-row swapped common-centroid: a CC row and beneath it the same row with A and B exchanged (REV §V-A L448–453). For a palindromic row this equals NTH's second-order construction (point-reflected copy with labels swapped), e.g. ABBA over BAAB (NTH Fig. 2(b); CC-04 note).
- `realizes`: Second-order gradient cancellation plus matched LOD and matched S/D wire lengths for CC transistor arrays (REV L448–453).
- `applicability`: Two devices, at least four units each, even per row. MOS in two rows; capacitor 2×2m arrays.
- `parameters`: Row length; the copy must be the 180° rotation with swapped labels (identical to a label swap only when the row is palindromic).
- `guarantees`: Orders 1 and 2, and every even order (A's cells are the point reflection of B's about the block centre; NTH §III case b and corollary). NTH Table I row (b): 0, 0, 0.24, 0.87, 1.70 % at cumulative orders 1–5. LOD and routing parasitics matched (REV L448–453).
- `does not guarantee`: Third and higher odd orders (0.24 % at cumulative order 3). Thermal fields unless they are polynomial.
- `inputs needed`: Unit count 4m; pitch. No process coefficient needed for construction; coefficients price the order-3 residual only.
- `verification`: Moment check: all moments with a+b ≤ 2 equal per device. LOD sums equal. Extraction check: lead R and via count equal per device (CC-31).
- `prefer over / degrade to`: Prefer over single-row CC whenever two rows fit; REV's optimised 5T-OTA uses four rows for the DP (80 units/device) and NMOS mirror (40 units/device) (REV L520–531). Upgrade to PAT-06 with n ≥ 3. Degrade to PAT-03 if only one row fits.
- `source`: REV §V-A L448–453, §V-B L520–531; NTH §III, Fig. 2, Table I(b); CC-04, CC-35.

### PAT-06
- `id`: PAT-06
- `technique`: Nth-order central-symmetric pattern. P_1 is any CC pattern with two units per device (ABBA or the 2×2 cross-quad). P_n is P_{n−1} placed beside its 180°-rotated copy; when n is even the labels A and B are swapped in the copy; the new centre C_n is the midpoint of the combined block (NTH §III items i–ii). Instances: order 2 is 1×8 ABBABAAB or 2×4; order 3 is 2×8 or 4×4; order 4 is 8×4; order 5 is 8×8 (NTH Figs. 2–4).
- `realizes`: Cancellation of every polynomial gradient of order 1 through n between two identical devices with 2^n units each (NTH abstract, §V).
- `applicability`: Exactly two devices, 1:1 ratio ("1-1 matching between two identical devices", NTH §III). 2^n units per device; the count m must be even for n > 1 (NTH §III). Grids: 2×2, 2×4 or 1×8, 4×4 or 2×8, 8×4, 8×8. Concatenate along the shorter side to stay near square (CC-04). MOS rows, caps, resistors.
- `parameters`: n; choice of P_1 (1-D or 2×2); concatenation axis at each step.
- `guarantees`: Orders 1..n for arbitrary coefficients (NTH eqs. 17–23 by induction). In addition, all odd orders when n is odd (each device is self-symmetric about C_n) or all even orders when n is even (A is the point reflection of B) (NTH §III corollary) [combination derived]. NTH Table I: (c) 4×4 zero through order 3, then 0.01 and 0.068 % at cumulative orders 4 and 5; (d) 8×4 zero through order 4, 0.0023 % at 5; (e) 8×8 zero through order 5.
- `does not guarantee`: Ratioed devices or more than two devices (not addressed by NTH). Non-polynomial fields such as the 1/t oxide model (DACP eq. 11). LOD and routing equality beyond what the row symmetry gives. Row-end dummies still needed.
- `inputs needed`: 2^n units per device; choose n = ⌊log2(units/2)⌋ (CC-04). Gradient coefficients only to price the first uncancelled order.
- `verification`: Moment check: all mixed moments with a+b ≤ n equal per device (CC-02). Symmetry classifier: self-symmetry for odd n, swap-symmetry for even n (CC-03). Extraction check: lead R and C equal per device.
- `prefer over / degrade to`: Prefer over ABBA or cross-quad whenever at least four units per device exist, since the area is the same (NTH §V). Prefer over circular symmetry (area-inefficient, non-Manhattan, NTH §I) and hexagonal tessellation (second order only, NTH §IV). Degrade to PAT-05 (n = 2) or PAT-03 (n = 1) at fewer units.
- `source`: NTH §III, §IV Table I, §V; CC-02, CC-03, CC-04.

### PAT-07
- `id`: PAT-07
- `technique`: Circular-symmetry and hexagonal-tessellation patterns (He et al., NTH ref. [6]), compared by NTH.
- `realizes`: Nonlinear gradient cancellation on a non-rectangular arrangement.
- `applicability`: Two devices; units at angles on a circle or hexagonal lattice; eight units in NTH Fig. 4(f),(g).
- `parameters`: Radius, angular positions.
- `guarantees`: Second-order circular symmetry: zero through cumulative order 3 (each device is self-symmetric about the circle centre, NTH §IV), then 0.026 and 0.18 % at orders 4 and 5. Hexagonal: zero through order 2, then 0.26, 0.50, 2.24 % (NTH Table I rows f, g).
- `does not guarantee`: Area efficiency or Manhattan placement: "unit cells need to be placed diagonally or in some particular angle … not easily realized in most of today's processes" (NTH §I).
- `inputs needed`: None for the engine; benchmark only.
- `verification`: Moment check as PAT-06. Extraction check not applicable.
- `prefer over / degrade to`: Never preferred over PAT-06 at equal order; the 4×4 reaches order 3 on a Manhattan grid. Listed so a pattern classifier can recognise and rank them.
- `source`: NTH §I, §IV Fig. 4(f),(g), Table I.

### PAT-08
- `id`: PAT-08
- `technique`: Array sizing, dummy fill and outer dummy ring. Choose r×s nearest to square for U units, fill leftover slots with dummies, surround the array with a ring of dummies (REV L309–314). Unit coordinates about the centre: x = (col − (s+1)/2)(W + S_h), y = ((r+1)/2 − row)(H + S_v) (DACP L271–284). Split-DAC dummy count D_C = r·s − (Σn_i + 2) (DACP L835–847).
- `realizes`: Bounded maximum distance from the centroid (a square minimises it, REV §I L65–73). Identical neighbourhood for every active unit: edge and fringe effects (REV L309–314), uniform OD width, spacing and pitch (REV L118–145), stress uniformity in FinFET/GAA (REV L510–522).
- `applicability`: All unit arrays (caps, MOS, resistors). Any U; odd U leaves at least one dummy slot.
- `parameters`: r, s, ring width (one unit), dummy terminal connection, tall vs wide.
- `guarantees`: Every active unit sees the same lateral environment. Qualitative in the sources; no quantified dummy benefit in any of these papers.
- `does not guarantee`: Gradient cancellation (dummies are inert). Electrical neutrality unless dummy plates or gates are tied to a quiet node (digest recipe CC-18, not a paper result).
- `inputs needed`: U, unit W and H, spacings S_h and S_v, aspect preference.
- `verification`: Structural: every active unit has eight neighbours. Extraction check: dummy-to-member coupling symmetric per device.
- `prefer over / degrade to`: Ring whenever area permits. Degrade to row-end dummies only (LOD equalisation, CC-35) when area is bound.
- `source`: REV §I L65–73, §III-C L309–314, §VI L510–522; DACP §II-C L271–284, §IV-C L835–847; CC-18, CC-35.

### PAT-09
- `id`: PAT-09
- `technique`: Odd unit counts and singletons. Binary DAC: C_0 and C_1 (one unit each) diagonally opposite at the centre (DACP L674–678). Split DAC: C_0 opposite a dummy near the centre; C_1 and the odd-count MSB unit as a symmetric pair (DACP L822–834). Transistor arrays: the odd cell goes at the layout edge (ALIGN, REV L439–441).
- `realizes`: Minimum centroid offset for the members that cannot be CC; the rest stay CC.
- `applicability`: Any array with odd per-device counts; k devices.
- `parameters`: Which centre slots; dummy pairing.
- `guarantees`: The residual systematic error is known: the singleton's distance from the array centre (CC-22). Centre placement protects the ratios that depend on C_0 (every C_k/C_0 does; DACP footnote 2 L689–696).
- `does not guarantee`: First-order cancellation for the odd members. DACP's alternative inner placement (C_0, C_1, C_2 and a dummy on the inner four, C_A on the next ring) "showed no significant difference" (DACP L822–834).
- `inputs needed`: Per-device unit counts.
- `verification`: Moment check reports the per-device centroid offset (CC-01). Extraction check: lead parasitics of the singleton vs the unit average.
- `prefer over / degrade to`: Prefer centre placement for the smallest members. Degrade to edge placement in transistor rows where diffusion sharing dominates (ALIGN).
- `source`: DACP §IV-A L674–678, §IV-C L822–834, footnote 2; REV §IV L439–441; CC-22, CC-25.

### PAT-10
- `id`: PAT-10
- `technique`: Chessboard CC placement for binary-weighted capacitor arrays: C_N (2^{N−1} units) on the black squares of the 2^N-slot grid, C_{N−1} on half of the white squares in chessboard order, alternate colours down to C_2, C_0 and C_1 at the centre (REV L338–346; DACP L638–644).
- `realizes`: Maximum dispersion (each bit spread over the whole array), which raises inter-bit correlation ρ_pq and lowers random ratio variance (CC-09, CC-10), plus CC by reflected pairs.
- `applicability`: Binary-weighted caps, N bits, 2^N units, square grid.
- `parameters`: N; which colour C_N takes; centre handling for C_0, C_1.
- `guarantees`: Order 1 by reflected pairs; all odd orders since every bit is self-symmetric [derived, NTH corollary]. Best linearity of the three DACP styles: 8-bit {|DNL|, |INL|} = {0.01, 0.06} LSB, 10-bit {0.05, 0.30}; robust at γ = 100 ppm: 10-bit {0.06, 0.34} (DACP Tables II–III).
- `does not guarantee`: Speed. 8-bit: 295 vias against 60 for the spiral, 5 vertical tracks per channel against 2, C_BB 73.3 fF against 6.1, critical-bit R_total 14.0 kΩ against 0.16, f3dB 16 MHz against 1411; 10-bit f3dB 0.8 MHz (DACP Tables I–II, L931–957).
- `inputs needed`: N, C_u; A_f, ρ_u, L_c for the random model; γ for the systematic model; per-layer r, c, c_c(s) for parasitics (CC-08, CC-34).
- `verification`: Moment check per bit (CC-02). ρ_avg and M_rand (CC-09, CC-10). 3σ INL and DNL with parasitics (CC-13). Extraction check: f3dB from the Elmore delay of the critical bit (CC-15), C_BB per channel.
- `prefer over / degrade to`: Prefer when linearity binds and speed does not. Degrade to block chessboard (PAT-12) or spiral (PAT-11) as f3dB binds (CC-24).
- `source`: REV §III-C L338–346; DACP §IV-A L638–644, §V Tables I–III; CC-20, CC-24.

### PAT-11
- `id`: PAT-11
- `technique`: Spiral CC placement for binary-weighted capacitor arrays: C_0 and C_1 diagonally opposite at the centre; then all of C_2, all of C_3, and so on along a spiral from the centre, each unit paired with its point reflection (d1, d2) → (−d1, −d2) (DACP L671–696).
- `realizes`: CC with the units of each bit contiguous in rows and columns, so bends and vias are minimal (vias only at the input connection) and f3dB is highest.
- `applicability`: Binary-weighted caps, any N, near-square grid.
- `parameters`: Spiral start direction, aspect.
- `guarantees`: Order 1 by reflected pairs; odd orders by self-symmetry [derived]. 8-bit: 60 vias, 514 µm wire, C_BB 6.1 fF, critical-bit R_total 0.16 kΩ, f3dB 1411 MHz; 10-bit f3dB 81 MHz (DACP Tables I–II). Runtime 1.11 s at 10 bits in Python (Table VI).
- `does not guarantee`: Dispersion, hence worst linearity: 8-bit {0.05, 0.08}, 10-bit {0.22, 0.31} LSB. Fails under a large nonlinear gradient: at γ = 100 ppm the 10-bit spiral gives {4.93, 2.56} LSB (DACP Table III).
- `inputs needed`: Same as PAT-10, plus γ_max for the stress check.
- `verification`: Same metrics as PAT-10, evaluated at γ_nominal and γ_max under the exact t0/t model (CC-05).
- `prefer over / degrade to`: Prefer when f3dB binds and γ is small. Degrade to block chessboard (PAT-12) when linearity or γ robustness binds (CC-24).
- `source`: DACP §IV-A L671–696, §V Tables I–III, VI; CC-05, CC-19, CC-24.

### PAT-12
- `id`: PAT-12
- `technique`: Block chessboard (BC), DACP Algorithm 1. An inner full chessboard holds C_0..C_k. Outer corridors hold bit pairs (C_i, C_{i+1}) for i = k+1..N step 2: C_i in b_s×b_s blocks in chessboard order plus their diagonal-symmetric copies, C_{i+1} in the leftover corridor cells. Assumes k and N−k even so no dummies are needed (DACP L645–786).
- `realizes`: Dispersion where accuracy comes from (LSBs and C_0) and few vias where RC comes from (MSBs), because the MSB's relative variation is already lower (σ/μ ∝ 1/√n) while its RC sets f3dB (DACP footnote 2).
- `applicability`: Binary-weighted caps with k and N−k even; square-ish grid.
- `parameters`: k (core size), b_s[i] per bit pair.
- `guarantees`: Order 1 by symmetric copies. 8-bit: {0.02, 0.07} LSB, 339 MHz, 234 vias, R_total 0.67 kΩ; 10-bit {0.11, 0.30} at 29 MHz (DACP Tables I–II).
- `does not guarantee`: Either extreme. The (k, b_s) family must be enumerated and evaluated (CC-21).
- `inputs needed`: N, k, b_s, plus the random, systematic and parasitic coefficients of PAT-10.
- `verification`: As PAT-10.
- `prefer over / degrade to`: The middle of the spiral–BC–chessboard trade (CC-24). Degrade to spiral for speed, chessboard for linearity.
- `source`: DACP §IV-A Algorithm 1 (PDF p.8), §V Tables I–II; CC-21, CC-24, CC-25.

### PAT-13
- `id`: PAT-13
- `technique`: Split-DAC CC placement with attenuation capacitor. Ratio list [1, 1, 2, …, 2^{L−1} | 1, 2, …, 2^{N−L−1}]. C_A = (C_T^LSB / C_T^MSB)·C_u, drawn as two half-slots symmetric near the centre (smaller than C_u, so most perturbed by routing C). Odd-count caps as PAT-09. Then alternate LSB and MSB bits along the spiral (C_2, C_7, C_3, C_8, …) in reflected pairs. BC variant: chessboard for the small caps, blocks for the large ones, equal block sizes for same-sized LSB and MSB caps placed diagonally symmetric (DACP §IV-C).
- `realizes`: CC for two sub-arrays sharing one array; the first automated split-DAC CC generator (DACP L123–125).
- `applicability`: Split charge-sharing DACs with parameters N and L; C_A is not an integer multiple of C_u.
- `parameters`: L, style (spiral or BC), block sizes.
- `guarantees`: 8-bit split against binary: area 362 against 2509 µm², f3dB 19977 against 1411 MHz, |DNL| 0.06 against 0.05, |INL| 0.04 against 0.08 LSB (DACP Fig. 16(c)). The split's critical-bit R is higher, so it used 2 parallel wires against 4 for binary (DACP L1009–1087).
- `does not guarantee`: C_A exactness without PAT-14. A unit bridge cap instead costs a 1-LSB gain error (DACP L77–84).
- `inputs needed`: N, L, C_u, C_A; the coefficients of PAT-10.
- `verification`: INL and DNL with the split-DAC nonideal transfer (DACP eqs. 25–31; CC-16). C_A within 1 % of ideal (CC-16 recipe). Extraction check: C_TS and C_TB per sub-array.
- `prefer over / degrade to`: Prefer over binary at N ≥ 8 when area or speed bind. Degrade to binary when the C_A precision cannot be met.
- `source`: DACP §I, §II-B, §IV-C, §V Fig. 16; CC-16, CC-23, CC-32.

### PAT-14
- `id`: PAT-14
- `technique`: Non-unit capacitor sized for equal edge sensitivity. MOM finger cap C is proportional to l·H with l = w − c (end gap c) and H = N·p. Under an edge bias δ, ΔC/C = 2δ(H + l)/(H·l). Choose l and H so that this equals the unit's value and l·H gives the target C (two equations, two unknowns; snap H to the finger pitch) (DACP §III-D eqs. 37–45).
- `realizes`: A non-integer capacitor whose systematic edge-bias error equals the unit's, so the ratio to C_u holds under process bias.
- `applicability`: Any non-unit cap; C_A is conventionally 1–2× C_u in a rectangle (DACP L607–616). MOM with w − c, MIM with w (DACP L81–84).
- `parameters`: l, H.
- `guarantees`: Equal relative edge-bias error to the unit (perimeter-to-area matching).
- `does not guarantee`: Random matching (area) or gradient (position); needs central placement (PAT-13).
- `inputs needed`: Target C, unit l_u and H_u, finger pitch p, end gap c.
- `verification`: Check (H + l)/(H·l) equals the unit's. Extraction check: C_A against design.
- `prefer over / degrade to`: Prefer over a unit bridge cap (1-LSB gain error). Degrade to a 1–2× C_u rectangle only if the solve fails to snap.
- `source`: DACP §III-D L534–616; CC-17.

### PAT-15
- `id`: PAT-15
- `technique`: ALIGN ratio-greedy multi-row CC transistor placement, LOD-aware and diffusion-shared. Multiplicity vector M → M_half; odd leftovers at the edge. Repeatedly place the device with the largest Ratio (fraction of its cells unplaced) that can share diffusion with the row end, alternating left and right of the CC column; if that device already occupies this column in another row, prefer a different device (LOD). Fill rows, then reflect the half layout about the CC point (REV §IV L445–464). Example M = [2, 2, 4, 8, 8], M_half = [1, 1, 2, 4, 4] (REV Fig. 8).
- `realizes`: CC for k ratioed devices in two dimensions with diffusion sharing and LOD diversity, without Euler-path enumeration.
- `applicability`: k devices, arbitrary integer ratios, several rows, MOS only.
- `parameters`: Row width, row count, tie-break rules.
- `guarantees`: Order 1 by reflection. Cheaper than Euler enumeration (REV L445–447). Accounts for LDE and parasitic mismatch (REV L445–448).
- `does not guarantee`: Higher orders. Exact LOD equality (heuristic). R_s equality of the interdigitated kind.
- `inputs needed`: M, row width, KVTH0 to score LOD.
- `verification`: Moment check per device (CC-01, CC-02). LOD sum per device (CC-35). Extraction check: fishbone R per device (PAT-27).
- `prefer over / degrade to`: Prefer for ratioed mirror banks. Degrade to Euler path (PAT-16) when exact diffusion sharing is mandatory. Degrade to 1-D ABBA (PAT-03) for two devices.
- `source`: REV §IV L445–464, Fig. 8(c)–(h); CC-37, CC-35.

### PAT-16
- `id`: PAT-16
- `technique`: Diffusion-sharing half graph and Euler path (Long, REV ref. [2]). Vertices are source/drain nets, edges are fingers (multiplicity = finger count). Halve the multiplicities to M_half. An Euler path on M_half is a diffusion-shared ordering of half the fingers; reflect it about the CC point (REV L396–444).
- `realizes`: Diffusion sharing along the whole row with CC by reflection.
- `applicability`: k devices with even finger counts (odd counts put a cell at the edge). One row or hierarchical rows.
- `parameters`: Path choice among Euler paths.
- `guarantees`: No diffusion break when the half multigraph has an Euler path (at most two odd-degree vertices).
- `does not guarantee`: LOD equality, routing. Enumerating all Euler paths is expensive (REV L445–447).
- `inputs needed`: Finger counts and net connectivity.
- `verification`: Diffusion-break count is zero. Moment check per device. Extraction check: none specific.
- `prefer over / degrade to`: Degrade to the ALIGN greedy (PAT-15) for speed. Insert dummies or gaps when more than two vertices have odd degree (CC-38).
- `source`: REV §IV L396–444, Fig. 8(b); CC-38.

### PAT-17
- `id`: PAT-17
- `technique`: Stochastic and ILP unit assignment for arbitrary ratios. Pair-sequence SA (REV refs. [13], [46]): the placement is a list of unit pairs symmetric about the CC point ordered from the innermost ring outward; three perturbation operators; a repair step restores adjacency for non-integer ratios. C-CBL SA over a global sequence pair ([45]). ILP ([38]) with exclusivity, exact ratio counts, and one track per unit in one of its four adjacent channels. GA over a CP-sequence ([27]) with fitness Φ = (α·C_u/C_u,max + β·DNL_max/0.5 + γ·INL_max/0.5)^{−1}, infinite penalty when DNL or INL exceed 0.5 LSB, plus a shield ILP (REV §III-C L323–377).
- `realizes`: CC with maximised dispersion or routability for ratio sets that have no constructive pattern.
- `applicability`: k devices, non-binary ratios, any grid.
- `parameters`: Move set, cost weights, iteration budget.
- `guarantees`: CC by construction (symmetric pairs). Routability when [46]'s simultaneous placement and global routing is used (target one track per channel).
- `does not guarantee`: Optimality or a runtime bound. [13] and [45] ignore routing (REV L359–383).
- `inputs needed`: Ratios, grid, ρ model, cost weights.
- `verification`: Moment check; ρ_avg and M_sys (CC-06, CC-09); tracks per channel. Extraction check: per-bit lead parasitics.
- `prefer over / degrade to`: Use only when PAT-10 to PAT-13 do not apply; seed from the constructive pattern (CC-43).
- `source`: REV §III-C L323–377, eq. 24; CC-43.

### PAT-18
- `id`: PAT-18
- `technique`: Identical unit cells with an equal layout-dependent-effect environment. Every matched device is built from one unit master on a common grid: uniform poly pitch, uniform OD width, row-based y-direction OD spacing, diffusion-shared x spacing; well edges far away or equidistant; equal LOD sums (REV §II-A-2).
- `realizes`: Cancellation of every LDE except LOD and WPE, which remain for placement and dummies (REV L135–145). Mask-induced mean W/L errors become common mode (PEL L64–67).
- `applicability`: All MOS matched sets; mandatory for on-grid FinFET and GAA layouts (REV §VI L510–522).
- `parameters`: Unit W, L, nf; pitch; well margin.
- `guarantees`: OD spacing and pitch equality by construction. LOD equality when Σ_i [1/(SA_i + 0.5L_g) + 1/(SB_i + 0.5L_g)] is equal across devices (REV eq. 11). WPE equality when channel-to-well-edge distances are equal (REV L149–152).
- `does not guarantee`: Gradient cancellation. A single-row CC inherently breaks LOD (REV §V-A L428–437).
- `inputs needed`: KVTH0 for LOD; a WPE coefficient (not given by any source); well geometry.
- `verification`: LOD sum per device; well distance per device on four sides (CC-36). Extraction check: none.
- `prefer over / degrade to`: Always. Mitigate LOD with end dummies or the two-row swap (PAT-05).
- `source`: REV §II-A-2 eq. 11 L98–145, §V-A; PEL L64–67; CC-35, CC-36, MM-13.

### PAT-19
- `id`: PAT-19
- `technique`: Common orientation and signed current direction for every unit of a matched set; no 90° rotation inside a set.
- `realizes`: Equal current factor β. Rotation affects only β, and A_β for a rotated pair "varies considerably with wafer and batch", attributed to mobility (PEL L98–110, Fig. 3d). For narrow poly resistors, orientation relative to the wafer flat decides whether long-distance mismatch appears at all (LDM Fig. 10).
- `applicability`: All matched MOS and resistors.
- `parameters`: None.
- `guarantees`: Removes the orientation β term. In-line and side-by-side placement showed no measurable difference at 2.5 µm (PEL L122–124), so the arrangement itself is free.
- `does not guarantee`: Anything about gradients.
- `inputs needed`: None.
- `verification`: Structural orientation check. Extraction check: none.
- `prefer over / degrade to`: Hard constraint; no degrade.
- `source`: PEL §Matching L98–124; LDM §III L300–313; MM-10, MM-11, MM-27.

### PAT-20
- `id`: PAT-20
- `technique`: FinFET gate-misalignment-aware orientation balancing. Each unit's orientation (source left or right) is a variable; equal, or ratio-proportional, counts per device so the printed-gate shift cancels per device. Placement stays diffusion-sharing aware and dispersion maximising; routing is a parasitic-aware MST (REV §IV L408–423, refs. [50], [55]).
- `realizes`: Matched current ratios under gate misalignment, which raises V_th and lowers I_D.
- `applicability`: FinFET mirrors and diff pairs.
- `parameters`: Orientation per unit.
- `guarantees`: First-order cancellation of the misalignment shift per device.
- `does not guarantee`: The sensitivity coefficient is not given (REV).
- `inputs needed`: Misalignment coefficient from the deck (absent in all sources).
- `verification`: Orientation count per device. Extraction check: current ratio under a misalignment corner if the model exists.
- `prefer over / degrade to`: Combine with PAT-15; no degrade.
- `source`: REV §IV L408–423; CC-42.

### PAT-21
- `id`: PAT-21
- `technique`: Unit capacitor structure choice. Finger MOM: highest density, FinFET-friendly, exposes C_TS and C_TB. Sandwich, pillar and mortise-tenon: top plate enclosed by the bottom plate, no C_TS, easier routing, lower density. MOM over MIM in multi-metal and FinFET nodes (REV §III-B; DACP L102–111).
- `realizes`: The array's parasitic class; an enclosed structure shields the top-plate node.
- `applicability`: All capacitor arrays.
- `parameters`: Structure, layers, finger count.
- `guarantees`: Enclosed structures eliminate C_TS (REV L328–330). A parameterised mortise-tenon generator exists (REV ref. [28]).
- `does not guarantee`: Density. The unit's own matching is set by area: σ_u² = A_f²/(W·L) (REV eq. 21).
- `inputs needed`: Metal stack; the sensitivity class of the top-plate net (CC-14).
- `verification`: Extraction check: C_TS and C_TB per bit (CC-14). Moment check not applicable.
- `prefer over / degrade to`: Enclosed when the top plate is sensitive and area permits; finger otherwise.
- `source`: REV §III-B L301–330, Fig. 5; DACP §I L102–111; CC-14, CC-33.

### PAT-22
- `id`: PAT-22
- `technique`: Routing: connected same-bit groups with mirrored branches. Build a graph of units with edges between 4-neighbours of the same bit; BFS gives the components (groups) and a BFS tree per group; each tree edge becomes a same-layer branch wire; every connection is mirrored to the point-symmetric location (DACP §IV-B-1 L748–761, Fig. 7).
- `realizes`: Minimal branch wiring and symmetric lead parasitics per bit.
- `applicability`: Any gridded unit array.
- `parameters`: Adjacency (4-neighbour), branch layer.
- `guarantees`: Symmetric branch wiring per reflected pair; via-free branches inside a group.
- `does not guarantee`: Connection between disjoint groups (needs trunks, PAT-23).
- `inputs needed`: Placement.
- `verification`: Branch length symmetric per bit; group count per bit (which is the trunk demand). Extraction check: branch R and C per bit.
- `prefer over / degrade to`: Always inside arrays; followed by PAT-23.
- `source`: DACP §IV-B-1; CC-26.

### PAT-23
- `id`: PAT-23
- `technique`: Routing: trunk and bridge bottom-plate routing with track minimisation. Branch wires join units inside a group or a group to a trunk; trunk wires are vertical channel tracks joining disjoint groups; bridge wires join trunks at the periphery, symmetrically. Steps: channel selection, track assignment, routing (DACP §IV-B-2 L763–789). Routability analysis finds overlapping channel spans and minimises the maximum tracks per channel with a target of one (REV L350–361, ref. [46]).
- `realizes`: Equal-length-by-symmetry bottom-plate leads at minimum channel width.
- `applicability`: Binary and split cap arrays; bottom plates exit at the array bottom to a clustered switch bank.
- `parameters`: Channel capacity, bit order of bridges.
- `guarantees`: Fewer tracks means a narrower pitch and lower C_BB: 8-bit spiral 6.1 fF against chessboard 73.3 fF (DACP Table I).
- `does not guarantee`: f3dB unless via count is controlled (PAT-25).
- `inputs needed`: Groups from PAT-22, channel pitch.
- `verification`: Maximum tracks per channel. Extraction check: C_BB per bit, trunk R per bit.
- `prefer over / degrade to`: Always after PAT-22; widen the channel pitch only where demand exceeds one (CC-27).
- `source`: DACP §IV-B-2, Table I; REV §III-C L350–361, Fig. 7; CC-27.

### PAT-24
- `id`: PAT-24
- `technique`: Routing: non-overlapped top and bottom plate wiring, and a top-plate MST. Keep top-plate and bottom-plate wires on disjoint tracks and layers so they never overlap or run adjacent (DACP L791–795). Route the top plate as the MST over unit adjacency weighted by spacing: vertical branches per column, one horizontal branch per adjacent column pair (DACP §IV-B-3 L813–826). Split DAC: BFS joins adjacent groups, an MST joins the remaining groups of each side array, and C_A lands on adjacent top plates of each side (DACP §IV-C-3).
- `realizes`: Removes code-dependent nonlinearity (C_TB) and minimises gain error (C_TS) (CC-14).
- `applicability`: All capacitor arrays.
- `parameters`: Layer assignment per plate.
- `guarantees`: C_TB negligible (DACP Table I omits it for this reason). Minimum top-plate wire length.
- `does not guarantee`: Symmetry where a plate must cross the other; a shield is needed there (PAT-26).
- `inputs needed`: Placement, layer stack.
- `verification`: Extraction check: C_TB per bit near zero, total C_TS. Moment check not applicable.
- `prefer over / degrade to`: Always; the fallback is a shield tied to a quiet net.
- `source`: DACP §IV-B L791–826, §IV-C-3 L860–913; CC-14, CC-28, CC-29.

### PAT-25
- `id`: PAT-25
- `technique`: Routing: parallel wires for the critical bit on quantised-width layers. Route the bit with the largest Elmore delay with k parallel minimum-width wires, which gives parallel vias at every direction change; after the MSB is paralleled the next bit may become critical, so iterate; widen the column spacing when the channel lacks room (DACP L790–795, L996–1013).
- `realizes`: An emulated wide wire where widths are quantised; an f3dB target.
- `applicability`: FinFET and other unidirectional-layer processes; any high-R net.
- `parameters`: k per bit; channel pitch.
- `guarantees`: k = 2 gives more than 2× f3dB, between 2× (wire-dominated) and 4× (via-dominated 2×2 mesh), with diminishing returns as k grows because wire C rises (DACP Fig. 15). Binary 8-bit used 4 wires, split used 2 (DACP L1051–1058).
- `does not guarantee`: Chessboard stays via-bottlenecked (DACP L1013).
- `inputs needed`: Per-bit RC tree, per-layer r and c, via R.
- `verification`: f3dB = 1/(2(N+2)·ln 2·τ) with τ the worst Elmore delay over bits (DACP eq. 32; CC-15). Extraction check: critical-bit R_total and via count (CC-31).
- `prefer over / degrade to`: Prefer over a wide wire wherever widths are quantised; stop at the first k meeting the target.
- `source`: DACP §III-C eq. 32, §IV-B, §V Fig. 15; CC-15, CC-30, CC-31.

### PAT-26
- `id`: PAT-26
- `technique`: Routing: ratio-proportional lead parasitics and ratio-preserving shields. The hybrid chessboard (PASTEL, REV ref. [43]) matches each bit's routing wirelength to its capacitor ratio so lead C scales with C_k; a shield ILP (REV ref. [27]) places shields so that parasitic-inclusive ratios stay ideal (REV L349–353, L375–377).
- `realizes`: Lead-capacitance matching: C_k/C_j unchanged by routing parasitics.
- `applicability`: Ratioed cap arrays including non-binary ratios; resistor ladders by analogy.
- `parameters`: Dummy wire length per bit, shield coverage per bit.
- `guarantees`: Ratio preservation to the extent that L_k/n_k is constant over bits.
- `does not guarantee`: Minimum wire; added length trades against f3dB.
- `inputs needed`: Per-bit wire length and unit count n_k.
- `verification`: Spread of L_k/n_k (max/min − 1) (CC-44). Extraction check: lead C per bit proportional to n_k.
- `prefer over / degrade to`: Prefer for non-binary ratio banks (SC filters, gain caps) where INL and DNL are undefined (CC-06). Degrade to symmetric mirrored routing (PAT-22) for binary arrays.
- `source`: REV §III-C L349–353, L375–377; CC-06, CC-44.

### PAT-27
- `id`: PAT-27
- `technique`: Routing: fishbone EM/IR-aware routing of CC transistor arrays. A spine plus ribs per device terminal with wire widths optimised against electromigration and IR drop (ALIGN, REV L448–452). Alternative: parasitic-aware MST (REV ref. [50], L422–423).
- `realizes`: Equal R per matched device at an EM-safe width.
- `applicability`: Multi-row MOS arrays.
- `parameters`: Spine layer, width per current.
- `guarantees`: EM and IR compliance; symmetric R when spines mirror.
- `does not guarantee`: In a single-row CC, A and B source/drain wire lengths inherently differ, and in FinFET nodes unidirectional layers forbid detours while vias add large R jumps (REV L438–447).
- `inputs needed`: Device currents, stack EM limits, sheet resistances.
- `verification`: Spine R per device. G_m with R_s: G_m = g_m(v_in − i_ac·R_s)/v_in, compared between A and B (REV eq. 25; CC-41). Extraction check: R from each source to the AC ground node.
- `prefer over / degrade to`: Prefer for arrays carrying real current; degrade to a plain MST for tiny devices.
- `source`: REV §IV L448–452, L422–423, §V-A L438–465; CC-39, CC-41.

### PAT-28
- `id`: PAT-28
- `technique`: Kelvin (force/sense) connection for a shared reference node. Route the shared current-carrying node (source rail, gate reference) as a star from one tap, or add a separate sense branch that carries no current, so interconnect IR drop does not appear as a device offset (LDM §II Fig. 4).
- `realizes`: Removes a systematic offset that masquerades as a gradient: 90 mΩ/□ interconnect at 320 µA drops 8.3 mV/mm, against a pair σ of about 1.6 mV (LDM L180–204, Table II).
- `applicability`: Any matched set sharing a current-carrying node over more than about 100 µm; resistor ladders; mirror banks; comparator arrays.
- `parameters`: Tap location, sense branch route.
- `guarantees`: Regulation error below 10 µV in LDM's structure (L180–204).
- `does not guarantee`: Gradient or random terms.
- `inputs needed`: Current per member, sheet resistance, route lengths and widths.
- `verification`: ΔV = I·R□·Δ(length/width) within the systematic allowance (MM-33). Extraction check: common-node R per member.
- `prefer over / degrade to`: Prefer for long-span sets; degrade to a symmetric tree when currents are tiny.
- `source`: LDM §II L47–89, §III L180–204; MM-33.

## 2. Theory

### 2.1 Gradient model and the moment condition

- A device parameter at a point follows a polynomial field: a constant plus, for each order i from 1 to n, the sum over j of coefficient g_{j,i−j} times x^j y^{i−j} (NTH eqs. 1–4).
- A unit cell is small enough that the field inside it is negligible, so the unit carries the field value at one representative point, and a device's parameter is the sum of the field over its unit positions (NTH eq. 5). Ideal matching is equality of the two sums (NTH eq. 6).
- The difference between devices A and B is therefore the sum over all coefficients of g_{j,i−j} times the difference of the devices' moments Σ x^j y^{i−j} over their units (CC-02, from NTH eqs. 3–6). A layout cancels every gradient of order up to n, for arbitrary coefficients, if and only if every mixed moment with j + (i−j) ≤ n is equal between the devices. For ratioed devices the moments are per-unit means (REV eq. 1).
- Order 1 is centroid coincidence: equal mean x and equal mean y (NTH eqs. 13–16; REV eqs. 1–3). Interdigitation (ABAB) does not satisfy it (REV §I L73–78).
- Order 2 needs three equalities, in x², xy and y² (CC-02). Equal mean radius squared alone is not enough: it misses the x² − y² and xy terms.
- The expansion centre of the top-order term can be moved anywhere; moving it adds only lower-order terms (NTH eqs. 7–12). This is why a recursive construction can cancel the top order about a new centre without disturbing the orders already cancelled.
- Parity corollary (NTH §III L216–222): if each device's units are centrally symmetric about one point by themselves, every odd order cancels; if device A's units are the point reflection of device B's, every even order cancels. Any reflected-pair construction (spiral, chessboard, block chessboard, ALIGN's reflected half) makes each device self-symmetric and so cancels all odd polynomial orders; its even-order behaviour depends on dispersion [derived].
- Nth-order construction: two (n−1)st-order patterns point-symmetric about a new centre, labels swapped when n is even; 2^n units per device; m even for n > 1 (NTH §III). Proof by induction over eqs. 17–23.
- Gradient sign and direction are random across wafers and chips (LDM L185–201; PEL L33–38 treat the deterministic wafer cone as stochastic because the die position is unknown; DVP L8–13 call it a statistical gradient parameter). The centroid must coincide in both axes; a 1-D interdigitation cancels one axis only (MM-32).

### 2.2 Pattern against cancelled gradient order

Residuals are NTH Table I, in percent mismatch, for a gradient containing all orders up to the column's order (columns are cumulative, so a residual at column k includes every uncancelled order below k). Gradient magnitudes are not given (NTH §IV).

| Pattern | Units per device | Grid | Proved cancelled orders | Parity bonus | Residual at cumulative order 1 / 2 / 3 / 4 / 5 | Source |
|---|---|---|---|---|---|---|
| Clustered | any | — | none | — | not simulated | REV §V |
| Interdigitated ABAB | any even | 1 row | none (centroid offset of half a pitch) | — | not simulated | REV §I L73–78 |
| 1-D ABBA | 2 | 1×4 | 1 | all odd (self-symmetric) | not in Table I; x² moments differ 9:1 [derived] | NTH Fig. 1(a); CC-02 |
| Cross-quad | 2 | 2×2 | 1 | all odd; x² and y² also cancel, xy does not [derived] | 0 / 2.77 / 5.22 / 7.43 / 10.39 | NTH Table I (a) |
| 2nd-order central symmetric | 4 | 2×4 or 1×8 | 1–2 | all even (swap-symmetric) | 0 / 0 / 0.24 / 0.87 / 1.70 | NTH Table I (b) |
| 3rd-order central symmetric | 8 | 4×4 or 2×8 | 1–3 | all odd | 0 / 0 / 0 / 0.01 / 0.068 | NTH Table I (c) |
| 4th-order central symmetric | 16 | 8×4 | 1–4 | all even | 0 / 0 / 0 / 0 / 0.0023 | NTH Table I (d) |
| 5th-order central symmetric | 32 | 8×8 | 1–5 | all odd | 0 / 0 / 0 / 0 / 0 | NTH Table I (e) |
| 2nd-order circular symmetry | 4 | circle | 1–3 (self-symmetric) | all odd | 0 / 0 / 0 / 0.026 / 0.18 | NTH Table I (f), §IV |
| Hexagonal tessellation | 4 | hexagon | 1–2 | — | 0 / 0 / 0.26 / 0.50 / 2.24 | NTH Table I (g) |
| Spiral, chessboard, block chessboard (reflected pairs) | 2^{k−1} per bit | near square | 1 | all odd [derived]; even orders by dispersion | see DACP Table III in §4 | DACP §IV-A |

### 2.3 Non-polynomial and statistical field models

- Oxide-thickness gradient for capacitors: t = t0 + γ(x cos θ + y sin θ), capacitance proportional to t0/t, with γ and θ fixed per array and random across arrays (REV eqs. 8–9; DACP eq. 11). The 1/t form has even-order terms in its expansion, so a centroid-matched array with poor dispersion still fails as γ grows: at γ = 100 ppm the 10-bit spiral reaches |DNL| 4.93 and |INL| 2.56 LSB while the chessboard gives 0.06 and 0.34 (DACP Table III; CC-05). The systematic metric is M_sys, the worst relative error of any capacitor ratio over θ (REV eq. 10; CC-06).
- Variation decomposition: ΔP = global + uncorrelated + spatially correlated; variances add (REV eqs. 6–7). Global variation causes no mismatch. Correlated variation for active devices follows ρ(r) = (σ_g² + exp(−(r/R_L)²)·σ_s²)/σ_P² with correlation distance R_L (REV eqs. 17–19). For capacitors ρ_ab = ρ_u^{D} with D the Euclidean distance over L_c (REV eq. 12; DACP eqs. 13–14). Higher dispersion raises the correlation between ratioed capacitors and lowers their ratio variance (REV L318–322; CC-09). The sum-of-correlations closed form for the ratio variance is var(C_p/C_q) = A_f²/(C_u² q⁴ W L)·[q²(p + 2S_p) + p²(q + 2S_q) − 2pq S_pq], with S the pairwise correlation sums (REV eq. 16; CC-10).
- Long-distance profiles measured on silicon range from none, to linear, to quadratic over a few millimetres, and they change wafer to wafer and chip to chip (LDM L181–201, Fig. 11). A quadratic profile is cancelled only by matching second moments; no source gives its coefficient (MM-26).

## 3. Mismatch sizing

Formulas as plain statements. All A constants are pair constants: they give the σ of the difference of two equal devices (PEL eq. 1; LDM eq. 1; DVP eq. 1; MM-02).

- Random plus distance (Pelgrom): the variance of the pair difference equals A_P² over the gate area W·L plus S_P² times the spacing D squared (PEL eq. 1; MM-01). Validity: many small additive events whose correlation distance is far smaller than the device (PEL L22–28). A single device has σ equal to the pair σ over √2 (LDM eq. 4). For two devices of unequal areas a1 and a2 the pair variance is (A²/2)(1/a1 + 1/a2) (MM-02).
- Unit count against unit size: a device built from n parallel units has σ falling as √n relative to one unit; an 80 µm device drawn as four 20 µm units has half the σ of one 20 µm unit (DRE §III-C; MM-19). So the random term is set by total drawn area, and the split into units is free for the pattern to decide: 2^n units per device for an nth-order pattern (PAT-06), 2^N for a binary DAC, integer ratios for mirrors [derived consequence]. Unit area is total area over unit count.
- Electrical domain: for a mirror at equal V_GS, σ²(ΔI/I) = (g_m/I_D)²·σ²(ΔV_T) + σ²(Δβ/β); for a diff pair at equal current, σ²(V_os) = σ²(ΔV_T) + (I_D/g_m)²·σ²(Δβ/β) (from PEL eq. 2; DRE eq. 4; LDM eq. 3; MM-09). V_T dominates for overdrive below about 0.2–0.3 V: at 0.2 V the V_T term is over 90 % of the variance (LDM L140–144; PEL L97; MM-07). Correlation between ΔV_T and Δβ is about zero for close pairs and −0.35 at 250–500 µm spacing (PEL L111–119). The square-law sum can overestimate σ_Id by up to 2× because V_T and β share the oxide-thickness dependence (DRE L131–135).
- Reference constants: PEL at 2.5 µm, 50 nm oxide: A_Vt0 30 (n) / 35 (p) mV·µm, A_β 2 / 3 %·µm, S_Vt0 about 4 µV/µm; reproducible to 5 % in a batch and 20 % across batches and factories (PEL Table 1; MM-05). LDM pair A_ΔVGS: 19 / 8.75 / 6.7 mV·µm nMOS at 0.5 / 0.35 / 0.25 µm (LDM Table I). A_Vt and A_K scale with oxide thickness; A_β does not (PEL L119–121; MM-12).
- Area-law limits: V_T mismatch departs from 1/√area for wide/short and narrow/long devices because V_T depends on L and W through short-channel, reverse short-channel, narrow-width and halo effects; V_T is not a proper mismatch parameter and σ_Vt is not the input offset (DRE L15–20, L83–138; MM-03, MM-08). Characterisation and application should be in the same SPICE domain with independent physical parameters (DRE §II, §IV; MM-29). The β law has edge terms A_W²/(W²L) and A_L²/(WL²) that matter as W and L shrink (PEL eq. 3; DRE eqs. 1–3; MM-04).
- Mirror sizing levers: under current bias, raising L lowers intrinsic mismatch and raises overdrive, so matching improves twice; raising W lowers intrinsic mismatch but lowers overdrive, so it "can give rise to little or no improvement"; mirror mismatch "depends strongly on L and not W" (DRE §III-A; MM-21). With integer ratios, 1 reference : 4 outputs gives 1.52 % against 3.05 % for 4 : 1 and 3.86 % for 1 : 1 at 2×2 µm², 0.13 µm; the multiplicity belongs in the output device (DRE Table III; MM-20).
- Capacitor units: σ_u/C_u = A_f/√(W·H) (REV eq. 21; DACP eq. 13). A capacitor of p units has variance σ_u²(p + 2S_p), where S_p sums the pairwise correlations inside it (REV eq. 13). The DAC transfer takes 3σ of the switched-on capacitance and of the total, plus the systematic ΔC from the t0/t gradient, plus C_TB and C_TS, into every code (DACP eqs. 19–24; CC-13), and the spec is |DNL| and |INL| within ±0.5 LSB (REV L263–268; DACP eqs. 17–18; CC-12). Unit size is then the smallest C_u that passes at the nominal and a stress γ with the chosen pattern; a larger C_u reduces mismatch and parasitic impact but raises power and area, and minimising routing-parasitic mismatch permits a smaller C_u (REV L286–299). DACP's working points: C_u = 5 fF for 6–12 bits and 10 fF for 14 bits with A_f² = 0.85 %·1 fF, ρ_u = 0.9, L_c = 1 mm, γ = 10 ppm (DACP L909–911, Table V). The MSB's relative variation is lower (σ/μ ∝ 1/√n), so accuracy is set by the LSBs and by C_0, which every ratio depends on (DACP footnote 2; CC-25).
- When distance binds: the distance term equals the area term at D* = A/(S·√(WL)) (MM-06). PEL: spacing can be ignored for gate areas below 100 µm², and matters "only for large area devices with a considerable spacing" (PEL L88–90, L150–151). With PEL constants D* is 7.5 mm over √(WL in µm²) [derived]. The spacing coefficient grows as L shrinks: DVP at 130 nm gives S_D = 1.63, 0.78–0.94, 0.58–0.64 mV/mm at L = 0.12, 0.24, 0.48 µm, with S_D² proportional to 1/L_eff² (DVP Table 1, Fig. 8; MM-24). A least-squares fit is S_D² = 0.1835 + 0.03533/L² in (µV/µm)² with L in µm [derived, MM-25]. Resistor S_D depends only weakly on width at 130 nm (0.163–0.212 %/mm, DVP Table 1) but strongly at 0.35 µm (0.19 to 1.2 %/mm for W 2.2 to 0.55 µm, LDM Table II). The span at which the gradient equals the pair σ is x* = σ_pair/|ΔP/Δx|: 16 / 1.4 / 0.16 mm for 0.35 µm nMOS at L = 1.71 / 0.71 / 0.35 µm [derived from LDM Table II; MM-34]. Long-distance mismatch can exceed pair mismatch over 1 mm for short-channel transistors and narrow resistors (LDM §IV). The coefficient is anisotropic: narrow poly resistors parallel to the wafer flat show none, perpendicular ones show dominant long-distance mismatch on the same chip (LDM Fig. 10; MM-27).
- Systematic allowance: pair and distance parts are independent, so with a total 1σ budget σ_b and random floor σ_rand the systematic share is √(σ_b² − σ_rand²), and the allowed centroid separation for a pair is that share over S (DVP eq. 1; MM-31). When σ_rand alone exceeds the budget, layout cannot fix it; the gate-area multiplier is (σ_rand/σ_b)². PEL needed 9× gate area, 0.1 mm², to go from 1.7 to 0.6 mV op-amp offset (PEL L143–145; MM-14). The budget is spent once across all systematic terms, not once per rule (MM-31).
- Error ledger: random terms add in quadrature; deterministic signed terms (gradient at a known direction, thermal, LOD, WPE, common-node IR drop) add linearly as a mean shift (PEL's band-gap decomposition, L128–142; MM-15, MM-16). Systematic effects show as a shift or skewness of the normal probability plot; independent random events give centred symmetric distributions (DVP L86–111; MM-36).
- From mismatch to yield (Graeb): a layout-induced systematic shift is a mean shift of a device's local statistical parameter (Graeb §3.9 L2334–2337, §4.8.5 eq. 143; GRAEB-02). The covariance is a global block plus an independent local diagonal whose entries shrink with gate area (Graeb eq. 61; GRAEB-03; the book's "and increasing distance" is a wording error against the Pelgrom model it cites, §3.9 L2308–2309). A spec's σ_f is the root sum of squared sensitivities times local variances (Graeb eq. 175; GRAEB-06). The worst-case distance β is the spec margin over σ_f; 3 is a three-sigma design; partition yield is Φ(β) (Graeb eqs. 177, 300–302; GRAEB-07, GRAEB-08). The margin a layout may spend on parasitics and systematic shifts is the schematic margin minus β_target·σ_f (GRAEB-16). Mismatch-sensitive specs have semidefinite Hessians with two worst-case vectors in opposite directions per pair; a layout offset shifts the ridge, lowering β on one side, so the smaller side counts (Graeb §5.4 L4355–4368, §6.3.1, §6.3.4; GRAEB-19). Mismatch-producing pairs are detectable from sensitivities: parameter similarity near −1 with equal magnitudes (Graeb eq. 74; GRAEB-20). Per-device allowances follow a tolerance assignment: maximise the sum of allowances subject to each spec's sensitivity-weighted sum staying within (β_schematic − β_target)·σ_f (Graeb §4.8.6, §7.2.6; GRAEB-38). Never stack ±kσ corners across devices: a ±3σ box corresponds to 4.2 to 19.0 σ_f (Graeb Table 13; GRAEB-27).

## 4. Measured evidence

### 4.1 Silicon measurements (model, not patterns)

- Area law: variances of V_t, β and K scale as 1/area at 2.5 µm (PEL Fig. 3; Table 1). Distance matters only for large-area devices with large spacing; S_Vt0 about 4 µV/µm (PEL Fig. 2a, L88–90).
- Placement arrangement: no significant difference between parallel and in-line placement, nor from 100 °C heating, scratch protection, or implant dose (PEL L122–124). Rotation by 90° affects only β and varies strongly by wafer and batch (PEL L98–110, Fig. 3d).
- Correlation: ΔV_t and Δβ uncorrelated for close pairs, −0.35 at large spacing (PEL L111–119).
- Geometry dependence: V_t mismatch fails 1/√area for wide/short and narrow/long devices; DOEs near L = W hide this (DRE L15–20, L129–138). Mirror matching set by L, not W, under current bias (DRE Figs. 6–8). nMOS against pMOS ranking depends on bias: at 10 µA current bias pMOS 1.09 % beats nMOS 1.22 %; voltage-driven at 1.70 V nMOS 0.589 % beats pMOS 1.09 % (DRE Table II). Ratioed mirrors: 1:1 3.86 %, 1:4 1.52 %, 4:1 3.05 %, 4:4 1.41 % (DRE Table III).
- Long-distance mismatch across 0.5, 0.35, 0.25 µm: negligible for long-L transistors and wide resistors; linear and sometimes quadratic for short-L and narrow devices; 0.35 µm nMOS gradient 0.1 / 0.8 / 9.8 mV/mm at L = 1.71 / 0.71 / 0.35 µm against pair σ 1.6 / 1.1 / 1.6 mV (LDM Table II). Orientation to the wafer flat switches the effect on and off for narrow poly resistors (LDM Fig. 10). Quadratic profiles over a few mm in all three technologies, which "cannot be compensated by means of circuit techniques" (LDM Fig. 11, L315–320).
- 130 nm distance coefficients: S_D grows as L shrinks with S_D² proportional to 1/L_eff²; resistor S_D 0.163–0.212 %/mm with weak width dependence; resistor A_R about 1.70 %·µm across five geometries [derived] (DVP Table 1, Fig. 8; MM-35).
- Routing as a false gradient: unregulated measurement through 90 mΩ/□ interconnect at 320 µA showed 8.3 mV/mm, several times the pair σ; force/sense regulation removes it to below 10 µV (LDM L180–204, Fig. 5).

### 4.2 Simulated pattern rankings

- Gradient order: NTH Table I (§2.2 table). Fifth-order 8×8 is zero through order 5; hexagonal only through order 2; circular symmetry through order 3 (NTH §IV).
- Capacitor array styles at 8 bits, 12 nm models (DACP Tables I–II): linearity chessboard {0.01, 0.06} > block chessboard {0.02, 0.07} > spiral {0.05, 0.08} LSB; speed spiral 1411 > BC 339 > Lin 2013 [1] 56 > chessboard 16 MHz; vias 60 / 234 / 60 / 295; C_BB 6.1 / 15.8 / 17.6 / 73.3 fF. At 10 bits chessboard {0.05, 0.30} at 0.8 MHz, BC {0.11, 0.30} at 29 MHz, spiral {0.22, 0.31} at 81 MHz. All within 0.5 LSB at γ = 10 ppm. At γ = 100 ppm the 10-bit spiral fails ({4.93, 2.56}) while the chessboard holds ({0.06, 0.34}) (Table III). The dependence on ρ_u is non-monotonic (Table IV). Split 8-bit: 362 µm² and 19977 MHz against 2509 µm² and 1411 MHz for binary with comparable linearity (Fig. 16(c)).
- Routing effects: parallel wires give more than 2× f3dB at k = 2 with diminishing returns (DACP Fig. 15). Chessboard needs 5 vertical tracks per channel against 2 for the spiral (DACP L931–957). Non-overlapped plate routing makes C_TB negligible (DACP L937–938). C_TS gives gain error, C_TB code-dependent nonlinearity, C_BS only power and f3dB (DACP §II-A; CC-14).
- Transistor rows at 14 nm (REV §V): effective G_m ranking ID > CC > clustered because of R_s (REV eq. 25). Single-row CC has inherent LOD mismatch and S/D wire mismatch (REV L428–447). 5T-OTA: the optimised layout (4-row CC for the 46 µm DP and 18.4 µm NMOS mirror, clustered 2.3 µm PMOS mirror) has the best σ and a good μ; the all-CC layout is worse because the small PMOS mirror in one CC row has high parasitic and LOD mismatch (REV L520–531). StrongARM (devices 1.6–6.1 µm): clustered is optimal; CC adds X/Y capacitance mismatch (REV L481–498). At R_L = 1000 µm the uncorrelated term dominates and the pattern barely matters; at R_L = 10 µm clustering is clearly worse (REV L476–480).

### 4.3 Dummies

- No source measures a dummy effect. REV states that an outer dummy ring "often suppresses edge (fringe) effects" (L309–314) and that FinFET/GAA stress "calls for dummies" (L510–522). LOD mismatch in a single CC row is fixed by end dummies or the two-row swap (REV §V-A; CC-35). DACP pairs C_0 with a dummy in the split DAC and found no significant difference between two inner dummy arrangements (L822–834).

## 5. Open problems named by the papers

- When CC beats non-CC remains open; the answer depends on unit device, uncorrelated-to-systematic ratio and parasitics (REV abstract L23–26, §VI L504–509).
- Well-matched, low-capacitance, low-parasitic unit structures are still sought (REV L504–509). FinFET and GAA impose on-grid, fixed-pitch, unidirectional lower metals where every turn costs a via, and stress needs dummies (REV L510–522).
- Capacitor-array algorithms do not transfer to transistors (diffusion sharing, LDE); constructive transistor patterns do not route or share diffusion; dispersion work is limited to two transistors (REV §IV L381–395). REV gives no formula for "dispersion" (CC-09 note L388–391).
- No measured spatial correlation or gradient magnitude exists for capacitors; DACP's ρ_u, L_c and γ are inherited from older papers (DACP L1014–1019). The ρ_u dependence is non-monotonic (Table IV). The 1/t nonlinearity defeats linear cancellation at large γ (Table III).
- The nth-order pattern covers two identical devices only; ratioed sets, more than two devices, and odd counts are not addressed; m must be even for n > 1; gradient magnitudes in Table I are not stated (NTH §III, §IV).
- V_t is not a proper mismatch parameter; mismatch must be characterised in the SPICE domain over bias and geometry including weak inversion, short channel and halo devices, and DOEs must span wide/short and narrow/long (DRE §I, §II, §IV).
- Quadratic long-distance profiles cannot be compensated by circuit techniques and have no published coefficient; profiles change wafer to wafer (LDM L181–201, L315–320).
- Accuracy limits set by local mismatch must be reconsidered for devices far apart; S_D for pMOS and for β is not given (DVP §Conclusion; MM-28).
- The cause of the wafer- and batch-dependent A_β under rotation is attributed to mobility but not resolved (PEL L98–110).
- Graeb: the sizing-rule constraints are referenced ([58]) but their thresholds are not given; for mismatch parameters the geometric yield problem has spurious stationary points and "suitable measures have to be taken" (Graeb §6.3.4 L5950–6010; GRAEB-40, GRAEB-45).
