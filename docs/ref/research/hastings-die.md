# Hastings 3e: technique cards for bipolar/diode construction, ESD, floorplanning, guard rings and reliability

Sources (digests of R. A. Hastings, *The Art of Analog Layout*, 3rd ed.): `ref-hastings-09` (ch. 9–11, items H09-xx), `ref-hastings-14` (ch. 14, H14-xx), `ref-hastings-15` (ch. 15 and appendices, H15-xx), `ref-hastings-01` (ch. 1–4, H01-xx), `ref-hastings-05` (ch. 5, H05-xx). Where a fact appears only in a digest's section summary and not in an H-item, the citation reads `H09 §9.2.2` (digest, section). Card prefixes: **BJT** bipolar construction and matching, **DIO** diodes, **ESD** protection structures, **GRD** guard rings, taps, latch-up, merging and isolation, **FLR** floorplan, padring, power/ground, Kelvin and star routing, routing policy, **REL** failure mechanisms as layout rules, **PDK** process facts a recipe must query. MOS matching and passive arrays are covered by the sibling reports and appear here only where they are floorplan or reliability rules.

---

## 1. Technique cards

### 1.1 Bipolar transistors (BJT)

#### BJT-01 Matching-class taxonomy
- `id`: BJT-01
- `technique`: Assign every matched bipolar or diode group a class before selecting any array recipe: minimal, moderate or exceptional. The class sets the tolerance and decides whether each later rule is a cost, a budget or a hard constraint.
- `realizes`: a numeric mismatch budget per group (random share + systematic share) that all other BJT/DIO cards consume.
- `applicability`: all BJT and diode matched groups. Minimal = ±2 mV ΔV_BE or ±8 % ΔI_C; moderate = ±0.5 mV or ±2 % (±1 % for bandgap cores, ±1–2 mV untrimmed op-amps); exceptional = ±0.1 mV or ±0.5 %, which needs post-package trim or matched emitter degeneration, and for laterals heavy degeneration plus base-current cancellation and cold β > 20. Figures are 6σ, 10 years, −40…125 °C, plastic package (H09-01).
- `parameters`: class source: user override, circuit role (bandgap/PTAT core → moderate; general op-amp input → minimal; trimmed reference → exceptional), or an explicit offset target (H09-01).
- `guarantees`: a checkable target; CC becomes a cost for minimal and hard for moderate/exceptional (H09-06); die-axis placement only for moderate/exceptional (H09-18).
- `does not guarantee`: achievability. Exceptional matching is hard in ratioed structures (H09-04); a heat source ≥ 1 W precludes it unless heavily degenerated (H09-16).
- `pdk data needed`: none directly; class drives the k_A sizing in BJT-04.
- `verification`: report 6·σ_random (H09-02) plus thermal, stress and Early residues against the class limit (H09-15, H09-18, H09-23).
- `prefer over / degrade to`: root of the ladder. Unknown class → moderate for reference cores, minimal elsewhere (H09-01).
- `source`: H09-01, H09-02, H09-04, H09-06, H09-16, H09-18.

#### BJT-02 Identical unit emitters; ratio by unit count only
- `id`: BJT-02
- `technique`: Build every ratioed or matched bipolar from identical unit emitters (same drawn size, shape and contact geometry); the ratio is the unit count. Base and collector geometry matter much less. Laterals additionally need an identical collector opening per emitter.
- `realizes`: ratio N:1 with ΔV_BE = V_T·ln N (H09-02); equal I_S between matched devices.
- `applicability`: vertical NPN/PNP, substrate PNP, lateral PNP, diode-connected BJTs, any class. Ratios needing more than 8–10 units are impractical (area, thermal sensitivity) (H09-03). Non-integer ratios go to degeneration (BJT-12).
- `parameters`: unit emitter size and shape (BJT-04), unit count per device, array topology (BJT-05…BJT-11).
- `guarantees`: exact drawn-area ratio; the sublinear drawn-to-effective area error cancels because every unit is identical (H09-03). A 1 % area error would cost 0.25 mV (H09-02).
- `does not guarantee`: gradient cancellation (needs common centroid, BJT-05) or random mismatch below k_A/√A_E (H09-02).
- `pdk data needed`: minimum emitter width, emitter-over-contact enclosure, whether the PDK BJT is fixed-geometry (e.g. sky130 pnp_05v5) so only the multiplier may change (H09-08).
- `verification`: netlist emitter areas or multipliers in a matched group are integer multiples of one unit; otherwise diagnose and round or refuse (H09-03).
- `prefer over / degrade to`: always prefer over a scaled single emitter. Degrade to the PDK multiplier when geometry is fixed (H09-49).
- `source`: H09-02, H09-03, H09-08, H09-30, H09-49.

#### BJT-03 Ratio band selection for ratioed pairs and quads
- `id`: BJT-03
- `technique`: Keep ratios inside the band where PTAT signal grows faster than gradient residue: even integer ratios 4:1–16:1 for pairs, 4:1:1:4–8:1:1:8 for quads; 8:1 is the most popular pair, 4:1:1:4 the favoured quad.
- `realizes`: a ratio constraint whose error sensitivity is acceptable for the class.
- `applicability`: ratioed pairs/quads in bandgaps and PTAT sensors. VPTAT ∝ ln N while quadratic gradient residue ∝ N, so the optimum lies between 6:1 and 16:1; 2:1 and 2:1:1:2 only for minimal class; 64:1 is too large (H09-04).
- `parameters`: N (pair), N:1:1:N (quad). ΔV_BE = V_T·ln N (8:1 → 54 mV); quad ΔV_BE = V_T·ln(A1·A2/(A3·A4)) (4X,4X,1X,1X → 72 mV) (H09-04).
- `guarantees`: an even ratio admits a point-symmetric array (BJT-05).
- `does not guarantee`: accuracy: for 8:1 a 1 mV error is 2 % of PTAT and comes from only 0.5 °C (H09-15).
- `pdk data needed`: none.
- `verification`: warn when N is odd, < 4 or > 16 at moderate class; report N, ΔV_BE, %/°C and %/mV sensitivity (H09-04).
- `prefer over / degrade to`: prefer 8:1; degrade to 4:1 (2:1:2 array) when area forbids; 2:1 only minimal (H09-04, H09-07).
- `source`: H09-04, H09-07, H09-15.

#### BJT-04 Unit emitter size and shape
- `id`: BJT-04
- `technique`: Size the unit emitter from the random-mismatch target and bound it by gradient sensitivity; draw it as a circle, octagon or square with a concentric contact of the same shape.
- `realizes`: random share of the class budget: s(ΔV_BE) = k_A·V_T/√A_E, s(I_C2/I_C1) = k_A/√A_E (H09-02).
- `applicability`: vertical devices: emitter width ≥ 2× the minimum drawn width (minimum = contact width + 2× emitter-over-contact, e.g. 2 µm + 2×1 µm = 4 µm); matched emitters typically 8–40 µm wide, 4–100× minimum area; exceptional class prefers arrays of moderate emitters over one huge emitter (H09-08). Laterals: minimum emitters, circular while the circle's radius is below the square's centre-to-vertex distance (H09-08). Diodes: area-defining width 2–10× minimum; moderate diode-connected devices 10–50 µm²; Schottky ≥ 10–15 µm (H09-08). Exception: low-β devices with high base sheet resistance may match better with long thin emitters (H09-09).
- `parameters`: A_E ≥ (6·k_A·V_T/(η·ΔV_max))² (derived example: k_A = 0.02 µm, V_T = 25.7 mV, ΔV_max = 0.5 mV → 38 µm²) (H09-02); shape ∈ {circle(32/64 sides), octagon, square}; polygon side count divisible by 4 (H09-09); contact geometry = emitter geometry, concentric; silicide block matches too (H09-09).
- `guarantees`: typical k_A = 2 %·µm gives 0.33 % and 86 µV for 6×6 µm at 25 °C (H09-02); concentric contacts keep the current centroid on the emitter centroid (H09-09).
- `does not guarantee`: current-density independence in retrograde-well BiCMOS devices, where Eq. 10.8/10.9 hold only at constant density (H09-02); vertices of circles fall off grid, so matched circles must be identical on-grid polygons (H01-27).
- `pdk data needed`: k_A per BJT model, minimum emitter width, emitter-over-contact enclosure, 45° allowance for octagons (H09-02, H09-09, H01-27).
- `verification`: contact-array centroid = emitter centroid within one grid; A_E ≥ sizing bound; width within 2–10× minimum for the class (H09-08, H09-09).
- `prefer over / degrade to`: octagon when 45° edges are legal, else square with centred contact array (H09-09); fixed PDK geometry → keep and report achieved σ (H09-08).
- `source`: H09-02, H09-08, H09-09, H01-27.

#### BJT-05 Point-symmetric common-centroid unit fill
- `id`: BJT-05
- `technique`: Assign array slots to each device in pairs that are 180° images about the grid centre, so each device's first moment equals the grid centre by construction. A device with an odd unit count takes the centre slot, which forces an odd×odd grid (or a 1-D row); at most one device may be odd. Equal devices are dealt diagonally. Dummy (unconnected) emitters fill symmetric holes when allowed.
- `realizes`: common centroid for emitters, and for base contacts when each unit carries its own concentric base ring (H09-06, H09-14, H09-49).
- `applicability`: all matched bipolar arrays at moderate/exceptional class (hard), minimal (cost) (H09-06). Because I_C is exponential in V_BE, CC removes only the linear thermal gradient; the nonlinear residue is minimised by compactness (H09-06).
- `parameters`: grid shape, dev_nf per device, dummy fill policy; exported symmetry axes S1/S2 (H09-07).
- `guarantees`: per-device centroid offset = 0 exactly; empty slots only in point-symmetric sets (H09-49).
- `does not guarantee`: second-moment (quadratic) equality; that is scored separately and favours compact topologies such as 3×3 (H09-07).
- `pdk data needed`: whether dummy emitters are allowed (H09-49).
- `verification`: property test over dev_nf ∈ {1..4}×{1..16}: Σ(unit positions) = count × centre for every device (H09-49). A centre-out fill by radius and angle is not CC for 1:2, 1:4, 1:6 or 2:2 (H09-49).
- `prefer over / degrade to`: prefer the ratio-specific templates BJT-06…BJT-11 and let placement choose; degrade to a 1-D ABA row (BJT-08) when the grid cannot be made square (H09-07).
- `source`: H09-05, H09-06, H09-07, H09-14, H09-49.

#### BJT-06 2:1:2 array for a 4:1 pair
- `id`: BJT-06
- `technique`: Place the 1X unit in the centre with the 4X device split into two halves on either side, rotated so emitters come together, symmetric about both axes through the 1X (Fig. 10.24A).
- `realizes`: CC for a 4:1 ratioed pair with minimal footprint.
- `applicability`: 4:1 pairs, any class; also the building block of 4:1:1:4 quads (BJT-11).
- `parameters`: row versus cross arrangement; orientation of S1 toward the heat source (BJT-18).
- `guarantees`: first-moment coincidence of emitters; with concentric base rings per unit, base contacts also coincide (H09-14).
- `does not guarantee`: quadratic residue as low as a 3×3 (H09-07).
- `pdk data needed`: unit spacing from the unconnected-emitter rule (BJT-13).
- `verification`: BJT-05 property check; symmetry about both axes.
- `prefer over / degrade to`: prefer over a 2×3 grid with an empty slot, which is off-centroid by (½, ¼) pitch (H09-49); degrade to ABA row (BJT-08).
- `source`: H09-07, H09-49.

#### BJT-07 Eight-around-one 3×3 for an 8:1 pair
- `id`: BJT-07
- `technique`: Put the single unit at the centre of a 3×3 grid and the eight units of the large device around it (Fig. 10.24B). Base contacts form their own common centroid slightly offset from the emitter centroid.
- `realizes`: CC for 8:1 with the lowest quadratic residue: the array spans one third of the equivalent 1-D length and the quadratic residue falls by nearly 10× (H09-07).
- `applicability`: 8:1 pairs (the most popular ratio) (H09-04).
- `parameters`: unit pitch; collector contacts outside the array so emitters and base contacts come closer (H09-14).
- `guarantees`: symmetric under both axes and the diagonals; emitters and base contacts both CC.
- `does not guarantee`: equality of NBL-shadow exposure unless the NBL is oversized (BJT-20).
- `pdk data needed`: as BJT-06.
- `verification`: BJT-05 property check (the one case the current generator already passes) (H09-49).
- `prefer over / degrade to`: prefer over any 1-D 8:1 arrangement; degrade to two-row (BJT-09) when a 3×3 does not fit.
- `source`: H09-04, H09-07, H09-14, H09-49.

#### BJT-08 1-D ABA row oriented along isotherms
- `id`: BJT-08
- `technique`: Interleave devices in one row (A B A, or halves of the large device about the small one) and orient the long symmetry axis S2 parallel to the anticipated isotherms.
- `realizes`: linear gradient cancellation when a square array is not possible; a 6:1 pair as two halves either side of the small device (H14-40).
- `applicability`: any ratio; elongation acceptable; one dominant heat source whose isotherms are predictable (H09-19).
- `parameters`: S2 orientation; with one power device: power device at the centre of one die end, pair at the other end with S1 on the die axis (H09-19).
- `guarantees`: first-moment cancellation along the row; identical isotherm for both halves when S2 ∥ isotherms (H09-19).
- `does not guarantee`: cancellation of gradients across the row or of quadratic terms, which are larger than for a 3×3 (H09-07).
- `pdk data needed`: none beyond BJT-13 spacing.
- `verification`: angle between S2 and the local isotherm (perpendicular to ∇T from the power map) (H09-19).
- `prefer over / degrade to`: prefer 3×3 or 2:1:2 when they fit; ABA is the fallback topology (H09-07).
- `source`: H09-07, H09-19, H14-40.

#### BJT-09 Two-row array for multiples of 4:1
- `id`: BJT-09
- `technique`: For 16:1 and other multiples of 4:1, place the large device in two rows about an axis S2 that passes through the single emitter (Fig. 10.26B); S2 parallel to isotherms. Base contacts then need a double-base unit or an extended 1X to stay common-centroid.
- `realizes`: CC for large even ratios that cannot form a compact square.
- `applicability`: N a multiple of 4, N ≥ 8, when a 3×3 or 4×4-with-centre is impossible (H09-07).
- `parameters`: row count, S2 orientation, base-contact style (double-base) (H09-07).
- `guarantees`: emitter first moment on S2.
- `does not guarantee`: base-contact centroid without the double-base variant (H09-07).
- `pdk data needed`: double-base construction legality (BJT-15).
- `verification`: BJT-05 property check plus base-contact centroid offset budget ≤ one grid for moderate class (H09-14).
- `prefer over / degrade to`: prefer 3×3 for 8:1; degrade to ABA.
- `source`: H09-07, H09-14, H09-19.

#### BJT-10 Cross-coupled quad for equal pairs
- `id`: BJT-10
- `technique`: Split each transistor of an equal pair into two halves and place them on the diagonals of a 2×2 (½Q1, ½Q2 / ½Q2, ½Q1). Emitters and base contacts must share a centroid; collector contacts need not, so they go outside the array to bring emitters and bases closer. Use CBE rather than CEB order because the V_BE temperature coefficient exceeds the base-contact potential's coefficient (H09-14).
- `realizes`: CC of emitters and of base contacts (base-contact thermoelectric potentials) for differential pairs and current-mirror pairs.
- `applicability`: equal pairs of verticals, laterals (split-collector cross-coupled pairs reach ±0.1 % per Gilbert) and diode pairs (H09-14, H09-31, H09-43). Exceptional class: several cross-coupled pairs in parallel forming a dispersed 2-D CC array (H09-14).
- `parameters`: unit halves (m = 2 per side when the PDK device is fixed-geometry, since splitting changes the netlist) (H09-49); collector-strap side.
- `guarantees`: zero first-moment offset for emitters and base contacts.
- `does not guarantee`: cancellation of local filler stress, which is location independent (H09-21).
- `pdk data needed`: fixed-geometry flag; CBE/CEB legality (H09-32).
- `verification`: base-contact centroid offset between A and B ≤ one grid for class ≥ moderate (H09-14). An AA/BB row assignment has one pitch of offset and fails (H09-49).
- `prefer over / degrade to`: prefer over side-by-side AB; degrade to AB only for minimal class and one-unit fixed devices (H09-49).
- `source`: H09-14, H09-31, H09-43, H09-49.

#### BJT-11 Ratioed quad as two stacked ratioed pairs
- `id`: BJT-11
- `technique`: Build a 4:1:1:4 (or N:1:1:N) quad as two ratioed pairs whose primary axes coincide, giving a 2-D common centroid; if their axes cannot coincide, treat each pair independently.
- `realizes`: CC for quad ΔV_BE = V_T·ln(A1·A2/(A3·A4)) (72 mV for 4:1:1:4) (H09-04).
- `applicability`: ratioed quads 4:1:1:4 to 8:1:1:8 (H09-04).
- `parameters`: pair template (BJT-06/07), stacking axis S1, merged-emitter spacing (BJT-13).
- `guarantees`: both pairs see the same linear gradient.
- `does not guarantee`: independence from the Early effect (equal V_CE still required, BJT-21).
- `pdk data needed`: as BJT-06.
- `verification`: BJT-05 check per pair and for the union.
- `prefer over / degrade to`: prefer stacked pairs; degrade to independent pairs (H09-07).
- `source`: H09-04, H09-07.

#### BJT-12 Emitter degeneration: transfer the matching burden to resistors
- `id`: BJT-12
- `technique`: Insert matched emitter resistors sized for equal voltage drop (inverse to emitter area: 1X 4 kΩ, 2X 2 kΩ, 3X 1.33 kΩ at 25/50/75 µA). Transistor mismatch is reduced by V_T/(V_T + V_d): 50 mV → ÷3, 100 mV → ÷6. The resistors themselves become the matched array (interdigitated against gradients).
- `realizes`: relaxed BJT tolerance; arbitrary or non-integer ratios (3.4:1 from 3X + 10 kΩ versus 1X + 34 kΩ) (H09-26).
- `applicability`: current mirrors and sources, not ratioed PTAT pairs/quads, not split-collector laterals (common emitter) (H09-26). Class rules: verticals none for minimal, ≥ 50 mV moderate, ≥ 100 mV exceptional; laterals ≥ 50 mV minimal/moderate, ≥ 100 mV exceptional, 200–300 mV to set a 10:1 ratio between unlike emitters; 200 mV suffices for arbitrary ratios; 250–500 mV makes transistor size irrelevant (H09-26).
- `parameters`: V_d = I_E·R from the op-point; resistor unit and interdigitation (passives report).
- `guarantees`: Early-effect mismatch also scales by V_T/(V_T + V_d) (H09-23).
- `does not guarantee`: anything if the resistors are not matched to the class tolerance; costs headroom V_d.
- `pdk data needed`: resistor recipes (passives report).
- `verification`: V_d ≥ class minimum; resistor group constrained as a matched array; BJT budget relaxed by (V_T + V_d)/V_T (H09-26).
- `prefer over / degrade to`: prefer for laterals (minimum emitters, low V_A) (H09-26); not applicable to ratioed pairs, which must rely on BJT-05…BJT-11.
- `source`: H09-23, H09-26.

#### BJT-13 Multi-emitter spacing in a common base; base enlargement
- `id`: BJT-13
- `technique`: When several emitters share one base, keep neutral base between emitters ≥ 2× the base junction depth and enlarge the base-over-emitter overlap by a couple of µm; space emitters so depletion edges stay ≥ one junction depth apart (if unknown, use emitter-to-emitter spacing + base junction depth). Never use a "connected-emitter" spacing rule for matched emitters; use the unconnected rule plus margin (diodes +1–2 µm). For moderate/exceptional class draw the base 1–2 µm wider around every emitter than the layout rule.
- `realizes`: unit emitters that keep their full effective area; immunity to misalignment-induced lateral beta variation (H09-12, H09-13).
- `applicability`: vertical BJTs and matched PN diodes with merged bases. Merged collector (common tank) is fine because the collector does not control the current (H09-12). Laterals: never two emitters in one collector opening (H09-12).
- `parameters`: base junction depth, match margin (default 1 µm, scaled to process) (H09-13).
- `guarantees`: adjacent emitters do not lose effective area to depletion merging (H09 §9.1).
- `does not guarantee`: cross-injection immunity in laterals (GRD-08).
- `pdk data needed`: base junction depth, unconnected emitter spacing, base-over-emitter enclosure (H09-12).
- `verification`: custom matched-emitter spacing check on marked groups (H09-12).
- `prefer over / degrade to`: per-unit concentric base rings avoid the issue entirely (current generator) (H09-12); merged base only to save area.
- `source`: H09-12, H09-13, H09 §9.1.

#### BJT-14 CMOS substrate-PNP unit, including the short-emitter effect
- `id`: BJT-14
- `technique`: Small square PMoat emitter with one centred contact inside an N-well base; the base contact is an NMoat ring around the emitter, contacted on one side only; the P substrate is the collector with substrate contacts adjacent (ideally abutting). Larger devices use one or more minimum-width emitter strips, not a big square. Minimise emitter contacts and silicide (short-emitter effect: more contact or silicide lowers β); silicide-block everything except under the contact.
- `realizes`: the only bipolar in plain N-well CMOS; usable β for bandgaps and thermal sensors (H09-11, H01-24).
- `applicability`: analog CMOS; β 50–100 in 4/3 µm, 10–20 in 5 V CMOS, barely > 1 in deep submicron (H01-24, H09-48). Retrograde wells may preclude it (H09-11). Substrate current ≤ ~1 mA per device and ≤ ~10 mA total with nearby contacts; > 1 mA needs substrate contacts all around (H09-11). Scribe-street contacts handle 10–20 mA; beyond that fill unused area with substrate contacts (H01-08).
- `parameters`: emitter contact count (single at low current; full array for high current), silicide block, base-ring contact side, collector-ring coverage from I_C (H09-10, H09-11).
- `guarantees`: lowest emitter-contact recombination for the given emitter (H09-10).
- `does not guarantee`: a beta plateau: CMOS/BiCMOS bipolars rarely have one, so diode-connected ratioed pairs must run near peak β or compensate base current (H09-24). Substrate-PNP β rolls off from ~1 µA/µm² versus ~30 µA/µm² for a vertical NPN; minimum-emitter R_E ~100 Ω versus ~10 Ω (H09 §9.2.2).
- `pdk data needed`: silicide-block layer and its enclosures, N-well retrograde flag, substrate-current budget (H09-10, H09-11).
- `verification`: per-device I_C from the op-point against the 1 mA and 10 mA thresholds; silicide-block enclosure legality (H09-11).
- `prefer over / degrade to`: single-contact variant at low current; full contact array at high current (H09-10). Degrade to the PDK fixed cell when present.
- `source`: H09-10, H09-11, H09-24, H09-48, H01-08, H01-24, H09 §9.2.2.

#### BJT-15 Vertical NPN unit construction (buried-layer processes)
- `id`: BJT-15
- `technique`: Emitter overlaps its contact equally on all sides with the contact as large as allowed; base overlaps emitter for lateral punchthrough plus misalignment; base contact elongated along the full base width on one side, or on both sides (double-base, R_B ≈ ¼); tank holds maximum NBL that touches or overlaps the deep-N+ sinker; sinker width ≥ 2× epi thickness, drawn NBL to the sinker's outer edge; sinkers on both sides cut NBL resistance 4×, a ring more. CEB order gives lower R_C; CBE is preferred for matched quads and diode-connected transistors. Stretch devices only when routing forces it.
- `realizes`: low R_C and R_B for a given emitter; consistent unit geometry for arrays (H09-32, H09-38).
- `applicability`: standard bipolar and BiCMOS CDI NPNs. Omit the sinker only at low current (R_C several kΩ; CDI NPN ~1 kΩ, ≤ 1–2 mA). Diode-connected NPN: > a few hundred µA needs a sinker, ≥ 10 mA a power layout with a full ring; tolerate ≤ 400 mV collector drop at 25 °C, 200 mV at 150 °C (H09-38).
- `parameters`: base contact {ring, one side, two sides}; terminal order {CBE, CEB}; sinker {none, one side, both, ring}; emitter stripe width (compact emitters crowd: 18 mV of pinched-base debias doubles the local current; stripes wider than 25–50 µm not advisable; β rolls off above ~10 µA/µm²) (H09-33).
- `guarantees`: R_C sized from current; identical unit per array.
- `does not guarantee`: SOA for power operation (BJT-23…BJT-25).
- `pdk data needed`: epi thickness, NBL/sinker layers and enclosures, maximum emitter stripe (deck key exists in sky130 as 25 µm) (H09-33, H09-38).
- `verification`: R_C versus the current-driven drop budget; stripe width ≤ deck maximum (H09-33, H09-38).
- `prefer over / degrade to`: CBE for matched, CEB for single high-current devices (H09-32). In STI CMOS the analogue is the N-well collector ring with contact density from I_C (H09-38).
- `source`: H09-32, H09-33, H09-38.

#### BJT-16 Lateral PNP unit and split collectors
- `id`: BJT-16
- `technique`: Minimum circular emitter (just enclosing a minimum circular contact) centred in a circular hole of a rectangular collector; drawn base width = hole radius − emitter radius; one collector end extended for its contact; base contact by an emitter-diffusion strip at the tank end (no sinker); maximum NBL enclosing the emitter and reaching the inner collector edge by ≥ 150 % of epi thickness. Split collectors are identical arcs placed symmetrically; segment size by periphery: A_C = (P_E/P_EU)·(P_C/ΣP_C). Larger devices use hot-dog (elongated) emitters or arrays of minimum emitters, each in its own identical collector opening.
- `realizes`: matched laterals, 1:1 mirrors from split collectors (±1 %), low-V_A input devices with V_CEO ≈ NPN V_CBO (H09-30, H09 §9.1.2).
- `applicability`: standard bipolar and BiCMOS with NBL; CMOS poly-ring lateral PNP where retrograde wells exist (β 50–100, collector efficiency > 0.9) (H09 §9.3.1). Square laterals allow only half/quarter splits (H09-30). Lateral β rolls off above ~100 µA per minimum emitter, < 10 at ~250 µA; deep-P+ emitter extends to ~0.5 mA (H09 §9.2.3). Collector efficiency must exceed 0.95, often > 0.99 (H09 §9.1.4).
- `parameters`: emitter radius (minimum), base width, split vector (periphery units), emitter arraying (square/hex), tank sharing policy.
- `guarantees`: identical symmetric segments match within ±1 %; Gilbert cross-coupled split pairs ±0.1 % (H09-30, H09-31).
- `does not guarantee`: ratioed accuracy from unequal segments (gaps); matching once one collector saturates (destroys all others); degeneration is impossible (common emitter); split-collector groups are capped at moderate class (H09-31). Diode-connected laterals lose 0.1–1 % to the substrate (DIO-01).
- `pdk data needed`: lateral construction declared (poly ring or NBL), NBL/epi data, 45° or circle support (H09-30, H01-27).
- `verification`: equal arc lengths; op-point check that no collector saturates; collector efficiency > 0.999 before allowing a shared tank at exceptional class, never-saturating for moderate (H09-30, H09-31).
- `prefer over / degrade to`: separate identical tanks over a shared tank; P-bar isolation not recommended for matching (H09-30). Degrade to a vertical device if the process has no NBL (lateral β < 1 without NBL) (H01-24).
- `source`: H09-30, H09-31, H01-24, H01-27, H09 §9.1.2, §9.1.4, §9.2.3, §9.3.1.

#### BJT-17 Field plates on lateral, epi-base, shallow-well and DMOS bipolars
- `id`: BJT-17
- `technique`: Lateral PNP: a metal (or poly) plate tied to the emitter covering all exposed base between drawn emitter and drawn collector, overlapping the collector by 2–3 µm. Epi-base NPN: plate over the base except its contact, touching the deep-N+ ring. Shallow-well NPN: PMoat channel-stop ring (also base contact) plus a metal plate from the emitter. DMOS NPN: poly gate plate tied to the emitter. Verti-lat substrate PNP: plate over the semicircular tank end.
- `realizes`: stable β and I_C versus voltage; blocks surface charge over the exposed base.
- `applicability`: every lateral PNP regardless of process voltage (without it β drifts and I_C steps at 5–10 V; β shifted > 30 % at < 5 V in one case) (H09-29, H05-43); any device with lightly doped surface between terminals.
- `parameters`: plate layer (lowest metal or poly), overhang (misalignment + base outdiffusion for HV collectors) (H05-43).
- `guarantees`: shields the silicon beneath from charge or conductors above, so low-voltage upper metal may cross it (H05-44).
- `does not guarantee`: protection where the plate does not extend; partial plates must extend well past the point where the voltage drops below V_TF (H05-43).
- `pdk data needed`: lowest metal/poly over active legality; CMOS poly-ring lateral construction (H09-29).
- `verification`: plate present on the emitter net; no other net on that layer over the exposed base (router obstruction) (H09-29).
- `prefer over / degrade to`: plate (easier) over channel stop; both where gaps exist (REL-13).
- `source`: H09-29, H05-43, H05-44.

#### BJT-18 Thermal placement for matched bipolars and diodes
- `id`: BJT-18
- `technique`: Convert the class budget into an allowed ΔT (V_BE TC ≈ −2 mV/°C, I_C TC ≈ 8 %/°C) and enforce distance and half-plane rules from heat sources: minimal ≥ 250 µm from devices ≥ 250 mW and not adjacent to any > 50 mW; moderate ≥ 100–250 µm (laterals ≥ 250 µm) from any > 50 mW and on the opposite side of the die; exceptional as far as possible, die aspect 1.5:1 or 2:1. Separate input and output stages on opposite die sides; common-centroid the inputs.
- `realizes`: ΔT budget: max ΔT = η·(class limit − random share)/(2 mV/K); moderate ±0.5 mV with no random share → 250 mK, minimal ±2 mV → 1 K (H09-15).
- `applicability`: all matched BJT/diode groups; thermal feedback matters at gains > 10,000 (H09-17). A heat source ≥ 1 W precludes exceptional matching unless heavily degenerated; a heat-sunk package lowers far-field gradients if separation ≥ 1 mm (H09-16). ~6 V Zeners are the only low-TC exception (H09-16).
- `parameters`: power class thresholds (50 mW, 250 mW, 1 W), distance, half-plane, array axis orientation (BJT-08).
- `guarantees`: ±1 mV offsets need ±0.5 °C; the budget is computed from physics, not habit (H09-15).
- `does not guarantee`: nonlinear residue inside the array (compactness, BJT-05).
- `pdk data needed`: V_BE TC per model (default 2000 µV/K), Seebeck coefficient 0.1–1.0 mV/°C for contact pairs at different temperatures (H09-15, H01-43).
- `verification`: ΔT between members from the power map versus budget; distance and half-plane checks (H09-16); report equivalent offset ΔV = 2 mV/°C·ΔT (H01-43).
- `prefer over / degrade to`: distance rules work when the power map is uncertain; the ΔT field is preferred when it exists (H09-16).
- `source`: H09-15, H09-16, H09-17, H01-43.

#### BJT-19 Die location for mechanical stress (bipolar and diode groups)
- `id`: BJT-19
- `technique`: Place matched bipolars/diodes near the die centre when no significant heat source exists, on a die symmetry axis (X or Y) for moderate/exceptional class, ≥ 250 µm inside die edges, never near corners. With a heat source, put the source on one side and the matched devices on the other, 125–250 µm from the far edge, preferably on a symmetry axis (thermal beats stress). CSP: centre of a group of four bumps near die centre, aligned with the axis through adjacent bumps, never near corner bumps.
- `realizes`: minimum stress gradient across the array (piezojunction ΔI_S = −I_S(ζ_L σ_L + ζ_T σ_T + ζ_LT τ_LT)) (H09-20).
- `applicability`: all matched bipolar groups; plastic packages (mold cures at 150–200 °C and compresses the die radially) (H09-21). 100 MPa in-plane shifts a vertical NPN 0.4 mV on (111) and 0.9 mV on (100), a lateral PNP 0.8 and 0.3 mV; equal stress cancels in ΔV_BE (H09-20). On (100) prefer vertical-PNP bandgaps (ζ_T ≈ ⅓ of NPN); not on (111) (H09-20).
- `parameters`: die rect, block origin, wafer orientation, package type, bump map, external heat sources (die context) (H09-18).
- `guarantees`: smallest gradient at the die centre; corner exclusion compatibility (FLR-03).
- `does not guarantee`: filler stress, which is local and location independent (SOT-23 bandgaps −1.1 mV mean, 2.3 mV σ) and is not cancelled by CC or placement; mitigations are a thick metal plate over the array (15 µm Cu moved −5.06 mV/2.64 mV σ to −2.26 mV/1.38 mV σ), polyimide, low-stress mold, on-chip stress sensors (H09-21). Long-term drift and hysteresis limit exceptional matching (H09-21).
- `pdk data needed`: wafer orientation, piezojunction table (ζ values, Table 10.3), thick top-metal/polyimide availability (H09-20, H09-21).
- `verification`: edge keep-out 250 µm hard, corner keep-out hard, on-axis cost (moderate) or budget (exceptional); without die context report "unknown" and use the block's symmetry axis as proxy (H09-18).
- `prefer over / degrade to`: thermal placement wins over stress placement when they conflict (H09-18).
- `source`: H09-18, H09-20, H09-21, H15-12, H01-44.

#### BJT-20 NBL shadow compensation
- `id`: BJT-20
- `technique`: Grow the drawn NBL around every emitter by the pattern shift plus emitter outdiffusion plus two-mask misalignment; if the shift direction is unknown grow all sides; if the magnitude is unknown use 150 % of epi thickness. Laterals: NBL over the inner collector perimeter ≥ 150 % epi so the shadow misses the exposed base. Alternatively orient CEB arrays with the main axis parallel to the shift so the shadow lands in contacts.
- `realizes`: equal emitter environment under the buried layer; a 2:1 array with one shadowed emitter would err by 0.5 % (0.13 mV) (H09-22).
- `applicability`: buried-layer processes with epitaxial pattern shift (shift up to ~2× epi thickness). STI/CMP processes have no NBL shadow (etched alignment markers), so the rule is off (H09-22, H01-19). Minimally matched verticals may skip it; Schottkys must never be crossed even at minimal class (H09-22).
- `parameters`: shift magnitude and direction, epi thickness.
- `guarantees`: no emitter of a matched set sits on the shadow edge.
- `does not guarantee`: anything about mask offsets the fab already applies (H01-19).
- `pdk data needed`: `nbl_shadow {shift, direction}` or absence (STI) (H09-22).
- `verification`: NBL enclosure of emitters ≥ computed margin for class ≥ moderate.
- `prefer over / degrade to`: oversize NBL (simple) over axis orientation; off for STI decks.
- `source`: H09-22, H01-19.

#### BJT-21 Operating-point preconditions for matched bipolars
- `id`: BJT-21
- `technique`: Before spending layout budget, check from the op-point that the circuit lets the pair match: equal V_CE (systematic error ≈ ΔV_CE/V_A: verticals V_A 100–300 V → 0.3–1 %/V, laterals 50–200 V → 0.5–2 %/V plus collector-efficiency collapse near saturation); current density well below high-level injection (verticals ≤ ⅓–½ of the onset where the semilog I_C–V_BE line bends; laterals 30–50 %, which can be ~1 µA per minimum emitter); β floor (mirror β ≥ 10; cold β > 20 for exceptional); a beta plateau when the circuit ratios emitter currents (diode-connected pairs).
- `realizes`: a systematic-error audit the layout cannot fix; the recipe choice (cascode, hot-dog emitters, degeneration) when a check fails.
- `applicability`: all ratioed and matched BJT groups; standard NPN β rolls off above ~10 µA/µm² and below ~10 pA/µm² of drawn emitter (H09-24). ΔV_BE = V_T·ln N holds over up to eight decades only in low-level injection (H09 §10.2.3).
- `parameters`: thresholds per class (H09-23, H09-24, H09-48).
- `guarantees`: the pair is not already out of class before layout.
- `does not guarantee`: layout quality; these are circuit facts.
- `pdk data needed`: V_A, HLI onset per unit, peak-β current, β at corners per BJT model (H09-23, H09-24).
- `verification`: error_% = ΔV_CE·(100/V_A)·V_T/(V_T + V_d) against class budget; J = I_C/n_units versus onset; β at corners (H09-23, H09-24, H09-48). Reverse V_EB limit is REL-12.
- `prefer over / degrade to`: when V_CE differs, add cascodes (circuit) rather than tightening layout; when J is too low for laterals, use hot-dog emitters (H09-24).
- `source`: H09-23, H09-24, H09-48, H09 §10.2.3.

#### BJT-22 Emitter and base ballast sizing for multi-finger devices
- `id`: BJT-22
- `technique`: Give each emitter finger a ballast drop of 2–3 V_T (50–75 mV) at full current (≈ 500–750 mΩ in the 400 mA four-finger example), which tolerates ≈ 25 mΩ of finger-to-finger interconnect imbalance; keep intrafinger debias ΔV_BE = L·R_S·I_E/(2W) ≤ 5 mV; base-lead debias ≤ 2–4 mV; feed serpentine base leads from both ends (÷4); prefer comb over serpentine base metal; add a second-metal plate over the fingers through rows of vias.
- `realizes`: equal finger currents (I_E ratio = exp(ΔV_BE/V_T); 1/2/3 mV lead drops gave 24–26 % skew) and runaway immunity (H09-34, H09-35).
- `applicability`: any multi-finger or multi-emitter BJT above ~10 mA / 100 mW; acute above 100 mA / 500 mW (H09 §10.1). Interdigitated example: ~1 □ emitter-diffusion pairs at 5 Ω/□ → 2.5 Ω per finger → 50 mV at 20 mA, so size fingers for ~20 mA (H09-34).
- `parameters`: ballast resistance per finger, finger length/width, base-lead style, metal plate; if total finger-to-finger debias cannot stay < 100 mV, redesign (aspect ratio, more pads, more/thicker metal) (H09-34).
- `guarantees`: bounded current skew (Table 10.1 range; budget max/min ≤ 1.1 derived) (H09-34).
- `does not guarantee`: RBSOA (turn-off focusing), which depends on topology (BJT-25).
- `pdk data needed`: emitter sheet resistance, metal sheet resistance, via resistance (H09-34).
- `verification`: solve the per-finger exponential network with extracted emitter metal and ballast by fixed-point iteration (the book's method); distributed-current IR check using Eq. 10.2 (half of I·R) (H09-34, H09-35).
- `prefer over / degrade to`: diffusion ballast (interdigitated) or distributed ballast (WENC, BJT-24); degrade to poly ballast resistors in RF SiGe (H09 §10.1.4).
- `source`: H09-34, H09-35, H09 §10.1, §10.1.4.

#### BJT-23 Interdigitated-emitter power NPN
- `id`: BJT-23
- `technique`: Parallel emitter fingers 8–25 µm wide with base contacts on both sides of every finger including the outermost; per-finger ballast from ~1 □ of emitter diffusion; base debias ≤ 2–4 mV with comb metal; sinker width ≥ 2× epi thickness, sinkers on both sides (NBL resistance ÷ 4) or a ring; NBL drawn to the outer sinker edge; the deep-N+ ring doubles as a hole-blocking guard ring.
- `realizes`: switched-mode power stage with excellent RBSOA and frequency response, fair FBSOA (with ballast) (H09-36 Table 10.2).
- `applicability`: switched or pulsed roles: switched ≤ 20 µA/µm² of emitter; pulsed ≤ 1 µs pulses ≥ 250 ns apart, RMS ≤ 20 µA/µm², metal per intermittent-EM rules (H09-36).
- `parameters`: finger count/width/length from I_max/J_limit, ballast, base metal style (BJT-22).
- `guarantees`: many small emitters resist turn-off current focusing (centre of a wide emitter turns off last) (H09 §10.1.1).
- `does not guarantee`: FBSOA as good as WENC; FBSOA and RBSOA cannot both be optimised (H09 §10.1.1).
- `pdk data needed`: emitter sheet R, epi thickness, sinker/NBL layers (H09-36, H09-38).
- `verification`: BJT-22 debias solve; current density per finger versus role limit (H09-36).
- `prefer over / degrade to`: prefer for switching; WENC (BJT-24) for linear; cruciform for compactness (BJT-25).
- `source`: H09-36, H09-38, H09 §10.1.1, §10.1.2.

#### BJT-24 Wide-emitter narrow-contact (WENC) power NPN
- `id`: BJT-24
- `technique`: Wide emitter fingers with a narrow central contact so the emitter diffusion itself ballasts: emitter-over-contact overlap 10–30 µm chosen from ΔV_E = R_SE·W_O²·J_E (5 Ω/□, 15 µm, 20 µA/µm² → 23 mV) and ΔV_B = R_SB·W_O²·J_E/β (200 Ω/□, β 25 → 36 mV); the contact stops short of the finger ends with end overlap equal to side overlap; metal drops ≤ 5–10 mV; tie both ends of the serpentine base lead. In DLM CDI processes: base ring around each finger, emitter M2 plate, base M1 grid, collector M1 ring plus U-shaped M2.
- `realizes`: good FBSOA, RBSOA and frequency ("best all-round"); linear-mode devices (H09-36).
- `applicability`: linear mode ≤ 150 µW/µm² and ≤ 10 µA/µm² of emitter (conservative); pulsed gate drivers (H09-36).
- `parameters`: W_O, finger length, metal scheme.
- `guarantees`: distributed ballast without separate resistors.
- `does not guarantee`: the best RBSOA (interdigitated is better) (H09-36).
- `pdk data needed`: emitter and base sheet resistance, β, metal stack (H09-36).
- `verification`: ΔV_E and ΔV_B within budget; end-overlap rule; metal drop ≤ 5–10 mV (H09-36).
- `prefer over / degrade to`: default power NPN; degrade to interdigitated for hard switching.
- `source`: H09-36, H09 §10.1.2, §10.1.4.

#### BJT-25 Christmas-tree, cruciform and power-layout selection
- `id`: BJT-25
- `technique`: Select the power BJT style from the SOA role (Table 10.2): interdigitated (FBSOA fair with ballast, RBSOA excellent, frequency excellent); WENC (good/good/good); Christmas-tree or H-emitter (FBSOA excellent, RBSOA poor; never for switching or pulsed loads; separating emitter banks with predrive between them tripled FBSOA); cruciform (3-D distributed ballast, FBSOA excellent, RBSOA good, most compact, small-contact EM concern; split into spaced sections because compactness causes local heating).
- `realizes`: SOA-appropriate power stage; emitter area = I_max/J_limit(role) (H09-36).
- `applicability`: power bipolars (β ≥ 10 minimum acceptable in high-level injection); small-signal layouts suffice to ~10 mA / 100 mW (H09 §10.1).
- `parameters`: style, section count and spacing (BJT-27), current-density role.
- `guarantees`: style matches the failure mode the application exercises (thermal runaway in linear mode, current focusing at turn-off) (H09 §10.1.1).
- `does not guarantee`: both SOAs at once (H09 §10.1.1).
- `pdk data needed`: as BJT-23/24.
- `verification`: role from op-point/testbench (linear V_CE > 0.5 V continuous, switched, pulsed) and J ≤ role limit (H09-36).
- `prefer over / degrade to`: WENC default; interdigitated for switching; cruciform when area-bound; Christmas-tree only for linear (H09-36).
- `source`: H09-36, H09-37, H09 §10.1, §10.1.1, §10.1.2.

#### BJT-26 Power lateral and substrate PNP arrays
- `id`: BJT-26
- `technique`: Power lateral PNP: square or hexagonal array of minimum emitters, each with its own identical collector opening, inside a continuous deep-N+ ring that is both base contact and hole-blocking ring; 0.25–1 mA per emitter before β < 5; the beta roll-off self-ballasts. A deep-P+ emitter extension gives 2–3× current density. Power substrate PNP: interdigitated emitter stripes ≤ 25 µm wide (pinched-base resistance may exceed 10 kΩ/□), unbroken substrate ring when DLM exists; needs a backside contact (thinned die, solder or silver sinter); the rigid mount raises stress, so matched parts are placed carefully with common centroid.
- `realizes`: PNP power stages without a dedicated power PNP process.
- `applicability`: standard bipolar and BiCMOS with NBL (laterals); any process with backside contact (substrate PNP) (H09 §10.1.3, §9.2.2).
- `parameters`: emitter count from I/(0.25–1 mA), packing (square/hex), stripe width ≤ 25 µm.
- `guarantees`: lateral arrays are "nearly indestructible" because β roll-off ballasts (H09 §10.1.3).
- `does not guarantee`: debias immunity of substrate PNPs above ~1 mA per device without surrounding substrate contacts (H09-11).
- `pdk data needed`: deep-N+/NBL, pinched-base sheet R, max emitter stripe (H09-33).
- `verification`: per-emitter current ≤ 1 mA; stripe width ≤ deck maximum.
- `prefer over / degrade to`: lateral array when NBL exists; substrate PNP when backside contact exists.
- `source`: H09-11, H09-33, H09 §9.2.2, §10.1.3.

#### BJT-27 Power-device sectioning and sense-device placement
- `id`: BJT-27
- `technique`: Split a dissipating device into banks with other circuitry (predrive) between them or interleave sinker strips, maximising the minimum distance between sections. Place a sense transistor as two equal sections on the power device's symmetry axis, each about halfway between centre and periphery, so each sits at the mean temperature; one sense device embedded in the middle is worse (hottest spot); adjacent on the axis is least good.
- `realizes`: lower peak temperature; a sense ratio that tracks the mean device temperature (H14-19, H14-20, H09-37).
- `applicability`: power BJTs and MOS; RF SiGe cells of tiny emitters widely separated (H09-37).
- `parameters`: bank count k, spacing, sense section positions on the T = T̄ contour.
- `guarantees`: residual |T_sense − T̄| computed from the thermal map (H14-20).
- `does not guarantee`: lower total area or wiring; the trade is explicit.
- `pdk data needed`: none beyond thermal model constants (κ_Si ≈ 1.3 W/cm/°C) (H05-04).
- `verification`: peak-T budget per section; sense-section isotherm residual (H14-20, H09-37).
- `prefer over / degrade to`: two sense halves on axis → single embedded → adjacent (H14-20).
- `source`: H09-37, H14-19, H14-20, H05-04.

#### BJT-28 Base-side ballasting against current hogging
- `id`: BJT-28
- `technique`: For parallel-base NPNs where one device may saturate, put a matched resistor in each base lead, sized inversely to emitter area (half the resistance for twice the area). A diffused base ballast must not share the NPN's tank, otherwise the parasitic PNP simply moves into the resistor (Fig. 9.8).
- `realizes`: a saturated member takes ≤ 10 % of the reference current (exercise target) (H09-27).
- `applicability`: NPN mirrors or arrays with a saturating member; Schottky clamps are the alternative (H09 §9.1.3).
- `parameters`: resistor values, tank assignment.
- `guarantees`: bounded hogging.
- `does not guarantee`: substrate injection suppression (that needs the deep-N+ ring, BJT-29).
- `pdk data needed`: resistor recipes; tank isolation rule.
- `verification`: resistor ratio inverse to unit counts; isolation rule: ballast resistor not in the ballasted transistor's tank (H09-27).
- `prefer over / degrade to`: Schottky clamp when the process has one (H09 §9.1.3).
- `source`: H09-27, H09 §9.1.3.

#### BJT-29 Guard structures for saturating bipolars and power diodes
- `id`: BJT-29
- `technique`: NPN with more than a few mA of base drive: continuous deep-N+ ring merged with NBL (hole-blocking, ≥ 100:1 doping ratio) that also lowers R_C; keep it even with anti-saturation circuits. Lateral PNP: deep-N+ ring, or a secondary (ring) collector grounded (hole-collecting guard), tied to base (self-limiting) or feeding an anti-saturation loop (stability not modelled). Substrate PNP > 1 mA: substrate contacts around it. Power diodes above a few mA: NBL plus deep-N+ ring; above 100 mA ring width ≥ 2× epi and NBL to the ring's outer edge. Saturating members of merged laterals get their own tank.
- `realizes`: no hole injection into the substrate from saturation; no latch-up or debias of neighbours (H09-28, H09-47).
- `applicability`: junction-isolated bipolar and BiCMOS; guard efficiency drops at high injection (H09 §11.1.4).
- `parameters`: ring width, ring net (ground, base, detector), contact density from I_C.
- `guarantees`: hole blocking > 95–98 % with correct NBL/deep-N+ geometry (GRD-07).
- `does not guarantee`: stability of an anti-saturation loop (H09-47).
- `pdk data needed`: deep-N+/NBL layers, doping ratio; in STI CMOS the deep-N-well ring is the analogue (H09-28).
- `verification`: op-point V_CE < ~0.3 V or forward-conducting diode flags the requirement; ring closure and NBL enclosure (H09-28, GRD-07).
- `prefer over / degrade to`: hole-blocking ring over hole-collecting; Schottky clamp prevents saturation altogether (H09 §9.1.3).
- `source`: H09-28, H09-47, H09 §9.1.3, §11.1.4.

#### BJT-30 Junction corner rounding for high voltage and Zeners
- `id`: BJT-30
- `technique`: Where a base/HSR diffusion runs near its breakdown, fillet or chamfer every 90° vertex (inside and outside): fillet radius ≥ 150 % of junction depth, chamfer segments of similar length; fillets beat chamfers; concentric fillets keep spacings. E-B Zeners are round or oval; matched Zeners use large circular geometries with circularly symmetric contacts.
- `realizes`: planar breakdown instead of corner breakdown (a 120 V planar junction can break at 60 V at corners) (H09-40).
- `applicability`: HV bipolar, Zeners, ESD clamps (fillet base corners of V_CES clamps and drain corners of thick-field NMOS) (H14-50, H14-46).
- `parameters`: radius or chamfer length; enabled when the net voltage is within a deck fraction of the diffusion BV.
- `guarantees`: corner field relief.
- `does not guarantee`: legality in Manhattan-only decks (H01-27).
- `pdk data needed`: 45° allowance, junction depth, BV per diffusion (H09-40, H01-27).
- `verification`: every qualifying vertex rounded; polygon on grid (H01-27).
- `prefer over / degrade to`: fillet → chamfer → stair-step approximation.
- `source`: H09-40, H14-46, H14-50, H01-27.

### 1.2 Diodes (DIO)

#### DIO-01 Diode-connected transistor as the precision diode
- `id`: DIO-01
- `technique`: Use an NPN with collector and base tied as the anode and emitter as the cathode (higher β, faster than PNP); CBE layout; merge the collector-base contact where the deck supports it (many modern decks do not). Matched diode-connected groups follow the BJT rules.
- `realizes`: a diode with R_S = R_B/β_F + R_E ≈ 10–20 Ω for a minimum device; ≈ 0.65 V at 1 µA/µm² at 25 °C, +18 mV per current doubling, −2 mV/°C (H09-41).
- `applicability`: any process with a vertical NPN; > a few hundred µA needs a sinker, ≥ 10 mA a power layout with a deep-N+ ring; tolerate ≤ 400 mV ohmic collector drop at 25 °C, 200 mV at 150 °C (H09-41, H09-38). Substrate-PNP diodes above 1 mA may saturate from debiasing; lateral-PNP diodes lose 0.1–1 % to the substrate and are unfit for accurate current matching (H09-41).
- `parameters`: unit emitter (BJT-04), terminal order CBE, merged C–B contact variant.
- `guarantees`: inherits CC/thermal/stress machinery of BJT-05…BJT-19; diode-connected NPNs match to a few mV and handle amps (H09 §11.4).
- `does not guarantee`: a beta plateau in CMOS/BiCMOS devices for emitter-current-ratioed pairs (H09-24).
- `pdk data needed`: merged C–B contact legality, sinker availability (H09-41).
- `verification`: reverse bias ≤ ⅔ V_EBO (REL-12); collector drop budget (H09-38, H09-41).
- `prefer over / degrade to`: prefer over PSD/N-well junction diodes for matching (DIO-03); degrade to substrate PNP diode in plain CMOS below 1 mA (H09-41).
- `source`: H09-24, H09-38, H09-41, H09 §11.4.

#### DIO-02 Diode class compatibility and forbidden matches
- `id`: DIO-02
- `technique`: Refuse or cap matched groups by diode class: PN, Zener and Schottky never match each other; poly diodes and surface Zeners reach not even minimal class; buried Zeners are better but winner-takes-all (not Pelgrom); Al Schottkys are never matched; ratioed Schottky pairs and PSD/N-well ratioed pairs are avoided; ideality ≤ 1.1 for minimal, < 1.05 (preferably < 1.03) for moderate ratioed pairs.
- `realizes`: catches impossible matching requests at annotation.
- `applicability`: all diode matched groups. Surface Zeners differ by 50–100 mV (up to 1 V); E-B Zener lateral ballast R = J·R_S·r1·ln(r2/r1) gives only ~4 mV against ~50 mV needed, hence the quatrefoil cross-coupled layout and preference for PN/MOS stacks (H09-42, H09 §11.3.2). Schottky n > 1.1 signals inhomogeneity; small diodes are worse (n 1.006 at 100 µm → 1.41 at 5 µm) (H09 §11.3.3).
- `parameters`: `diode_class` and `ideality` tags per model; class cap (buried Zener ≤ minimal).
- `guarantees`: a diagnostic naming the violated rule.
- `does not guarantee`: anything about the surviving groups' layout.
- `pdk data needed`: diode class and ideality per model (H09-42).
- `verification`: annotator refusal table; matched Schottkys > 10–15 µm, identical, equal current density, guard-ringed, low current, compact, NBL/deep-N+ or annular cathode (H09 §11.3.3).
- `prefer over / degrade to`: diode-connected BJTs (DIO-01) over any junction diode for matching; PN/MOS stacks over Zeners for references (H09-42).
- `source`: H09-42, H09 §11.3.2, §11.3.3.

#### DIO-03 Matched diode arrays
- `id`: DIO-03
- `technique`: Keep the area-defining geometry identical (PN: the junction; field-plated Schottky: the contact opening; guard-ringed Schottky: the ring's inner perimeter); the enclosing isolation matters little, and several diodes may share one isolation region if only majority carriers flow in it. Adjacent placement, compact clusters, cross-coupled pairs for equal diodes and multiple cross-coupled pairs for exceptional class. Merged PN junctions spaced 1–2 µm beyond the unconnected rule. Matched PSD/N-well diodes as long thin PMoat strips between NMoat contacts to cut well resistance.
- `realizes`: CC and thermal rules for diodes with the same machinery as BJTs (−2 mV/°C) (H09-43).
- `applicability`: PN and Schottky diode groups at minimal class, mandatory proximity and CC for moderate/exceptional; never more than a few hundred µm apart (H09-43).
- `parameters`: unit geometry, array template (BJT-05, BJT-10), strip form for PSD/N-well.
- `guarantees`: equal area-defining perimeter and area per unit.
- `does not guarantee`: a cure for N-well resistance × β variation in PSD/N-well ratioed pairs (avoid them) (H09-42).
- `pdk data needed`: unconnected junction spacing, well sheet resistance (H09-43).
- `verification`: CentroidGroup, ThermalGradient and heat keep-outs on diode groups (H09-43); same die-location rules as BJT-19 (H09 §11.3.4).
- `prefer over / degrade to`: cross-coupled pairs → interdigitated rows → single.
- `source`: H09-42, H09-43, H09 §11.3.4.

#### DIO-04 Zener diode construction and placement
- `id`: DIO-04
- `technique`: Emitter-base Zener (V_EBO ≈ 6.8 V, range 6–12 V) uses the NPN layout with round emitters; the tank is tied to anode, cathode or a node ≥ anode, never floating; no sinker needed, NBL optional; a field plate works only over thin oxide and biased positive; an emitter field plate of metal-1 tied to the emitter extends several µm past the drawn junction against charge spreading. Buried Zeners (breakdown ≥ 1 µm below the surface, beyond the 140 nm electron / 60 nm hole thermalization distance) do not walk out: emitter-in-iso, P+ plug (0TC 5.0–5.4 V), implanted buried Zener; buried-Zener isolation must never tie to a high-impedance node (a field failure was fixed by grounding it). Nonisolated Zeners sharing substrate potential stay away from injectors of more than a few hundred µA. PSD/N-well Zener: poly field-plate ring tied to the anode forces subsurface breakdown; ≤ 1–2 µA per µm of cathode periphery; outermost strips PSD; silicide block ≥ 1–2 µm between contact and moat edge (or +1–2 µm moat over contact) for ballast.
- `realizes`: a clamp or reference with controlled breakdown and limited walkout (surface Zeners walk out 200–300 mV, sometimes > 1 V) (H09-45, H05-36).
- `applicability`: references prefer buried Zeners run at higher current density (less RTS noise); below 5 V soft breakdown makes low-voltage gate clamps impractical; avoid poly-emitter Zeners (H09 §11.1.2, §11.2.2).
- `parameters`: style {E-B, buried, PSD/N-well, series stack with diode TC compensation}, tank net, plate net, current per periphery.
- `guarantees`: tank/isolation ERC (GRD-19); corner rounding (BJT-30).
- `does not guarantee`: Pelgrom-style matching (winner-takes-all) (H09-42).
- `pdk data needed`: V_EBO, thin-oxide availability under plates, buried-Zener module, silicide block (H09-45).
- `verification`: ERC that Zener tank/isolation nets connect to a low-impedance node; injector-distance keep-out for nonisolated Zeners; periphery current limit (H09-45).
- `prefer over / degrade to`: buried → E-B with plate → PSD/N-well; PN/MOS stacks for matched references (H09-42).
- `source`: H09-40, H09-42, H09-45, H05-36, H09 §11.1.2, §11.2.2.

#### DIO-05 Schottky diode construction
- `id`: DIO-05
- `technique`: Field-relief guard ring (base or PSD ring) for matched and higher-voltage Schottkys; metal-flange field plate over thinned oxide for small low-voltage ones; NBL plus deep-N+ plug or an annular cathode contact to lower series resistance; a full deep-N+ ring for amps (also hole-blocking). Contact-array processes need a fully silicided moat anode. Low-voltage Schottkys have no shape constraint and may fill leftover space.
- `realizes`: clamps (need ≥ 150 mV lower V_F than the B–C junction; PtSi loses clamping when hot) and fast low-drop diodes (H09-44).
- `applicability`: processes with PtSi/PdSi (bipolar) or CoSi₂/NiSi (CMOS) silicides; barrier optimum 0.75–0.80 V, < 0.6 V gives excessive hot leakage; aluminium Schottkys vary lot to lot (H09-44, H09 §11.1.3).
- `parameters`: guard ring vs field plate, cathode style, anode area ≥ 10–15 µm for matching (H09-44).
- `guarantees`: BV = base/epi BV with a guard ring; matched Schottkys operate low enough that the guard ring does not conduct (H09-44).
- `does not guarantee`: no injection: a PN-guard-ringed Schottky injects into its cathode once the drop exceeds a PN V_F; even field-plated ones inject ~0.5 % (GRD-15, H05-47).
- `pdk data needed`: Schottky device and silicide type, field-plate oxide (H09-44).
- `verification`: ideality limits (DIO-02); NBL shadow never crosses a Schottky (BJT-20).
- `prefer over / degrade to`: guard ring for large/matched, field plate for small (H09-44).
- `source`: H09-44, H05-47, H09 §11.1.3.

#### DIO-06 Power diode with hole-blocking ring
- `id`: DIO-06
- `technique`: Large single anode filled with contact so the epi ballasts; BiCMOS: PSD anode inside N-well + NBL + deep-N+ ring (hole-blocking) handles amps; NBL to the outer edge of deep-N+; above 100 mA ring width ≥ 2× epi thickness; emitter over the sinker for the cathode contact.
- `realizes`: a forward diode that does not inject into the substrate above a few mA (H09-28, H09 §11.1.4, §11.2.2).
- `applicability`: junction-isolated processes; a zero-biased hole-collecting ring suffices only for a few mA (H09 §11.2.2).
- `parameters`: anode area from current, ring width, NBL enclosure.
- `guarantees`: hole blocking per GRD-07 geometry.
- `does not guarantee`: full guard efficiency at high injection; stored charge (H09 §11.1.4).
- `pdk data needed`: deep-N+/NBL layers, epi thickness.
- `verification`: ring closure; NBL enclosure of deep-N+ outer edge; width ≥ 2× epi above 100 mA.
- `prefer over / degrade to`: hole-blocking ring over hole-collecting ring above a few mA.
- `source`: H09-28, H09 §11.1.4, §11.2.2.

### 1.3 ESD protection (ESD)

#### ESD-01 Pad-based protection network
- `id`: ESD-01
- `technique`: Every pin except substrate ground gets one primary device to the substrate ring, adjacent to its pad with short wide leads, so any pin-to-pin strike crosses ≤ 2 devices; metal between any two bondpads ≤ ~2 Ω. Pads joined by stout on-chip metal (ideally a plate, also carrying package/board imbalance current) may share a device; separately bonded pads of one pin (Kelvin) each need their own device (≈ 1 nH/mm bondwire, several volts per nH at CDM); separate grounds are joined by antiparallel diodes; differential inputs get an INP–INM clamp; multiple reference domains need inter-reference clamps and then larger devices because a strike may cross ≥ 3 devices.
- `realizes`: ΔV = V_E2(I_pk) + V_E3(−I_pk) + I_pk·R_VSS with R_VSS ≤ 2 Ω → 2.6 V at 2 kV HBM (H14-54); failsafe behaviour under supply sequencing (H14-54).
- `applicability`: analog dice with sequenced supplies; die-to-die connections inside a package and probe/trim pads need no protection (H14 intro, H15-16).
- `parameters`: HBM/CDM targets (PDK-11), ring width (ESD-13), device per pin (ESD-05).
- `guarantees`: ≤ 2 devices per strike when every pin has its device (H05-22).
- `does not guarantee`: CDM protection of internal gates (ESD-11) or latch-up immunity (GRD-05).
- `pdk data needed`: maximum metal sheet resistance at 25 °C per layer, TLP-characterized device table (H14-55, H14-43).
- `verification`: every non-substrate pad has a primary device; extracted pad-to-pad resistance worst case between diametrically opposed pads (H14-42, H14-55, H15-15).
- `prefer over / degrade to`: pad-based for analog; rail-based (ESD-02) for high-speed low-C pins; custom GND/PGND (ESD-03) for power parts (H14-54).
- `source`: H14-42, H14-54, H14-55, H14-58, H05-22, H05-23, H15-15, H15-16, H14 intro.

#### ESD-02 Rail-based network
- `id`: ESD-02
- `technique`: Dual diodes from each pad to VDD and VSS rails plus one rail clamp; widen both rails to a combined ≤ 2 Ω or distribute rail clamps.
- `realizes`: ΔV = V_D2 + I_pk·R_VDD + V_E1 + I_pk·R_VSS + V_D3 with diode drops 2–3 V each and R_VDD, R_VSS ≤ 1 Ω each (H14-54).
- `applicability`: high-speed pins needing small, low-capacitance devices (H14-45 dual diodes).
- `parameters`: rail widths, clamp count/placement.
- `guarantees`: small pad devices.
- `does not guarantee`: failsafe operation: the diode forward-biases whenever the pin exceeds the supply, including during sequencing (H14-45).
- `pdk data needed`: as ESD-01.
- `verification`: rail resistance between any pad and the clamp; ΔV per pad pair.
- `prefer over / degrade to`: degrade to pad-based when supplies are sequenced (H14-54).
- `source`: H14-45, H14-54, H14-55.

#### ESD-03 Split GND/PGND network for power outputs
- `id`: ESD-03
- `technique`: Separate GND and PGND with antiparallel diodes between them; the output clamp returns to PGND so negative output transients do not disturb the substrate ground.
- `realizes`: quiet analog ground during output kickback (H14-54).
- `applicability`: parts with a power output stage; the same principle as separate injector/victim ring returns (GRD-03).
- `parameters`: APD size (ESD-08), ground pin assignment.
- `guarantees`: one extra device in the strike path is accounted for (ESD-01).
- `does not guarantee`: CDM safety at the domain crossing without clamps (ESD-11).
- `pdk data needed`: none beyond ESD-01.
- `verification`: domain-crossing clamps present; APD between grounds (H14-58).
- `prefer over / degrade to`: use when an output pin can swing below ground; else ESD-01.
- `source`: H14-54, H14-58, H14-10.

#### ESD-04 Primary-device acceptance criteria
- `id`: ESD-04
- `technique`: Accept a clipping device only if V_c (at 1 µA, sometimes 100 nA) exceeds the maximum operating voltage, V_pk (at 1.3 A for 2 kV HBM) stays below junction and oxide breakdown, and I_f (from TLP) is well above I_pk; leakage rises after stress. Snapback devices add I_t2 ≫ I_HBM and one of: V_h > V_op,max; V_t1 > V_op with available current below I_h; or V never above V_t1 (discouraged); low-V_h devices only on current-limited pins. Rate-fired devices trigger on ≈ V_pk within < 50 ns (~1 V/ns); not on pins slewing faster than 100 V/µs unless a disable circuit exists; disable-type devices are forbidden on pins needing system-level ESD.
- `realizes`: a table-driven per-pad check (H14-43).
- `applicability`: every primary device; large junctions may self-protect but filament-prone lightly doped drift regions cannot absorb HBM regardless of size (H14-43).
- `parameters`: pin specs V_op,max, slew, available current, system-level requirement.
- `guarantees`: keeps HBM peak below junction breakdown and CDM peak below field/interlevel oxide breakdown (H14-43).
- `does not guarantee`: anything without TLP data.
- `pdk data needed`: per-device V_c, V_t1, V_h, I_h, I_t2, R_on (Table 14.2 and TLP) (H14-45).
- `verification`: the criteria above per pad.
- `prefer over / degrade to`: n/a (gate).
- `source`: H14-43, H14-45.

#### ESD-05 Primary device selection by pin type
- `id`: ESD-05
- `technique`: CMOS inputs: GGNMOS or GCNMOS primary plus failsafe GGNMOS CDM clamps at the input devices (BTNMOS if the source can sustain snapback). BiCMOS inputs around 5 V: V_ECS clamp (V_t 7–12 V, no ECGR; the CDM clamp must handle the V_ECS peak). Large MOS outputs self-protect with the ESD device's layout (silicide block), an ECGR and ample substrate contacts. Large BJT collector-base junctions self-protect; NPN emitters on pins need antiparallel diodes. CMOS supply pins: GCNMOS if V_pk < BV_DSS of protected devices and V_h > V_op,max, else active FET with a disable circuit tied to the rail. High-voltage pins: PNP first, then power Zener (possible rate firing), then V_CES/V_CER if the current cannot sustain snapback. Stacking two devices costs ≈ 4× area. Last resort: active FET, possibly LDMOS. Table 14.2 behaviours: buffered Zener clipping V_c = 0.7 V_EBO; V_CES snapback V_t1 = V_CES, V_h = V_CEO(sus); V_ECS V_t1 = V_EBO, V_h = 0.8 V_EBO; APD V_c = 0.3 V; dual diodes V_c = V_DD; thick-field NMOS V_t1 ≈ 0.9 BV_DSS, V_h ≈ ½ BV_DSS; GGNMOS V_t1 = BV_DSS, V_h ≈ ½ BV_DSS; GCNMOS/BTNMOS lower V_t1, V_h ≈ ½ BV_DSS; active FET V_c ≈ 0.9 BV_DSS; MVSCR V_t = BV_DSS, V_h very low; lateral PNP V_h ≈ ½ V_t1. Every device except V_ECS needs an ECGR.
- `realizes`: the EsdPlan: pad → device type (H14-42, H14-60).
- `applicability`: all pad-based networks.
- `parameters`: pin class {input, output, supply, HV, Kelvin, probe}.
- `guarantees`: a device whose trigger/hold window fits the pin (ESD-04).
- `does not guarantee`: area minimality (stacks ×4; SCR small but latches) (H14-58, H14-49).
- `pdk data needed`: device availability per process; ESD-14 ring construction.
- `verification`: ESD-04 per pad; ECGR present (H14-45).
- `prefer over / degrade to`: failsafe devices (V_CES, V_CER, V_ECS, buffered Zener, APD) over dual diodes (H14-45).
- `source`: H14-45, H14-49, H14-58, H14-60.

#### ESD-06 Grounded-gate NMOS (GGNMOS) layout
- `id`: ESD-06
- `technique`: L at the minimum for the device style; drain NMoat overlap of contact 1–3 µm above minimum; silicide block pulled back 1–3 µm from the NMoat edge (silicided S/D is fragile); no LDD if optional; total W ≈ 600 µm for 2 kV HBM; per-finger ballast so V_t2 > V_t1 and all fingers turn on together; fillet or chamfer drain corners; characterize an array of test-structure variants.
- `realizes`: snapback clamp with V_t1 ≈ BV_DSS, V_h ≈ ½ BV_DSS (H14-46, H14-45).
- `applicability`: CMOS inputs and supply pins; it cannot protect same-style NMOS without series resistance because it triggers slightly above BV_DSS (ESD-07) (H14-47).
- `parameters`: W_total, W_finger, overlap, SAB pullback, ballast (RW, ESD-12).
- `guarantees`: multi-finger turn-on when V_t2 > V_t1 (H14-46).
- `does not guarantee`: protection without an ECGR (H14-46).
- `pdk data needed`: silicide-block layer, ESD contact spacing rules, LDD optionality (H14-46).
- `verification`: ESD-04; silicide-block extension; ECGR (H14-46).
- `prefer over / degrade to`: GCNMOS when same-style gates must be protected without series R (H14-47). Thick-field NMOS (metal gate tied to drain overlapping the junction) is obsolete (H14-46).
- `source`: H14-45, H14-46, H14-47.

#### ESD-07 Gate-coupled, backgate-triggered and active-FET clamps
- `id`: ESD-07
- `technique`: GCNMOS: C1 (PMOS gate capacitor) and R1 (small poly resistor) with R1·C1 = 20–50 ns lower V_t1 to about half the zero-V_GS value; keep V_GS low so current flows through the BJT; a disable circuit with R2·C2 ≥ 1 µs and M2 holds M1 off while powered (R2/C2 may serve several clamps). BTNMOS: deliberate backgate debias; substrate-pumped version splits predrive M2 into halves either side of M1, M2's source goes to a PMoat ring enclosing both, C1 a small PMOS cap with an HSR poly R1; an N-well strip on the supply cuts lateral majority flow. Active FET: pure MOS conduction, protection linear in W (1 kV → 2 kV by doubling W), always the largest option.
- `realizes`: rate-fired protection of same-style gates without series resistance (H14-47).
- `applicability`: supply pins and inputs; disable circuit required on high-slew pins (H14-43); the PMOS C1, M1 and M2 sit behind an ECGR (H14-47).
- `parameters`: R1C1, R2C2, W (active FET), symmetry of M2 halves about M1.
- `guarantees`: BTNMOS often smaller than GCNMOS (lower V_t1, higher I_t2) (H14-47).
- `does not guarantee`: use on pins requiring system-level ESD when a disable circuit is used (H14-43).
- `pdk data needed`: PMOS cap, HSR poly, BV_DSS.
- `verification`: time constants from netlist values; ESD-04; ECGR.
- `prefer over / degrade to`: GCNMOS → BTNMOS (smaller) → active FET (last resort) (H14-60).
- `source`: H14-43, H14-47, H14-60.

#### ESD-08 Diode ESD arrays (antiparallel, dual, NSD/P-epi, PSD/N-well)
- `id`: ESD-08
- `technique`: Minimum-width NMoat/PMoat fingers at minimum spacing in square or near-square arrays to limit metal resistance; size from the allowed series resistance (a few Ω; ≤ 3 Ω keeps ≈ 5 V across a forward diode at 1.3 A). APD between grounds: D1 NMoat/P-epi interdigitated with substrate contacts, D2 PMoat/N-well (a substrate PNP). Dual diodes: D1 to substrate, D2 to the supply pin through wide metal (ESD path ≤ 2 Ω). NSD/P-epi has inherent distributed ballast, so fully silicide it with maximum contacts and metal; adding a P-well shrinks the device but concentrates heat so the area barely changes. PSD/N-well arrays: outermost strips PSD.
- `realizes`: clipping clamps: APD V_c = 0.3 V, dual diodes V_c = V_DD; also antenna diodes and ESD diodes in plain CMOS (H14-48, H09-46).
- `applicability`: ground-to-ground, high-speed pins; pin-connected diodes need latch-up precautions and an ECGR (H14-48, H09-46).
- `parameters`: finger count and length (minimize R_series + R_metal at aspect ≈ 1), silicide policy.
- `guarantees`: robust (APD) (H14-48).
- `does not guarantee`: failsafe behaviour for dual diodes (ESD-02).
- `pdk data needed`: diffusion and contact resistance, silicide availability (H09-46).
- `verification`: R_S ≤ V_clamp/I_peak; ECGR present.
- `prefer over / degrade to`: APD for grounds; dual diodes only where failsafe is not needed.
- `source`: H14-48, H09-46.

#### ESD-09 Bipolar ESD clamps (buffered Zener, V_CES/V_CER, V_ECS)
- `id`: ESD-09
- `technique`: Buffered Zener: a Zener drives an NPN so the Zener resistance divides by β; R1 ≈ 1 kΩ lowers leakage gain, raises V_CER and resists dV/dt turn-on; 300–600 µm² of emitter gives 2 kV HBM; all in one tank with a shared deep-N+; Zeners stackable below the NPN's V_CER; injects electrons (ECGR) and shows some rate firing (smaller R1 reduces it). V_CES/V_CER clamp: V_t1 from V_CES (≈ 65 V for a 40 V NPN) down to V_CER set by R1, V_h = V_CEO(sus) ≈ 45 V; fillet base corners; base overlap of contact 1–2 µm above rule, emitter overlap a couple of µm; 300–500 µm² for 2 kV; ECGR. V_ECS clamp: reverse-active NPN with V_t1 ≈ V_EBO and V_h ≈ 60–80 % of V_EBO; under 600 µm² gave 2 kV HBM / 200 V MM, larger survived 10 kV; two emitters share one base contact; seldom needs an ECGR because the collector is grounded with NBL and a large sinker; stacking 2–3 possible but series resistance forces larger devices.
- `realizes`: failsafe clamps for bipolar/BiCMOS pins (H14-50, H14-45).
- `applicability`: BiCMOS inputs around 5 V (V_ECS), HV pins (power Zener, V_CES/V_CER) (H14-60).
- `parameters`: emitter area scaled to the HBM target, R1, stack count.
- `guarantees`: failsafe (no reliance on other pins) (H14-45).
- `does not guarantee`: protection of junctions avalanching below 10 V (a 60 V trigger / 40 V snapback primary cannot) (H14-59).
- `pdk data needed`: V_EBO, V_CES, V_CEO(sus), NBL/sinker.
- `verification`: ESD-04; emitter area versus target; ECGR except V_ECS (H14-45).
- `prefer over / degrade to`: V_ECS → buffered Zener → V_CES, by size (H14-50).
- `source`: H14-45, H14-50, H14-59, H14-60.

#### ESD-10 Lateral PNP and medium-voltage SCR clamps
- `id`: ESD-10
- `technique`: CMOS lateral PNP: closely interdigitated narrow emitter and collector fingers, no ballast needed (holes filament less); snaps back deeply (base conductivity modulation), suiting HV pins; the floating-base variant has lower V_t (V_CEO), almost no snapback and no pin-to-N-well injection, but still gets an ECGR. MVSCR: triggered by an NMoat/P-epi Zener (V_t ≈ BV_DSS), V_h can be ~1 V and I_h a few mA (a high-holding variant reached 70 mA); use only if V_op,max < V_h or supply current < I_h; stacking erases the area benefit.
- `realizes`: small HV clamps (PNP) and the smallest low-voltage clamp (SCR) (H14-49).
- `applicability`: HV pins (PNP first choice) (H14-60); SCR only on pins that cannot sustain its hold current (H14-49).
- `parameters`: finger pitch, floating vs tied base, SCR trigger.
- `guarantees`: deep snapback for HV (PNP).
- `does not guarantee`: latch-up immunity of the SCR by design (H14-49).
- `pdk data needed`: BV_DSS, N-well/P-epi parameters.
- `verification`: ESD-04 hold-current rule; ECGR (H14-49).
- `prefer over / degrade to`: PNP → power Zener → V_CES/V_CER on HV pins (H14-60).
- `source`: H14-49, H14-60.

#### ESD-11 Secondary and CDM protection at the protected gate
- `id`: ESD-11
- `technique`: Place the CDM clamp or RC filter next to the gate oxide it protects, tied to that circuit's own supply and ground (2 Ω of metal at 10 A is 20 V). RC filter: R1·C1 ≥ 10 ns (0.25 pF → 40 kΩ), C1 next to the protected device with a short ground, R1 anywhere on the line. CDM clamp: series R1 plus small single-finger GGNMOS clamps gate→source and source→gate each meeting RW; the version with an extra GGNMOS is failsafe and preferred; omit clamps for absent gate types; rail taps joined near the protected transistors with a lead wide enough for the CDM current; clamps at every supply/ground domain crossing. An ECGR goes around the clamp devices unless R1 > ~25 kΩ. Bipolar inputs: a diode-connected E–B clamp with series R1 and a balancing R2 = R1 in the other input to cancel base-current IR offset. Oxide check: V_ox = V_h + I_pk·(R_int + R_metal) ≤ oxide limit (90 Å 5 V oxide ≈ 22 V; 50 Å ≈ 24 MV/cm for 100 ns); a 5 V GCNMOS (6 V, 4 Ω) + 2 Ω metal gives 13.8 V at 1.3 A but 46 V at 10 A CDM.
- `realizes`: gate-oxide survival of 500 V CDM (> 5 A in ~0.25 ns, 1–2 ns) (H14-44, H14-52, H14-53).
- `applicability`: every pin-connected gate oxide and bipolar E–B junction (E–B avalanche permanently degrades low-current β) (H14-59, REL-12).
- `parameters`: R1, C1, clamp finger size, Proximity(clamp, protected device), rail-join distance.
- `guarantees`: current limited to (V_primary − V_secondary)/R_series (500 Ω + 50 Ω Zener → 44 mA, 9.2 V) (H14-44).
- `does not guarantee`: matching: R2 must be added to keep the diff pair balanced (H14-59).
- `pdk data needed`: oxide thickness and ESD-duration field limit, PEX metal resistance (H14-53).
- `verification`: R1·C1 ≥ 10 ns; Proximity met; V_ox from extracted metal for HBM and CDM currents (H14-52, H14-53); checklist item 7 (H15-43).
- `prefer over / degrade to`: failsafe clamp with extra GGNMOS → RC filter.
- `source`: H14-44, H14-52, H14-53, H14-59, H15-43.

#### ESD-12 Drain ballast (RW product) and self-protecting outputs
- `id`: ESD-12
- `technique`: An output NMOS triggers like a GCNMOS through C_GD, so a parallel GGNMOS does not help; use a low-V_t rate-fired clamp, an enlarged and ballasted output device, or GGNMOS/GCNMOS plus a series resistor. Per-finger ballast R_ballast = RW/W_finger (RW = 5 mm·Ω with 50 µm fingers → 100 Ω; five fingers at 100 Ω give 20 Ω effective and still meet RW). Silicide block over the drain restores ballast removed by silicide; increase diffusion overlap of contact for intrafinger ballast.
- `realizes`: multi-finger turn-on and filament immunity of pin-connected outputs (H14-51, H14-60, H05-08).
- `applicability`: output MOS on non-rail pins; power devices.
- `parameters`: RW (deck), W_finger, SAB extension or series poly.
- `guarantees`: each finger carries its share.
- `does not guarantee`: protection without the ECGR and substrate contacts (H14-60).
- `pdk data needed`: RW per device style, silicide-block layer, ESD contact-to-gate spacing (H14-51, H05-08).
- `verification`: per-finger R ≥ RW/W_finger; silicide block present.
- `prefer over / degrade to`: ballasted self-protecting output → clamp plus series R.
- `source`: H14-51, H14-60, H05-08.

#### ESD-13 ESD-path metallization and resistors
- `id`: ESD-13
- `technique`: Keep the HBM metal drop ≤ ~3 V (≤ 2.3 Ω at 1.3 A), with 2 Ω allotted to the substrate ring; ring width W_G ≅ R_S·(W_D + L_D − 4d_C + 2√2·d_C)/(4 Ω) using the process maximum R_S at 25 °C and stacking all metals; size cross-sections adiabatically, A = √(ρ·τ·I_pk²/(2·C_V·ΔT)) with τ = 225 ns and ΔT ≈ 50 °C (Al 6.5 µm², Cu 4.3 µm² at 1.3 A; 0.5 µm Al needs ≥ 13 µm width); resistors in the path use ρ = R_s·t (200 Ω poly at 2.5 mΩ·cm carrying 20 mA → 3.7 µm², 7 µm wide; 0.2 µm TiSi₂ → 0.40 µm², 2.0 µm); 45° chamfers on inside corners extending back ≥ half the lead thickness; stress slots along current flow or parallel narrower leads; layer changes overlap fully and fill with maximum vias.
- `realizes`: ≤ 2 Ω pad-to-pad paths without fusing (H14-55, H14-56, H14-57).
- `applicability`: all ESD-path nets; "20X/40X EM" rules have little justification (H14-56).
- `parameters`: die W_D, L_D, corner exclusion d_C, metal stack, ΔT.
- `guarantees`: adiabatic rise bounded for a 225 ns HBM pulse (H14-56).
- `does not guarantee`: CDM (10 A) drops: 2 Ω → 20 V, hence ESD-11 at the gate (H14-52).
- `pdk data needed`: ρ and C_V per conductor (Al 2.7 µΩ·cm/2.42 J/°C/cm³, Cu 1.7/3.45, CoSi₂ 15/0.56, NiSi 10.5/3.45, TiSi₂ 15/0.85, Si varies/1.66), thickness, max sheet R (H14-56).
- `verification`: per-segment width ≥ A/t; chamfers and via fill on ESD nets; pad-pair resistance (H14-55, H14-57).
- `prefer over / degrade to`: solid plate over parallel leads where slotting is not required.
- `source`: H14-55, H14-56, H14-57, H14-52, H05-06.

#### ESD-14 Peripheral ESD cluster behind one ECGR
- `id`: ESD-14
- `technique`: Place ESD devices at the periphery next to their pads and enclose the whole pad/ESD band with one electron-collecting guard ring that separates it from the core; PMOS capacitors and disable devices of GCNMOS clamps sit behind the ring too; CDM clamps get their own ECGR unless R1 > ~25 kΩ.
- `realizes`: latch-up and noise isolation of the core from the main injectors (H14-26, H14-45).
- `applicability`: every pad-based or rail-based die; bump/pillar dice put ESD at the periphery or next to bumps (H15 §15.3).
- `parameters`: ring construction (GRD-05), tie net (highest supply).
- `guarantees`: fewer rings than ringing every victim (injectors are fewer) (H14-26).
- `does not guarantee`: protection of V_ECS clamps (none needed) (H14-45).
- `pdk data needed`: GRD-05 data.
- `verification`: every Table 14.2 device inside the ring; ring continuity (GRD-12).
- `prefer over / degrade to`: one shared band over per-device rings.
- `source`: H14-26, H14-45, H14-47, H14-52, H15 §15.3.

### 1.4 Guard rings, taps, latch-up, merging and isolation (GRD)

#### GRD-01 Injector and aggressor classification
- `id`: GRD-01
- `technique`: Mark as a minority-carrier injector every diffusion (source/drain, collector, emitter, diode, resistor body) on a package pin that is neither a supply nor substrate ground, including through series resistance below ~50 kΩ (10 kΩ "may trigger"; conservative designers use 50 or 100 kΩ; 100 kΩ has failed in very-low-current circuits); every node capacitively coupled to a switching output (< 1 pF latched adjacent logic); every device with a deliberately forward-biased backgate (charge pumps, clamps); Schottky diodes (especially PN-guard-ringed); saturating BJTs and forward-biased Schottkys in shared regions; a collector pulled below ground. Mark as noisy every clock net, charge-pump node or node whose swing × slew is large (a > 20 V swing through ~1 fF upset a µA mirror). Pin polarity: positive-supply pins see no below-ground transients, negative-supply pins none above ground, substrate pins are exempt.
- `realizes`: the aggressor list that drives every ring, tub, merge and keep-out decision (H14-01, H05-47, H14-23, H14-08, H15-43).
- `applicability`: all dice; latch-up is intermittent and application dependent, rarely found by simulation or test (H14 §14.2).
- `parameters`: R_inj threshold (default 50 kΩ per ch. 14/15, 10 kΩ per ch. 5), swing/slew threshold, op-point V_BS forward threshold (~−0.3 V) (H14-23).
- `guarantees`: exhaustive enumeration through resistor paths and capacitors, which "are not always easily identified" by hand (H14-01).
- `does not guarantee`: correct polarity without pin classes (user input).
- `pdk data needed`: none; needs netlist ports with pin classes.
- `verification`: every injector has a ring (GRD-11) and its own region (GRD-16).
- `prefer over / degrade to`: n/a (input).
- `source`: H14-01, H14-08, H14-23, H05-47, H15-43, H14 §14.2.

#### GRD-02 Minimum-width substrate-contact ring around the injector
- `id`: GRD-02
- `technique`: Ring each noise injector with substrate (majority-carrier) contacts at minimum width; rings around victims are only an addition. In thin isolated layers make the annular contact ≥ t/2 wide (t = layer thickness).
- `realizes`: majority-carrier noise attenuation (contacts around the injector attenuate more than contacts around the victim; widening the ring helps only a little) (H14-09, H05-52).
- `applicability`: every noisy or injector device; rings that will collect more than a few mA must not merge with isolation or substrate ties (H14-34).
- `parameters`: width (min, or t/2 in thin layers), contact density from expected current.
- `guarantees`: pinned backgate potential and a recombination sink for majority carriers.
- `does not guarantee`: collection or blocking of minority carriers (that needs GRD-05…GRD-07) (H14-27).
- `pdk data needed`: tap minimum width, layer thickness t (PDK-01/02).
- `verification`: ring closure; GRD-03 return path; GRD-12 capacity.
- `prefer over / degrade to`: always; add GRD-04 when sensitive blocks are near.
- `source`: H14-09, H14-27, H14-34, H05-52.

#### GRD-03 Separate return paths for injector and victim rings
- `id`: GRD-03
- `technique`: Return the injector ring's ground and the victim ring's ground by separate leads to the ultimate ground: separate pins ideally, otherwise separate bondwires to one pin; never let substrate contacts feed a sensitive ground lead.
- `realizes`: no shared segment through which injected current can lift the sensitive ground (H14-10, H14-35).
- `applicability`: whenever a sensitive block and an injector share a die; the GND/PGND split of ESD-03 is the pad-level form.
- `parameters`: sub-net names (noisy ring ground, quiet ring ground), declared star point or port.
- `guarantees`: tree-level separation can be checked exhaustively on the routed net.
- `does not guarantee`: isolation at GHz, where bondwire inductance dominates (H14-12).
- `pdk data needed`: none.
- `verification`: routed trees share no segment; joins only at the declared star point or pad (H14-10).
- `prefer over / degrade to`: separate pins → separate bondwires → on-die star point.
- `source`: H14-10, H14-35, H14-12.

#### GRD-04 N-type noise-blocking ring outside the tap ring
- `id`: GRD-04
- `technique`: Outside the injector's substrate-contact ring, add an N-type ring (NSD, or N-well which is deeper and more effective but larger) tied to a clean supply or ground; it partially blocks the conductive substrate path and extracts some capacitively injected noise.
- `realizes`: reduced substrate coupling to nearby sensitive blocks (H14-11).
- `applicability`: noisy devices with a sensitive block within a few epi thicknesses; does not replace substrate contacts (H14-11).
- `parameters`: layer (NSD/N-well), gap (deck N-well-to-PSD spacing), tie net.
- `guarantees`: additional attenuation on lightly doped substrates when combined with distance (H14 §14.1).
- `does not guarantee`: blocking of electrons under it (an N-well in deep lightly doped wells is of doubtful value) (H14-28).
- `pdk data needed`: N-well spacing to P+ taps.
- `verification`: tie net is quiet; concentric outside GRD-02.
- `prefer over / degrade to`: N-well ring when area allows; NSD ring otherwise; isolated tub (GRD-09) when the deck has one.
- `source`: H14-11, H14-28, H14 §14.1.

#### GRD-05 Electron-collecting guard ring (ECGR)
- `id`: GRD-05
- `technique`: Around an electron injector (NSD on a pin that can go below ground, ESD devices) draw an N-type ring using every available N layer, immediately adjacent to the injector. Pure CMOS: an NMoat ring at least as wide as the P-epi thickness (or the P-well depth in retrograde processes), because low supplies cannot deepen the depletion region; an N-well ring contacted with as much NMoat as possible and tied to a supply; if the ring must be grounded, a wide NMoat ring beats an N-well ring. Tie to the highest supply (deeper depletion, less debias); a grounded ring that could saturate gets a second supply-tied ring around it. Strap every portion with continuous metal sized for tens to hundreds of mA; along a die edge the ring segment is useless, so end an L or U ring at the edge. Standard bipolar: deep-N+ + NBL + emitter; BiCMOS: NBL + deep-N+ + NMoat + N-well (> 90 % efficient, groundable at 100–200 mA with deep-N+).
- `realizes`: electron collection before they reach N-wells elsewhere (H14-28, H14-30, H05-56).
- `applicability`: every injector of electrons; P− substrates are far more latch-up prone and ECGRs weaken there (re-spins reported); a P+ substrate makes the ring 10–100× more effective (H14-32, H05-56); gate drivers and inductive-load drivers always need one as near the offender as possible (H14 §14.2.1).
- `parameters`: width, layers, tie net, strap width, open side toward the die edge.
- `guarantees`: efficiency > 90 % with buried layers (H14-32); "significantly improved latch-up immunity" for a wide NMoat ring in CMOS (H14-28).
- `does not guarantee`: survival under severe kickback ("no amount of guard rings … will guarantee") (H14-32); metal gaps debias the ring and make it reinject (H05-56).
- `pdk data needed`: P-epi thickness or P-well depth, substrate type, NBL/deep-N+ availability (H14-28, H14-32).
- `verification`: width ≥ depth; I_collect·R_vertical < supply headroom (H14-30); strap continuity; ring adjacent to the injector.
- `prefer over / degrade to`: HBGR (GRD-07) when buried layers exist and space is limited; isolated tub (GRD-09) as the 100 % form; degrade to spacing plus hole rings on P− substrates (H14 §14.2.1).
- `source`: H14-28, H14-30, H14-32, H05-56, H14 §14.2.1.

#### GRD-06 Hole-collecting guard ring (HCGR)
- `id`: GRD-06
- `technique`: Inside the N-well (or tank) around a hole injector (PSD on a pin that can exceed the supply), draw a PMoat ring tied to ground with width ≥ the N-well depth; even minimum width helps. In standard bipolar the HCGR is a reverse-biased base ring tied to ground or a negative supply; it may merge with isolation only if it collects no more than a few mA and contains substrate contacts. BiCMOS with a P buried layer: PBL + P-well around a shallow N-well is a 100 % HCGR, contacted by a grounded PMoat ring as wide as possible.
- `realizes`: hole collection before they reach the substrate (H14-29, H05-57).
- `applicability`: requires a retrograde N-well whose bottom doping is ≥ 10× the middle; useless otherwise, where the alternative is an NMoat backgate ring (majority) plus spacing (H14-29).
- `parameters`: width, tie (ground, or tank potential with debias risk), merge policy.
- `guarantees`: inside an HBGR it raises blocking above 99 % (H14-33).
- `does not guarantee`: blocking (pure CMOS cannot build an HBGR) (H14-29).
- `pdk data needed`: `nwell_retrograde`, N-well depth, PBL availability (H14-29).
- `verification`: retrograde flag true before emission; width ≥ well depth; GRD-12 capacity.
- `prefer over / degrade to`: HBGR when deep-N+/NBL exist; else HCGR; else majority N+ ring plus spacing.
- `source`: H14-29, H14-33, H05-57, H14 §14.2.2, §14.2.6.

#### GRD-07 Hole-blocking guard ring (HBGR)
- `id`: GRD-07
- `technique`: A deep-N+ (sinker) ring or deep-trench ring completely encircling the injector plus an NBL (or radical retrograde well) floor; drawn NBL and drawn N-well extend at least to the outer edge of drawn deep-N+ (NBL beyond the trench outer edge for DTI); the N+/N− doping ratio must be ≥ ~100:1, with deep-N+ above 10²⁰ cm⁻³ when the well surface exceeds 10¹⁸ cm⁻³.
- `realizes`: hole blocking > 95 %, often > 98 %, > 99 % with an HCGR inside; DTI permeation measured under 5 % (H14-33).
- `applicability`: buried-layer and DTI processes; NBL doping ≥ 50× the N-well bottom (H01-19); more effective than an ECGR in standard bipolar, so preferred when space is limited (H05-58). Permeation P = (A·D_H·τ_L/(W_H·V_L))·(N_L/N_H) (example 0.0018) (H05-55).
- `parameters`: ring width (≥ 2× epi for > 100 mA, BJT-29), enclosure margins.
- `guarantees`: any gap lets holes escape, so closure is the invariant (H05-58).
- `does not guarantee`: benefit with a permeable NBL, which can increase substrate injection (HCGRs are unaffected) (H14-33); LV BiCMOS wells with surface doping > 10¹⁸ defeat deep-N+ (H05-55).
- `pdk data needed`: deep-N+/NBL/DTI layers and dopings (PDK-03).
- `verification`: closed ring; enc(NBL, deep-N+ outer) ≥ 0; enc(N-well, deep-N+ outer) ≥ 0 (H14-33).
- `prefer over / degrade to`: HBGR → HCGR → N-well tie plus spacing.
- `source`: H14-33, H05-55, H05-58, H01-19.

#### GRD-08 P-bars, N-bars and tank splitting for merged laterals
- `id`: GRD-08
- `technique`: Between lateral PNPs sharing a tank insert a P-bar (base diffusion strip, ends extending into isolation) or heavily doped N-bar; give any lateral that may saturate its own tank; split a split-collector device whose segment may saturate. Symmetric arrays at equal emitter current and V_BC cancel cross-injection and may share a tank.
- `realizes`: cross-injection limited to about 1 % unsaturated (a saturating collector in a two-LPNP tank sends ~¼ of its current across) (H14-15).
- `applicability`: merged lateral PNPs, LPNP mirrors, NPN collector merged with PNP base (SCR, GRD-20).
- `parameters`: bar layer and doping, tank partition.
- `guarantees`: most, not all, cross-injection blocked; ask what 1 % leakage does to the circuit (H14-15).
- `does not guarantee`: matching (P-bar isolation not recommended for matched laterals, BJT-16).
- `pdk data needed`: base/isolation layers (bipolar decks).
- `verification`: op-point saturation check per member (V_CE margin) (H14-15).
- `prefer over / degrade to`: separate tanks when 1 % would break the circuit (H14-15).
- `source`: H14-15, H05-57.

#### GRD-09 Isolated tub for sensitive or injecting NMOS
- `id`: GRD-09
- `technique`: Put the NMOS in a P-region floored by NBL and walled by deep-N+ (or DTI with a tilted As implant), in CMOS a deep N-well tub, with its own ground return on a separate pin.
- `realizes`: a 100 %-efficient ECGR; an audio preamp beside a ~1 A switcher heard no noise (H14-12, H14-32).
- `applicability`: sensitive NMOS blocks near switchers; injector NMOS; less effective at GHz because of bondwire and pin inductance (H14-12). SOI is not immune when PMOS and NMOS share a tank: isolate injectors and scatter taps through both wells (H14-32).
- `parameters`: tub extent, tie net (quiet supply for the N-walls), return pin (GRD-03).
- `guarantees`: complete minority-carrier enclosure at DC and low frequency.
- `does not guarantee`: immunity to capacitive coupling through the tub junction at high frequency.
- `pdk data needed`: `dnwell` (or NBL + deep-N+ or DTI) and its enclosure rules (H14-12).
- `verification`: tub closure; separate return route (GRD-03).
- `prefer over / degrade to`: tub when available; else GRD-05 ring plus GRD-04.
- `source`: H14-12, H14-32.

#### GRD-10 Supply-tied wells as free collectors between injector and victim
- `id`: GRD-10
- `technique`: Place supply-tied N regions (PMOS wells, tanks) on the straight path between an injector and a sensitive block; they act as effective electron collectors.
- `realizes`: extra ECGR action at zero area cost (H14-31).
- `applicability`: placement cost term; standard bipolar tanks with as much NBL and deep-N+ as possible (H14 §14.2.1).
- `parameters`: which wells count (tied to a supply, not floating).
- `guarantees`: geometric shadowing credit.
- `does not guarantee`: a quantified efficiency.
- `pdk data needed`: none.
- `verification`: ray from injector to victim intersects a supply-tied well bbox.
- `prefer over / degrade to`: a bonus on top of GRD-05, never a replacement.
- `source`: H14-31, H14 §14.2.1.

#### GRD-11 Guard-ring taxonomy and type selection by injected carrier
- `id`: GRD-11
- `technique`: Distinguish majority-carrier tap rings (P+ around NMOS, N+ around PMOS, tied to the device bulk) from minority-carrier guard rings (opposite-type diffusion tied to a supply or ground): ECGR = N in P collecting electrons; HCGR = P in N collecting holes; EBGR = P+/P− high-low junction (rare, needs PBL/deep-P+); HBGR = N+/N− high-low junction. Select by the carrier: NSD/P-epi diffusion on a pin that goes below ground injects electrons → ECGR (or tub); PSD/N-well on a pin above the supply injects holes → HCGR or HBGR.
- `realizes`: the correct ring for every GRD-01 injector; victims get majority contacts plus distance (H14-27, H05-54).
- `applicability`: all processes; minority carriers travel hundreds to thousands of µm (electrons ~800 µm in 10 Ω·cm P-Si), so taps alone cannot stop them (H05-54, H05-52).
- `parameters`: ring kind, tie net (supply for ECGR, ground for HCGR), width (GRD-05/06).
- `guarantees`: the ring's purpose is encoded in its type.
- `does not guarantee`: anything if a majority tap is mislabelled as a minority ring (a known defect in Philis's current naming) (H14-27).
- `pdk data needed`: as GRD-05…GRD-07.
- `verification`: ring polarity opposite to the surrounding region; tie net is a rail.
- `prefer over / degrade to`: blocking → collecting → majority tap plus spacing.
- `source`: H14-27, H05-54, H05-52, H05-47.

#### GRD-12 Ring current capacity and metal strap sizing
- `id`: GRD-12
- `technique`: Size ring contacts and strap metal from the current the ring must collect (default the 100 mA latch-up test current, 50 ms, both polarities) against ≤ 0.3 V debias; strap continuously with metal (no gaps), sized for tens to hundreds of mA transients; forbid merging a ring with isolation or substrate ties when it collects more than a few mA; P regions on pins exposed to ~100 mA get their own tank with nearby substrate contacts.
- `realizes`: rings that survive the latch-up test (grounded low-R rings survive 100–200 mA only when every portion is strapped) (H05-56, H14-34, H05-49).
- `applicability`: every minority ring and injector tap ring.
- `parameters`: I_collect, V_debias (0.3 V), strap width from EM and self-heating.
- `guarantees`: a numeric debias check per ring instead of a fixed resistance cap (H14-34).
- `does not guarantee`: efficiency (that is GRD-05…GRD-07).
- `pdk data needed`: contact resistance, metal sheet R, EM limits (PDK-05).
- `verification`: I·R_ring ≤ 0.3 V; continuity on the strap layer; crossings go over on a higher metal (H05-56).
- `prefer over / degrade to`: n/a.
- `source`: H14-34, H05-49, H05-56.

#### GRD-13 Substrate and well contact strategy by substrate type
- `id`: GRD-13
- `technique`: (a) Thin lightly doped layer on a heavily doped sublayer (P− epi on P+, N-well over NBL, retrograde well): vertical R_V = ρ·t/A_dif; disperse small contacts anywhere (an array of 10 µm² contacts spaced ≥ 2t has 13 % of the resistance of one large equal-area contact); fill unused area with contacts; ring majority injectors with an annular contact whose width benefit tapers beyond t/2. (b) Thick lightly doped substrate: contacts within half the die thickness of every device (~100 µm; nothing farther than 100–200 µm on a 250 µm die), each as large as space permits; contacts farther than half the die thickness help little; the "closer ring is lower R" intuition is wrong (10 µm dot + 5 µm ring: 110 Ω at 5 µm spacing, 22 Ω at 100 µm). (c) Thin isolated layers (wells without buried layer, SOI, P-well on P−): annular contact ≥ t/2 wide around majority injectors as close as possible; small contacts ≤ 20–50·t apart because resistance rises with spacing (constriction). Standard bipolar isolation (10–20 Ω/□): ring majority injectors, minimize metal gaps, scatter contacts.
- `realizes`: substrate/well debias ≤ 0.3 V at the latch-up test current (R_V ≤ 3 Ω → 0.33 mm² of contact at ρ = 10 Ω·cm, t = 10 µm) (H05-50, H05-51, H05-52, H05-53).
- `applicability`: by deck substrate type (PDK-01); a typical CMOS die's scribe-street contacts handle 10–20 mA of injected substrate current, beyond which unused area is filled with contacts (H01-08); on P+ substrates fill with contacts rather than bypass capacitors when substrate resistance would exceed 1 Ω (H15-60).
- `parameters`: contact size, pitch, ring width, fill policy; contact area ∝ injected current placed on the injector-to-victim path (H01-08).
- `guarantees`: a computed R_sub metric (parallel combination of per-tap resistances) against V_debias/I_test (H05-50).
- `does not guarantee`: minority-carrier stopping (GRD-11).
- `pdk data needed`: substrate type, epi ρ and t, well depth, die thickness (PDK-01/02/10).
- `verification`: R_sub ≤ budget; max device-to-tap distance ≤ d_tap (½ die thickness or deck latch-up rule); tap-to-tap ≤ k·t in wells (H05-51, H05-52).
- `prefer over / degrade to`: strategy (a) on epi-on-P+ decks; (b) on bulk P− (sky130 is bulk P−); (c) inside wells.
- `source`: H05-50, H05-51, H05-52, H05-53, H01-08, H15-60.

#### GRD-14 Tap distance and backgate ties (latch-up)
- `id`: GRD-14
- `technique`: Keep every point of every active area within the deck's maximum tie distance of a tap of its own bulk net (sidecar values 3 µm in sky130, 20 µm in IHP); place backgate contacts immediately adjacent to NMOS; scatter low-R ties through merged structures when there is no low-R sublayer; use integrated (butted) taps in large devices; PSD/NSD must lie under every tap contact.
- `realizes`: pinned epi surface potential and reduced R1/R2 of the parasitic SCR (H01-07, H14-24, H05-48).
- `applicability`: all MOS; SOI/DTI tanks holding both polarities need many ties in both wells (H14-24).
- `parameters`: tie_max_dist, tap pitch inside wide devices.
- `guarantees`: exhaustive distance check for every diffusion point (H01-07).
- `does not guarantee`: minority-carrier immunity (GRD-11).
- `pdk data needed`: tie_max_dist per bulk type, tap implant enclosure (H01-07).
- `verification`: global nearest-tap distance; dp penalty when a move exceeds it (H01-07).
- `prefer over / degrade to`: butted tap (GRD-18) → tap row in cell → separate tap cell.
- `source`: H01-07, H14-24, H05-48.

#### GRD-15 Merge-legality predicate for shared wells, tanks and backgates
- `id`: GRD-15
- `technique`: Allow a set of devices to share a well/tank/backgate only if all hold: (1) same bulk net; (2) no member injects minority carriers (saturating BJT, forward-biased Schottky, pin diffusion) unless every other member tolerates it; (3) worst debias I_well·R_share stays below the forward-bias trigger (~300 mV at 150 °C; 100 mA through a 5 Ω sinker gives 0.5 V; 2 µA through a 500 Ω shared base gives 1 mV; 1000 □ of 50 mΩ/□ metal at 100 µA gives 0.5 mV); (4) no mixing of noisy with sensitive (sensitive with sensitive and noisy with insensitive are fine; if forced, add plentiful majority contacts); (5) matched devices share no resistive region in their critical path (shared tank/collector is fine, a shared base is not; a shared well with current through it shifts V_T by body effect); (6) the well voltage stays at or above every enclosed P region (Zener strings at the cathode end); (7) a resistor sharing a well tie with a current-drawing transistor has I·R_tie < 0.3 V (deep-N+ tie ≤ 100 Ω removes the risk; without it hundreds to thousands of Ω); (8) pin-connected and noisy resistors do not share wells with sensitive ones; (9) every well-tie contact is fully enclosed by N+ (a contact touching lightly doped N forms a Schottky injector with ~0.5 % minority current, enough to latch if β > 200). Rejected pairs get a spacing floor of 2× well spacing.
- `realizes`: safe area recovery from merging (three NPNs in one tank ≈ 70 % of separate; three PMOS in one well with shared gate < 50 %) (H14-03, H14-04, H14-05, H14-06, H14-13, H14-16, H14-17, H14-21, H14-25).
- `applicability`: every candidate merge the placer tries; CMOS/BiCMOS practice merges backgates only (H14 §14.1.5). Ask the three questions: can any member inject, can any member debias the shared region (a deep-N+ plug handles tens of mA), can noise coupling upset it; when in doubt separate (H14 §14.1.5).
- `parameters`: R_share from well sheet resistance × distance to the nearest tap; I_well from the op-point or the 100 mA test current for pin devices.
- `guarantees`: a Θ residual per shared region.
- `does not guarantee`: correctness of parasitic-BJT simulation, which "can mislead" (H14 §14.1).
- `pdk data needed`: well sheet resistance, tie resistance (deep-N+), well depth (PDK-02).
- `verification`: all nine clauses on the union before `well_bridges` or ring clustering merge anything (H14-03).
- `prefer over / degrade to`: separate wells whenever a clause fails.
- `source`: H14-03, H14-04, H14-05, H14-06, H14-13, H14-16, H14-17, H14-21, H14-25, H14 §14.1, §14.1.5.

#### GRD-16 Low-risk merge whitelist and own-region rules
- `id`: GRD-16
- `technique`: Always merge: sections of one matched device (watch tank/well modulation and emitter spacing), fingers of a power device, a sense transistor with its power device, back-to-back LDMOS with a common drain (interdigitate the two sources), Schottky-clamped NPNs, I²L multi-emitter NPNs, Darlingtons (shared tank with a deep-N+ bar contact of 5–10 Ω handles hundreds of mA), base turnoff resistors in the NPN tank (a substrate-tied end may run into isolation with a substrate contact nearby). Give its own backgate region to: any MOS whose source/drain is on a pin that is neither a supply nor substrate ground (else add minority rings and low-R backgate contacts on adjacent complementary devices); devices with deliberately forward-biased backgates; saturating lateral PNPs; Zeners whose cathode could exceed the tank voltage; shared-collector BJTs get a deep-N+ plug and never merged bases with a fluctuating tank.
- `realizes`: the whitelist and the own-region list of GRD-15 (H14-18, H14-22, H14-23).
- `applicability`: all processes; only saturating NPNs and large power devices need full deep-N+ rings, a bar contact suffices otherwise (H14 §14.1.2).
- `parameters`: pattern membership from the annotator catalog.
- `guarantees`: consistent treatment of every output device found from the port list (H14-22).
- `does not guarantee`: area recovery beyond backgate sharing in CMOS (H14 §14.1.5).
- `pdk data needed`: none.
- `verification`: output MOS has own well; charge-pump devices flagged (H14-22, H14-23).
- `prefer over / degrade to`: n/a.
- `source`: H14-18, H14-22, H14-23, H14 §14.1.2, §14.1.4, §14.1.5.

#### GRD-17 Well partitioning by bulk and supply; PMOS well sharing
- `id`: GRD-17
- `technique`: The well key of a PMOS is its bulk net; PMOS referenced to different supplies occupy separate N-wells; any number of same-backgate PMOS share one well (the deep well outdiffuses a lot, so merging saves substantial area); matched pairs must have identical bulk and source nets (or equal V_SB); an NMOS whose bulk is not the substrate needs an isolated P-well (deep N-well) or is flagged.
- `realizes`: area recovery (< 50 % with shared gate) without body-effect or bias violations (H01-05, H01-06, H14-04).
- `applicability`: N-well CMOS; placement should reward adjacency of same-group PMOS rather than bridging after placement (H01-05).
- `parameters`: group by bulk net; well spacing by voltage (GRD-21).
- `guarantees`: well identity for every matched group and supply domain (H01-06).
- `does not guarantee`: legality without GRD-15 (injector, debias, noise).
- `pdk data needed`: N-well depth/outdiffusion, well spacing rules (H01-04).
- `verification`: well tie = highest supply of the source network; PMOS with a signal-net bulk flagged (H01-01).
- `prefer over / degrade to`: share where GRD-15 allows; else separate with voltage-dependent spacing.
- `source`: H01-05, H01-06, H01-04, H14-04.

#### GRD-18 Butted source–tap
- `id`: GRD-18
- `technique`: When an NMOS source ties to substrate potential, abut the PSD substrate contact against the NSD source (implants split at the shared edge); never abut PSD and NSD at different potentials (the N+/P+ junction leaks and breaks down at a very low voltage).
- `realizes`: compact taps and minimal tie distance (H01-09, GRD-14).
- `applicability`: end source regions with net(S) = net(B); decks that allow the abutment when the two are connected by metal (H15-44 note 1).
- `parameters`: butted variant flag per device.
- `guarantees`: lower backgate resistance at the source.
- `does not guarantee`: matching symmetry (keep butted taps identical across a matched group).
- `pdk data needed`: abutment rule and implant clipping enclosures (H01-10).
- `verification`: ERC: every touching N+/P+ pair is on one net (H01-09).
- `prefer over / degrade to`: butted → separate tap row.
- `source`: H01-09, H01-10, H15-44.

#### GRD-19 Isolation-junction bias ERC
- `id`: GRD-19
- `technique`: Check at every op-point corner (and transient when available) that every isolation junction stays reverse-biased: P substrate below the lowest voltage of any isolated N region; each N-well at or above the highest P region inside it; PMOS backgate ≥ source; diffused/well resistor tank tied to the more positive end (NBL under base resistors against vertical punchthrough); gate-oxide capacitor well ≥ 1 V above the poly (else capacitance collapses; ±20 % tolerance at 0.86 fF/µm² for 400 Å); Zener tank tied to anode, cathode or ≥ anode, never floating; buried-Zener isolation never on a high-impedance node; DI handle tied to the lowest-voltage pin.
- `realizes`: no forward-biased isolation junction, which injects minority carriers and causes glitches or latch-up (H01-01, H01-22, H01-23, H09-45, H05-46).
- `applicability`: all junction-isolated and DI processes; the deck cannot check well bias when the well is not a net (H01-01).
- `parameters`: margin (≥ 0 V hard).
- `guarantees`: every well against every bias corner, including wells shared by devices of different source potentials (H01-01).
- `does not guarantee`: transient excursions not simulated.
- `pdk data needed`: node voltages in the op-point (H05-17).
- `verification`: well-tap graph from placed geometry; V(well tap) ≥ max V(P-diff inside); V(sub) ≤ min V(N regions) (H01-01).
- `prefer over / degrade to`: n/a (hard ERC).
- `source`: H01-01, H01-22, H01-23, H09-45, H05-46, H05-17.

#### GRD-20 SCR loop-gain and PNPN enumeration
- `id`: GRD-20
- `technique`: For every PNPN path in the merged-well graph (a P diffusion in an N region adjacent to an N diffusion in a P region, joined by nets) compute β_N·β_P·(1 − η_C) with η_C the fraction collected or blocked by an intervening bar or ring; latch-up occurs when it exceeds 1 (β_N = 300, β_P = 10 need η_C > 0.997, which a bar cannot assure). Never merge an NPN driving a lateral PNP that could saturate; the same holds for BiCMOS NPN + LPNP in one well, a vertical NPN with an integrated field-plated Schottky, and the CMOS lateral NPN/PNP pair with well (R1) and substrate (R2) debias as trigger.
- `realizes`: enumerated SCR candidates with estimated loop gain and debias resistances (H14-14, H05-48).
- `applicability`: all merged structures and CMOS cell neighbourhoods; latch-up needs both transistors active, β_N·β_P > 1 over some current range, and a supply able to sustain it, and persists until power cycles (H05-48).
- `parameters`: ring efficiency by type (GRD-05…07), parasitic β from deck data.
- `guarantees`: every candidate listed, not spotted by inspection.
- `does not guarantee`: accurate β without deck parasitic-BJT data.
- `pdk data needed`: parasitic β, latch-up spacing, substrate/well resistances (PDK-01/02).
- `verification`: loop gain < 1 with margin; taps within distance (GRD-14).
- `prefer over / degrade to`: prevent injection (GRD-01/16) → collect/block (GRD-05…07) → reduce R1/R2 (GRD-13/14).
- `source`: H14-14, H05-48, H14-17.

#### GRD-21 Voltage-dependent spacing and recognition markers
- `id`: GRD-21
- `technique`: Space same-type wells/diffusions on different nets by 2×0.8·x_j (outdiffusion) + 1.5×(W(V1) + W(V2)) (depletion widths from Table 3.1) + 2× linewidth control; use deck per-voltage rules when they exist. Apply high-voltage spacings only where needed via recognition pseudolayers (one per voltage level except the highest, enclosing whole devices or coded per edge; N edges default to the highest class, P edges to the lowest) and let LVS check the marker against a per-symbol or per-parameter voltage rating. Interconnect: the required dielectric spacing depends on the voltage difference between adjacent leads, not on either lead's absolute voltage (ILO ≈ 2 MV/cm nominal, but a drawn 0.25 µm space may hold only ~25 V because of microloading, OPC and damascene taper; low-k is weaker); an `_hv` suffix is a crude substitute. Choose the lowest voltage option that meets the spec, because V_CEO sets the isolation spacings.
- `realizes`: exact area per net pair instead of blanket HV spacing (NDIF–NDIF 5.8 µm at 10 V vs 7.8 µm at 20 V in the worked example) (H01-02, H09-39, H15-36).
- `applicability`: any design above core voltage; standard bipolar 40 V needs 6–10 µm wells (H01-04); every HV diffusion corner rounded (BJT-30).
- `parameters`: net voltage ranges [V_min, V_max] over corners; depletion table by background doping and x_j; marker layers.
- `guarantees`: no under-spaced HV pair, no wasted area on two HV nets at the same potential (H15-36).
- `does not guarantee`: correctness without net voltage data (H05-17).
- `pdk data needed`: per-voltage spacing rules or depletion table, x_j, LWC, HV marker layers, ILD voltage table (PDK-06, PDK-12).
- `verification`: pairwise ΔV spacing on placed and routed geometry; marker ↔ net voltage consistency (H09-39).
- `prefer over / degrade to`: deck per-voltage rules → formula → blanket HV spacing.
- `source`: H01-02, H01-04, H09-39, H15-36, H09 §9.2.4.

### 1.5 Floorplan, padring, power/ground and routing policy (FLR)

#### FLR-01 Area estimation and packing factors
- `id`: FLR-01
- `technique`: Estimate cell area as A_cell ≅ P_f·ΣA_device with P_f = 1.2–1.8 for two-metal CMOS/BiCMOS, 1.3–2.5 for DLM standard bipolar, 1.5–3.0 for SLM; device estimators: resistor A ≅ 1.2·R·W_r(W_r + S_r)/R_S, capacitor A ≅ 1.1·C/C_a with C_a = 0.885·ε_r/t (pF/µm², t in Å), MOS A ≅ 1.3·W_g(L_g + S_gg), power MOS A ≅ R_SP/(R_DS(on) − R_P) at worst-case R_DS(on) ≈ 2× nominal (±25 % process, +50 % 25→125 °C), bipolars from laid-out reference devices. Core A_C ≅ R_f·P_f(die)·ΣA_cell + P_f(die)·ΣA_pwr with routing factor R_f: SLM ≤ 1.2, 1.5-layer 1.2–1.4, DLM channel 1.1–1.3, maze+channel ≈ 1.1, pure maze ≈ 1.0 (cells with little upper metal), TLM 1.1–1.2; P_f(die) 1.1–1.2 for 20–30 loose cells, < 1.05 tight. Die A_die ≅ (√A_C + 2W_pr + W_s)²; pad-limited when P = P_min/(4√A_die) > 1 with P_min ≅ (N_p + 4)(W_p + S_p) + 4W_s (+ 8× corner distance if ruled), then A_die = P_min²/16.
- `realizes`: a die outline and per-block area targets within ±20 % before any placement (H15-01, H15-02, H15-03, H15-04, H15 §15.1).
- `applicability`: every floorplan; more metal layers add little to analog P_f (H15-01); maze routing saves 5–10 % area but raises coupling risk and is harder to modify (H15-03).
- `parameters`: P_f, R_f, scribe (saw 50–75 µm, laser 25 µm), pad width 2–3× wire diameter, seal + ground ring 20–50 µm, guard rings ≥ 25 µm, padring 100–150 µm (H15-04).
- `guarantees`: closed-form core-vs-pad limitation; early schematic values for area-dominant parts (large caps, high-value resistors, matched components, power devices) make it accurate (H15 §15.1.1).
- `does not guarantee`: routing feasibility (FLR-06).
- `pdk data needed`: sheet resistances, capacitance densities, pad/scribe/seal widths (PDK-05, PDK-09).
- `verification`: achieved P_f and R_f per block reported each epoch against the ranges (H15-01, H15-03).
- `prefer over / degrade to`: n/a.
- `source`: H15-01, H15-02, H15-03, H15-04, H15 §15.1, §15.1.1.

#### FLR-02 Die assembly build order
- `id`: FLR-02
- `technique`: Build in this order: scribe seal and ground ring → ESD devices, bondpads and guard rings → power devices → analog cells → logic → top-level routing → final verification. Hundreds to thousands of top-level signals benefit from autorouting; a few dozen are better hand-routed (or routed with explicit class policies).
- `realizes`: a floorplan showing die outline, scribe and corner exclusion zones, ground ring, bondpads, ESD, guard rings, sized block polygons and key metal (power, buses) (H15 intro).
- `applicability`: every die; designs are "largely art", so the tool must replace judgement with explicit metrics (H15 §15.6).
- `parameters`: order is fixed; block sizes from FLR-01.
- `guarantees`: the fixed periphery exists before blocks compete for space.
- `does not guarantee`: anything about block placement quality (FLR-05).
- `pdk data needed`: none.
- `verification`: the checklist of section 4.
- `prefer over / degrade to`: n/a.
- `source`: H15 intro, H15 §15.4, §15.6.

#### FLR-03 Scribe streets, seal ring, ground ring, corner exclusion and slotting
- `id`: FLR-03
- `technique`: Scribe street ≥ 75 µm for saw dicing (blade 20–30 µm, kerf margins 15–25 µm; < 50 µm impossible), ~25 µm for laser (stealth-dicing damage zone ≈ 6 µm); test structures live in the scribe. Seal ring: every metal layer joined by continuous contact/via strips, optional PO flap-down into the scribe, over a continuous PSD substrate-contact diffusion, so it doubles as the substrate ground ring at the lowest potential (VSS in dual-supply parts); DI processes use deep-trench rings with large-radius corner fillets. Corner exclusion zones when the longer die side exceeds 2.5 mm: d_c ≥ 5 % of the longer dimension (5 mm die → 250 µm), nothing active inside, not even the ground ring (fabs differ on anti-delamination structures and seal routing). Anti-delamination slots in wide metal: long axis orthogonal to the shear-stress gradient (parallel to the die edge, or to the exclusion-zone diagonal), slot W_S × L_S with maximum gaps S_L and S_W, alternate columns offset, adjacent-layer slots offset, drawn on a slot layer (FINAL_MET = MET − SLOT); slotted resistance R_slot/R_unslot ≅ W·L_S/((S_L + L_S)(W − N_S·W_S)) + S_L/(S_L + L_S).
- `realizes`: moisture/ion/crack barrier, substrate ground reference, stress-safe corners (H15-12, H15-14, H05-32, H15 §15.3.1, §15.3.2).
- `applicability`: every die; plastic-package stress is high at edges and maximal (shear) at corners, larger dice worse, causing metal extrusion shorts and diagonal cracks (H15 §15.3.2).
- `parameters`: W_s, seal width, d_c, slot geometry.
- `guarantees`: ground ring metal also carries ESD current (ESD-13 width).
- `does not guarantee`: EM/IR of slotted leads unless R_slot and the reduced cross-section W − N_S·W_S are used (H15-14).
- `pdk data needed`: scribe width, seal construction, corner-exclusion rule, slot rule (PDK-09).
- `verification`: seal continuity and joining to ground metal; corner zones empty; slots oriented per rule (checklist 13–14) (H15 §15.5).
- `prefer over / degrade to`: n/a.
- `source`: H15-12, H15-14, H05-32, H15 §15.3.1, §15.3.2, §15.5.

#### FLR-04 Bondpads, probe pads, bond-over-active and PO openings
- `id`: FLR-04
- `technique`: Pad opening ≈ 3× wire diameter (75 µm for 25 µm wire), pitch ≈ 5× (125 µm); fine pitch 60 µm with 20 µm wire; circular metal-exclusion zone ≈ 50 µm larger in diameter than the opening; leads within 25 µm of a pad ≥ 10 µm wide; pad-1 mark. Probe pads need no ESD; POR ≥ max(2× tip, 0.75× pitch); dual ranks offset by ≥ one pitch. BOAC: pad on the topmost metal, few topmost vias under it, next-to-top metal sparse and thin, dense lower-level via arrays encouraged; no matched circuitry under thin-Al BOAC pads; under thick top metal place matched devices well inside or well away from the thick-metal edge and never let a thick-metal edge cross a matched device. Bondwire R_w = 4ρL/(πD²) (1 mm of 25 µm 4N Au = 47 mΩ; TCR Au 3700 ppm/°C, Cu 4000; +37 % from 25 to 125 °C); 2–3 wires per package pin; pads of one pin joined by wide on-die metal. Bondwire current capacity per Table 5.1 (Au 25 µm: 1.8/1.3/1.1/1.0 A at 1/2/5/10 mm; Cu 50 µm: 7.2/4.3/2.9/2.5 A; mold k = 0.65 W/m/°C, ΔT = 75 °C). Minimize the number and area of PO openings; test pads on a separate test-POR layer closed for production; metal overlaps pad openings on all sides; fuse openings minimal with nothing but the fuse inside; fuses well away from sensitive analog, especially matched MOS.
- `realizes`: bondable, probeable, corrosion-resistant periphery whose stress and resistance are accounted for (H15-16, H15-17, H15-13, H15-11, H05-07, H05-22, H05-31).
- `applicability`: every die; "all numerical values are merely examples" and come from the assembly house (H15-16).
- `parameters`: wire diameter and material, pad count per supply = ⌈I_supply/I_wire(L, D)⌉ (H05-07), BOAC scheme.
- `guarantees`: pads sized for the wire; bondwire resistance and TCR subtracted from IR and Kelvin budgets (H15-11).
- `does not guarantee`: matching under pads (keep matched devices out).
- `pdk data needed`: pad rules, BOAC density caps, thick-metal flag (PDK-05, PDK-09, PDK-10).
- `verification`: checklist 11 (pads sized/located, keep-away, lead width, pad-1 mark) (H15 §15.5); thick-metal edge versus matched cell distance (H15-13).
- `prefer over / degrade to`: thick-metal BOAC with matched devices well inside → no BOAC over matched.
- `source`: H15-11, H15-13, H15-16, H15-17, H05-07, H05-22, H05-31.

#### FLR-05 Block placement by pad adjacency, mirroring and heat flow
- `id`: FLR-05
- `technique`: Place blocks next to their pads (pads start adjacent to lead fingers; bondwires must not cross or closely approach); place identical blocks as mirror images of one cell with symmetric pads; put power pads at the top/bottom; elongate an amplifier so the sensitive input circuitry sits far from output-stage heat; put the power device at the centre of one die end and matched references at the other end on the die axis; pick the smallest leadframe that fits with 10–20 % X/Y margin (oversized mount pads delaminate); round the die edge to the mask-vendor increment. Matched components occupying an array live in one cell (~50–100 components per cell); layout-only cells (ground rings, power-device portions) are tolerated by LVS.
- `realizes`: short pad leads, similar electrical characteristics of duplicates, thermal separation (H15-07, H15-08, H15-05, H09-19, H15 §15.2).
- `applicability`: every floorplan; careless block placement creates channel choke points (FLR-06).
- `parameters`: pad order (given), mirror axis, heat-source position.
- `guarantees`: identity of duplicates (variant, orientation, routing parasitics) when mirrored (H15-07).
- `does not guarantee`: matched-device stress safety (BJT-19) or corner clearance (FLR-03).
- `pdk data needed`: none.
- `verification`: pad adjacency; mirror symmetry; input/output distance along one die axis (H09-17).
- `prefer over / degrade to`: mirrored pair → translated copy.
- `source`: H15-05, H15-07, H15-08, H09-17, H09-19, H15 §15.2.

#### FLR-06 Routing channel reservation and choke-point avoidance
- `id`: FLR-06
- `technique`: Reserve routing area explicitly (20 % of the core in the worked example) as strips; primary channels near the die centre carry 10–20 % of all top-level signals each and taper toward the edges like watercourses; every feeder carries ≥ 5 signals; channel widths cannot grow once routing starts; die-wide alternating preferred directions per metal; keep cells on the lowest layers so upper layers stay routable over them; dummy-metal block regions and power devices using all metals obstruct maze routing.
- `realizes`: routable top level without late choke points (H15-09, H15-23, H15 §15.1.3).
- `applicability`: every top level; channel width W_C = N·P_R + S_m (FLR-17).
- `parameters`: reserve fraction, gcell capacity per layer, feeder minimum.
- `guarantees`: capacity checked for every placement move rather than found too late (H15-09).
- `does not guarantee`: coupling safety (FLR-11).
- `pdk data needed`: per-layer pitch (FLR-17).
- `verification`: demand versus capacity per gcell; min-cut between netlist halves (H15-09).
- `prefer over / degrade to`: channel routing (emphasized) → maze where cells leave upper metal free (H15-03).
- `source`: H15-09, H15-23, H15-03, H15 §15.1.3.

#### FLR-07 Power and high-current leads
- `id`: FLR-07
- `technique`: Annotate every high-current lead with its DC (duty-averaged) current and, where it matters, its maximum resistance; EM sets the width floor (W_min = I_max/(J_max·t)), resistance usually forces wider; keep such leads short; route wide leads first from the channel edges inward, no jogs; quantize wide widths to integer track multiples W_N = N·P_R + S_m (0.7 µm pitch, 0.3 µm space → 1.0, 1.7, … µm) so partial runs can be swapped for N tracks; at layer changes and branches extend both leads across each other and fill the overlap with as many vias as fit; EM must hold at every point (no neck-down) while resistance averages along the lead (widen where room exists); mark each lead's current and R_max on the floorplan; thick top metal for supply trunks.
- `realizes`: supply and output nets meeting EM and IR budgets (H15-10, H15-24, H15-25, H15-26, H15-37, REL-01).
- `applicability`: all supply, output and bias-trunk nets; poly carries only static digital signals (unsilicided 20–50 Ω/□, silicided < 5 Ω/□) (H15 §15.4.3).
- `parameters`: I_dc, R_max = ΔV_allow/I, width series, via fill density.
- `guarantees`: every segment sized from its own tree current and every net measured against R_max (H15-10).
- `does not guarantee`: current crowding at bends (FLR-16) or ESD adequacy (ESD-13).
- `pdk data needed`: J_max per layer and cut with derating data, sheet R, via array spacing (PDK-05).
- `verification`: power plots: min width along each current path, cut count per layer change ≥ ⌈I/I_cut⌉, J/J_max heat map (H15-41); resistance of crucial leads computed (checklist 4–5) (H15 §15.5).
- `prefer over / degrade to`: solid wide lead → parallel narrower leads (slotting) (H14-57).
- `source`: H15-10, H15-24, H15-25, H15-26, H15-37, H15-41, H15 §15.4.3, §15.5.

#### FLR-08 Star node for matched returns
- `id`: FLR-08
- `technique`: Return the shared terminal (emitters, sources, analog ground taps) of every matched group to a single point so that drops from other currents become common-mode; no member branch may carry foreign current; branch resistances from the root to each member equal within budget; apply to emitters and to base contacts of bipolar quads.
- `realizes`: elimination of offsets like 10 □ × 30 mΩ/□ × 1 mA = 0.3 mV between two matched emitters' ground taps (H15-33, H09-14).
- `applicability`: any matched group whose shared net also carries other devices' current (net current ≠ Σ member currents) (H15-33).
- `parameters`: root location, branch width.
- `guarantees`: topological star (degree ≥ members at the root) plus balanced resistance.
- `does not guarantee`: bondwire or package drops (FLR-09 Kelvin).
- `pdk data needed`: metal sheet R.
- `verification`: no shared segment between member branches and foreign paths; root-to-member R equal within ΔR budget (H15-33).
- `prefer over / degrade to`: star → balanced common-node routing.
- `source`: H15-33, H09-14.

#### FLR-09 Kelvin (force/sense) connections
- `id`: FLR-09
- `technique`: Force leads carry the current; sense leads carry negligible current and connect at the resistor (or load) terminals; in precise circuits match the two sense-lead currents and resistances so their residual drops cancel; "Kelvin at the bondpad" must physically connect at the pad; separate schematic names through metal resistors prevent force/sense shorts but do not guarantee location; Kelvin pads on one pin each need their own ESD device.
- `realizes`: sense voltages free of force-path IR drop (H15-34, H14-42).
- `applicability`: current-sense resistors, output voltage sense, remote ground sense; bondwire resistance (47 mΩ/mm for 25 µm Au) decides when a separate sense pin is needed (H15-11).
- `parameters`: tap-point location, sense pair balance.
- `guarantees`: sense branch shares no segment with the force branch; tap at the terminal shape (H15-34).
- `does not guarantee`: anything if the sense taps sit on the force lead away from the terminal.
- `pdk data needed`: none.
- `verification`: checklist 12 (Kelvin/star physically correct) (H15 §15.5); geometric tap-point check (H15-34).
- `prefer over / degrade to`: separate sense pin → on-die sense at the terminal → star node (FLR-08).
- `source`: H15-34, H15-11, H14-42, H15 §15.5.

#### FLR-10 Net-class taxonomy and routing policy
- `id`: FLR-10
- `technique`: Classify every net and route by class. Power: no tunnels or jumpers, width ≥ spec (marked red). Noisy: never crosses sensitive leads or the devices they connect to (yellow). Sensitive: no tunnels, no substrate contacts in sensitive ground leads (green). Noncritical: none. Table 15.5 adds: high-voltage (label the voltage), high-current (label the width), HV+HC, sensitive analog, noisy digital, static digital (may route in poly), other (metal only). Eight sensitive categories: inputs of high-gain amplifiers and precision comparators; ADC inputs; precision reference outputs; analog grounds of accurate circuits; high-value resistor networks; very low-voltage signals; bias lines into low-current accurate circuits; any very-low-current circuitry. Name suffixes carry intent: `_n` noisy, `_s` sensitive, `_hv` high voltage, `b`/`z` negative logic, `1p5`/`3p3` voltage domain. Also list matched components and devices needing guard rings.
- `realizes`: a per-net policy for width, layers, jumpers, keep-outs and shielding (H14-35, H15-27, H15-28, H15-06).
- `applicability`: every net; noisy = fast slew or switching while others cannot tolerate it (H15 §15.4.4).
- `parameters`: class, slew (clock: VDD/t_edge), node impedance.
- `guarantees`: policy applied to every net instead of relying on colour-coded schematics (H14-35).
- `does not guarantee`: correct inference without op-point data (H15-28).
- `pdk data needed`: none.
- `verification`: no jumper on power/sensitive; no tap feeding a sensitive ground; noisy-over-sensitive count = 0 (FLR-11).
- `prefer over / degrade to`: op-point inference → name suffixes → user tags (a name may only add sensitivity, never remove it) (H15-06).
- `source`: H14-35, H15-06, H15-27, H15-28, H15 §15.4.4.

#### FLR-11 Noisy/sensitive separation and the coupling budget
- `id`: FLR-11
- `technique`: Never route noisy signals above, below (adjacent layer) or beside sensitive ones; when a crossing is unavoidable make it at right angles; interpose a quiet line (static digital, supply, ground) between parallel noisy and sensitive runs because lateral capacitance can equal vertical capacitance in dense stacks; never route digital over sensitive analog circuitry; treat fast-slewing analog nodes (charge pumps, switch nodes) as aggressors; evaluate ΔV = C·R·dV/dt per victim/aggressor pair (a minimum M1×M2 right-angle crossing of 1 µm leads over 10 kÅ ILD is ≈ 0.05 fF, which at 1 V/ns into 200 kΩ makes 50 mV spikes; 10 fF into 100 kΩ at 1 GV/s makes 1 V; sub-0.1 ns edges exceed 1 GV/s); back-annotate selectively (only noisy→critical capacitances, the rest lumped to ground); substrate coupling rises with frequency.
- `realizes`: C_max(v, a) = ΔV_allow(v)/(R_node(v)·SR(a)) per pair, summed over aggressors (H14-07, H14-08, H15-29, H15-30, H15-35).
- `applicability`: every noisy/sensitive pair; Hastings' own 1 fF case was found only by back-annotation (H14-07).
- `parameters`: R_node from the op-point, SR from net class, ΔV_allow from the offset budget, keep-out distances.
- `guarantees`: every overlap and crossing priced; inter-layer overlap C = ε·A/t_ild plus fringe included (H15-30).
- `does not guarantee`: anything with same-layer-only coupling models (a known Philis gap) (H15-30).
- `pdk data needed`: ILD thicknesses and plate/fringe capacitances per layer pair (PDK-05).
- `verification`: Σ_a C(v,a)·SR(a)·R(v) in mV per victim; zero parallel adjacent-layer overlap between noisy and sensitive (checklist 10) (H15 §15.5).
- `prefer over / degrade to`: distance → right-angle crossing with shield (FLR-12) → slew limiting (FLR-13).
- `source`: H14-07, H14-08, H15-29, H15-30, H15-35, H14 §14.1, H15 §15.5.

#### FLR-12 Shield plates and shield leads
- `id`: FLR-12
- `technique`: At an unavoidable noisy/sensitive crossing insert a plate on an intermediate layer tied to the sensitive node's own low-impedance reference (a reference's analog ground, a diff pair's domain ground), extending 2–3 µm past the intersection; in DLM route one signal in poly under an M1 plate; between parallel runs interpose a quiet lead; full coaxial shielding (signal on M3 with M3 side shields and M2/M4 plates) costs three routing layers; a grounded field plate under a lead also blocks capacitive noise into the silicon beneath (resistors, diffusions).
- `realizes`: crossing capacitance redirected to the reference (H15-32, H05-44).
- `applicability`: top-sensitivity nets with ≥ 4 free metals for coax; every crossing the FLR-11 budget rejects.
- `parameters`: plate layer, extension, reference net per victim, shield type (plate, lead, coax).
- `guarantees`: shield adds capacitance to the victim's reference, priced against its C budget (H15-32).
- `does not guarantee`: benefit when the reference is a noisy global ground (choose per victim) (H15-32).
- `pdk data needed`: layer stack.
- `verification`: shield present at every rejected crossing; reference net impedance low.
- `prefer over / degrade to`: plate → quiet lead → coax (most layers).
- `source`: H15-32, H05-44, H15 §15.1.3.

#### FLR-13 Slew limiting of static digital control lines
- `id`: FLR-13
- `technique`: Put a few tens of kΩ in series with the gate driving a static digital control line (100 kΩ with 1 pF of line gives ≈ 10 MV/s); value depends on line length (compute from extracted C or add C); not for clocks or data buses.
- `realizes`: an aggressor whose slew fits the FLR-11 budget: R_s = V_swing/(SR_target·C_line) (H15-31).
- `applicability`: static control lines crossing sensitive regions.
- `parameters`: R_s, C_line.
- `guarantees`: slew bounded.
- `does not guarantee`: timing.
- `pdk data needed`: poly resistor recipe.
- `verification`: SR ≈ V_swing/(R_s·C_line) ≤ target after extraction (H15-31).
- `prefer over / degrade to`: slew limiting when rerouting fails.
- `source`: H15-31.

#### FLR-14 Single-level crossing techniques and crossing-count placement
- `id`: FLR-14
- `technique`: Where only one routing layer is available (single-level metal, under MIM/MOM capacitors over active circuitry), minimise crossings by mock-layout search (paper dolls): arrange matched sets symmetrically about one axis; split a 6:1 pair into halves either side of the small device; cross over resistors except field-plated ones, sensitive ones and lightly doped high-sheet ones (2 kΩ/□ HSR suffers voltage modulation); rearrange device terminals (CEB vs CBE; CBE has slightly more R_C); stretch devices to pass leads between terminals (adds R/C; if one matched device is stretched, stretch all); route through merged tunnels (a base with two contacts; large currents debias); insert tunnels (area, R, C; never for high-current leads; must respect junction voltages; each approved by the circuit designer and resimulated); rearrange bondpads (system and package approval). Tunnel data: base 100–200 Ω/□ (a few hundred Ω, tens to hundreds of fF; widening trades R for C and leakage); emitter, deep-N+ and NBL ≈ 10 Ω/□; emitter-in-iso ≈ 1.5 fF/µm² and breaks down at a few volts (ground leads, or signals if BV ≥ 6 V); stacked emitter + deep-N+ + NBL 3–5 Ω/□; an NBL tunnel bridges two tanks with isolation acting as a P-bar. Route metal-only nets first and poly-tolerant static signals last; overload a full channel with non-preferred metal or poly.
- `realizes`: a crossing-free arrangement, measured as flight-line crossing count (H14-36, H14-37, H14-38, H14-39, H14-40, H15-25, H15 §15.4).
- `applicability`: single-layer regions; with two or more metals placement is limited only by matching and packing (H14 §14.3).
- `parameters`: terminal-order variants, stretch variants (same for the whole matched group), tunnel budget.
- `guarantees`: tunnels back-annotated and class-checked (H14-39).
- `does not guarantee`: low parasitics; stretched devices have more R and C (H14-37).
- `pdk data needed`: tunnel layer sheet R, capacitance, breakdown (bipolar decks).
- `verification`: crossing count after placement; no jumper on power/sensitive classes (FLR-10).
- `prefer over / degrade to`: rearrange terminals → cross over resistors → stretch → merged tunnel → tunnel → move pads (H14 §14.3.2).
- `source`: H14-36, H14-37, H14-38, H14-39, H14-40, H15-25, H14 §14.3, §14.3.1, §14.3.2.

#### FLR-15 Via redundancy, stacking and enclosure model
- `id`: FLR-15
- `technique`: Place at least two cuts at every layer change where space allows (stress voiding is probabilistic per via; adjacent vias protect each other beyond redundancy), then n = max(2, ⌈I/I_cut⌉). Respect the via technology: aluminium vias are fixed-size, not stackable (VIA2 not on VIA1; VIA2 on CONT and VIA3 on VIA1 usually allowed), symmetric overlap of both metals; W-plug vias stack, lower-metal enclosure mandatory on two sides, upper optional/asymmetric; single/dual damascene stack with both enclosures optional/asymmetric. Same-size cuts only; high current uses arrays of minimum vias.
- `realizes`: reliable layer changes with minimal pitch P_VTV = W_v + 2E_mv + S_m (E_mv = smallest opposite-side enclosure) (H15-18, H15-19, H15-20).
- `applicability`: every via; always double on power, bias and matched-signal nets (H15-19).
- `parameters`: cut count, stack legality, enclosure direction.
- `guarantees`: residual single-cut list is explicit (H15-19).
- `does not guarantee`: EM adequacy unless the leading-row rule is used (FLR-16).
- `pdk data needed`: stackable(via_i, via_j), asymmetric enclosure, exact cut size (PDK-05, PDK-09).
- `verification`: single-cut count metric; no illegal stacks.
- `prefer over / degrade to`: pairs everywhere → singles only where space forbids.
- `source`: H15-18, H15-19, H15-20, H01-25.

#### FLR-16 Current crowding at bends, via-array placement and contact faces
- `id`: FLR-16
- `technique`: Replace a 90° interior angle on a high-current lead by two 135° angles; put via arrays in straight segments where each column carries roughly equal current (at a 90° corner the inside-corner via is most loaded, the outside least); shape via arrays as a row perpendicular to the current on the leading edge, because only the contact faces toward the current conduct (an end contact one face, a contact between two leads two faces; of two vias at a lead end only the front face of the front via conducts much); with aluminium step coverage the sidewall metal is thinned (10 kÅ at 50 % → 5 kÅ), ignore step coverage with a refractory barrier; W-plugs void where conventional current flows into the plug, and a plug with barrier below only is worse for upper-to-lower current.
- `realizes`: EM-safe bends and via arrays (H15-39, H05-14, H05-15).
- `applicability`: nets above about half the EM limit; ESD paths (chamfers ≥ half lead thickness) (H14-57).
- `parameters`: chamfer size, array shape, direction factor per via polarity.
- `guarantees`: effective cuts = leading-row cuts ≥ ⌈I/I_cut⌉ (H05-14).
- `does not guarantee`: anything when the array sits at a bend (derate by a corner factor, not given; use 1/n_columns per inside via or FEA) (H15-39).
- `pdk data needed`: 45° allowance, via EM direction factor (H05-15, H15-22).
- `verification`: no 90° inside corner on flagged nets; arrays off corners; leading-row count (H15-39, H05-14).
- `prefer over / degrade to`: 135° bends → stair-step when 45° is illegal.
- `source`: H15-39, H05-14, H05-15, H14-57, H15-22.

#### FLR-17 Routing pitch, track lattice and grid discipline
- `id`: FLR-17
- `technique`: Per layer compute P_LTL = W_m + S_m, P_VTV = W_v + 2E_mv + S_m (preferred), P_VTL = (W_v + W_m)/2 + E_mv + S_m (staggered vias congest channel intersections), channel width W_C = N·P_R + S_m; make the routing grid an integer multiple of the coding increment and all path widths integer multiples of twice the coding increment so centreline-defined edges land on grid (a 0.35 µm path on a 0.05 µm grid puts edges on 0.025 µm; widen to 0.4 µm, P_VTV → 0.7 µm); with different rules per layer use the larger rule on all layers if the difference is small; 45° routing packs tighter along corner-exclusion diagonals but puts vertices off grid and may require wider diagonal segments (use the diagonal width for the pitch).
- `realizes`: a legal per-layer track lattice and gcell capacity (H15-20, H15-21, H15-22, H01-26).
- `applicability`: every router and generator; Table 15.4 example: 0.3 µm width/space, 0.25 µm via, 0.05/0.1 µm overlaps → P_LTL 0.6 µm, P_VTV 0.65 µm (H15-20).
- `parameters`: per-layer width, space (incl. end-of-line), cut size, min one-side enclosure, grid.
- `guarantees`: no rounding-induced DRC failures (H15-21).
- `does not guarantee`: capacity under keep-outs (FLR-06).
- `pdk data needed`: all of the above per layer (PDK-05, PDK-09).
- `verification`: off-grid check; pitch ≥ computed minimum.
- `prefer over / degrade to`: via-to-via lattice → line-to-line lattice; one pitch for all layers only as a stopgap.
- `source`: H15-20, H15-21, H15-22, H01-26.

#### FLR-18 Dummy-metal block, fill and substrate-contact fill
- `id`: FLR-18
- `technique`: Cover matched devices with dummy-metal block layers so CMP fill (which disturbs matching, notably MOS) stays off them, accepting that the blocks obstruct maze routing; generate dummy fill automatically per density window elsewhere (CMP dishes recessed areas over hundreds of µm; even chip art must obey design rules); on P+ substrate dice fill empty area with substrate contacts (trade against bypass capacitance in logic-heavy designs); extend fill-block over PSD antenna diodes by at least the well junction depth.
- `realizes`: matching protection plus density compliance plus substrate resistance (H15-61, H01-40, H15-60, H05-29).
- `applicability`: every die; checklist item 9 (H15 §15.5).
- `parameters`: block extent, density window, fill tile, tap tile.
- `guarantees`: fill never touches matched cells; density exact.
- `does not guarantee`: routability over blocked regions (FLR-06).
- `pdk data needed`: density rules, fill-block layer, tap tile (PDK-09).
- `verification`: density windows pass; matched cells inside blocks; R_sub budget (GRD-13).
- `prefer over / degrade to`: symmetric fill inside matched arrays when a block is impossible.
- `source`: H15-61, H15-60, H01-40, H05-29, H15 §15.5.

#### FLR-19 Hierarchy and naming for LVS and assembly
- `id`: FLR-19
- `technique`: Partition by simulation units (~50–100 components per cell, a schematic that fits a B-size sheet, few designs need more than 10–12 levels); matched components occupying an array reside in the same cell; layout hierarchy mirrors schematic hierarchy but LVS tolerates layout-only cells (ground rings, power-device parts); names start with a lower-case letter, use lower-case/digits/underscore, ≤ 32 characters (GDSII); pin-direction ERC: a net on only inputs is floating, a net on two outputs is shorted; emit pins as TEXT labels in GDS; keep one unit system (nm silicon) and the eight orthogonal orientations only.
- `realizes`: LVS-clean, archivable output (H15-05, H15 §15.1.1, H01-30, H01-28, H01-31).
- `applicability`: every design; zero topological LVS errors are never waivable, parametric errors (e.g. guard-ring diode areas across hierarchy) may be waived after review (H15-42).
- `parameters`: cell size, naming, waiver ledger.
- `guarantees`: a matched group never splits across cells (H15-05).
- `does not guarantee`: n/a.
- `pdk data needed`: GDS layer/datatype for pins (PDK-09).
- `verification`: checklist 1–3, 16–17 (DRC waivers with process engineering, LVS topological zero, symbolization, archival of DB, decks, GDSII, layer maps, checksums, two copies) (H15 §15.5).
- `prefer over / degrade to`: hierarchical GDS → flat when the external flow demands.
- `source`: H15-05, H15-42, H01-28, H01-30, H01-31, H15 §15.1.1, §15.5.

### 1.6 Failure mechanisms as layout rules (REL)

#### REL-01 Electromigration width, derating and Black's law parameters
- `id`: REL-01
- `technique`: Width W_min = I_max/(J_max·t_min) per segment with t_min counting only the Al/Cu (not barrier layers); derate J to the junction temperature with J2 = J1·exp[(E_a/(n·k))(1/T2 − 1/T1)], k = 8.62×10⁻⁵ eV/K; Black's law t50 = A50·J⁻ⁿ·exp(E_a/kT) with n = 2 nucleation-dominated (Al without barrier), n = 1 growth-dominated (Al with refractory barrier and W vias, most Cu), n > 2 hinting at thermal gradients; E_a ≈ 0.7 eV Al (0.5 eV pure Al), ≈ 0.8 eV Cu, 0.5–1.5 eV others, ≈ 0.5 eV high-tin solder. Typical rule: 5×10⁵ A/cm² at 105 °C with E_a = 0.7 eV (MIL-M-38510 for Al under glass); 105 → 125 °C gives D = 0.58; 50 mA on 1 µm metal needs 10 µm. Cu-doped Al (0.5 %) lives > 10× longer; Cu carries ≥ 5× Al at 105 °C but above ~175 °C Al may be better; lead-free solder joints carry ≈ 2 % of Al's current density. Remedies: wider or parallel leads, more contacts/vias, more or larger bondwires.
- `realizes`: 100,000 h life at rated T_J for every segment (H05-10, H05-11, H05-12, H15-38, H05-01).
- `applicability`: all current-carrying metal; failures appear above ~1×10⁵ A/cm² in Al and become critical near 1×10⁶ (H05 §5.1.3, H01-42); EM holds at every point, resistance only on average (H15-37).
- `parameters`: (T_ref, E_a, n) per metal, J_max per layer and per cut, duty (REL-03), local temperature (REL-05).
- `guarantees`: per-segment sizing from the tree current.
- `does not guarantee`: anything when the deck lacks derating data (Philis decks today) (H15-38).
- `pdk data needed`: J_max per layer/cut with (T_ref, E_a, n), metal thickness, Blech product (PDK-05).
- `verification`: hard per-segment width; power plots for pinch points and via counts (H15-41).
- `prefer over / degrade to`: deck data → family defaults (Al 0.7 eV/n = 2, Cu 0.8 eV/n = 1) flagged as assumed (H05-10).
- `source`: H05-10, H05-11, H05-12, H05-01, H15-37, H15-38, H01-42, H05 §5.1.3, H15 §15.3.4.

#### REL-02 Blech immortality
- `id`: REL-02
- `technique`: Leads shorter than the Blech length do not electromigrate: L_B ≈ (1200 A/cm)/J for pure Al on TiN (24 µm at 5×10⁵ A/cm²); allow w < I/J when (I/w)·L < (jL)_B for the whole connected run on that layer between vias/branches.
- `realizes`: short stubs not over-widened (H05-13).
- `applicability`: when the deck gives a Blech product; strongly process dependent; analog EM rules usually ignore it; dynamic EM tools apply Blech rules to short leads (H05-13, H15-40).
- `parameters`: (jL)_B per layer.
- `guarantees`: immortality below the product.
- `does not guarantee`: validity across diffusion barriers (vias end the domain).
- `pdk data needed`: Blech product (PDK-05).
- `verification`: j·L per domain.
- `prefer over / degrade to`: use only with deck data; else REL-01.
- `source`: H05-13, H15-40.

#### REL-03 Pulsed and AC electromigration
- `id`: REL-03
- `technique`: Pure AC above ~10 kHz rarely fails (keep self-heating checks); unidirectional pulses with duty D: t_pulse = t_DC/D for f < 1/τ and t_DC/D² for f > 1/τ (τ > 0.1 ms vacancy recombination); so at 50 % duty life quadruples and an Al lead (n ≈ 2) may be half as wide; size switched nets from duty-averaged current (W ≥ I_peak·D^(1/n)/(J·t) for f < 1/τ, D^(2/n) for f > 1/τ, derived); exercise 15.12 uses duty-cycle averaging for 360 mA at 50 %.
- `realizes`: compact clock and switch nets without EM risk (H05-16, H15-38, H15-40).
- `applicability`: switched-capacitor, clock and gate-driver nets; waveforms are hard to determine for analog, so a per-net waveform class is needed (H15-40).
- `parameters`: I_peak, D, f, n.
- `guarantees`: equal life to the DC rule.
- `does not guarantee`: Joule-heating safety (use I_rms) (H05-16).
- `pdk data needed`: n per metal (PDK-05).
- `verification`: per-net current = max(|I_DC|, duty-scaled I_peak, I_rms for heating).
- `prefer over / degrade to`: DC worst case when the waveform is unknown.
- `source`: H05-16, H15-38, H15-40.

#### REL-04 Self-heating, junction temperature and resistor width
- `id`: REL-04
- `technique`: T_J = T_A + θ_JA·P_D (or T_C + θ_JC·P_D); ambient grades: commercial 0–70 °C, industrial −40–85 °C, military −55–125 °C, automotive −40 to 85–150 °C; small SMD packages θ_JA > 150 °C/W, power packages θ_JC < 10 °C/W; max T_J 125 or 150 °C with 100,000 h life, ~1,000 h at the thermal-shutdown temperature ≈ T_J,max + 25 °C. Device self-heating for L > W smaller than the die thickness (~250 µm): ΔT = ln(4L/W)/(π·κ·L)·P_D with κ_Si ≈ 1.3 W/cm/°C (25 µm square at 100 mW → 14 °C). Deposited resistors over oxide: ΔT = I²·t_ox·R_s/(κ_ox·W²), κ_ox ≈ 0.011 W/cm/°C, W_min = I_max·√(t_ox·R_s/(κ_ox·ΔT)); design ΔT ≈ 5 °C so gradients do not invalidate nearby EM figures, never above 50 °C (stress); pulses < ~1 µs are adiabatic (REL-05). Leakage doubles every ~8 °C; µA circuits shift above ~175 °C and fail above ~200 °C; contact Seebeck 0.1–1.0 mV/°C degrades mV-matched circuits under self-heating gradients.
- `realizes`: the temperature at which every Arrhenius rule is evaluated and the width of every deposited resistor (H05-01, H05-02, H05-03, H05-04, H05-05, H01-43).
- `applicability`: all dice; poly and thin-film resistors are EM-immune so heating is their only current limit; poly over oxide self-anneals and drifts under power, melts before a diffused resistor fails, and is bad for ESD paths (H01-20, H05-25).
- `parameters`: grade, θ, life hours, ΔT budget.
- `guarantees`: per-device hot spot fed to EM derating of attached wires and to matching ΔT (H05-04).
- `does not guarantee`: package-level thermal paths (θ data is external).
- `pdk data needed`: field-oxide thickness under resistors, resistor sheet R, κ (defaults above) (PDK-08).
- `verification`: W ≥ W_min at ΔT = 5 °C; T_J per corner reported; leakage guard for high-impedance nodes (H05-03).
- `prefer over / degrade to`: diffused resistor when a deposited one would overheat (Table 5.5) (H05 §5.5).
- `source`: H05-01, H05-02, H05-03, H05-04, H05-05, H01-20, H01-43, H05-25, H05 §5.5.

#### REL-05 Adiabatic pulse sizing
- `id`: REL-05
- `technique`: For pulses shorter than ~1 µs heat does not escape: W_min = (I_max/t_R)·√(ρ·τ/(d·c_V·ΔT)) (t_R thickness, ρ resistivity, d density, c_V volumetric heat: poly ≈ 1.66, Al ≈ 2.42 J/°C/cm³); for an HBM strike τ = 1.5 kΩ × 150 pF = 225 ns and A = √(ρ·τ·I_pk²/(2·C_V·ΔT)).
- `realizes`: ESD-path and pulse-power conductor and resistor widths (H05-06, H15-46, H14-56).
- `applicability`: pin/ESD nets, Zener-zap and fuse paths, pulsed power stages (BJT-25 pulsed role).
- `parameters`: I_pk, τ, ΔT (50 °C typical).
- `guarantees`: bounded temperature rise per pulse.
- `does not guarantee`: steady-state safety (REL-04).
- `pdk data needed`: ρ, d or C_V, thickness per conductor (PDK-05).
- `verification`: per-segment width on flagged nets.
- `prefer over / degrade to`: n/a.
- `source`: H05-06, H15-46, H14-56.

#### REL-06 Ballasting against thermal runaway and filamentation
- `id`: REL-06
- `technique`: Give every parallel current path series resistance: per-finger emitter/source ballast, larger diffusion-over-contact overlap for intrafinger ballast, silicide block where silicide removed the drain ballast; bound intra-array ΔT well below 0.03·T_J because +10–15 °C doubles I_C at constant V_BE (V_BE TC ≈ −2 mV/°C) and hot spots form in tens to hundreds of µs; avalanche filaments form in < 1 ns and can be triggered by ESD; the Al–Si eutectic fails at ~580 °C, SiO₂ melts at 1600 °C, Si at 1400 °C, and one molten filament destroys the die; secondary breakdown in power BJTs always traces to debiasing.
- `realizes`: stable current sharing in power and ESD devices (H05-08, H05-09, BJT-22, ESD-12).
- `applicability`: multi-finger BJTs and MOS, ESD devices, high-g_m power MOS (Spirito effect); Zener-zap fuses use filamentation deliberately (H05 §5.1.2).
- `parameters`: ballast per finger, ΔT budget between fingers.
- `guarantees`: negative feedback against localization.
- `does not guarantee`: RBSOA (topology, BJT-25).
- `pdk data needed`: silicide-block layer, ESD contact spacing, emitter sheet R.
- `verification`: finger current skew solve (BJT-22); thermal field ΔT across the array (H05-09).
- `prefer over / degrade to`: distributed diffusion ballast → discrete resistors.
- `source`: H05-08, H05-09, H05 §5.1.2, H09 §10.1.1.

#### REL-07 Dielectric breakdown (TDDB) rules
- `id`: REL-07
- `technique`: Limit every oxide to V = E·t with E_max ≈ 3.5–4 MV/cm for 300–500 Å dry oxide and 4–4.5 MV/cm for thinner oxides (100,000 h at 125 °C; instantaneous strength ≈ 11 MV/cm); fields crowd at sharp conductor edges and thicker oxides are somewhat more fragile; scale with area and time (Weibull β ≈ 2: t2 = t1·(A1/A2)^(1/β); AHI t2 = t1·exp(G(1/E2 − 1/E1)), G ≈ 350 MV/cm; McPherson t2 = t1·exp(γ(E1 − E2)), γ ≈ 2.5–3.5 cm/MV) to run small or low-duty oxides above the global limit and large MOS capacitors below it; allow an over-voltage stress test of up to 2× operating voltage once for ~100 ms; poly over STI is weaker than poly over LOCOS (trench cone defects), metal is not affected, so route HV crossings in metal; deep-N+/NBL getter heavy metals within ~100 µm and improve gate-oxide integrity where oxygen precipitation is absent; never use grown oxide over deep-N+ as a capacitor dielectric.
- `realizes`: oxide field per gate and per capacitor plate pair checked at real voltage (H05-18, H05-19, H05-20, H05-21).
- `applicability`: all gates, MOS caps, poly-over-field crossings, HV metal-over-metal; ESD duration limits differ (90 Å ≈ 22 V; 50 Å ≈ 24 MV/cm for 100 ns) (H14-53).
- `parameters`: t_ox per oxide, duty, area.
- `guarantees`: area-aware limits per dielectric.
- `does not guarantee`: anything without per-net voltages (H05-17).
- `pdk data needed`: oxide thicknesses, E_max, getter layer and distance (PDK-06).
- `verification`: |ΔV|max/t_ox ≤ E_max(t_ox, A, duty) per pair; no HV poly over STI field.
- `prefer over / degrade to`: thicker dielectric or lower voltage; OVST provision (H05 §5.5).
- `source`: H05-18, H05-19, H05-20, H05-21, H14-53, H05 §5.5.

#### REL-08 Antenna ratios, repair and diodes
- `id`: REL-08
- `technique`: Per gate net and per etch stage, ratio = perimeter (plasma etch) or area (ashing) of the connected conductor at that stage divided by the gate-oxide area under the node's poly; typical poly limits 100 µm⁻¹ peripheral and 500 areal, each metal with its own limits; merge geometries closer than the latent threshold (oversize–undersize) across nets because min-spaced neighbours clear late and act as one antenna; P+ poly gates may be more vulnerable. Repair a poly antenna with a short metal-1 jumper next to the gate, a lower-metal antenna with a jumper through a higher metal near the gate; top metal cannot be jumpered and needs a junction. Junction credit: an NSD/substrate junction clamps negative excursions and clamps positive ones only if the maximum gate-oxide voltage ≤ ~150 % of its avalanche voltage; avalanche cannot go much below 6 V, so ≤ 3.3 V processes need both NSD/substrate and PSD/N-well junctions (the latter works by UV photocurrent at the N-well/substrate junction and needs no metal or poly within a few µm and a large enough N-well); many decks check metal/diffusion rules only when a metal/gate rule fails. Antenna diode: NSD = NMoat in substrate; PSD = PMoat in a floating N-well with fill-block extended ≥ the well junction depth.
- `realizes`: zero antenna violations (checklist 6) (H05-26, H05-27, H05-28, H05-29, H05-30, H15 §15.5).
- `applicability`: every gate net; worked limit: top metal areal 1000, peripheral 400 µm⁻¹, 1 µm lead on 0.7 µm² gate → L ≤ 139 µm (derived) (H05-26).
- `parameters`: per-layer limits, latent merge distance, diode type.
- `guarantees`: exact per-stage connectivity.
- `does not guarantee`: credit from a junction reached only at a later stage (H05-28).
- `pdk data needed`: antenna limits per layer, sidewall mode, diode layers (PDK-07).
- `verification`: ratio per stage; latent merge; polarity credit.
- `prefer over / degrade to`: jumper near the gate → diode (both polarities at ≤ 3.3 V).
- `source`: H05-26, H05-27, H05-28, H05-29, H05-30, H15 §15.5.

#### REL-09 Hot-carrier rules
- `id`: REL-09
- `technique`: Channel hot carriers inject at the drain–backgate junction with gate current peaking at V_GS ≈ 0.4·V_DS; PMOS needs ~2× the field; characterize allowed duty versus (V_DS, V_GS) for 10 % linear-g_m loss or 10 mV ΔV_T; matched transistors at unequal gate/drain voltages (comparator input pairs) degrade differently and develop offset, so equalize V_DS with cascodes; lengthening L by 0.5–2 µm buys a few volts of margin; LDD/DDD/RESURF drain engineering; deuterium anneal ≥ 10× slower. NMOS carry a blocking rating (switches: junction breakdown/punchthrough) and a lower operating rating (held in saturation): 4/3 µm example 15 V blocking, 7 V operating; plain 400 Å NMOS 5–10 V, PMOS 15–20 V, LDD NMOS 12–15 V; core devices have smaller voltage margin than I/O devices.
- `realizes`: screening of every matched set at its op-point and device-flavour choice (H05-33, H05-34, H01-15, H01-16).
- `applicability`: all MOS; a sizing/flavour issue the layout cannot fix, reported before layout (H01-15).
- `parameters`: V_DS,op, V_DS,block, V_GS,max per model; ΔV_DS threshold per matched set.
- `guarantees`: consistent screening.
- `does not guarantee`: lifetime without the HCI duty table.
- `pdk data needed`: ratings per model, HCI onset or duty table, marker layers for flavours (PDK-06).
- `verification`: saturated devices |V_DS| ≤ V_DS,op, others ≤ V_DS,block, |V_GS| ≤ V_GS,max; |ΔV_DS| across matched members.
- `prefer over / degrade to`: cascode → longer L → extended-drain model.
- `source`: H05-33, H05-34, H01-15, H01-16.

#### REL-10 NBTI mismatch of matched PMOS
- `id`: REL-10
- `technique`: Find matched PMOS operated at different V_GS (or stress duty) and bias them identically or eliminate the difference; NBTI shifts V_T negative under negative gate bias at temperature, partly recovers within seconds, AC < DC; worse with thin oxides, surface-channel dual-doped poly and oxynitride.
- `realizes`: long-term offset screening (H05-35).
- `applicability`: matched PMOS groups; PBTI for high-k NMOS.
- `parameters`: ΔV_GS threshold (not given; config).
- `guarantees`: every group checked.
- `does not guarantee`: a layout cure (circuit issue).
- `pdk data needed`: none.
- `verification`: op-point V_GS per member.
- `prefer over / degrade to`: n/a.
- `source`: H05-35.

#### REL-11 Emitter–base reverse-bias limit
- `id`: REL-11
- `technique`: Never reverse-bias a B–E junction beyond ~75 % of V_EBO; matched transistors ≤ 50 % of V_EBO, diode-connected ≤ ⅔, poly-emitter devices ≤ ½ (a couple of volts); pin-connected B–E junctions need ESD clamps or redesign.
- `realizes`: no avalanche-induced beta degradation (surface hot carriers; low-current β dropped ~80 % in one discrete NPN, vertical NPNs far more susceptible than laterals, poly-emitter worst; the matching loss counts as an ESD failure) (H05-37, H09-25, H14-59).
- `applicability`: every BJT; E–B avalanche permanently degrades low-current β (H09 §9.1.2).
- `parameters`: V_EBO per model (typ. 6.8 V, 6–12 V) (H09-45).
- `guarantees`: an ERC on simulation data for a hazard invisible in layout.
- `does not guarantee`: anything for transients not simulated.
- `pdk data needed`: V_EBO per model (PDK-04).
- `verification`: max reverse V_EB per corner and transient ≤ limit; pin-connected E/B has a clamp (ESD-11).
- `prefer over / degrade to`: n/a.
- `source`: H05-37, H09-25, H14-59, H09 §9.1.2.

#### REL-12 Parasitic channels and charge spreading
- `id`: REL-12
- `technique`: A parasitic MOS conducts when a lightly doped backgate, an opposite-type source and drain, a conductor or static charge between them, |V_GS| near or above the thick-field threshold V_TF (with body effect) and V_DS ≠ 0 coexist; thresholds per (conductor, backgate): 20 V N-well CMOS example poly ±15 V, metal-1 ±30 V, metal-2 ±50 V; modern CMOS top metal > 50 V; standard bipolar metal 40–42 V; derate V_TF by ≥ 25 % (protect P regions ≥ 30 V when V_TF is 40 V), field-plate all P regions above ~⅔ of the top-metal V_TF, inspect leads near P regions above ~⅔ of the lowest-metal V_TF. Remedy ladder: reroute so the lead does not bridge the HV P region to another P region → move to a higher metal (larger V_TF) → field plate (minimum-width poly or lower metal under the lead tied to ≥ the N-well voltage; overhang = outdiffusion + misalignment + 2·t_ox) → channel stop (minimum-width NMoat bisecting the lead; overhang ≥ two-level misalignment + 2·t_ox). Retract thick-oxide PMOS poly inside the N-well by N-well-over-PSD overlap + 1–2 µm when |V_GS| exceeds the poly PMOS V_TF. Field plating of HV resistors and devices: the highest-potential plate covers most; flange or channel-stop gaps, or overlap with an upper-metal plate; partial plates extend well past the V_TF crossing point; lateral PNP emitter plate overlapping the collector; HV charge spreading from mold-compound ions lowers breakdown long before inversion (> 200 V): top-metal plate, thick power Cu over PO, or semi-insulating PO. Base-over-isolation and channel-stop implants raise the thick-field threshold above the maximum operating voltage in the baseline process.
- `realizes`: no µA leakage paths that appear after long bias and vanish on decap (H05-38, H05-39, H05-40, H05-41, H05-42, H05-43, H05-45, H01-17).
- `applicability`: HV and thick-oxide designs; charge spreading was seen below 10 V with metal V_TF > 42 V (sodium amplification) (H05 §5.3.5).
- `parameters`: V_TF table, derate (⅔ or 0.75), plate/stop overhangs.
- `guarantees`: every crossing enumerated with real voltages.
- `does not guarantee`: anything without V_TF data (report "unknown", not pass) (H05-38).
- `pdk data needed`: V_TF per layer and backgate, misalignment, t_ox (PDK-06).
- `verification`: six-condition geometric pass over the final layout (H05-39).
- `prefer over / degrade to`: reroute → higher metal → plate → channel stop (H05-41).
- `source`: H05-38, H05-39, H05-40, H05-41, H05-42, H05-43, H05-45, H01-17, H05 §5.3.5.

#### REL-13 Contamination control: PO openings, fuses, seal
- `id`: REL-13
- `technique`: Minimize the number and area of protective-overcoat openings (pads, probe pads, fuses are moisture and ion ingress paths); test pads on a separate test-POR layer closed for production; metal overlaps pad openings on all sides; fuse openings minimal with nothing but the fuse inside or adjacent; fuses well away from sensitive analog, especially matched MOS; a continuous gap-free scribe seal (PSD under, metal over, continuous via rings between adjacent metals, PO flap-down) is the barrier; mobile sodium shifts V_T reversibly (200–250 °C unbiased bake) and returns under bias.
- `realizes`: dry-corrosion and mobile-ion immunity in plastic packages (H05-31, H05-32).
- `applicability`: every die; halides and brominated flame retardants attack Al (H05 §5.2.1).
- `parameters`: fuse keep-out radius (not given; config).
- `guarantees`: ingress minimised.
- `does not guarantee`: anything against a defective PO.
- `pdk data needed`: POR/test-POR layers.
- `verification`: PO opening count/area; fuse-to-matched distance; seal continuity (FLR-03).
- `prefer over / degrade to`: n/a.
- `source`: H05-31, H05-32, H05 §5.2.1, §5.2.2.

### 1.7 Process facts a PDK-agnostic recipe must query (PDK)

#### PDK-01 Substrate and epitaxy
- `id`: PDK-01
- `technique`: Query substrate type (P+ with P− epi, bulk P−, SOI/DI), epi thickness and resistivity, wafer orientation (100)/(111), die thickness after backgrind (~250 µm; down to 50 µm).
- `realizes`: the contact strategy of GRD-13 (epi-on-P+: scatter and fill; bulk P−: large contacts within half the die thickness; thin layers: annular t/2), ring efficiency (P+ substrate 10–100× better ECGR; P− far more latch-up prone), piezojunction coefficients (BJT-19), DI handle tie (GRD-19).
- `applicability`: all decks; sky130 is bulk P− (H05-51).
- `parameters`: `substrate ∈ {p_plus_epi, p_minus_bulk, soi}`, `epi_t`, `epi_rho`, `orientation`, `die_t`.
- `guarantees`: the right debias model is used.
- `does not guarantee`: anything if absent: report "unknown".
- `pdk data needed`: this card is the datum.
- `verification`: deck lint for presence.
- `prefer over / degrade to`: absent → ask the user; never assume epi-on-P+.
- `source`: H05-50, H05-51, H05-52, H05-56, H14-32, H09-20, H05-46, H15 §15.3.1.

#### PDK-02 Wells
- `id`: PDK-02
- `technique`: Query N-well (and P-well) depth, outdiffusion (≈ 0.8·x_j), retrograde flag and bottom/middle doping ratio, well sheet resistance, tie resistance with and without deep-N+, maximum tie distance.
- `realizes`: HCGR legality and width (GRD-06), ECGR width (GRD-05), well debias in the merge predicate (GRD-15), tap pitch ≤ 20–50·t (GRD-13), tie distance (GRD-14), well spacing (GRD-21), well-edge effects measured from the outdiffused outline (H01-04).
- `applicability`: all CMOS decks; a 40 V process needs 6–10 µm wells, a 20 V CMOS N-well ≈ 5 µm (H01-04).
- `parameters`: `nwell_depth`, `nwell_retrograde`, `nwell_rs`, `tie_r_deepn`, `tie_max_dist`.
- `guarantees`: ring gating on process physics instead of habit (H14-29).
- `does not guarantee`: n/a.
- `pdk data needed`: this card is the datum.
- `verification`: deck lint; `tie_max_dist` already exists in sidecars (3 µm sky130, 20 µm IHP) but is unused (H01-07).
- `prefer over / degrade to`: absent retrograde flag → no HCGR, majority ring plus spacing (H14-29).
- `source`: H14-05, H14-28, H14-29, H01-04, H01-07, H05-52.

#### PDK-03 Buried layers, sinkers, trenches and pattern shift
- `id`: PDK-03
- `technique`: Query NBL, deep-N+ (sinker), PBL, deep N-well (DNW), DTI availability; NBL/N-well doping ratio (≥ 50× bottom, ≥ 100× for blocking), deep-N+ core doping, sinker outdiffusion (width ≥ 2× epi), NBL pattern shift magnitude/direction (absent in STI/CMP processes), DTI termination scheme (N-well ends mid-trench, NBL short of it).
- `realizes`: HBGR construction (GRD-07), isolated tubs (GRD-09), lateral PNP legality (BJT-16: β < 1 without NBL), NBL shadow compensation (BJT-20), power rings (BJT-29), collector resistance (BJT-15).
- `applicability`: bipolar, BiCMOS, BCD decks; CMOS decks offer at most DNW.
- `parameters`: layer names, enclosures (NBL to deep-N+ outer edge), doping ratios, `nbl_shadow`.
- `guarantees`: closure and enclosure constraints generated per ring.
- `does not guarantee`: blocking when the well surface doping exceeds 10¹⁸ (H14-33).
- `pdk data needed`: this card is the datum.
- `verification`: deck lint.
- `prefer over / degrade to`: absent → collecting rings only (GRD-05/06).
- `source`: H14-33, H01-19, H09-22, H09-38, H14-12, H01-24.

#### PDK-04 Bipolar device data
- `id`: PDK-04
- `technique`: Per BJT model query construction (vertical NPN, substrate PNP, lateral PNP, poly-ring lateral), fixed-geometry flag, minimum emitter and contact enclosure, k_A (typical 2 %·µm), β at corners and current-density plateau, high-level-injection onset per unit, V_A, V_EBO, V_CEO/V_CBO, V_BE temperature coefficient (≈ −2 mV/°C), piezojunction coefficients (Table 10.3: (100) ζ_L NPN −28.4 / PNP 8.9, ζ_R 7.5 / 11.6, ζ_T 43.4 / 13.3; (111) 28.2 / 81.5, 21.7 / 29.7, 15.1 / −22.2, ×10⁻¹¹ Pa⁻¹), base junction depth, emitter/base/pinched-base sheet resistances, maximum emitter stripe.
- `realizes`: BJT-02…BJT-22 sizing and checks; silicon-only facts (β barely > 1 in deep-submicron substrate PNPs; lateral β < 1 without NBL) decide construction (H09-48, H01-24).
- `applicability`: every BJT recipe.
- `parameters`: as listed.
- `guarantees`: sizing from σ targets and op-point checks with model data.
- `does not guarantee`: anything beyond the characterized range.
- `pdk data needed`: this card is the datum; characterize offline (sweep ln I_C vs V_BE) when the deck lacks onset data (H09-24).
- `verification`: deck lint; `bjt_max_emitter_stripe` exists in sky130's sidecar (25 µm) but no generator reads it (H09 §9.2.2).
- `prefer over / degrade to`: missing k_A → treat as unknown σ, not zero.
- `source`: H09-02, H09-08, H09-15, H09-20, H09-23, H09-24, H09-25, H09-33, H09-48, H01-24.

#### PDK-05 Interconnect stack
- `id`: PDK-05
- `technique`: Per metal and cut query thickness (Al/Cu only, excluding barriers), sheet resistance (min/mean/max; use max at 25 °C for ESD), J_max with (T_ref, E_a, n) and Blech product, I_cut per via, via EM direction factor, stackability and asymmetric enclosure, exact cut size, via array spacing, wide-metal spacing and slot rule, thick-metal flag, ρ and C_V for adiabatic sizing (Al 2.7 µΩ·cm/2.42 J/°C/cm³, Cu 1.7/3.45), ILD thickness and plate/fringe capacitance per layer pair, poly and silicide sheet resistance (TiSi₂ rises below ~1 µm width; Ni/Co do not).
- `realizes`: REL-01…REL-03, REL-05, FLR-07, FLR-11, FLR-15…FLR-17, ESD-13, FLR-04 (thick-metal edges).
- `applicability`: every deck; analog processes have 3–4 metals with a thick top metal (H01-42).
- `parameters`: as listed.
- `guarantees`: physics-based widths instead of multipliers.
- `does not guarantee`: anything where derating fields are absent (no current Philis deck has them) (H15-38).
- `pdk data needed`: this card is the datum.
- `verification`: deck lint; family defaults flagged as assumed (H05-10).
- `prefer over / degrade to`: deck → family defaults (Al/Cu) → refuse to claim EM compliance.
- `source`: H05-10, H05-12, H05-13, H05-14, H05-15, H14-56, H15-13, H15-18, H15-20, H01-42, H01-47.

#### PDK-06 Oxides, thresholds and voltage spacings
- `id`: PDK-06
- `technique`: Query gate-oxide thicknesses per flavour with marker layers (thin/thick, LVMOS-style derived layers, natural V_t), E_max per thickness and duration, device voltage ratings (V_GS,max, V_DS,block, V_DS,op), thick-field thresholds V_TF per (conductor, backgate), ILD voltage-spacing table, HV recognition layers, depletion-width table (background doping, x_j, V), linewidth control and alignment tolerances.
- `realizes`: REL-07, REL-09, REL-12, GRD-21, BJT-30, ESD-11 oxide check, FLR-10 HV class.
- `applicability`: any design above core voltage or with dual oxides; sky130 recognizes 5 V devices via `hvi` but Philis draws no marker (H01-16).
- `parameters`: as listed.
- `guarantees`: flavour, markers and spacing from one table (H01-16).
- `does not guarantee`: checks when V_TF is absent (report unknown) (H05-38).
- `pdk data needed`: this card is the datum.
- `verification`: deck lint; marker ↔ model consistency.
- `prefer over / degrade to`: deck rules → derivation from x_j/LWC/alignment (PDK-12).
- `source`: H01-02, H01-15, H01-16, H05-17, H05-18, H05-38, H15-36, H09-39.

#### PDK-07 Antenna data
- `id`: PDK-07
- `technique`: Query per-layer peripheral and areal antenna limits, sidewall/perimeter mode, latent-merge distance, diode layers (NSD in substrate, PSD in floating N-well), avalanche voltage of NSD/substrate (≥ ~6 V), fill-block layer.
- `realizes`: REL-08.
- `applicability`: every CMOS deck; ≤ 3.3 V processes need both diode polarities (H05-28).
- `parameters`: as listed.
- `guarantees`: stage-accurate checks.
- `does not guarantee`: n/a.
- `pdk data needed`: this card is the datum.
- `verification`: deck lint.
- `prefer over / degrade to`: typical poly limits (100 µm⁻¹, 500) only as placeholders (H05-26).
- `source`: H05-26, H05-28, H05-29, H05-30.

#### PDK-08 Silicide, resistors and capacitors
- `id`: PDK-08
- `technique`: Query silicide type and silicide-block layer with enclosures, resistor recipes (sheet R min/mean/max, tolerance, minimum width, breakdown, isolation bias; e.g. poly 20 Ω/□ ±30 %, PSD 50 Ω/□ 20 V ±20 %, N-well 2 kΩ/□ 40 V ±40 %; BiCMOS LSR 5, MSR 200, HSR 1000 Ω/□ ±20/20/30 %), field-oxide thickness under resistors, MOS-cap density and bias condition (0.86 fF/µm² at 400 Å, ±20 %, well ≥ 1 V above poly), RW ballast product.
- `realizes`: BJT-14 (short-emitter silicide block), ESD-06/12 (silicide block, RW), REL-04 (resistor width), GRD-19 (bias ERC), FLR-14 (routing over HSR), DIO-04 (Zener ballast).
- `applicability`: all decks; silicided processes need a block mask for resistors above a few kΩ (H01-21).
- `parameters`: as listed.
- `guarantees`: recipe choice by tolerance, voltage and power (H01-46).
- `does not guarantee`: n/a.
- `pdk data needed`: this card is the datum; `sheet_tolerance` is empty in Philis sidecars (H01-46).
- `verification`: deck lint.
- `prefer over / degrade to`: n/a.
- `source`: H01-20, H01-21, H01-22, H01-23, H01-46, H01-47, H14-51, H09-10, H05-05.

#### PDK-09 Design-rule classes the deck must express
- `id`: PDK-09
- `technique`: Confirm the deck can express: minimum width per layer; exact cut sizes with object-class exceptions (bondpads); same-layer and inter-layer spacing; enclosure including asymmetric two-side enclosure; extension/overhang; exact-distance and no-touch (via must not touch contact = stacking prohibition); voltage-conditional spacing; region-conditional spacing (NMOAT–NMOAT inside vs outside N-well); connectivity-conditional abutment (PMOAT may abut NMOAT when connected by metal); containment-conditional rules; derived layers and mask generation (NMOAT → MOAT + NSD, slot layers FINAL = MET − SLOT); optional modules as separable rule sets; coding increment and grid; process electricals; plus chapter-15 classes: wide-metal spacing, EM per layer/cut with derating, antenna ratios, density windows, pad/probe-pad rules, slotting, corner exclusion, diagonal widths, via stacking, ESD path resistance, latch-up distance. Allowed angles (Manhattan only vs 45°), GDSII limits (≤ 200 vertices, ≤ 32-character names, layers 0–255 common).
- `realizes`: rule-exact generators and sign-off parity (H15-44, H01-33, H01-27, H01-31).
- `applicability`: every deck audit; Philis decks lack voltage-conditional spacing, EM derating fields, stacking, slotting, corner exclusion and ESD resistance (H15-44).
- `parameters`: rule grammar.
- `guarantees`: in-loop checks match sign-off semantics (merge before check, vertex-to-vertex spacing, `exact`, `must touch`) (H01-33).
- `does not guarantee`: n/a.
- `pdk data needed`: this card is the datum.
- `verification`: deck lint per class.
- `prefer over / degrade to`: n/a.
- `source`: H15-44, H01-25, H01-26, H01-27, H01-31, H01-33, H01-34.

#### PDK-10 Die and package context
- `id`: PDK-10
- `technique`: Query (from the user or assembly house, not the PDK) die outline and block origin, die thickness, package type (plastic, CSP with bump map, heat-sunk), θ_JA or θ_JC with T_C, ambient grade, leadframe material (Cu vs Alloy 42 stress), mold compound, bondwire material/diameter/length, corner-exclusion rule, external heat sources, pin classes and pad order.
- `realizes`: BJT-18, BJT-19, FLR-03, FLR-04, FLR-05, REL-04, GRD-13 (die thickness), ESD-01 (pin classes).
- `applicability`: whenever Philis places a block on a die; without it, die-location rules report "unknown" and use the block's symmetry axis as proxy (H09-18).
- `parameters`: as listed.
- `guarantees`: stress and thermal context becomes an explicit input (H01-44).
- `does not guarantee`: n/a.
- `pdk data needed`: this card is the datum.
- `verification`: presence check; otherwise unknown.
- `prefer over / degrade to`: n/a.
- `source`: H09-18, H01-44, H05-01, H15-11, H15-12, H14-01.

#### PDK-11 Qualification targets
- `id`: PDK-11
- `technique`: Query the ESD levels (HBM 150 pF/1.5 kΩ: 1 kV accepted since 2010, 2 kV often demanded → I_pk ≈ V/1.5 kΩ, 1.3 A at 2 kV; CDM 500 V, proposed 250 V; MM obsolete; HMM/IEC 61000-4-2 330 Ω, 8 kV, human-accessible pins only), the latch-up test (100 mA, 50 ms, both polarities, ≥ 10 % supply rise = fail), the life target (100,000 h at T_J,max) and the OVST provision (2× once, 100 ms).
- `realizes`: peak currents for ESD-04, ESD-13, REL-05; ring/tap sizing for GRD-12/13; derating temperature for REL-01.
- `applicability`: every die.
- `parameters`: `hbm_v`, `cdm_v`, `hmm_pins`, `latchup_test_ma`, `life_hours`.
- `guarantees`: one source of truth for all sizing (H05-24).
- `does not guarantee`: n/a.
- `pdk data needed`: this card is the datum.
- `verification`: presence.
- `prefer over / degrade to`: defaults 2 kV HBM / 500 V CDM / 100 mA when unstated.
- `source`: H05-24, H05-49, H05-01, H14-44, H05 §5.1.4.

#### PDK-12 Derivation fallbacks when a rule is missing
- `id`: PDK-12
- `technique`: When a deck lacks a dimension a generator needs, derive it from error terms and say so: minimum feature F ≈ λ/(2·NA) (365 nm, NA 0.6 → 0.3 µm, used at 0.5 µm); linewidth control ≈ 10 % of F per feature; mask alignment ≈ 20 % of F per alignment, two alignments ≈ 1.6× one; outdiffusion 0.8·x_j (5 µm well spreads 4 µm per side; 10 µm drawn → 18 µm); dilution below ≈ 2·x_j (keep width ≥ x_j); depletion widths from Table 3.1 with ≥ 50 % margin; process bias ≤ 10 % of film thickness and geometry dependent (identical geometry for matched devices); practical junction depth ≈ 15 µm; worked example: NDIF width 2.0, NDIF space 5.8/7.8 µm at 10/20 V, CONT 1.0 exact, CONT space 1.0, NDIF over CONT 0.4, METAL width/space 1.8, METAL over CONT 0.4, POR width 10.0, POR space 4.2, METAL over POR 1.4 µm.
- `realizes`: missing enclosures (e.g. implant over tap: enc ≥ 0.8·well depth), explained clearances, deck lint (empty `linewidth_control_nm`, unused `n_well_depth`) (H01-03, H01-04).
- `applicability`: only where the deck is silent; deck rules always win.
- `parameters`: F, LWC, alignment, x_j, V.
- `guarantees`: consistent, explainable fallbacks.
- `does not guarantee`: sign-off legality (the deck decides).
- `pdk data needed`: F, x_j per layer, LWC, alignment.
- `verification`: fixture test against the worked example; lint flags.
- `prefer over / degrade to`: deck rule → derived value flagged as derived.
- `source`: H01-02, H01-03, H01-04, H01-36, H01-37, H01-38.

---

## 2. Numeric rules of thumb

### 2.1 Bipolar and diode matching

| Rule | Value | Source |
|---|---|---|
| Thermal voltage at 25 °C | ≈ 26 mV | H09 §9.1 |
| V_BE temperature coefficient; I_C sensitivity | −2 mV/°C; 8 %/°C (80,000 ppm/°C); ±1 mV ↔ ±0.5 °C | H09-15 |
| Ratioed pair ΔV_BE | V_T·ln N; 8:1 → 54 mV; quad 4:1:1:4 → 72 mV; 1 % area error → 0.25 mV | H09-02, H09-04 |
| Random mismatch | s(ΔV_BE) = k_A·V_T/√A_E, k_A ≈ 2 %·µm; 6×6 µm → 0.33 %, 86 µV | H09-02 |
| Classes | minimal ±2 mV/±8 %; moderate ±0.5 mV/±2 %; exceptional ±0.1 mV/±0.5 % | H09-01 |
| Ratio band | even 4:1–16:1 (optimum 6:1–16:1); quads 4:1:1:4–8:1:1:8; > 8–10 units impractical | H09-03, H09-04 |
| 3×3 eight-around-one | spans ⅓ of 1-D length; quadratic residue ÷ ~10 | H09-07 |
| Vertical unit emitter | width ≥ 2× minimum (e.g. 4 µm); 8–40 µm typical; 4–100× minimum area | H09-08 |
| Diode unit | 2–10× minimum; 10–50 µm² moderate diode-connected; Schottky ≥ 10–15 µm | H09-08 |
| Circle polygons | 32 or 64 sides, count divisible by 4 | H09-09 |
| Degeneration | 50 mV ÷3, 100 mV ÷6; ≥ 50 mV moderate, ≥ 100 mV exceptional; 200–300 mV for 10:1 unlike emitters; 250–500 mV makes size irrelevant | H09-26 |
| Merged-base spacing | neutral base ≥ 2× base x_j; base over emitter +1–2 µm; merged junctions +1–2 µm beyond unconnected rule | H09-12, H09-13 |
| Heat separation | ≥ 250 µm from ≥ 250 mW; 100–250 µm from > 50 mW; ≥ 1 W precludes exceptional; heat-sunk helps if ≥ 1 mm | H09-16 |
| Die location | ≥ 250 µm from edges; never corners; 125–250 µm from the far edge with a heater | H09-18 |
| Piezojunction | 100 MPa → NPN 0.4 mV (111) / 0.9 mV (100); LPNP 0.8 / 0.3 mV | H09-20 |
| Package shift | SOT-23 bandgaps −1.1 mV mean, 2.3 mV σ; 15 µm Cu shield −5.06/2.64 → −2.26/1.38 mV | H09-21 |
| NBL shadow | up to 2× epi; use 150 % epi when unknown; 2:1 array with one shadowed emitter → 0.5 % (0.13 mV) | H09-22 |
| Early effect | verticals V_A 100–300 V → 0.3–1 %/V; laterals 50–200 V → 0.5–2 %/V; 150 V example → 0.7 % | H09-23 |
| Current density ceiling | verticals ≤ ⅓–½ HLI onset; laterals 30–50 % (~1 µA per minimum emitter); NPN roll-off > 10 µA/µm², < 10 pA/µm² | H09-24 |
| Substrate-PNP roll-off | ~1 µA/µm² (vs ~30 µA/µm² vertical NPN); R_E ~100 Ω vs ~10 Ω | H09 §9.2.2 |
| Reverse E–B limit | ≤ 75 % V_EBO general; ≤ 50 % matched; ≤ ⅔ diode-connected; poly emitter ≤ ½ (1–2 V) | H05-37, H09-25 |
| Beta floors | mirror β ≥ 10; cold β > 20 for exceptional laterals | H09-48 |
| Split collectors | identical symmetric segments ±1 %; cross-coupled ±0.1 % | H09-30, H09-31 |
| Lateral PNP field plate | overlaps collector 2–3 µm; without it β shifts > 30 % below 5 V | H09-29, H05-43 |
| Collector efficiency | > 0.95 required, often > 0.99; shared tank for matched laterals only if > 0.999 | H09 §9.1.4, H09-30 |
| Lateral β roll-off | > ~100 µA per minimum emitter; β < 10 at ~250 µA; deep-P+ emitter → ~0.5 mA | H09 §9.2.3 |
| Substrate-PNP current | ≤ 1 mA/device, ≤ 10 mA total with nearby contacts; scribe contacts 10–20 mA | H09-11, H01-08 |
| Diode-connected NPN | R_S 10–20 Ω; 0.65 V at 1 µA/µm²; +18 mV per doubling; ≤ 400 mV collector drop at 25 °C, 200 mV at 150 °C; sinker above a few hundred µA; power layout ≥ 10 mA | H09-41, H09-38 |
| Zener | V_EBO ≈ 6.8 V (6–12 V); walkout 200–300 mV, up to > 1 V; buried 0TC 5.0–5.4 V; thermalization 140 nm e⁻ / 60 nm h⁺; PSD/N-well ≤ 1–2 µA/µm periphery; silicide block 1–2 µm | H09-45, H05-36 |
| Schottky | barrier optimum 0.75–0.80 V, < 0.6 V leaky; clamp ≥ 150 mV below B–C; ideality ≤ 1.1 minimal, < 1.05 (1.03) moderate | H09-44, H09-42 |
| HV fillets | radius ≥ 150 % of junction depth; 120 V planar junction may break at 60 V at corners | H09-40 |

### 2.2 Power bipolars

| Rule | Value | Source |
|---|---|---|
| Small-signal layout limit | OK to ~10 mA / 100 mW; acute above 100 mA / 500 mW | H09 §10.1 |
| Finger ballast | 2–3 V_T (50–75 mV) at full current; tolerates ≈ 25 mΩ interconnect; > 100 mV total debias → redesign | H09-34 |
| Finger skew example | 1/2/3 mV lead drops → 24–26 % skew at 400 mA | H09-34 |
| Intrafinger / base debias | ≤ 5 mV (ΔV_BE = L·R_S·I_E/(2W)); base ≤ 2–4 mV; both-end feed ÷ 4; WENC metal ≤ 5–10 mV | H09-35 |
| Current density by role | linear ≤ 150 µW/µm² and ≤ 10 µA/µm²; switched ≤ 20 µA/µm²; pulsed ≤ 1 µs, ≥ 250 ns apart, RMS ≤ 20 µA/µm² | H09-36 |
| Interdigitated geometry | emitter width 8–25 µm; sinker ≥ 2× epi; two sinkers ÷ 4 NBL resistance | H09-36, H09-38 |
| WENC overlap | 10–30 µm; ΔV_E = R_SE·W_O²·J_E (23 mV example); ΔV_B = R_SB·W_O²·J_E/β (36 mV) | H09-36 |
| Power lateral PNP | 0.25–1 mA per minimum emitter before β < 5 | H09 §10.1.3 |
| Thermal runaway | +10–15 °C doubles I_C; hot spots in tens–hundreds of µs | H05-09 |
| Hole-blocking ring for diodes | mandatory above a few mA; > 100 mA width ≥ 2× epi | H09-28 |
| Sense device | two halves on the symmetry axis halfway between centre and periphery | H14-20 |

### 2.3 Merging, guard rings, latch-up and taps

| Rule | Value | Source |
|---|---|---|
| Lateral diffusion | 0.8× junction depth; two 10 µm diffusions ≥ 16 µm apart, ~20 µm with depletion | H14-04, H01-04 |
| Merge area gain | 3 NPNs in one tank ≈ 70 %; 3 PMOS in one well < 50 % | H14-04 |
| Debias examples | 1000 □ × 50 mΩ/□ × 100 µA = 0.5 mV; 2 µA × 500 Ω = 1 mV; 100 mA × 5 Ω sinker = 0.5 V | H14-05 |
| Forward-bias trigger | ≈ 300 mV at 150 °C (3 kΩ at 100 µA); deep-N+ tie ≤ 100 Ω vs hundreds–thousands Ω | H14-05, H14-16 |
| Schottky injector | ~0.5 % minority current; latches if β > 200 | H14-17 |
| SCR criterion | β_N·β_P·(1 − η_C) > 1; 300 × 10 needs η_C > 0.997 | H14-14 |
| LPNP cross-injection | ~¼ of a saturated collector's current; < 1 % unsaturated | H14-15 |
| Injector resistance | pins through < 50 kΩ (ch. 14/15); < 10 kΩ "may trigger", 50–100 kΩ conservative; ≥ 100 µA sunk triggers; < 1 pF coupling latched | H14-01, H15-43, H05-47 |
| Latch-up test | 100 mA, 50 ms, both polarities; ≥ 10 % supply rise = fail; rings survive 100–200 mA if strapped | H05-49, H05-56 |
| ECGR width | ≥ P-epi thickness or P-well depth (CMOS) | H14-28 |
| HCGR | PMoat width ≥ N-well depth; needs retrograde well bottom ≥ 10× middle | H14-29 |
| HBGR | N+/N− ≥ 100:1; deep-N+ > 10²⁰ when well > 10¹⁸; > 95–98 %, > 99 % with HCGR; DTI < 5 % permeation; NBL ≥ 50× well bottom | H14-33, H01-19 |
| Ring efficiency | BiCMOS ECGR > 90 %; P+ substrate 10–100× electron reduction; permeation example 0.0018 | H14-32, H05-56, H05-55 |
| Substrate contact on epi/P+ | R_V ≤ 3 Ω (100 mA, 0.3 V) → 0.33 mm² at 10 Ω·cm, 10 µm; dispersed 10 µm² contacts ≥ 2t apart = 13 % of one large contact; ring width gain tapers > t/2 | H05-50 |
| Bulk P− substrate | contacts within ½ die thickness (~100 µm); 10 µm contacts 1 kΩ far, 840 Ω at 20 µm; dot + ring 110 Ω at 5 µm, 22 Ω at 100 µm | H05-51 |
| Thin layers/wells | annular ≥ t/2; small contacts ≤ 20–50·t apart; electrons travel ~800 µm in 10 Ω·cm P-Si | H05-52 |
| Isolation sheet | 10–20 Ω/□ (standard bipolar) | H05-53 |
| Tie distance (sidecars) | 3 µm sky130; 20 µm IHP | H01-07 |
| P+ substrate fill | fill with contacts if substrate R would exceed 1 Ω | H15-60, H05-50 |
| Well depth vs voltage | 40 V → 6–10 µm; 20 V CMOS ≈ 5 µm; depletion stays out of top 2–3 µm | H01-04 |
| Depletion widths (1e15, x_j 1 µm) | 3/4/5/8 µm at 5/10/20/40 V; ×1.5 margin | H01-02 |
| Noise-ring widening | helps "only a little" → minimum width | H14-09 |

### 2.4 ESD

| Rule | Value | Source |
|---|---|---|
| HBM 2 kV | I_pk ≈ 1.3 A; rise ~15 ns; τ ≈ 225–230 ns | H14-44 |
| CDM 500 V | > 5 A (10–12 A in examples) in ~0.25 ns, 1–2 ns duration | H14-44 |
| IEC 61000-4-2 8 kV | ~30 A peak < 1 ns, ~16 A for 50–100 ns; handled off-chip | H14 §14.4 |
| Metal between pads | ≤ 2 Ω (2.3 Ω for 3 V at 1.3 A); R_VSS ≤ 2 Ω → 2.6 V pad-based; rail-based R_VDD, R_VSS ≤ 1 Ω, diodes 2–3 V each | H14-54, H14-55, H05-23 |
| Clamp arithmetic | 7 V Zener + 2 Ω → 9.6 V at 1.3 A, 31 V at 12 A; 500 Ω + 50 Ω secondary → 44 mA, 9.2 V | H14-44 |
| Oxide during ESD | 90 Å 5 V oxide ≈ 22 V; 50 Å ≈ 24 MV/cm for 100 ns; GCNMOS + 2 Ω: 13.8 V at 1.3 A, 46 V at 10 A | H14-53 |
| Adiabatic cross-section | A = √(ρτI²/(2C_VΔT)), τ 225 ns, ΔT 50 °C → Al 6.5 µm², Cu 4.3 µm²; 0.5 µm Al ≥ 13 µm wide | H14-56 |
| Chamfer | inside corners 45°, ≥ half the lead thickness back | H14-57 |
| GGNMOS | W ≈ 600 µm for 2 kV; overlap +1–3 µm; silicide block 1–3 µm | H14-46 |
| GCNMOS | R1·C1 20–50 ns; disable R2·C2 ≥ 1 µs; V_t1 ≈ ½ at V_GS ≈ ½ V_GD | H14-47 |
| Rate-fired | trigger ~1 V/ns (< 50 ns); disable needed above 100 V/µs slew | H14-43 |
| RC filter | R1·C1 ≥ 10 ns (0.25 pF → 40 kΩ); ECGR around clamps unless R1 > 25 kΩ | H14-52 |
| Bipolar clamps | buffered Zener 300–600 µm² emitter, R1 ≈ 1 kΩ; V_CES 300–500 µm², V_t1 ≈ 65 V / V_h ≈ 45 V for 40 V NPN; V_ECS < 600 µm², V_t 7–12 V | H14-50 |
| Drain ballast | R = RW/W_finger; 5 mm·Ω / 50 µm → 100 Ω | H14-51 |
| Stacking | each device ~2× → stack ~4× area | H14-58 |
| SCR | V_h ~1 V, I_h few mA (70 mA variant) | H14-49 |
| Bondwire | ≈ 1 nH/mm; several volts per nH at CDM | H14-42 |
| ESD diode series R | ≤ 3 Ω for ≈ 5 V at 1.3 A | H09-46 |
| V_c / V_pk definitions | V_c at 1 µA (or 100 nA); V_pk at 1.3 A | H14-43 |

### 2.5 Reliability

| Rule | Value | Source |
|---|---|---|
| EM rule | 5×10⁵ A/cm² at 105 °C, E_a 0.7 eV; n = 2 Al, 1 Cu; 105→125 °C D = 0.58; 50 mA/1 µm → 10 µm | H05-11, H05-12, H15-38 |
| EM onset | failures above ~1×10⁵ A/cm²; critical near 1×10⁶ | H05 §5.1.3, H01-42 |
| Blech | 1200 A/cm ÷ J → 24 µm at 5×10⁵ | H05-13 |
| Pulsed EM | t_DC/D (f < 1/τ), t_DC/D² (f > 1/τ), τ > 0.1 ms; AC > 10 kHz seldom fails | H05-16 |
| Cu vs Al | Cu −35 % resistance, ×10 EM; Cu ≥ 5× Al at 105 °C; parity ≈ 175 °C; 0.5 % Cu in Al > 10× life | H01-42, H05-10, H15-38 |
| Step coverage | 10 kÅ at 50 % → 5 kÅ sidewall; ignore with refractory barrier | H05-14 |
| Self-heating | ΔT = ln(4L/W)/(πκL)·P, κ_Si 1.3 W/cm/°C; 25 µm sq, 100 mW → 14 °C | H05-04 |
| Resistor heating | κ_ox 0.011 W/cm/°C; design ΔT 5 °C, max 50 °C; c_V poly 1.66, Al 2.42 J/°C/cm³ | H05-05, H05-06 |
| Package | θ_JA > 150 °C/W small SMD; θ_JC < 10 °C/W power; T_J,max 125–150 °C; shutdown +25 °C; 100,000 h vs ~1,000 h | H05-01 |
| Leakage | doubles every ~8 °C; parametric > 175 °C, functional > 200 °C | H05-03 |
| TDDB | 3.5–4 MV/cm (300–500 Å), 4–4.5 thinner; 11 MV/cm strength; G ≈ 350 MV/cm; γ 2.5–3.5 cm/MV; β ≈ 2; OVST 2× once 100 ms | H05-18, H05-19 |
| Gettering | deep-N+/NBL within ~100 µm | H05-21 |
| Antenna | poly 100 µm⁻¹ peripheral, 500 areal; clamp positive only if V_ox,max ≤ 150 % avalanche; avalanche ≥ ~6 V → both diodes at ≤ 3.3 V; fill-block ≥ well x_j | H05-26, H05-28, H05-29 |
| Hot carriers | gate current peak at V_GS ≈ 0.4 V_DS; PMOS 2× field; L + 0.5–2 µm → a few volts; NMOS 15 V blocking / 7 V operating (4/3 µm) | H05-33, H05-34, H01-15 |
| Thick-field thresholds | poly ±15 V, M1 ±30 V, M2 ±50 V (20 V process); modern top metal > 50 V; derate 25 % / ⅔ | H05-38, H05-40 |
| Plate/stop overhang | field plate = outdiffusion + misalignment + 2 t_ox; channel stop ≥ two-level misalignment + 2 t_ox; HV PMOS poly inside well by N-well-over-PSD + 1–2 µm | H05-41, H05-42 |
| Zener walkout | +200–300 mV then −300–500 mV (BiCMOS); buried ≥ 1 µm deep does not walk | H05-36 |
| Beta degradation | low-current β −80 % after E–B avalanche; 250 °C/2 min bake reverses 60 % | H05 §5.3.3 |
| Seebeck | 0.1–1.0 mV/°C per contact | H01-43 |
| Bondwire current (Au 25 µm / Cu 50 µm) | 1.8/1.3/1.1/1.0 A and 7.2/4.3/2.9/2.5 A at 1/2/5/10 mm, ΔT 75 °C | H05-07 |

### 2.6 Die assembly and routing

| Rule | Value | Source |
|---|---|---|
| Area estimate accuracy | ±20 % | H15 §15.1 |
| Packing factor P_f | two-metal CMOS/BiCMOS 1.2–1.8; DLM bipolar 1.3–2.5; SLM 1.5–3.0; die 1.1–1.2 loose, < 1.05 tight | H15-01 |
| Routing factor R_f | SLM ≤ 1.2; 1.5-layer 1.2–1.4; DLM 1.1–1.3; maze ≈ 1.0–1.1; TLM 1.1–1.2; maze saves 5–10 % | H15-03 |
| Periphery widths | scribe 50–75 µm saw, 25 µm laser; pad 2–3× wire; seal + ring 20–50 µm; guard rings ≥ 25 µm; padring 100–150 µm | H15-04 |
| Corner exclusion | ≥ 5 % of longer side when > 2.5 mm (5 mm → 250 µm) | H15-12 |
| Bondpad | opening 3× wire (75 µm), pitch 5× (125 µm); exclusion circle +50 µm; leads within 25 µm ≥ 10 µm wide | H15-16 |
| Bondwire resistance | 47 mΩ per mm of 25 µm 4N Au; TCR 3700 (Au) / 4000 (Cu) ppm/°C; +37 % 25→125 °C; 2–3 wires per pin | H15-11 |
| Routing reserve | 20 % of core; primary channels 10–20 % of signals each; feeders ≥ 5 signals | H15-09 |
| Wide-lead widths | W_N = N·P_R + S_m (0.7/0.3 → 1.0, 1.7, …) | H15-24 |
| Pitch example | 0.3 µm w/s, 0.25 µm via, 0.05/0.1 enclosures → P_LTL 0.6, P_VTV 0.65 µm; 0.35 µm path → widen to 0.4 µm | H15-20, H15-21 |
| Star-node offset | 10 □ × 30 mΩ/□ × 1 mA = 0.3 mV | H15-33 |
| Coupling | 0.05 fF per 1 µm M1×M2 crossing (10 kÅ ILD); 1 V/ns into 200 kΩ → 50 mV; 10 fF into 100 kΩ at 1 GV/s → 1 V; < 0.1 ns edges > 1 GV/s | H14-07, H15-29 |
| Slew limiting | 100 kΩ × 1 pF → 10 MV/s | H15-31 |
| Shield plate | extends 2–3 µm past the crossing; full coax costs 3 layers | H15-32 |
| ILD voltage | 2 MV/cm nominal (200 V/µm) but 0.25 µm space ≈ 25 V | H15-36 |
| Poly jumpers | unsilicided 20–50 Ω/□, silicided < 5 Ω/□; static digital only | H15 §15.4.3 |
| Tunnels | base 100–200 Ω/□; emitter/deep-N+/NBL ≈ 10 Ω/□; stacked 3–5 Ω/□; emitter-in-iso 1.5 fF/µm², BV a few volts (≥ 6 V for signals) | H14-39 |
| Cell size | 50–100 components; ≤ 10–12 hierarchy levels; names ≤ 32 chars | H15-05, H15 §15.1.1 |
| Via pairs | ≥ 2 cuts per layer change | H15-19 |

---

## 3. Conflicts and trade-offs

1. **FBSOA vs RBSOA**: wide emitters with distributed ballast (WENC, Christmas-tree) resist thermal runaway but focus current at turn-off; many small emitters (interdigitated) do the opposite; both cannot be optimised in one device, so the role decides (H09-36, H09 §10.1.1).
2. **Compactness vs dispersion**: CC removes only the linear part of a thermal gradient, so compact arrays (3×3) minimise the nonlinear residue (H09-06, H09-07), while exceptional class wants several cross-coupled pairs dispersed to average random and stress variation (H09-14, H09-08). Resolve by class: compact for moderate, dispersed multi-quad for exceptional.
3. **Thermal vs stress placement**: the die centre has the lowest stress gradient, but with a heater the matched devices go to the far side 125–250 µm from the edge; thermal wins (H09-18).
4. **Emitter size**: larger unit emitters cut random mismatch (k_A/√A_E) but raise gradient sensitivity and current crowding (18 mV debias doubles local current); laterals lose β with larger emitters, so they stay at minimum (H09-08, H09-33).
5. **Degeneration**: relaxes transistor matching by V_T/(V_T + V_d) and makes size irrelevant above 250 mV, but costs headroom, moves the burden onto a matched resistor array, and is impossible for ratioed PTAT pairs and split collectors (H09-26, H09-31).
6. **CBE vs CEB**: CBE keeps emitters and base contacts closer for CC (base contacts carry thermoelectric potentials); CEB has lower R_C for single power devices (H09-14, H09-32).
7. **Merging vs isolation**: a shared tank or well saves 30–50 % area, but any injector, debias above ~300 mV or noisy/sensitive mix in the shared region risks latch-up or offset; the predicate decides and the answer "separate" is the default when in doubt (H14-04, H14-03, H14 §14.1.5).
8. **Grounded vs supply-tied ECGR**: a grounded ring survives latch-up current only with deep-N+ and full metal strapping; a supply-tied ring tolerates debias but dissipates power at high voltage (H14-30, H05-56).
9. **Ring width**: widening a substrate-tap ring around an injector helps little (minimum width), but the CMOS ECGR's only lever is width (≥ epi thickness) because low supplies cannot deepen the depletion region (H14-09, H14-28).
10. **Tap strategy by substrate**: scatter small contacts and fill on epi-on-P+, but on bulk P− make few large contacts within half the die thickness because a ring closer to the injector is not lower in resistance there; in thin wells spacing beyond 20–50·t stops helping (H05-50, H05-51, H05-52).
11. **HBGR vs ECGR**: in standard bipolar the hole-blocking ring beats the electron collector when space is short, but a permeable NBL can turn an HBGR into a net injector while HCGRs are unaffected (H05-58, H14-33).
12. **HCGR tie**: tank potential risks debias through a high-resistance base diffusion; ground limits the tank voltage to the P-iso/NBL breakdown and burns power at high voltage (H05-57).
13. **Pad-based vs rail-based ESD**: pad-based is failsafe under sequenced supplies (analog default); rail-based dual diodes are small and low-capacitance for high-speed pins but conduct whenever the pin exceeds the supply (H14-54, H14-45).
14. **ESD device size vs behaviour**: the SCR is smallest but latches by design; the active FET never latches but is the largest; stacking doubles each device so a stack costs ~4×; rate-fired devices need disable circuits on slewing pins yet disable types are forbidden on system-level ESD pins (H14-49, H14-58, H14-43).
15. **Primary clamp vs internal oxide**: a 60 V-trigger primary cannot protect a 10 V junction, and 2 Ω of metal is 20 V at CDM current, so secondary protection must sit at the gate, costing area and capacitance on the signal path (H14-59, H14-52).
16. **Maze vs channel routing**: maze saves 5–10 % area and R_f → 1.0, but is slower, harder to modify and more prone to unexpected coupling; dummy-metal blocks over matched devices and power devices using all metals block it (H15-03, H15-61).
17. **Full shielding vs routing resources**: coaxial shielding of one signal consumes three routing layers; a plate tied to the wrong (noisy global) reference hurts (H15-32).
18. **Tunnels and poly jumpers**: they remove crossings at the price of R, C, voltage limits and resimulation, and are forbidden on power and sensitive classes (H14-39, H14-35).
19. **Wide-lead quantization**: integer track multiples waste a little width but let partial runs be replaced by N lattice wires (H15-24).
20. **EM vs resistance**: EM must hold at the narrowest point while resistance averages, so widen where room exists and accept short necks for R but never for EM (H15-37).
21. **Blech and pulsed relief vs uncertainty**: both allow narrower leads but depend on process data and on waveforms that are hard to determine for analog; use only with deck data (H05-13, H05-16, H15-40).
22. **Poly vs diffused resistors**: poly is dielectrically isolated (≥ 50 V to substrate, low parasitic C) but conducts heat poorly, drifts under power and fails in ESD paths; wells are robust but ±40 % variable (H01-20, H05-25, H01-22, H01-46).
23. **Substrate contacts vs bypass capacitance**: on P+ substrates fill empty area with contacts when substrate resistance would exceed 1 Ω, otherwise with decoupling (H15-60, H05-50).
24. **Bond-over-active**: thick top metal lets lower layers be used freely under pads, but thin-Al BOAC pads and thick-metal edges are unsafe over matched devices (H15-13, H15-17).
25. **Voltage markers vs blanket spacing**: recognition layers and pairwise ΔV spacing save area but need per-net voltage data; without it only blanket HV spacing is safe (H09-39, H15-36, H01-02).
26. **Short-emitter trade**: fewer emitter contacts raise β in CMOS PNPs but raise emitter resistance, so the choice follows the operating current (H09-10).
27. **Ring merging**: rings collecting ≤ a few mA may merge with isolation/substrate ties to save area; above that they must stay separate because debias occurs even with contacts in the ring (H14-34).
28. **Field plate vs channel stop**: the plate is usually easier and also shields, but both need overhang and the plate adds capacitance to its tie net (H05-41, H05-44).
29. **Anti-saturation loops vs guard rings**: a secondary collector feeding an anti-saturation loop risks instability (no model); keep the hole-blocking ring regardless (H09-47, H09-28).
30. **Fixed PDK bipolars vs cross-coupling**: a one-unit equal pair can only be cross-coupled by splitting each device, which changes the netlist for fixed-geometry PDK devices; advise m = 2 per side instead (H09-49).

---

## 4. Floorplan rules: ordered checklist for a constructive floorplanner

1. **Collect inputs**: netlist with subcircuit ports and pin classes (supply, substrate ground, power ground, input gate, output, HV, Kelvin, probe); op-point with node voltages and per-device I, V_DS/V_GS/V_EB and power; die context (outline, thickness, package, orientation, θ, bondwire, external heaters); qualification targets; deck facts (PDK-01…PDK-12). Without ports no injector can be classified (H14-01, H05-17, H09-18, PDK-10, PDK-11).
2. **Classify**: injectors and noisy devices (GRD-01), sensitive nets and devices (FLR-10), matched groups with class (BJT-01), power devices by class (≥ 250 mW major, > 50 mW significant, ≥ 1 W precludes exceptional) (H09-16), HV nets with voltage ranges (GRD-21).
3. **Estimate area**: per cell ΣA·P_f, core with R_f, die with padring; decide core- vs pad-limited; choose the die aspect (1.5:1–2:1 when exceptional matching must be far from a heater); round to the mask increment and the smallest leadframe with 10–20 % margin (FLR-01, FLR-05, H09-16).
4. **Draw the periphery**: scribe streets, seal ring joined on every metal, substrate ground ring at the lowest potential, corner exclusion zones (5 % of the long side when > 2.5 mm) left empty, slot rules for wide metal (FLR-03).
5. **Order the pads**: follow the pin list and lead fingers (no crossing wires), symmetric pads for mirrored blocks, power pads top/bottom, Kelvin sense pads adjacent to their force pads, pads per supply from bondwire current, pads of one pin joined by wide metal; probe pads on a test-POR layer (FLR-04, FLR-09, H05-07, H05-22).
6. **Place ESD**: one primary device per non-substrate pin adjacent to its pad (pad-based for analog), APD between separate grounds (GND/PGND for power outputs), one ECGR band separating pads/ESD from the core, ring metal sized for ≤ 2 Ω pad-to-pad and the adiabatic cross-section (ESD-01, ESD-03, ESD-05, ESD-13, ESD-14).
7. **Place power devices**: at the centre of one die end or one side so isotherms are predictable; sectioned into spaced banks; sense devices as two halves on the symmetry axis; own backgate/tank; deep-N+ or DNW rings for saturating/injecting devices; separate PGND return (BJT-25, BJT-27, BJT-29, GRD-16, GRD-03, H09-19).
8. **Place the sensitive core on the opposite side**: matched references near the die centre or on a die axis, ≥ 250 µm from edges, never in corners, 125–250 µm from the far edge when a heater exists; ≥ 250 µm from ≥ 250 mW devices and 100–250 µm from > 50 mW; input stage far from output stage along one axis; arrays' S2 along isotherms; nothing matched under thin BOAC pads or across thick-metal edges (BJT-18, BJT-19, BJT-08, FLR-04, FLR-05).
9. **Place noisy digital and switching blocks** away from the core, behind their own minimum-width tap rings with separate ground returns, optionally an N-type noise-blocking ring, or in DNW tubs; put supply-tied wells between injectors and victims (GRD-02, GRD-03, GRD-04, GRD-09, GRD-10).
10. **Partition wells and tanks**: one well per bulk/supply net; merge same-backgate PMOS only where the nine-clause predicate passes; output devices, forward-biased-backgate devices, saturating laterals and Zeners get their own regions; rejected pairs keep 2× well spacing; voltage-dependent spacing between wells on different nets (GRD-15, GRD-16, GRD-17, GRD-21).
11. **Ring every injector by carrier**: ECGR (wide NMoat or supply-tied N-well, adjacent, continuously strapped, L/U at the die edge) for electron injectors; HCGR (grounded PMoat in a retrograde well) or HBGR (closed deep-N+/NBL) for hole injectors; P-bars between merged laterals; size straps for the 100 mA test (GRD-05, GRD-06, GRD-07, GRD-08, GRD-11, GRD-12).
12. **Distribute taps**: by substrate type (scatter and fill on epi-on-P+; large contacts within half the die thickness on bulk P−; annular t/2 and ≤ 20–50·t pitch inside wells); every active point within the tie distance; butted taps where source = bulk; fill free area with contacts on P+ substrates (GRD-13, GRD-14, GRD-18, FLR-18).
13. **Mirror duplicates and keep arrays whole**: identical blocks as mirror images of one cell; every matched array in one cell; identical variants and orientations within a group (FLR-05, FLR-19, H14-38).
14. **Reserve routing**: 20 % of the core as explicit channels, primary channels near the centre carrying 10–20 % of signals each, feeders ≥ 5 signals, alternating preferred directions, cells kept on the lowest layers; check demand against capacity and remove choke points before routing (FLR-06, FLR-17).
15. **Annotate nets**: class, DC/duty-averaged current, R_max, voltage range, slew; list matched components and guard-ring devices (FLR-10, FLR-07).
16. **Route power and ground first**: wide leads from channel edges, integer track multiples, via-filled overlaps at layer changes and branches, 135° bends and via arrays on straight runs, pairs of vias, EM at every point; star nodes for matched returns; Kelvin force/sense split at the terminal or the pad; separate quiet and noisy ground returns joined only at the star point or pad (FLR-07, FLR-08, FLR-09, FLR-15, FLR-16, GRD-03).
17. **Route sensitive nets next**: no noisy net above, below or beside; right-angle crossings with shield plates tied to the victim's own reference; quiet lines between parallel runs; no jumpers or substrate contacts in sensitive grounds; slew-limit static control lines that still violate the C·R·dV/dt budget (FLR-11, FLR-12, FLR-13).
18. **Apply HV rules**: pairwise ΔV spacing in silicon and dielectric, recognition markers, rounded junction corners, field plates or channel stops where leads cross HV P regions, HV poly never over STI field, no HV net over thin emitter oxide (GRD-21, BJT-30, REL-12, REL-07, H05-25).
19. **Place secondary ESD at the gates**: CDM clamps or RC filters next to every pin-connected gate oxide and at every supply/ground domain crossing, tied to that circuit's rails; E–B clamps with balancing resistors on bipolar inputs; check V_ox with extracted metal (ESD-11).
20. **Run reliability sizing**: EM widths with temperature derating and duty, adiabatic ESD paths, resistor widths at ΔT ≤ 5 °C, oxide fields, hot-carrier and NBTI screens on matched sets, E–B reverse-bias limits, parasitic-channel pass (REL-01…REL-12).
21. **Antenna**: ratios per stage with latent merging; jumpers near gates; diodes of both polarities below 3.3 V with UV keep-outs and fill-block (REL-08).
22. **Fill**: dummy-metal block over matched devices, density fill elsewhere, substrate-contact fill on P+ substrates, fill-block over antenna diodes; fill-block obstructs maze routing, so do this after routing (FLR-18).
23. **Sign off (ch. 15.5)**: DRC diagnostics reviewed with process engineering; zero topological LVS errors (never waivable), parametric errors reviewed; power plots for pinch points and via counts; resistance of crucial high-current leads computed; zero antenna violations; ESD path resistance and CDM clamps at every pin-connected gate and domain crossing; minority-carrier guard rings on every diffusion within < 50 kΩ of a vulnerable pin (substrate pins exempt, supply pins by polarity); dummy-metal block over matched devices; noisy/sensitive separation or shielding; pad and probe-pad geometry, keep-away, lead width, pad-1 mark; Kelvin/star physically correct including at the bondpad; corner exclusion and slotting; seal joined to ground metal and scribe width; substrate-contact fill on P+ dice; symbolization and mask-work notice; archival of database, libraries, decks, GDSII, layer maps, checksums in two copies (H15-42, H15-43, H15-41, FLR-19, H15 §15.5).
