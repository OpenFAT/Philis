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
