# Hastings passives: technique cards for a recipe-driven layout engine

Sources, all digests of R. A. Hastings, *The Art of Analog Layout*, 3e:

- `H06-xx`: item ids in `docs/plans/ref-hastings-06-ch06-07-resistors-capacitors-inductors.md` (ch. 6 resistors, ch. 7 capacitors and inductors). `H06 §x.y` cites a section of that digest's narrative where no item id carries the fact.
- `H08-xx`: item ids in `docs/plans/ref-hastings-08-ch08-matching-resistors-capacitors.md` (ch. 8 matching). `H08 §x.y` likewise.
- `ref-99 §n`: `docs/plans/ref-hastings-99-figure-descriptions.md`. For chapters 6–8 it holds no per-figure prose, only cross-checked values for Figs 6.27, 6.28, 7.7, 8.2, 8.5, 8.8, 8.12, 8.18, 8.19, 8.20 (§2.1, §2.2, §2.8, §4).

Card prefixes: `RES` resistor construction, `CAP` capacitor construction, `IND` inductors (brief), `MAT` matching and array techniques shared by resistors and capacitors. Precision classes: MIN ±1–5%, MOD ±0.1–1%, EXC ±0.01–0.1%, all six-sigma over 10 years, −40…125 °C, plastic package (H08-02).

---

## 1. Technique cards

### Resistor construction

#### RES-01 Straight-strip resistor with a bias-corrected value model
- **id**: RES-01
- **technique**: One rectangular body of drawn width W_d and length L_d between two full-width heads (implanted or silicided), on field oxide; the value is computed from sheet resistance, width/length bias and head terms rather than from L/W alone.
- **realizes**: Absolute target R at a chosen width; minimum parasitic capacitance; the unit element that every other resistor card composes (H06-01, H06-21).
- **applicability**: Poly (gate 25–50 Ω/□, HSR ≈ 500 Ω/□, VHSR ≥ 5 kΩ/□), NSD/PSD, base, well, metal bodies; any class; body on thick field oxide with no active beneath (H06-20, H06-21). Head implant must cover the full body width (H06-24).
- **parameters**: W_d; L_d (solved from R); head style (implanted head sheet R_h and overlap L_h, or silicide block with silicided heads); contacts per head; recipe/layer (H06-01, H06-24).
- **guarantees**: R = R_s(L_d + L_b)/(W_d + W_b) for plain strips (Eq 6.7); R = R_s(L_d − 2L_b)/(W_d + W_b) + 2R_h(L_h + L_b)/(W_d + W_b) for implanted heads (Eq 6.22); R ≈ R_s·L_d/(W_d + W_b) for silicide-blocked bodies (Eq 6.23) (H06-01). No lateral current-spreading error because the head has no width step (H06-24). Poly over field oxide couples ≈ 0.05 fF/µm² to substrate, distributed (dual-π: C/4, C/2, C/4) (H06-17).
- **does not guarantee**: Absolute tolerance better than the process (±20–25% sheet, ±50% for VHSR and pinch) (H06-07); matching (see MAT cards); value under naive segmentation (RES-02); linearity at high field (RES-17).
- **pdk data needed**: R_s per recipe; head sheet R_h; W_b (plus a width-dependent bias table below ≈ 2·x_j for diffusions and near the bamboo regime for poly); L_b; L_h; implant and silicide-block enclosures; contact ρ_c; body-to-substrate area capacitance; grain size d_g (H06-01, H06-08, H06-15, H06-17).
- **verification**: LVS parameter check of modeled R against the netlist within grid quantization (H06-01); head implant encloses full width (H06-24); no diffusion under the body (H06-21).
- **prefer over / degrade to**: Poly is the general-purpose choice; degrade to diffused for power/ESD, well only when high-sheet poly is absent, metal for sub-ohm, thin film for extreme precision (H06-20). Use dogbone heads (RES-04) only when the contact does not fit the body width (H06-24).
- **source**: H06-01, H06-07, H06-08, H06-15, H06-17, H06-20, H06-21, H06-24.

#### RES-02 Series segmentation into identical straight strips with jumpers
- **id**: RES-02
- **technique**: Split the body into N equal strips side by side at a fixed gap, joined head-to-head by metal jumpers; current direction alternates strip to strip.
- **realizes**: Aspect-ratio fit of long resistors; the identical-section basis for matched and ratioed sets (H06-06); thermoelectric cancellation when N is even (H08-45); fewer geometry-induced errors than a serpentine (H08-46).
- **applicability**: Any body material; required instead of serpentine at MOD and above (H08-46); N ≥ 2, even preferred; segment length ≥ 10 µm for accurate resistors (H06-03).
- **parameters**: N; L_seg; gap (voltage-dependent, RES-11); jumper layer, width and via count (MAT-09); folded-in or folded-out jumpers (MAT-10); body extension past the active region (MAT-07).
- **guarantees**: R_total = N·[R_s(L_seg − 2L_b)/(W + W_b) + 2R_h(L_h + L_b)/(W + W_b) + 2R_c] when L_seg is solved per N (H06-02); head and contact terms cancel in ratios because every strip is identical (H06-06, H08-10); zero net Seebeck EMF with alternating directions and even N (H08-45); lateral end correction < 1% at ≥ 10 □, vertical crowding negligible at L ≥ 20·thickness (H06-03).
- **does not guarantee**: Equal jumper resistance between members (MAT-09); the value when L_seg = L/N is used naively, which adds 2(N−1)(R_head + R_c): a 1 kΩ base resistor with two 43 Ω contacts is +9% (H06-02); gradient immunity (MAT-04).
- **pdk data needed**: RES-01 data; ρ_c per contact type; jumper sheet R and via R; minimum segment length; voltage-dependent poly spacing (H06-15, H06-22, H08-20).
- **verification**: Σ(segment direction) = 0 per series string (H08-45); |R_model − R_target| ≤ quantization and head/contact share ≤ class budget (H06-02); L_seg ≥ 10 µm and ≥ class floor (H06-03, H08-41).
- **prefer over / degrade to**: Prefer over a bent serpentine (RES-03) for every matched set (H08-46); prefer fewer, longer parallel segments over dogbone heads when a segment would be < 10 □ (H06-02, H06-15). Degrade to N = 1 when the aspect ratio allows.
- **source**: H06-02, H06-03, H06-06, H06-15, H06-22, H08-10, H08-20, H08-41, H08-45, H08-46.

#### RES-03 Bent continuous serpentine
- **id**: RES-03
- **technique**: One continuous body folded with square corners or rounded 180° turns, no jumpers.
- **realizes**: Compact fit of a single non-critical resistor; high-voltage resistors when turns are rounded and corners behind the contacts filleted (H06-05).
- **applicability**: MIN class only (rule R16) (H08-46); any body material; HV resistors (filleting raises HSR breakdown by a few volts) (H06 §6.5.4).
- **parameters**: Corner count and type (square or circular); straight lengths A, B; equal turn spacing (H08-18); both heads close together, non-identical serpentines with oppositely oriented heads (H08-45).
- **guarantees**: Value with turn terms: square corner +0.56 □, R = R_s[(2A + B)/W + 1.12] (Eq 6.9); unequal-width corner by Hall's Eq 6.10 (0.5587 □ at a = 1); 180° circular end +2.96 □ (Eq 6.11) (H06-05).
- **does not guarantee**: Matching at MOD/EXC (H08-46); immunity to turn-spacing bias and to a sinker beside the body (H08-18); thermoelectric cancellation unless heads are co-located (H08-45).
- **pdk data needed**: Corner squares per turn style; R_s; minimum spacing (H06-05).
- **verification**: Corner count × N_corner added to the value model (H06-05); turn spacings equal (H08-18); heads adjacent (H08-45).
- **prefer over / degrade to**: This is the degraded form of RES-02; promote to RES-02 at MOD and above (H08-46).
- **source**: H06-05, H08-18, H08-45, H08-46, H06 §6.5.4.

#### RES-04 Dogbone heads
- **id**: RES-04
- **technique**: Enlarge the head around the contact so a contact wider than the body fits.
- **realizes**: Contactability of narrow bodies and lower contact resistance per head (H06-06, H06-15).
- **applicability**: Bodies narrower than the minimum contact; any class provided every section in a set has the same head (H06-06); in mixed-length arrays dogbones force embedded dummies between different lengths (H08-43; Fig 8.18: 10 dogbones with dummies at the 4th and 7th positions, ref-99 §2.8).
- **parameters**: Head width W_h; head overhang W_o; contact width W_c.
- **guarantees**: Current spreading lowers R by 0.7 □ (W_o = W_d, W_c = W_d) to 0.3 □ (W_o = ½W_d), usually < 0.3 □ (Table 6.3) (H06-06).
- **does not guarantee**: Any matching advantage; the accuracy advantage is illusory and matched resistors should be identical sections regardless of head (H06-06); packing density (H06-06).
- **pdk data needed**: ΔR table versus (W_o, W_c); contact rules (H06-06).
- **verification**: Every section of a matched set has the same head geometry (H06-06); head/contact share ≤ budget (H06-02).
- **prefer over / degrade to**: Prefer full-width strip heads (RES-01) wherever the contact fits (H06-24); prefer parallel longer segments (RES-02) over dogbones for segments < 10 □ (H06-02, H06-15).
- **source**: H06-02, H06-06, H06-15, H06-24, H08-43, ref-99 §2.8 (Fig 8.18).

#### RES-05 End-effect-immune segment sizing with contact-end correction
- **id**: RES-05
- **technique**: Choose the segment length so contact end effects, vertical current crowding and poly grain nonlinearity vanish, or add the Ting–Chen correction explicitly for short segments.
- **realizes**: Absolute accuracy of short segments (trim LSBs, partial segments); linearity (H06-03, H06-04).
- **applicability**: All resistors; mandatory correction for segments < 10 □ with contacts narrower than the strip (H06-04); class length floors 3×/5×/10× minimum for MIN/MOD/EXC (H08-41).
- **parameters**: L_seg; contact width relative to W; correction on/off.
- **guarantees**: Lateral end correction < 1% at ≥ 10 □ (H06-03); Ting–Chen Eq 6.8, both ends together, strictly valid for W_c ≫ W − W_c (5 µm strip, 3 µm contact: +0.05 □) (H06-04); vertical crowding negligible at L ≥ 20·t (H06-03); grain-barrier nonlinearity negligible at L ≥ 1000·d_g (≈ 50 µm), acceptable at 10 µm (H06-03, H06-13).
- **does not guarantee**: Contact resistance itself, which is R_c = (√(R_s·ρ_c)/W_c)·coth(L_c·√(R_s/ρ_c)) (Eq 6.19) (H06-15).
- **pdk data needed**: Film thickness t; grain size d_g; minimum segment length; contact width; ρ_c (H06-03, H06-04, H06-15).
- **verification**: L_seg ≥ max(10 µm, 10·W, 20·t, class floor) (H06-03, H08-41); EXC poly series path ≥ 1000·d_g or 200 µm if d_g unknown (H08-41).
- **prefer over / degrade to**: Lengthen segments before oversizing contacts; oversize contacts before dogbones (H06-02, H06-15).
- **source**: H06-02, H06-03, H06-04, H06-13, H06-15, H08-41.

#### RES-06 Width selection by tolerance class with narrow-width anomaly floors
- **id**: RES-06
- **technique**: Pick W from the absolute tolerance budget δR ≅ δW/W + δR_s (Eq 6.12) and the random-mismatch law, then clamp above the physical floors of the material.
- **realizes**: Absolute tolerance class (none, moderate, crucial) (H06-07); the width half of the random-mismatch budget (MAT-01).
- **applicability**: All resistors. Floors: poly ≥ 0.25–0.3 µm (bamboo effect, grains ≤ 0.1 µm) (H06-08); TiSi₂-silicided ≥ 0.25 µm (C49 phase below 0.2 µm, 80–100 vs 13–20 µΩ·cm, bimodal R) (H06-08); diffusions ≥ 2·x_j and wells ≥ 2–3·x_j (dilution makes W_b width-dependent) (H06-08, H06-25).
- **parameters**: W; tolerance class; recipe.
- **guarantees**: δR ≅ δW/W + δR_s; with ±20% R_s and ±0.05 µm linewidth: ±30%, ±25%, ±21% at 0.5, 1, 10 µm (H06-07). Guidelines: none → minimum width; moderate → 2–3× minimum, poly ≥ 0.3 µm, diffusion ≥ 2·x_j, no poly > 5 kΩ/□, no pinch resistors; crucial → trim, poly ≥ 0.5 µm, no lightly doped material (H06-07). Matching floors 150/200/400% of minimum width for MIN/MOD/EXC (rule R4) (H08-41).
- **does not guarantee**: Mismatch below the random law s_δ = (k_R/W)·√(1/R) (H08-06); immunity to small-area silicide outliers (3–5× the mean in string DACs; Co and Ni silicides immune) (H08-41).
- **pdk data needed**: Sheet tolerance %; linewidth control (≈ ±10% of minimum feature for deposited layers, plus ±5% of x_j for diffusions); x_j; d_g; silicide kind (H06-07, H06-08, H08-41).
- **verification**: W ≥ max(DRC minimum, anomaly floor, class floor) and δR(W) ≤ budget (H06-07, H06-08, H08-41).
- **prefer over / degrade to**: Widen before trimming for moderate tolerance; trim for crucial (H06-07).
- **source**: H06-07, H06-08, H06-25, H08-06, H08-08, H08-41.

#### RES-07 Diffused resistor with body tap, body biasing and segmentation into separate bodies
- **id**: RES-07
- **technique**: Base, emitter, NSD/PSD or well body inside a tank or well; each segment's body tied to that segment's positive end; split into separately biased bodies when the voltage across the resistor exceeds 2/3 of the junction rating.
- **realizes**: Power and ESD resistors (body in contact with silicon dissipates more than poly) (H06-17); tracking body modulation in dividers (H06-11); voltage rating above one junction (H06-19).
- **applicability**: Base 100–200 Ω/□ for 50 Ω–20 kΩ; emitter 2–10 Ω/□ for tenths of an ohm to ≈ 200 Ω with low VCR and TC; NSD/PSD 20–50 Ω/□ (2–5 silicided) for ESD clamps and power (H06 §6.5.1, §6.5.2, §6.5.8); width ≥ 2·x_j (H06-08); orientation-sensitive materials (MAT-08). Do not merge with minority-carrier injectors (H06-19).
- **parameters**: Body net (default: the segment's positive terminal); body count k; NBL overlap (5–8 µm under matched base resistors; ≥ 150% of epi thickness, or all sides, where the NBL shadow is unknown) (H06 §6.5.1, H08-26); field plate (RES-08).
- **guarantees**: The resistor–body junction stays reverse-biased when body = positive end; ratio drift from body modulation tracks when each equal segment has its own body (H06-11, H06-19); ≤ 2/3 of the body-junction rating (E-B avalanche for emitter and pinch) per segment (H06-19).
- **does not guarantee**: Freedom from VCR (base 100 ppm/V; HSR 2.5%/V; 700 Ω/□ P-type body modulation 1000 ppm/V and −20 ppm/V²) (H06-11); linearity at high field (RES-17); shadow immunity without overlap (H08-26).
- **pdk data needed**: x_j; junction BV; body-modulation coefficient; junction C 1–5 fF/µm²; NBL shift magnitude/direction and epi thickness on patterned-NBL processes (H06-11, H06-17, H08-26).
- **verification**: ERC: V_body ≥ max terminal voltage with margin; |V_P − V_N| ≤ (2/3)·BV else split (H06-19); equal resistor-to-body voltage across a matched set (H08-31).
- **prefer over / degrade to**: Poly (RES-01) for general use; diffused for power and ESD (H06-20). For matched HSR dividers: equal segments, one tank each tied to its positive end, field plate tied to the same end (H06 §6.5.4).
- **source**: H06-08, H06-11, H06-17, H06-19, H06-20, H08-26, H08-31, H06 §6.5.1–6.5.4, §6.5.8.

#### RES-08 Field-plated high-sheet or well resistor, with split field plates
- **id**: RES-08
- **technique**: Metal plate over the body tied to the segment's tank/well (its positive end), one plate per segment; optionally split with the gap halfway down each segment.
- **realizes**: Immunity to charge spreading; suppression of dielectric-absorption drift (0.1% seen in 2 kΩ/□ HSR under BPSG) (H08-30); HV operation above the thick-field threshold (H06-14).
- **applicability**: Diffused or implanted resistors above 50% (rule R20) of the top-metal thick-field threshold, every MOD one with R_s ≥ 500 Ω/□, every EXC one (H08-30); unpinched N-well resistors always (H06-25); split plates for diffused resistors > 1 kΩ/□ that must match better than ±0.5% (H08-30). Not needed for base resistors or poly below a few kΩ/□ (H08-30). The chapter-6 threshold is 2/3 of the thick-field threshold (H06-14).
- **parameters**: Plate net; per-segment or common; split position; overhang 2–3 µm (H08-29).
- **guarantees**: Blocks DC and low-frequency fields, not digital edges (H08-29); gaps between per-segment plates are not channel-stopped (H08-30).
- **does not guarantee**: Freedom from body modulation (H06-11); linearity of PMoat-pinched wells (severely nonlinear) (H06-25).
- **pdk data needed**: Thick-field threshold of the top metal; well x_j; R_s; charge-spreading flag per recipe (H06-14, H06-25, H08-30).
- **verification**: Plate present when the thresholds hold and tied to the right net; per-segment when R_s > 500 Ω/□ or more than a few volts span the array (H08-29, H08-30).
- **prefer over / degrade to**: Use HSR only when poly ≥ 500 Ω/□ is unavailable; optimal HSR sheet ≈ 2 kΩ/□, always with NBL (H06 §6.5.4). Base pinch (±50%, ±5% residual even when matched) and epi pinch are for non-critical values only (H06 §6.5.3, §6.5.5).
- **source**: H06-11, H06-14, H06-25, H08-29, H08-30, H06 §6.5.3–6.5.5.

#### RES-09 Metal resistor with Kelvin force/sense taps
- **id**: RES-09
- **technique**: Metal run or serpentine over field oxide where there is no CMP, optionally paralleled layers with many vias; separate force and sense taps, L_d measured between the sense taps' inner edges.
- **realizes**: 50 mΩ–50 Ω current-sense and ballast resistors with a VPTAT-compatible TCR (H06-23).
- **applicability**: Bipolar metal 10–15 kÅ at 20–30 mΩ/□; lower CMOS metals 3–5 kÅ at 50–90 mΩ/□ (H06 §6.5.6); EXC sense paths require Kelvin connections (H08-21).
- **parameters**: Layer(s); width; length; via count; tap positions; force leads straight and constant-width for ≥ 2× the resistor width before any bend or width change (H06-23); two-level layout with sense taps on an upper metal (less sensitive) (H06-23).
- **guarantees**: Sense path shares no segment with force current when drawn as a star at the taps (H06-23); via R is a small fraction of metal R with enough vias (H06-23).
- **does not guarantee**: Absolute value (R_s ±20%, often unmonitored) (H06-23); temperature independence (TCR ≈ +3300 ppm/°C) (H06-23).
- **pdk data needed**: Metal R_s per layer; via R; CMP flag; TCR (H06-23).
- **verification**: Sense nets branch only at the taps; I_sense·R_lead ≤ ε (H06-23); IR drop on terminal routes ≤ budget (H08-21).
- **prefer over / degrade to**: The only option below ≈ 1 Ω; degrade to emitter or NSD/PSD bodies for 1–200 Ω power resistors (H06 §6.5.2, §6.5.8).
- **source**: H06-23, H08-21, H06 §6.5.6.

#### RES-10 Thin-film resistor with laser trim
- **id**: RES-10
- **technique**: Deposited NiCr, Ta or SiCr strip (single- or two-mask flow), laser-trimmed at probe.
- **realizes**: EXC absolute and ratio accuracy; TC < 100–150 ppm/°C; no voltage modulation (H06 §6.5.10, H08-56).
- **applicability**: NiCr 20–200 Ω/□ (+50…+100 ppm/°C); Ta 20–200 Ω/□ (−100…−150); SiCr 100–2000 Ω/□ (0…−150) (H06 §6.5.10); EXC class (H08-56). A single-mask TFR blocks routing on the metal directly above and is incompatible with W plugs; the two-mask TFR/TFC flow avoids this (H06-14, H06 §6.5.10).
- **parameters**: Material; W; L; trim structure (RES-16); flow (single- or two-mask).
- **guarantees**: Sheet ±20%, laser-trimmed to ±0.05% or better, process TC variation < ±50 ppm/°C, minimal aging (H06 §6.5.10); k_A < 0.5%·µm (H08-56); nichrome π_L ≈ 0.9–1.3e-11 Pa⁻¹, sichrome ≈ 10× (H08-51).
- **does not guarantee**: Untrimmed absolute value; EXC without trim in realistic area (2.9 mm² per device for ±0.01% untrimmed) (H08-03).
- **pdk data needed**: TFR layer; R_s; TC; k_A; laser-link layer and clearance; routing blockage above (H06-14, H06-29).
- **verification**: No metal of the blocked level above a single-mask TFR (H06-14); 10–15 µm laser clearance (H06-29).
- **prefer over / degrade to**: Top of the resistor ladder for EXC; degrade to poly plus a trim network (RES-15) when no TFR exists (H06-20, H06-27).
- **source**: H06-14, H06-20, H06-27, H06-29, H08-03, H08-51, H08-56, H06 §6.5.10.

#### RES-11 Voltage-aware segment spacing and HV segmentation over biased wells
- **id**: RES-11
- **technique**: Set the gap between adjacent segments from the voltage between them; for HV poly, split the resistor over separately biased wells.
- **realizes**: Interlevel-oxide TDDB reliability at high voltage; STI cone-defect immunity (H06-21, H06-22).
- **applicability**: Poly resistors on decks with voltage-dependent poly spacing; HV poly over STI (cone defects break down below 20 V) (H06-21).
- **parameters**: N; gap per adjacent column pair; well count and bias.
- **guarantees**: Within one string of N equal segments the adjacent-turn voltage is ΔV = 2V_d/N (Eq 6.24; 6 V over 6 segments → 2 V), so gap = space(2V_d/N); between interdigitated members gap = space(|V_A − V_B|max) (H06-22).
- **does not guarantee**: Area efficiency at high voltage; cone-defect immunity without screening (H06-21).
- **pdk data needed**: Poly space versus voltage table; STI cone threshold voltage; well rules (H06-21, H06-22).
- **verification**: Gap ≥ space(ΔV) for every adjacent pair, from op-point node voltages (H06-22).
- **prefer over / degrade to**: Per-pair voltage spacing over a global worst-case gap (wastes area) or none (reliability risk) (H06-22).
- **source**: H06-21, H06-22.

#### RES-12 Extended low-sheet heads as lead-crossing corridors
- **id**: RES-12
- **technique**: Stretch the low-sheet head (base head of an HSR, silicided head of poly) so crossing leads pass over the head, never the body.
- **realizes**: Routing across a resistor without conductivity modulation (0.1%/V from a lead over 2 kΩ/□ HSR) (H06-14).
- **applicability**: HSR and high-sheet poly; at any class a connected lead may cross beside the head it connects to (H08-28).
- **parameters**: Head extension length; head sheet.
- **guarantees**: Head resistance R_h = k·R_sb·(S₂ + W_c/2)/(W_h + W_hb), with k ≈ 0.7 and → 1 for elongated heads (Eq 6.21) (H06-14, H06 §6.5.4).
- **does not guarantee**: Capacitive-coupling immunity (≈ 0.05 fF per 1 µm × 1 µm crossing; 1 mVpp on a 0.1 pF node from a 2 Vpp lead) (H06-18).
- **pdk data needed**: Head sheet and head bias (H06-01).
- **verification**: Foreign nets over bodies are keep-outs; over heads they are zero-penalty corridors (H06-14).
- **prefer over / degrade to**: Reroute first; then head corridor; then a shield plate (RES-13) (H06-14, H08-28).
- **source**: H06-14, H06-18, H08-28, H06 §6.5.4.

#### RES-13 Metal plate over a resistor array (Faraday shield, hydrogen plate, per-segment shields)
- **id**: RES-13
- **technique**: A metal layer between the bodies and overlying routing, tied to the circuit reference (or per segment to that segment), 2–3 µm overhang; a met1 plate that exposes only the heads lets met2 jumpers fold in.
- **realizes**: Lead-over immunity at DC and low frequency (H08-29); uniform hydrogenation across the array (metallized vs unmetallized poly mismatch up to 10%, with gradients microns from metal edges) (H08-27); permitted leads over MOD resistors when unavoidable (H08-28).
- **applicability**: Poly resistor arrays, MOD and EXC. EXC: met2 plate ≥ 10 µm beyond the active region in every direction, no met1 across the active area beneath it (H08-27). A common plate is fine only if poly ≤ ≈ 500 Ω/□ and ≤ a few volts span the array; otherwise per-segment plates tied to their segments with 2–3 µm overlap, or reroute (H08-29).
- **parameters**: Plate layer (met1 is the better hydrogen barrier, met2 less effective) (H08-27); net (no voltage relative to the resistors) (H08-27); overhang; segmentation; exposed heads.
- **guarantees**: Blocks DC and attenuates LF; static logic may cross; switching digital never, even shielded (H08-28, H08-29).
- **does not guarantee**: Protection from digital edges (H08-29); a common plate above 500 Ω/□ modulates the segments it spans (H08-29).
- **pdk data needed**: Metal stack; hydrogen sensitivity per recipe (CVD nitride carries 4–40% H; HSR poly sheet drops up to 30%) (H08-27).
- **verification**: Overhang ≥ 2–3 µm (≥ 10 µm EXC); no digital over the plate; per-segment plates when R_s > 500 Ω/□ or span > 3 V (H08-27, H08-29).
- **prefer over / degrade to**: Without a plate: no leads over or between matched segments and dummy fill blocked on all layers (H08-27); folded-out jumpers (MAT-10) (H08-27).
- **source**: H08-27, H08-28, H08-29.

#### RES-14 Tweakable-resistor provisions (sliding contact, sliding head, trombone, metal options)
- **id**: RES-14
- **technique**: Reserve geometry so one mask change retargets the value: body extended with the contact mid-travel under an elongated plate; sliding heads; sliding serpentine turns; spare contacted segments combined through the metal mask.
- **realizes**: Single-mask value adjustment for all subsequent units (H06-26).
- **applicability**: Sliding contact only where head material equals body material and body ≤ ≈ 200 Ω/□; sliding head for HSR, high-sheet poly and silicided heads; trombone requires every enclosing well, implant and silicide block to cover the slide region; metal options for any (H06-26).
- **parameters**: Tweak range %; slide travel; spare segment count and values.
- **guarantees**: The reserved travel covers the requested range (H06-26).
- **does not guarantee**: Per-unit accuracy, which needs trimming (RES-15) (H06 §6.6).
- **pdk data needed**: Enclosure rules of the enclosing layers (H06-26).
- **verification**: Enclosing layers cover the full slide region; spare segments contacted under metal (H06-26).
- **prefer over / degrade to**: Tweak when the whole lot shifts; trim (RES-15) for per-unit correction (H06 §6.6).
- **source**: H06-26, H06 §6.6.

#### RES-15 Discrete trim network from unit segments (fuses, Zener zaps, NVM, remote switches)
- **id**: RES-15
- **technique**: Main resistor plus weighted segments built from the set's unit segment, each shorted by a fuse, zap or MOS switch; the ladder shares the main resistor's common centroid.
- **realizes**: Per-unit absolute accuracy to ≈ ±0.05% (thin film ±0.01%) (H06-27); voltage trim (series weights) or current trim (parallel weights) (H06-27; Fig 6.27: series 4R_lsb, 2R_lsb, R_lsb; parallel R_msb, R_msb/2, R_msb/4, ref-99 §4).
- **applicability**: Bits N = ceil[3.32·log₁₀(X/δX)] (±0.25% over ±5% → 4 bits; 100 kΩ = 95 kΩ + 667 Ω·(1+2+4+8)); above ≈ 6 bits matching errors exceed 1 LSB, so add a secondary network or sub-2:1 steps; nonbinary 1:2:4:6:12:24:36:72:144 stays binary-searchable if each weight ≤ 2× the previous (H06-27).
- **parameters**: Bits and weights; differential LSB ΔR = R_B²/(R_A + R_B) (Eq 6.26) for tiny steps (H06-27); fuse kind; fuse on the least-vulnerable (grounded) end (H06-28); remote switch widths inversely proportional to segment R (4W/L, 2W/L, W/L for R_lsb, 2R_lsb, 4R_lsb) (H06-28); NVM MSB buffer non-inverting so the untrimmed value sits mid-range (H06-28).
- **guarantees**: Weights inherit the set's matching when built from identical units (H06-27); fuse leads ≥ 5× the link width, fuses next to their trimpads, trimpads shared via series/parallel fuse arrangements (H06-28; Fig 6.28, ref-99 §4); poly fuse programming 5–15 V, 1 ms, 50–150 mA, < 25 ns rise; metal fuse 5 V, 1 ms, several hundred mA (H06-28, H06 §6.6.2); Zener zap ≈ 10 Ω after 100–250 mA, no overcoat opening so post-package trim is possible (H06 §6.6.2).
- **does not guarantee**: Accuracy below ±0.05% (H06-27); TC stability if continuous electrical poly trim is used instead (lowers R up to 50% but shifts TC, unsuitable for ratios) (H06 §6.6.2).
- **pdk data needed**: Fuse layer and programming spec; trimpad rules; NVM availability; switch R_on model (H06-28).
- **verification**: Fuse-lead width ≥ 5·W_link; fuse–pad proximity; R_on,k/R_k ≤ ε; trim table covers ±X at δX (H06-27, H06-28).
- **prefer over / degrade to**: Remote MOS switches beside the resistor with minimum-width control lines, over long leads from edge fuses to central resistors (H06-56); NVM when compact and low-current (H06 §6.6.2); laser (RES-16) when a TFR exists.
- **source**: H06-27, H06-28, H06-56, H06 §6.6.2, ref-99 §4 (Figs 6.27, 6.28).

#### RES-16 Laser trim structures (notched bar, tophat, loop, ladder, ablated links)
- **id**: RES-16
- **technique**: Thin-film bar cut laterally to ≈ 90% of target then longitudinally for fine resolution; or discrete links ≈ 1 µm × 15 µm ablated.
- **realizes**: Continuous or discrete trim to ±0.01% on thin film (H06-27, H06-29).
- **applicability**: Thin film (and poly) at probe; links need 10–15 µm clearance to other circuitry; cuts run at 10–30 mm/s, which is costly (H06-29).
- **parameters**: Trim geometry; link count; clearance.
- **guarantees**: Discrete loop or ladder trims avoid the TC shift of continuous trimming (H06-29).
- **does not guarantee**: Low test cost (H06-29); link reliability outside the width window (too narrow will not rupture the overcoat, too wide splatters) (H06-29).
- **pdk data needed**: Laser-link layer; clearance; overcoat-opening rules (H06-29).
- **verification**: 10–15 µm keep-out around links (H06-29).
- **prefer over / degrade to**: Use with RES-10 for EXC; otherwise RES-15 (H06-20, H06-27).
- **source**: H06-27, H06-29.

#### RES-17 Current-density and power sizing (velocity saturation, self-heating, power density)
- **id**: RES-17
- **technique**: Size L from the voltage across the resistor and W from the allowed self-heating; cap the power density of matched arrays.
- **realizes**: Linearity under bias; matched self-heating; gradient-free matched arrays (H06-12, H06-13, H08-55).
- **applicability**: Every resistor with an op-point V and I; EXC arrays ≤ 1–2 µW/µm², MOD substantially more (H08-55); diffused bodies for high power (H06-17).
- **parameters**: L_total; W; N; recipe.
- **guarantees**: L_min = 10 µm/V · V_max for N-type silicon and 3.3 µm/V · V_max for P-type (critical fields 0.2 and 0.6 V/µm, safety factor 2; Eqs 6.17–6.18) (H06-13); self-heating β₂ ≅ α·t_ox·R_s/(κ·R₀²·W²) with κ(SiO₂) ≈ 0.013 W/cm/°C (Eq 6.16; the book's −1.5%/V² example disagrees with the equation by ×100, validate units) (H06-12).
- **does not guarantee**: Recovery after poly overheating (permanent R and TC shift) (H06-12); the array temperature rise without the ch. 5 thermal model (H08-55).
- **pdk data needed**: TCR α; oxide thickness; κ; power-density limit per class; critical fields (H06-12, H06-13, H08-55).
- **verification**: |β₂,A·V_A² − β₂,B·V_B²| ≤ budget; P/A_body ≤ class limit; L ≥ k·V_max (H06-12, H06-13, H08-55).
- **prefer over / degrade to**: Widen W first (β₂ ∝ 1/W² at fixed R and R_s); change to a diffused body for power (H06-12, H06-20).
- **source**: H06-12, H06-13, H06-17, H06-20, H08-55.

#### RES-18 TC-compensated composite resistor
- **id**: RES-18
- **technique**: Series or parallel combination of positive-TC low-sheet poly and negative-TC high-sheet poly, kept under one isotherm as a single common-centroid macro.
- **realizes**: Near-zero TC; series-parallel networks insensitive to high-sheet TC variation (Gregoire–Moon) (H06-10).
- **applicability**: Two poly recipes with opposite TC: 70 Ω/□ +500, 500 Ω/□ −1000, 10 kΩ/□ −7000 ppm/°C; 0TC occurs near 200–300 Ω/□ ±100 ppm/°C (H06-09).
- **parameters**: Section values and topology; co-location.
- **guarantees**: Both sections share the thermal field when not segmented apart (H06-10).
- **does not guarantee**: TC accuracy beyond the fab's high-sheet TC control, which is poorly monitored (H06-10).
- **pdk data needed**: tc1, tc2 and fit range per recipe (H06-09).
- **verification**: Sections placed as one centroid group; worst ΔR/R over the temperature range (H06-09, H06-10).
- **prefer over / degrade to**: A single 0TC recipe when the deck has one (H06-09).
- **source**: H06-09, H06-10.

#### RES-19 Resistor material selection ladder
- **id**: RES-19
- **technique**: Choose the recipe from the resistor's role by area (R/R_s·W²), tolerance, TC, VCR, power and drift.
- **realizes**: The tolerance, TC, VCR, power and drift class a role needs (H06-20).
- **applicability**: Resistors whose model is generic or substitutable. Ladder for matching: thin film (TC < 150 ppm/°C, k_A < 0.5%·µm, no modulation) > 500 Ω/□ N-poly (TC ≈ 1500 ppm/°C, k_A ≈ 2%·µm, adequate for MOD) > 2 kΩ/□ HSR (TC ≈ 3000 ppm/°C, up to 2%/V, struggles to reach MOD) (H08-56); diffused for power/ESD; well only without high-sheet poly; pinch only non-critical (H06-20).
- **parameters**: Recipe.
- **guarantees**: One material for all members of a set at every class; different materials mismatch ±20% or more, still ±5% over temperature after single-point trim (H08-56).
- **does not guarantee**: Absolute tolerance better than Table 6.7 for the chosen recipe (base 150 Ω/□ ±20%, HSR 2000 ±30%, poly 500 ±30%, N-well 2000 ±40%, metal 0.05 ±20%) (H06-20).
- **pdk data needed**: R_s, ±%, V_max per recipe; TC; VCR; drift class (high-sheet P-type poly drifts up several % over 10²–10³ h; N-type with P > B drifts less; compensated B > P drifts more) (H06-16, H06-20).
- **verification**: Matched-set key includes the model; warn when a high-drift recipe sets a bias current (H06-16, H08-56).
- **prefer over / degrade to**: As the ladder above.
- **source**: H06-16, H06-20, H08-56.

### Capacitor construction

#### CAP-01 Unit-capacitor array with preserved area/perimeter ratio
- **id**: CAP-01
- **technique**: Build each capacitor as n identical unit plates in parallel, with fringe modeled as c_a·A + c_p·P.
- **realizes**: Ratio accuracy against edge bias (10×10 vs 10×20 µm with −0.1 µm bias gives 0.497, a 0.6% error; unit arrays give exact integer ratios such as 3 pF and 5 pF from 1 pF units) (H08-11).
- **applicability**: Any plate capacitor type; integer ratios; customary for MOD, required for EXC (H08-11). Units: squares for EXC, 2:1 or 3:1 rectangles acceptable for MOD, no odd shapes or extra corners (OPC corners not repeatable); optimum unit 20×20 to 50×50 µm; MIN gains nothing from division (H08-57).
- **parameters**: Unit side s (from MAT-01); grid r×c (compact rows and columns, e.g. 32 as 4×8, or 5×7 with three unused); equal row pitch and equal column pitch, which may differ from each other (H08-57).
- **guarantees**: Equal A/P across members (H08-11); fringe C_F ≈ (εP/π)[ln(2eP/t) + ½·ln(1 + 4t_e/t)] (Eq 7.6; a 5 µm square over 200 Å is 43.1 fF plate + 2.6 fF fringe), with t/2 for a finite plate over a larger one (H06-30).
- **does not guarantee**: Non-integer ratios (CAP-02); gradient immunity (MAT-04); series-built small capacitors (parasitics uncontrolled) (H08-05).
- **pdk data needed**: c_area and c_perim per recipe; dielectric thickness; electrode thickness; default unit side (H06-30, H06-42).
- **verification**: Every member's A/P equals the unit's within grid snap; unit counts match the ratio (H08-11).
- **prefer over / degrade to**: Prefer over merged single plates for every ratioed set; a merged plate only for MIN singletons (H08-57).
- **source**: H06-30, H06-42, H08-05, H08-11, H08-57.

#### CAP-02 Non-integer ratio residual unit (rectangle, notch or tang)
- **id**: CAP-02
- **technique**: One non-unit capacitor whose A/P equals the unit's: with C1 a square of side W1, L2 = (C2/C1)(1 + √(1 − C1/C2))·W1 and W2 = (C2/C1)(1 − √(1 − C1/C2))·W1 (Eqs 8.19/8.20); or keep W2 = W1 and add a notch or tang.
- **realizes**: Exact non-integer ratio with equal periphery sensitivity (H08-11).
- **applicability**: C2 ≥ C1; the rectangular residual does not tile the grid because W2 ≠ W1 (H08-11); Fig 8.2 shows areas 1 and 1.3 (ref-99 §4).
- **parameters**: Residual shape (rectangle or notch/tang); placement centred on the array axis.
- **guarantees**: Bias-independent ratio when A/P matches (H08-11).
- **does not guarantee**: OPC repeatability of the extra corners (H08-57).
- **pdk data needed**: c_a, c_p; grid snap (H06-30).
- **verification**: (c_a·A + c_p·P)/C_unit equals the fraction exactly after snapping (H06-30).
- **prefer over / degrade to**: Whole units first; then one residual unit; never series units unless antiparallel (CAP-08) (H08-05, H08-11).
- **source**: H06-30, H08-05, H08-11, H08-57, ref-99 §4 (Fig 8.2).

#### CAP-03 Metal–insulator–metal capacitor (TiN/ARC, RBM or silicided-poly lower plate)
- **id**: CAP-03
- **technique**: Dedicated thin dielectric between metallic or silicided electrodes, insertable between two metals, optionally placed over circuitry wired in lower layers.
- **realizes**: Lowest VCR and TC because metal electrodes do not deplete (H06-42); EXC matching (metallic or silicided electrodes are best) (H08-58); area reuse over circuitry (H06-45).
- **applicability**: Analog BiCMOS TiN 3.1 fF/µm² ±20% 7 V (Table 7.3) (H06 §7.1.3); one mask (ARC SiON + TiN, or silicided-poly lower plate) or two masks (RBM lower plate) (H06 §7.1.3).
- **parameters**: Plate size; metal pair; over-circuit flag; shield (CAP-10).
- **guarantees**: VC 2–4 ppm/V for silicide–oxide–metal; 600 Å nitride between silicide and TiN gives −13 ppm/V, −4 ppm/V², 39 ppm/°C (H06-42).
- **does not guarantee**: The density of poly-poly or gate oxide (1.3–4.3 fF/µm²) (H06 §7.1.3); freedom from coupling to nets beneath when placed over circuitry (H06-45); zero nitride VCR (residual from Si–H bonds) (H06 §7.1.3).
- **pdk data needed**: MIM recipe; c_area; V_max (t and E_max); TC; VC; over_circuit_allowed; via-ring style (H06-32, H06-42, H06-45).
- **verification**: |V_P − V_N| ≤ V_max = t_min·E_max (H06-32); TDDB FIT from area, field and duty within the ≈ 10 FIT budget (H06-33).
- **prefer over / degrade to**: Top of the capacitor ladder for matching; degrade to poly-poly (CAP-06), then MOS (CAP-07); junction never for matching (H08-58).
- **source**: H06-32, H06-33, H06-42, H06-45, H08-58, H06 §7.1.3.

#### CAP-04 Lateral-flux MOM (horizontal bars, vertical pillars, woven, fractal)
- **id**: CAP-04
- **technique**: Interdigitated same-layer fingers with alternating electrodes, stacked across metals so each strip is surrounded by the other electrode; or vertical pillars (lateral flux in 2-D only); woven adds vias at same-electrode crossings.
- **realizes**: Capacitance from lateral spacing without a MIM mask; low series inductance (woven) (H06-43).
- **applicability**: Competitive when lateral spacing < ILO thickness (5–10 kÅ); pillars win when spacing ≪ ILO; lateral flux 0.07 fF/µm² ±30% 40 V (Table 7.3) (H06-43, H06 §7.1.3).
- **parameters**: Finger width and space (minimum); layer set; style; electrode assignment.
- **guarantees**: Density computable from the stack as vertical plus sidewall terms (H06-43); symmetric metal electrodes, so no depletion and no bottom-plate asymmetry (H06-42).
- **does not guarantee**: Absolute tolerance (±30%) (H06 §7.1.3); fractal styles trade series R against density (H06-43).
- **pdk data needed**: Metal thickness and spacing per layer; ILO thickness; k per layer pair (H06-43).
- **verification**: Density within x% of the best style for the deck; ESR/ESL for RF nets (H06-43).
- **prefer over / degrade to**: Choose per deck by computed density; MIM (CAP-03) when available; stack (CAP-05) when lateral spacing > ILO (H06-43). Trench capacitors (≥ 3 orders above planar, up to 0.4 µF/mm²) are a process option, not a layout choice (H06 §7.1.3).
- **source**: H06-42, H06-43, H06 §7.1.3.

#### CAP-05 Stack (sandwich) capacitor with a shielded middle plate
- **id**: CAP-05
- **technique**: Poly–M1–M2 (or three consecutive metals) using the interlevel oxide; the two outer plates form one electrode, the sandwiched plate the other.
- **realizes**: ≈ 2× single-ILO density without extra masks; high voltage rating; an electrode with almost no parasitic (H06-43, H06-40).
- **applicability**: ILO 5–10 kÅ: ≈ 30,000 µm² per pF single-layer, half for the three-plate stack; 0.05 fF/µm² ±30% 40 V (Table 7.3) (H06-43, H06 §7.1.3); compensation and bypass (H06 §7.1.3).
- **parameters**: Layer set; via ring (staggered contacts/vias for Al, stacked for W plugs) (H06-45); electrode assignment.
- **guarantees**: The sandwiched M1 plate is shielded; the poly/M2 electrode couples to substrate and overhead leads (H06-40).
- **does not guarantee**: Density competitive with MIM or poly-poly (H06 §7.1.3).
- **pdk data needed**: ILO thickness; k; via-ring rules (H06-43, H06-45).
- **verification**: Sandwiched plate on the high-impedance net (CAP-09) (H06-40).
- **prefer over / degrade to**: Use when no MIM exists and spacing > ILO; otherwise CAP-04 (H06-43).
- **source**: H06-40, H06-43, H06-45, H06 §7.1.3.

#### CAP-06 Poly–poly capacitor
- **id**: CAP-06
- **technique**: Poly-1 lower plate, thin oxide or ONO dielectric, poly-2 upper plate carrying the heaviest-doped implant; no oxide step crosses the plates.
- **realizes**: 1.3 fF/µm² ±25% 15 V linear capacitor for MOD matching (H06 §7.1.3, H08-58).
- **applicability**: Poly-gate CMOS and analog flows; oxide steps must not intersect the capacitor; deep-N+ under LOCOS thickens the oxide (not with STI) (H06-45); prefer a single-layer dielectric where C must be constant over frequency (ONO absorbs at ≥ 10 MHz) (H06-44); low-k or large-area dielectrics improve small-cap matching (H06-44).
- **parameters**: Plate size; dielectric (oxide or ONO); top-plate implant (heaviest-doped lowers series R and depletion VCR) (H06-36); orientation (CAP-09).
- **guarantees**: VCR ≈ 150 ppm/V and TC < 250 ppm/°C unsilicided (H06-36, H06-42); poly depletion w_d = εV/(qN_P·t), c = εA/(t + w_d·ε/ε_Si) (Eqs 7.20–7.21; 5e19 As, 200 Å, 4.5 MV/cm → 19 Å, 1.73 → 1.67 fF/µm²) (H06-36).
- **does not guarantee**: Symmetric breakdown (ONO up to 50% lower with the lower plate negative) (H06-44); freedom from poly-2 stringers in older ONO flows (fixed by patterning poly-2 first) (H06-45); TC below 50–100 ppm/°C for EXC (H08-59).
- **pdk data needed**: c_area; dielectric kind; V_max per polarity; TC; VC; step rules (H06-42, H06-44).
- **verification**: No active or oxide edge under the plates (H06-45); lower-plate polarity against the reduced limit (H06-44).
- **prefer over / degrade to**: Below MIM (CAP-03), above MOS (CAP-07) (H08-58); forbid ONO on switched-capacitor and ADC paths when an alternative exists (H06-44).
- **source**: H06-36, H06-42, H06-44, H06-45, H08-58, H08-59, H06 §7.1.3.

#### CAP-07 MOS capacitor (accumulation or inversion) with gate/channel resistance balance
- **id**: CAP-07
- **technique**: Transistor-like gate over active, S/D on all four sides (inversion) or a backgate-contact ring (accumulation); type and polarity chosen from terminal connectivity; biased ≥ 0.5–1 V beyond V_t or V_FB.
- **realizes**: Highest density (GOX 0.86–4.3 fF/µm², Table 7.3) (H06 §7.1.3); MOD matching when biased several volts into the flat region (H08-58).
- **applicability**: Selection on a P-substrate: one terminal at substrate → NMOS inversion or PMOS accumulation; at the positive supply → PMOS inversion; both floating with the lower-voltage electrode sensitive → PMOS inversion, gate low, source high; higher-voltage electrode sensitive → PMOS accumulation or isolated NMOS inversion (H06-38). Never for EXC (H08-58). Thin emitter-oxide MOS caps: emitter plate to a low-impedance node (H06 §7.1.3).
- **parameters**: W, L, sections; contacted gate ends (distributed gate R lumps to 1/3 of end-to-end with one end, 1/4 of that with both) (H06-39); sparse gate-contact array at several × minimum pitch, PMoat ring on four sides, elongated shape, split when spindly (H06-39); S/D tied to backgate in inversion (H06-37).
- **guarantees**: C within 10% at 0.5 V overdrive; ≥ 1 V minimizes VCR (H06-37); channel R_ch ≈ L/(W·k′·(V_GS − V_t)) (Eq 7.29) (H06-39); N+ sinker MOS cap varies ≤ 5% over voltage (H06 §7.1.3).
- **does not guarantee**: C near threshold; inversion without S/D (takes up to hundreds of ms) (H06-37); electrode interchangeability (CAP-09) (H06-40); MIM-class TC/VCR (H06-42).
- **pdk data needed**: C–V table or V_FB and V_t; t_ox; gate R_s; k′; antenna ratio limit (H06-37, H06-39).
- **verification**: Overdrive ≥ 0.5 V hard and ≥ 1 V warning from the op-point (H06-37); ESR = R_g,eff + R_ch ≤ budget (H06-39).
- **prefer over / degrade to**: Below poly-poly for linearity, above junction (H08-58); antiparallel pair (CAP-08) when the bias polarity reverses (H06-38).
- **source**: H06-37, H06-38, H06-39, H06-40, H06-42, H08-58, H06 §7.1.3.

#### CAP-08 Antiparallel split of asymmetric-electrode capacitors
- **id**: CAP-08
- **technique**: Two equal halves in parallel with swapped plate assignment (top of one to bottom of the other), placed common-centroid.
- **realizes**: First-order VCR cancellation: a silicided-lower-plate poly-poly cap goes from ≈ 150 to ≈ 2 ppm/V (H06-36); polarity-reversing MOS caps as two PMOS inversion halves of C/2 with backgates to their sources, dips offset by 2V_t (H06-38); the only acceptable series form in matched arrays (H08-11).
- **applicability**: Poly-poly, MOS and any depleting-electrode capacitor on a linearity-critical net (H06-36).
- **parameters**: Split into 2 halves; placement.
- **guarantees**: Odd-order voltage terms cancel (H06-36).
- **does not guarantee**: Bottom-plate parasitic symmetry, since both nets now carry one high-parasitic plate (H06-40); even-order terms (H06-36).
- **pdk data needed**: VC1 and VC2 per recipe (H06-42).
- **verification**: Both halves identical units with swapped orientation and coincident centroid (H06-36, H08-35).
- **prefer over / degrade to**: A MIM (CAP-03) when available; this split for asymmetric types (H06-36).
- **source**: H06-36, H06-38, H06-40, H06-42, H08-11, H08-35.

#### CAP-09 Plate orientation by net impedance, with a well shield under the bottom plate
- **id**: CAP-09
- **technique**: Map the high-parasitic electrode (lower diffusion or poly plate, the outer plates of a stack) to the low-impedance net; where the bottom plate cannot be low-impedance, put an N-well (NBL plus deep-N+ in BiCMOS) tied to a quiet node under the capacitor.
- **realizes**: Minimum parasitic on the sensitive node; substrate-noise shielding; the schematic's "outside foil" (curved plate) intent (H06-40, H08-25).
- **applicability**: MOS, poly-poly, stack and MIM — electrodes are never interchangeable (H06-40); rule C6 for all matched capacitors, most important with switching-converter substrate noise (H08-25); the same grounded well or tank also shields deposited resistors (H08-25).
- **parameters**: Plate-to-net map; well shield on/off and its net.
- **guarantees**: The lower-plate-to-substrate C₂ lands on the low-impedance node; the sandwiched plate has virtually none (H06-40, H06-41).
- **does not guarantee**: Freedom from ONO asymmetric breakdown unless polarity is also checked (H06-44).
- **pdk data needed**: Bottom-plate area C to substrate; well availability; net roles (H06-41, H08-25).
- **verification**: Impedance ranking supply/ground < driven output < high-impedance node; the mapping is recorded for LVS and performance extraction (H06-40).
- **prefer over / degrade to**: Orientation first (free); well shield when both nets are high-impedance or substrate noise is severe (H08-25).
- **source**: H06-40, H06-41, H06-44, H08-25.

#### CAP-10 Electrostatic top shield over a matched capacitor array
- **id**: CAP-10
- **technique**: Metal plate one level above the top electrode (met2 over poly-poly), tied together with the poly-1 or well shield below to a low-noise, low-impedance node; the dummies join it.
- **realizes**: Contained fringing (smaller dummy ring), leads allowed over the array, outside fields blocked, lead C made predictable (H08-23, H08-14).
- **applicability**: All MOD and EXC capacitors (rule C8) (H08-23); never route clocks or digital lines over it; noisy lines ≥ 3–4 µm clear of the shielded area (H08-23); arrays whose capacitors do not all have one side on a low-impedance node need parasitic analysis (H08-14).
- **parameters**: Shield layer; overhang 3–5 µm beyond the array when there are no dummies (H08-23); net (analog ground).
- **guarantees**: Lead C rises but is predictable (H08-23).
- **does not guarantee**: Immunity to digital edges routed over it (H08-23, H08-29).
- **pdk data needed**: Metal stack; ILD thickness (H08-22).
- **verification**: Shield covers the array plus overhang; 3–4 µm digital keep-out; tie net is quiet and low-impedance (H08-23).
- **prefer over / degrade to**: Shield plus narrow dummies over no shield plus wide dummies (fringing reaches 10–30 µm unshielded) (H08-14).
- **source**: H08-14, H08-22, H08-23, H08-29.

#### CAP-11 Capacitor dummy ring by precision class
- **id**: CAP-11
- **technique**: Ring of dummy units on all four sides at the active pitch, both dummy electrodes tied to a node; with a common low-impedance bottom plate, extend it under the dummies and tie their tops to the same node.
- **realizes**: Identical etch, litho and fringe environment for edge units (H08-14).
- **applicability**: Every arrayed matched capacitor (rule C7). MOD under a shield: dummy width ≤ 3× minimum; no shield: much wider (fringing 10–30 µm); EXC: exact unit copies on all sides (H08-14). Fig 8.5: 6 active units inside 14 dummies in a 5×4 grid (ref-99 §2.8, §4).
- **parameters**: Dummy width and count; tie node (shield or low-impedance bottom plate; never a noisy node or one its capacitance would disturb) (H08-15).
- **guarantees**: Dummy rows and columns need only span the etch penetration (≤ 10 µm if that is the measured depth) (H08-14).
- **does not guarantee**: Gradient cancellation (MAT-04).
- **pdk data needed**: Etch penetration depth; litho w_min (H08-12, H08-14).
- **verification**: Every edge unit has a same-pitch neighbour on all four sides; dummy plates connected (H08-14, H08-15).
- **prefer over / degrade to**: Exact copies for EXC; narrow dummies under a shield for MOD (H08-14).
- **source**: H08-12, H08-14, H08-15, ref-99 §2.8, §4 (Fig 8.5).

#### CAP-12 Capacitor type and dielectric selection ladder
- **id**: CAP-12
- **technique**: Pick the capacitor type from class, voltage, frequency behaviour and density.
- **realizes**: The linearity, TC and dielectric-absorption behaviour a role needs (H08-58, H06-44).
- **applicability**: Matching ladder: thick homogeneous grown or LPCVD oxide between metallic electrodes (optimal) > MIM > poly-poly (top-plate depletion) > MOS biased several volts into accumulation/inversion (moderate) > junction (unsuitable) (H08-58). Density ladder (Table 7.3, fF/µm²): thin GOX 4.3 > TiN 3.1 > thick GOX 2.2 > poly-poly 1.3 > GOX 0.86 > B-E junction 0.8 > lateral flux 0.07 ≈ emitter oxide 0.07 > stack 0.05 (H06 §7.1.3).
- **parameters**: Type; dielectric; V_max.
- **guarantees**: Oxide capacitors in plastic reach ±0.01% and perhaps ±0.001%, enough for 14–17-bit converters (H08 §8.3.2); TCs: thin oxide +20 ppm/°C, nitride +12, poly-poly < 50 (or < 250, the book gives both), junction +500…+700 (H06-42).
- **does not guarantee**: Absolute value (±10% thin film to ±30% junction); capacitors are hard to trim, so designs rely on ratios or trim resistors and currents (H06-46).
- **pdk data needed**: dielectric_kind, electrode_kind, tc, vc, V_max, density and tolerance per recipe (H06-42, H08-58).
- **verification**: Warn when an EXC set uses a composite dielectric, poly electrodes or a MOS cap; forbid ONO/TEOS (relax in ≈ µs) on charge-redistribution paths (H08-58, H06-44).
- **prefer over / degrade to**: As the ladders above.
- **source**: H06-42, H06-44, H06-46, H08-58, H06 §7.1.3, H08 §8.3.2.

#### CAP-13 Junction capacitor (comb or plate), reference only
- **id**: CAP-13
- **technique**: Reverse-biased B-E or S/D junction; comb when finger spacing S_f < 2c_p/c_a (Eq 7.18), else plate; spine with short fingers as compromise.
- **realizes**: Varactor or coarse bypass where nothing better exists (H06-34, H06-35).
- **applicability**: 0.8 fF/µm² at zero bias (75% at −1 V, 50% at −5 V), ±30%, +500…+700 ppm/°C; operate ≤ 75% of BV (6.8 V → 5 V); forward bias ≤ 400 mV at room temperature, −2 mV/°C, 200 mV at 125 °C (H06-35); unsuitable for matching (H08-58); peak near 0.7 V forward not usable (Fig 7.7: peak ≈ 0.7 V, avalanche ≈ −7 V, ref-99 §4).
- **parameters**: Comb vs plate; finger spacing; tank contact (required; NBL useless) (H06 §7.1.3).
- **guarantees**: c_j0 = c_a·A_d + c_p·P_d (40 V bipolar B-E: 0.82 fF/µm², 2.8 fF/µm) (H06-34).
- **does not guarantee**: Linearity (c_j = c_j0(1 − V/φ_B)^−m, m 0.33–0.5) (H06-34); low leakage at high temperature; latch-up immunity if forward-biased without a guard ring (H06-41).
- **pdk data needed**: c_a, c_p, BV, m (H06-34).
- **verification**: ERC V_R ≤ 0.75·BV and V_F ≤ V_F,max(T_max) (H06-35).
- **prefer over / degrade to**: Bottom of the ladder (H08-58).
- **source**: H06-34, H06-35, H06-41, H08-58, H06 §7.1.3, ref-99 §4 (Fig 7.7).

### Inductors (brief)

#### IND-01 Planar spiral synthesis and layer assignment
- **id**: IND-01
- **technique**: Octagonal (or square) spiral on the highest metal, strapping the top 2–3 metals when there is no thick top metal, underpass beneath, geometry searched for L_target at f_op.
- **realizes**: Target inductance of tens of nH at maximal Q (H06-47, H06-48).
- **applicability**: RF only (useless below 100 MHz); best parts 10–100 nH at Q ≈ 40 up to a few GHz (H06 §7.2); standard CMOS on low-ρ substrates rarely exceeds Q 5–10 (H06-53).
- **parameters**: Shape (Mohan K₁/K₂: square 2.34/2.75, octagon 2.25/3.55); N; width 10–15 µm at 1–3 GHz; spacing = minimum; d_o; d_i ≥ max(5w, d_o/3); ≤ 1–2 winding layers (strapped count as one); symmetric centre-tapped variant with crossover jumpers for differential circuits (H06-47, H06-50).
- **guarantees**: L = µ·K₁N²(d_o + d_i)/{2[1 + K₂(d_o − d_i)/(d_o + d_i)]} (Eq 7.34; the book's example gives 18.6 nH where the equation gives ≈ 24.8 nH, so validate) (H06-47); f_crit = p·R_s/(2µw²) (2.4 GHz example reproduces) and R_W = R_DC[1 + (f/f_crit)²/10] (H06-48); no metal-1 in the body, underpass never above (H06-49).
- **does not guarantee**: Q on substrates < 10 Ω·cm (H06-53); SRF > f_op without EM verification (H06-55).
- **pdk data needed**: Metal R_s and thickness per layer; substrate ρ; dedicated inductor metal (2–5 µm) if any (H06-48, H06-49, H06-53).
- **verification**: Export to a 3D/2.5D EM solver; SRF ≥ k·f_op; f_crit ≥ f_op; Q = 2πfL/R_s (H06-48, H06-55).
- **prefer over / degrade to**: Octagon over square (circular has least R, square most); thick dedicated metal over strapping (H06-49, H06-50).
- **source**: H06-47, H06-48, H06-49, H06-50, H06-53, H06-55, H06 §7.2.

#### IND-02 Inductor keep-out zone
- **id**: IND-02
- **technique**: Exclusion polygon = spiral bounding box grown by ½ the inductor width, plus the hollow centre, on all layers, for foreign metal, devices, junctions and dummy fill.
- **realizes**: Q preservation; no rectification or carrier injection under the coil (H06-51).
- **applicability**: Every inductor; guard rings and scattered substrate contacts on high-ρ substrates (H06-53).
- **parameters**: Margin ≥ ½ width (more if possible); fill policy (remove to ≈ ½ width or size to minimize eddy loss) (H06-51).
- **guarantees**: Nothing above or below the spiral except slotted shields (IND-03) (H06-51).
- **does not guarantee**: Substrate eddy loss on low-ρ substrates (H06-53).
- **pdk data needed**: Substrate ρ (prefer > 100 Ω·cm); fill rules (H06-53).
- **verification**: No foreign shape inside the polygon on any layer (H06-51).
- **prefer over / degrade to**: Hard constraint; no cheaper alternative (H06-51).
- **source**: H06-51, H06-53.

#### IND-03 Patterned (slotted) ground shield
- **id**: IND-03
- **technique**: Silicided-poly shield under the spiral, slotted radially so each strip's f_crit > f_op, tied at the perimeter to ground.
- **realizes**: Higher Q by replacing the lossy substrate path with a low-R one (H06-52).
- **applicability**: Standard processes without thick elevated metal or a MEMS cavity; the one exception to "nothing under the coil" (H06-52).
- **parameters**: Shield layer; strip width from Eq 7.38 with the shield's R_s; slot pattern (four quadrant combs) (H06-52).
- **guarantees**: Silicided poly gives the best combination of low shield R and low winding-to-shield C (H06-52).
- **does not guarantee**: Compensation for a low-ρ substrate (H06-53).
- **pdk data needed**: Shield layer R_s (H06-52).
- **verification**: Strip f_crit ≥ f_op (H06-48, H06-52).
- **prefer over / degrade to**: Elevated thick metal or a high-ρ substrate first; PGS second (H06-53).
- **source**: H06-48, H06-52, H06-53.

### Matching and array techniques (resistors and capacitors)

#### MAT-01 Precision class and random-mismatch area budget (unit sizing)
- **id**: MAT-01
- **technique**: Assign MIN/MOD/EXC to each matched set, split the mismatch budget into a random share and a systematic residue, and size unit area or width from s_δ = k_A/√A (Pelgrom) or s_δ = (k_R/W)·√(1/R).
- **realizes**: A six-sigma mismatch target per set (H08-01, H08-02).
- **applicability**: All matched resistors and capacitors; random share ≤ 75% MIN, ≤ 50% MOD, ≤ 25% EXC (H08-03); EXC resistors generally need thin film plus laser trim and are rarely compatible with significant power devices (H08 §8.3.1).
- **parameters**: Class; target %; random share f; unit area A or width W; unit count.
- **guarantees**: A ≥ (6·k_A/(f·ε))² per device; with k_A = 2%·µm (500 Ω/□ poly or a 10 V poly-poly cap): 500 µm² for ±1%, 0.12 mm² for ±0.1%, 46 mm² for ±0.01%; thin film (k_A < 0.5%·µm) 2.9 mm² for ±0.01% (H08-03). Resistors: s_δ ∝ 1/W and 1/√R, so low-value matched resistors need wider segments (H08-06). Capacitors: s_δ = k_C·√((C1+C2)/(2C1C2)), the smaller capacitor dominates and halving mismatch costs 4× capacitance (H08-05).
- **does not guarantee**: The systematic terms (gradient, leads, heads), budgeted separately as (1−f)·ε (H08-03); EXC without trim in realistic area (H08-03).
- **pdk data needed**: k_A and k_P (or k_R, k_C) per model with provenance; for poly, estimate k_R = d_g·√(η·R_s), η ≈ 2, valid to widths somewhat below 0.3 µm (H08-04, H08-08).
- **verification**: Monte Carlo δ = X1·x2/(X2·x1) − 1 per set; m_δ (systematic) and s_δ (random, N−1); UDL/LDL = m_δ ± 6s_δ within the class band; ≥ 30 samples across wafers and die positions (H08-01, H08-62).
- **prefer over / degrade to**: Compute the area; when it exceeds the cap emit "needs trim" (RES-15) instead of under-sizing (H08-03).
- **source**: H08-01, H08-02, H08-03, H08-04, H08-05, H08-06, H08-08, H08-62, H08 §8.3.1.

#### MAT-02 Identical-unit unitization for ratios (equal width, equal length, series/parallel composition)
- **id**: MAT-02
- **technique**: Every member is n_i identical units: the larger resistor as units in series, the smaller as units in parallel or series-parallel; capacitors as n_i identical plates.
- **realizes**: Immunity to width bias (2 vs 4 µm poly with −0.1 µm bias → 0.487, 2.6% error; 2×2 µm parallel → 0.500) (H08-09); to length and contact bias (20 vs 40 µm with −0.2 µm → 0.503; 2×20 µm series → 0.500) and to head/contact R (2 kΩ and 4 kΩ single strips with 50 Ω contacts mismatch 2.4%) (H08-10); lower mismatch for the small member of a large ratio (100k:10k with the 10k as 2×20k parallel roughly halves it; 9:1 as a 2-parallel × 2-series small resistor has 4× area, half the mismatch and interdigitates as 9:4 AABAABABAABAA) (H08-07).
- **applicability**: Any ratioed R or C set at every class (rules R2, R5; capacitor units customary MOD, required EXC) (H08-09, H08-11); same material required (H08-56).
- **parameters**: Unit (W, L_seg, heads); n_i; series-parallel (s, p) per member; unit value.
- **guarantees**: Ratio expressed in integer unit counts; the centroid uses sensitivity weights ∂R/∂R_i (1 for a series segment, (R/R_i)² for a parallel branch) (H08-33); equal segments also cancel end current crowding (H08-10).
- **does not guarantee**: Non-integer ratios (MAT-06); correction of width mismatch by trimming (edge material differs in doping, TC and modulation) (H08-09).
- **pdk data needed**: As RES-01 and CAP-01.
- **verification**: One drawn width per set; all segment lengths equal (or MAT-06 sensitivity passed); the model key is identical across members (H08-09, H08-10, H08-56).
- **prefer over / degrade to**: Whole-unit integer ratios first; then MAT-06; unequal widths forbidden at every class (H08-09).
- **source**: H06-06, H08-07, H08-09, H08-10, H08-11, H08-33, H08-56.

#### MAT-03 One-dimensional interdigitation patterns (ratio library and arbitrary scatter)
- **id**: MAT-03
- **technique**: Order unit segments along one axis from a pattern table; for arbitrary M:N scatter each device uniformly across M+N positions and mirror-balance.
- **realizes**: Centroid coincidence or minimal offset in 1-D, cancelling linear gradients (H08-39).
- **applicability**: Any array of units. Optimum ratio patterns (Table 8.5): 1:1 ABBA; 2:1 ABA; 3:1 AABAABAA; 3:2 ABABA; 4:1 AABAA; 4:3 ABABABA; 5:1 AAABAAAABAAA; 5:2 AABABAA; 5:3 ABAABABAABABAABA; 5:4 ABABABABA; 9:5 ABAABAABABAABA (H08-39). Fig 8.8's active order is ABAABA with two end dummies; Fig 8.12 is ABBA with two end dummies (ref-99 §2.8).
- **parameters**: Pattern string; end dummies; optional single central segment bisected by the axis (H08-34).
- **guarantees**: Preference ABBA > ABAB > AABB (ABAB leaves centroids one pitch apart and still needs dummies) (H08-35, H08 §8.2.7). Pair construction: each mirrored pair belongs to one device, has the same effect on its value (series vs parallel differ), and sits at equal distance from the axis; examples SPPS/PSSP, QRRRQ/RQRQR, SPRQRQRPS (H08-34).
- **does not guarantee**: Quadratic residue cancellation (MAT-05); thermoelectric cancellation (MAT-11).
- **pdk data needed**: None beyond unit rules.
- **verification**: Golden tests against Tables 8.2, 8.3, 8.5; coincidence |c_A − c_B|/extent ≤ tolerance (H08-35, H08-39).
- **prefer over / degrade to**: Exact CC pattern when each (device, role) count is even except at most one; else minimum-|Δcentroid| scatter (H08-34).
- **source**: H08-34, H08-35, H08-39, ref-99 §2.8 (Figs 8.8, 8.12), H08 §8.2.7.

#### MAT-04 Common-centroid array (four rules, 2-D cross-coupling)
- **id**: MAT-04
- **technique**: Arrange units so every member's electrically weighted centroid coincides, symmetric about both axes; in 2-D use cross-coupled quads (AB/BA, ABBA/BAAB, ABA/BAB, ABCCBA/CBAABC, AAB/BAA).
- **realizes**: Cancellation of linear gradients: ΔR = α·R·d_CC·∇T_CC for temperature (H08-32); oxide and poly thickness gradients (H08 §8.2.7); stress for capacitors (H08-53).
- **applicability**: Any matched set of ≥ 2 members. Cross-couple even two equal capacitors (2 sections each) and use more sections for large ones (rule C9) (H08-57). Required for EXC and for MOD when any R7 trigger holds: die > 15 mm², power > 250 mW, heat-sunk package, solder/sinter/eutectic mount, within 250 µm of the die edge or of a major heat source (H08-47, H08-35). Capacitors always CC regardless of stress (H08-53).
- **parameters**: Pattern (MAT-03 or 2-D); sections per device; bank grid (MAT-05).
- **guarantees**: Table 8.4: coincidence (paramount), symmetry about both axes, dispersion, compactness (H08-35); centroid X_R = Σ(∂R/∂R_i·X_i)/Σ(∂R/∂R_i) for arbitrary networks (H08-33); the device temperature equals T at its centroid under planar, uniform, linear-in-T and linear-in-position assumptions (H08-32).
- **does not guarantee**: Thermoelectric EMF (MAT-11) (H08-45); nonlinear residue (MAT-05); any gain when gradients are negligible, where ABBA is as good as CC (H08-38). Asymmetric CC arrays are discouraged (H08-35).
- **pdk data needed**: Thermal and stress fields for scoring only.
- **verification**: |c_A − c_B|/extent ≤ class tolerance; mirror symmetry about both axes; number of maximal CC sub-arrays; bbox aspect and fill (H08-35).
- **prefer over / degrade to**: 2-D CC suffices in practice; higher-order arrays only when the residue dominates (H08-40).
- **source**: H08-32, H08-33, H08-35, H08-38, H08-40, H08-45, H08-47, H08-53, H08-57, H08 §8.2.7.

#### MAT-05 Dispersion into CC sub-arrays and quadratic-residue control (nonlinear gradients)
- **id**: MAT-05
- **technique**: Split each member into many small CC sub-arrays, each obeying coincidence and symmetry, arranged in a mirror-symmetric 2-D bank grid; score by Q_i = Σw·|r − r_c|²/Σw per member.
- **realizes**: Reduction of the c₂(x−a)² residue that CC leaves (ABBAABBA has half the residue of AABBBBAA; a 4×4 ABAB/BABA with 9 cross-coupled pairs beats ABBA/BAAB with 4) (H08-36); priority sub-grouping for ≥ 3 devices (ABBACCABBA best for A–B, BBACAACABB for A–C) (H08-37).
- **applicability**: Large EXC arrays (dispersion vital) (H08-35); any array under large thermal or stress gradients (H08-38); banks until the segment-length limit or until a sub-array's aspect reaches ≈ 1:1, then arranged in 2-D (H08-42).
- **parameters**: Sections per device; bank count; bank grid; importance weight per pair.
- **guarantees**: Residue ∝ span² per sub-array; sub-arrays that each obey the rules release the whole array from them (H08-36, H08-35).
- **does not guarantee**: A free lunch: gaps waste area and raise the random term, so an optimum exists that the book calls rarely computable by hand (H08-38); full quadratic cancellation, which needs a 4-segment rotational array (cancels linear and most quadratic) or an octagonal array (both, non-Manhattan, costly) (H08-40).
- **pdk data needed**: Quadratic gradient coefficient if known; thermal field; stress model (H08-36).
- **verification**: max_{i,j}|Q_i − Q_j| × c₂; combined σ² = σ_rand²(A_active(n)) + (gradient·residue(n))² minimized over the section count (H08-36, H08-38).
- **prefer over / degrade to**: Plain ABBA when gradients are negligible; dispersion when they are not; the Manhattan 4-segment variant only when residue dominates and area allows (H08-38, H08-40).
- **source**: H08-35, H08-36, H08-37, H08-38, H08-40, H08-42.

#### MAT-06 Non-integer resistor ratios by per-resistor segment length or partial segments
- **id**: MAT-06
- **technique**: (a) Give each resistor its own segment length, lengthening both ends equally so the axis holds, and choose N to minimize S = |N/R_N − M/R_M| with M = round(N·R_M/R_N) (Eqs 8.27–8.28); or (b) all segments R₀ plus one partial segment per resistor, minimizing S = |(N+1)/(N+k) − (M+1)/(M+j)| over R₀ with M = trunc(R_M/R₀), N = trunc(R_N/R₀), j = R_M/R₀ − M, k = R_N/R₀ − N (Eqs 8.29–8.33).
- **realizes**: Arbitrary ratios with contact and head bias kept nearly ratiometric (H08-43, H08-44).
- **applicability**: MOD and EXC only with the sensitivity calculation (rule R5) (H08-43); dogbone or enclosed segments force embedded dummies between differing lengths (Fig 8.18: dummies at the 4th and 7th of 10 dogbones, 4th and 6th of 8 bars) (H08-43, ref-99 §2.8); avoid partial segments when possible (H08-44).
- **parameters**: N sweep and per-device L_seg; or R₀ sweep with M, N, j, k.
- **guarantees**: For 200 kΩ/146 kΩ, Eq 8.27 minima at N = 3, 8, 11 (e.g. R_N = 8×18.25k, R_M = 11×18.18k); Fig 8.19's bars agree with Eq 8.27 within 0.11e-6 (H08-43, ref-99 §2.2). Partial segments inset equally on both ends and extended to align with full ones (rules R8, R9) (H08-44).
- **does not guarantee**: Fig 8.20's optima (10.34 and 10.64 kΩ, M = 19, N = 14, j = 0.342, k = 0.120) as golden values for Eq 8.29: the figure plots the equation with j and k exchanged, so implement the printed form (ref-99 §2.1, H08-44).
- **pdk data needed**: Head and contact bias (process bias R = αR_i + β applies to every segment including the partial one) (ref-99 §2.1).
- **verification**: S ≤ the systematic budget share; Pareto over (S, area, aspect) (H08-43).
- **prefer over / degrade to**: Integer units (MAT-02) > per-resistor lengths > partial segments (H08-44).
- **source**: H08-43, H08-44, ref-99 §2.1, §2.2, §2.8 (Fig 8.18).

#### MAT-07 Resistor-array end dummies and body extension by class (etch, litho, dopant proximity)
- **id**: MAT-07
- **technique**: Same-pitch dummy segments at both array ends, bodies extended past the active region, and enlarged block-mask enclosures for matched classes.
- **realizes**: Equal etch microloading (penetrates ≥ 10 µm; 10 kÅ poly etched 90% anisotropically undercuts 0.1 µm and a 0.02 µm/side iso-dense bias is 2% on 1 µm resistors), equal iso-dense print bias (significant below ≈ 2× the minimum feature; negligible above w_min = λ/NA ≈ 0.6 µm for i-line), equal diffusion tails (H08-12, H08-13, H08-18).
- **applicability**: MIN diffused/implanted: no dummy; MIN deposited: one minimum-width dummy per end; MOD any type: one full-width dummy per end; EXC deposited: multiple dummies spanning 5–10 µm (§8.2.3 says ≥ 10 µm) (H08-13). Diffused dummies 3–4× x_j wide (6–8 µm for a 2 µm junction), copying heads and contacts, biased to the enclosing region (H08-13). Gate-doping block enclosure 150% of the rule for MOD/EXC (H08-60).
- **parameters**: Dummy count and width (≥ w_min, no contacts needed) (H08-12); pitch = active pitch; body extension beyond the inner contact edge ≥ 3× (MOD) or 5× (EXC) the minimum width, partial segments extended to align (H08-13); tie node: connected rather than floating (floating poly charges), never noisy or capacitively disturbing, designer consulted (H08-15).
- **guarantees**: End segments see the same neighbour environment as interior ones (H08-13).
- **does not guarantee**: Gradient cancellation (MAT-04); hydrogen uniformity (MAT-10).
- **pdk data needed**: Etch penetration depth; litho w_min; x_j; block-mask enclosure (H08-12, H08-13, H08-60).
- **verification**: Each end segment has a same-width neighbour at the array pitch; dummy span meets the class; dummies connected to a quiet net (H08-13, H08-15).
- **prefer over / degrade to**: Connected over floating dummies; full-width over minimum-width when room exists (H08-12, H08-15).
- **source**: H08-12, H08-13, H08-15, H08-18, H08-60.

#### MAT-08 Common orientation, piezoresistive axis choice and H+V split arrays
- **id**: MAT-08
- **technique**: Lock all members of a set to one orientation even when not adjacent; choose the axis by material; for orientation-sensitive materials build two identical CC arrays rotated 90° with each resistor split half and half.
- **realizes**: Immunity to H-V lithography bias (rule R6, all classes) (H08-16); stress-direction insensitivity, π = (π_L + π_T)/2 for equal H and V series segments (Eq 8.39) (H08-52).
- **applicability**: N-type diffused on (100): least stress-sensitive along layout X/Y (<110>); P-type diffused on (100): least sensitive at 45° (<100>) but usually drawn X/Y; poly, (111) and thin film are orientation-independent yet still share one orientation (H08-16); the H+V split pays for P-type monocrystalline on (100) and N-type on (111) (H08-52).
- **parameters**: Orientation; 50/50 split on or off.
- **guarantees**: ΔR = R(π_L·σ_L + π_T·σ_T + π_LT·τ_LT), π_LT ≈ 0 for monocrystalline silicon (Table 8.7, ×1e-11 Pa⁻¹: P <110>/(100) 71.8/−66.3; P <100>/(100) 6.6/−1.1; P poly 24/−10; N <100>/(100) −102/53.4; N <110>/(100) −31.2/−17.6; N poly −16/9.5); poly at −110 MPa shifts 4% (H08-51).
- **does not guarantee**: Random filler stress (MAT-13) (H08-54).
- **pdk data needed**: π_L and π_T along layout X/Y; wafer surface (100)/(111) (H08-51).
- **verification**: All body long axes parallel across the set; current direction balanced (H08-16).
- **prefer over / degrade to**: Single orientation by default; H+V split only for the sensitive monocrystalline cases (H08-52).
- **source**: H08-16, H08-51, H08-52, H08-54.

#### MAT-09 Ratiometric jumpers, equalized lead parasitics and Kelvin leads
- **id**: MAT-09
- **technique**: Make each jumper's R proportional to its segment's R (same layer; short jumpers lengthened with jogs, long ones widened; a via in every jumper if any has one, several vias each); make every unit's lead C equal (same width, jogs or dead-end stubs, all leads cross a metal or none, dummy metal excluded, ≥ 2 µm or 2–3× ILD spacing to other metal, else flanking shield lines); current-carrying leads short, direct and wide; Kelvin taps for EXC.
- **realizes**: Interconnect-neutral ratios (a 100 µm × 0.5 µm jumper at 50 mΩ/□ adds 1% to a 1 kΩ segment; a 100 µm lead at ≈ 0.13 fF/µm is 13 fF, 1.3% of 1 pF) (H08-20, H08-22).
- **applicability**: Resistor segments ≤ 1 kΩ especially (H08-20); MOD and EXC capacitors (rule C10) (H08-22); EXC (rule R23): each metal layer's and each via type's contribution scales with the ratio, via share a small fraction of the mismatch; Kelvin connections with sense currents accounted for (H08-20, H08-21).
- **parameters**: Jumper layer, width, length, via count; stub lengths; lead width; spacing.
- **guarantees**: Interconnected segments adjacent to minimize jumpers (H08-20); via R is random, hence several vias (H08-20); Fig 8.8 shows a jumpered ABAABA array with end dummies (ref-99 §2.8).
- **does not guarantee**: Hydrogen effects of metal over bodies (MAT-10).
- **pdk data needed**: Metal R_s; via R; lead C per µm (7.5 kÅ, 1 µm metal over 10 kÅ FOX ≈ 0.13 fF/µm) (H08-20, H08-22).
- **verification**: |R_route(A)/R_route(B) − R_A/R_B| ≤ ε and R_route ≪ allowed error; |C_lead(i)/n_i − C_lead(j)/n_j| ≤ ε·C_u from back-annotation; IR drop per terminal route ≤ budget (H08-20, H08-21, H08-22).
- **prefer over / degrade to**: Equal construction by the generator first; stubs and jogs to equalize; never lengthen the long lead (H08-22).
- **source**: H08-20, H08-21, H08-22, ref-99 §2.8 (Fig 8.8).

#### MAT-10 Metal-over-body policy: folded-out jumpers, hydrogen control, fill block, keep-out by class
- **id**: MAT-10
- **technique**: Route jumpers folded out, away from the bodies; block all metal including dummy fill over matched bodies; or cover the array with a plate (RES-13); price or forbid foreign nets over bodies by class and signal type.
- **realizes**: Uniform hydrogenation (aluminium blocks and getters hydrogen; metallized vs unmetallized poly mismatch up to 10%) (H08-27); no conductivity modulation (0.1%/V for met1 over 2 kΩ/□ HSR) or noise coupling (H08-28).
- **applicability**: MIN poly ≤ 500 Ω/□ may carry unconnected low-frequency analog or static digital leads, never switching digital or HF analog; MOD: no unconnected lead, connected leads only if unavoidable, unless a low-impedance field plate intervenes; EXC: met2 plate ≥ 10 µm overhang and no met1 across the active area (H08-27, H08-28); a connected lead may cross beside the head it feeds (H08-28); a folded-in jumper crossing every segment identically still modulates (H08-28).
- **parameters**: Fold direction; keep-out polygon over bodies (heads exempt for the own net); cost matrix by net class.
- **guarantees**: Fig 8.12's folded-out jumpers match better than folded-in ones even with dummy metal; a met1 plate exposing only the heads lets met2 jumpers fold in (H08-27, ref-99 §2.8).
- **does not guarantee**: Protection against digital edges, even shielded (H08-28).
- **pdk data needed**: Hydrogen sensitivity per recipe; whether the flow anneals at ≈ 400 °C (deep-submicron flows anneal less) (H08-27).
- **verification**: No foreign shape over body polygons per class; fill blocked on all layers over matched bodies (H08-27, H08-28).
- **prefer over / degrade to**: Reroute > head corridor (RES-12) > plate (RES-13) > priced crossing for MIN only (H08-28).
- **source**: H08-27, H08-28, ref-99 §2.8 (Fig 8.12).

#### MAT-11 Thermoelectric cancellation
- **id**: MAT-11
- **technique**: Connect series segments so current flows alternately in opposite directions, half each way, with an even count; for serpentines keep both heads close together and orient non-identical ones oppositely against misalignment.
- **realizes**: Cancellation of Seebeck EMF E_T = S·ΔT_C, S = 50–500 µV/°C at Al–Si (and Cu–Si) contacts, unchanged by silicide; 1 °C at 100 µV/°C is 0.1 mV, a 0.4% bipolar mirror error (H08-45).
- **applicability**: Every series string of a matched resistor (rule R11); common-centroid layout cannot cancel it (H08-45).
- **parameters**: Direction sequence; N even; head placement.
- **guarantees**: Σ(direction) = 0 per string (H08-45).
- **does not guarantee**: Anything with an odd count; a single segment is tolerated below EXC (H08-45).
- **pdk data needed**: None.
- **verification**: Per-string direction sum from unit orientation (H08-45).
- **prefer over / degrade to**: Alternating even strings > co-located heads > nothing (H08-45).
- **source**: H08-45.

#### MAT-12 Thermal placement relative to power devices
- **id**: MAT-12
- **technique**: Put power devices on a die symmetry axis; align matched arrays to the thermal axis as far from heat as possible; derive separation from power; add CC when the R7 triggers hold.
- **realizes**: Low ∇T across the set; the gradient decays roughly exponentially from the power-device edge (H08-47, H08-32).
- **applicability**: One source: power at one end, matched devices on the axis about halfway across the remaining die (≈ ¾ of the way from centre to far edge for EXC on power ICs), elongation to 1.3–1.5 helps; two sources: in line on one axis or one at each end with matched devices centred ≥ 0.5 mm away; four sources: aspect 1.5:1–2:1 (H08-47, H08-49). EXC needs TC < 500 ppm/°C near power and is rarely feasible with ≥ 1 W; MOD goes opposite multi-watt devices, or near small ones on their axis with array width ≤ device width and separation ≥ 1 µm per mW (½–¼ acceptable); MIN may sit within a few hundred µm of multi-watt devices if interdigitated (H08-47). EXC capacitors, especially poly-electrode ones (TC 50–100 ppm/°C), away from power (H08-59). Matched resistor dissipation ≤ 1–2 µW/µm² for EXC (H08-55).
- **parameters**: Axis alignment; distance; CC flag; die elongation.
- **guarantees**: ΔR = α·R·d_CC·∇T_CC, so the levers are lower TC, lower gradient, separation perpendicular to the gradient and d_CC → 0 (H08-32).
- **does not guarantee**: Transient thermal waves in non-heat-sunk packages; heat-sunk packages give steep gradients near power devices (H08 §8.2.7).
- **pdk data needed**: TCR per recipe (H06-09); thermal conductivities (Si 1.56, SiO₂ 0.013, mold 0.008–0.021, Cu alloys 3.0–3.6 W/cm·°C) (H08 §8.2.7).
- **verification**: α·∇T·(c_A − c_B) ≤ the systematic budget from the thermal field; self-heating added as sources (H08-32, H08-55).
- **prefer over / degrade to**: Minimize the stress-plus-thermal sum along the axis rather than a fixed ½ or ¾ (H08-49).
- **source**: H06-09, H08-32, H08-47, H08-49, H08-55, H08-59, H08 §8.2.7.

#### MAT-13 Stress-aware die location, aspect limits and filler averaging
- **id**: MAT-13
- **technique**: Place matched sets near the die centre on a die axis, never in corners; limit die aspect by package; for EXC under random filler stress split each device into copies ≥ 50–100 µm apart and average them, or build redundant arrays and select post-package.
- **realizes**: Minimal stress gradient (smallest at centre, larger at edges, largest at corners; anomalies within about one die thickness, typically 250 µm, of the edge) (H08-48); control of package shift (H08-54).
- **applicability**: MIN: out of corners and ≥ 50–100 µm from edges; MOD: interior or middle of a side inset 100–250 µm (middle of the longer side if peripheral); EXC: near the centre on a symmetry axis (also MOD monocrystalline; thin film need only avoid edges and corners); EXC capacitors several hundred µm from the sides (H08-48). Aspect (preferred/maximum): plastic ≤ 10 mm² 1.5:1/3:1, > 10 mm² 1.5:1/2:1; solder or eutectic ≤ 10 mm² 1.5:1/2:1, > 10 mm² 1.5:1/1.5:1; CSP 1.5:1/2:1; metal can and ceramic 2:1/any; > 1.5:1 risky for solder/eutectic above 3–4 mm (H08-50).
- **parameters**: Location; orientation on the (100) die axes (four-fold symmetry) (H08-48); split count and spacing; overcoat.
- **guarantees**: Capacitor piezo ΔC = C·ξ·(σx + σy) with ξ ≈ 5.8e-13 Pa⁻¹ on (100), orders below resistor π, so capacitors tolerate worse spots but still need CC (H08-53); an elastomeric overcoat ≥ filler diameter (10–30 µm; 10 µm polyimide or 10–15 µm power copper) gives 3× less random shift (H08-54).
- **does not guarantee**: Gradient reduction from overcoats (H08-54); any enforcement at block level without a die model (H08-48); stability of post-package trim (relaxation in the first 1–2 h, moisture swelling, thermal hysteresis) (H08-54).
- **pdk data needed**: Die thickness; package type and attach; mold filler class (H08-50, H08-54).
- **verification**: Class setbacks from edges and corners; axis attraction; exported floorplan requirement (H08-48).
- **prefer over / degrade to**: Centre > middle of the long side > never corners (H08-48).
- **source**: H08-48, H08-50, H08-53, H08-54.

#### MAT-14 Tank or body grouping with a common-factor group size
- **id**: MAT-14
- **technique**: Give resistors at different voltages separate tanks, each tied to its segment's positive end; share a tank among G segments where G is a common factor of every member's segment count; order the groups common-centroid.
- **realizes**: Equal resistor-to-body voltage across members (body modulation ≈ 0.1%/V for 160 Ω/□ base, several %/V for HSR) (H08-31).
- **applicability**: Diffused, well and HSR matched resistors; example 4 and 8 segments → groups of 4, R2's two groups either side of R1 (H08-31); low-sheet base resistors may share one tank (H08-31); tank biasing is the designer's decision and belongs on the schematic (H08-31).
- **parameters**: G; group order; body nets.
- **guarantees**: Modulation tracks when each equal segment group has its own body at its positive end (H06-11, H06-19).
- **does not guarantee**: A smaller VCR magnitude (H06-11).
- **pdk data needed**: Body-modulation coefficient per model (H08-31).
- **verification**: Spread of (V_body − V_segment) ≤ budget/coefficient; groups CC (H08-31).
- **prefer over / degrade to**: One shared tank for low-sheet material; per-group tanks for HSR (H08-31).
- **source**: H06-11, H06-19, H08-31.

#### MAT-15 Identical underlayer and environment (moat setback, nothing beneath, topography, well proximity, foreign diffusions)
- **id**: MAT-15
- **technique**: All members over the same field oxide with no well or active edge within the setback; block every layer under matched capacitors or draw a solid tied plate there; keep matched sets away from tall topography, well edges and foreign diffusions.
- **realizes**: A uniform oxide, thickness and stress environment (poly over N-well vs P-well differs > ±1% without CMP; LOCOS beaks taper over microns and STI stress radiates microns) (H08-24).
- **applicability**: Accurate resistors ≥ 5 µm and MOD capacitors ≥ 5–10 µm from moat edges; nothing routed beneath more accurate capacitors (rules R17, C5) (H08-24); matched poly resistors spaced from poly-poly capacitors by at least the poly-2 width along the joining line, MOS matching affected within ≈ 30 µm (rule R25) (H08-17); matched diffusions ≥ 2–3 µm inside drawn well edges (ions scatter > 1 µm) (H08-19); MOD/EXC diffused resistors ≥ 150% of minimum spacing to other diffusions, deep-N+ sinkers behind a head not beside the body (rule R24) (H08-18).
- **parameters**: Setbacks; under-array blockage or solid plate; halo keep-outs by class.
- **guarantees**: Identical underlayer geometry per member (H08-24).
- **does not guarantee**: Gradient cancellation (MAT-04).
- **pdk data needed**: CMP flag; well depth; enclosure rules (H08-19, H08-24).
- **verification**: Geometric identity of underlayers across members; no active edge within the setback; no foreign shape under the capacitor bounding box (H08-24).
- **prefer over / degrade to**: Hard constraint for MOD/EXC, a cost for MIN (H08-17).
- **source**: H08-17, H08-18, H08-19, H08-24.

---

## 2. Numeric rules of thumb

| Topic | Value | Source |
|---|---|---|
| Resistor width bias | Diffusion W_b ≈ 20% of x_j (1.25 µm x_j → 0.25 µm, 5% on a 5 µm base resistor); poly W_b a significant fraction of a µm | H06-01, H06 §6.5.7 |
| Segment length | Accurate resistors ≥ 10 µm; end correction < 1% at ≥ 10 □; L ≥ 20·thickness; poly L ≥ 1000·d_g (≈ 50 µm), 10 µm acceptable | H06-03 |
| Contact-end crowding | 5 µm strip with 3 µm contacts: +0.05 □ (both ends) | H06-04 |
| Serpentine turns | Square corner +0.56 □ (≈ ½ □); 180° circular end +2.96 □ | H06-05 |
| Dogbone heads | −0.7 □ (W_o = W_d) to −0.3 □ (W_o = ½W_d), usually < 0.3 □ | H06-06 |
| Contact resistance | ρ_c (Ω·µm²): Al–base 750, RBM–base 2500, PtSi–base 1250, Al–emitter 40, TiSi₂–NSD 30, TiSi₂–PSD 100; 1 kΩ base with 8×8 µm contacts: 43 Ω/contact, 9% | H06-15, H06-02 |
| Sheet tolerance | ±20–25% most layers; ±50% VHSR poly and pinch; 500 Ω/□ poly ±20%, 5 kΩ/□ ±50% | H06-07, H06-20 |
| Linewidth control | ≈ ±10% of minimum feature (deposited); +±5% of x_j (diffusion) | H06-07 |
| Width guidelines (absolute) | None: minimum; moderate: 2–3× min, poly ≥ 0.3 µm, diffusion ≥ 2·x_j, no poly > 5 kΩ/□, no pinch; crucial: trim, poly ≥ 0.5 µm, no lightly doped | H06-07 |
| Narrow-width floors | Poly bamboo harmless > ≈ 0.25 µm; TiSi₂ ≥ 0.25 µm (C49 below 0.2 µm); wells ≥ 2–3·x_j | H06-08, H06-25 |
| Resistor TCs (ppm/°C) | Al +3800, Cu +4000, metal ≈ +3300; 160 Ω/□ base +1500; 7 Ω/□ emitter +600; 5 kΩ/□ base pinch +2500; 2 kΩ/□ HSR +3000; 500 Ω/□ poly −1000; 25 Ω/□ poly +1000; 70 Ω/□ poly +500; 10 kΩ/□ poly −7000; 0TC poly 200–300 Ω/□ ±100; 10 kΩ/□ N-well +6000 | H06-09, H06-23 |
| Voltage coefficients | Base 100 ppm/V; HSR 2.5%/V; base pinch 6%/V + 2%/V²; 700 Ω/□ P-type body 1000 ppm/V, −20 ppm/V²; lead over 2 kΩ/□ HSR 0.1%/V; poly ≤ 1 kΩ/□ largely immune | H06-11 |
| Velocity saturation | L_min = 10 µm/V · V_max (N-Si), 3.3 µm/V · V_max (P-Si); fields 0.2 / 0.6 V/µm, safety 2 | H06-13 |
| Body/tank rating | ≤ 2/3 of resistor–body (or E-B) breakdown per segment; field plate above 2/3 (ch. 6) or 50% (R20) of the thick-field threshold | H06-19, H06-14, H08-30 |
| HSR | 1–10 kΩ/□; optimal ≈ 2 kΩ/□; BV 20–30 V; always NBL beneath; head k ≈ 0.7 → 1 elongated | H06 §6.5.4 |
| NBL under matched base resistors | Overlap 5–8 µm; shadow 50–150% (DCS) or 100–200% (SiCl₄) of epi thickness; ≥ 150% overlap if unknown | H06 §6.5.1, H08-26 |
| Poly parasitics | ≈ 0.05 fF/µm² over field oxide (1 µm × 1000 □ ≈ 50 fF); 1 µm × 1 µm crossing ≈ 0.05 fF; 2 Vpp on 0.1 pF node → ≈ 1 mVpp | H06-17, H06-18 |
| Junction capacitance | 1–5 fF/µm² (diffused resistor body); emitter oxide 0.7 (thin) / 0.2 (thick) fF/µm² | H06-17, H06 §6.5.2 |
| Voltage-dependent spacing | Adjacent turns ΔV = 2V_d/N; spacing elsewhere = block operating voltage; STI cones break down < 20 V | H06-22, H06-21 |
| Metal resistors | 50 mΩ–50 Ω; 20–30 mΩ/□ (bipolar 10–15 kÅ), 50–90 mΩ/□ (CMOS 3–5 kÅ); R_s ±20%; force leads straight ≥ 2× width | H06-23, H06 §6.5.6 |
| Thin film | NiCr 20–200 Ω/□ +50…+100; Ta 20–200 −100…−150; SiCr 100–2000 0…−150 ppm/°C; sheet ±20%; trimmed ±0.05% or better; TC var < ±50 ppm/°C | H06 §6.5.10 |
| Hydrogen | HSR poly sheet −30% from nitride H; metallized vs unmetallized poly up to 10%; P-type HSR drifts up several % over 10²–10³ h | H08-27, H06-16 |
| Trim bits | N = ceil[3.32·log₁₀(X/δX)]; ±0.25% over ±5% → 4 bits; > 6 bits needs secondary network; nonbinary weights each ≤ 2× previous; trimmed accuracy ±0.05% (TF ±0.01%) | H06-27 |
| Fuses | Leads ≥ 5× link width; poly fuse 5–15 V, 1 ms, 50–150 mA, < 25 ns rise; metal fuse 5 V, 1 ms, several hundred mA, < 1 µs rise; Zener zap ≈ 10 Ω after 100–250 mA; switch W ∝ 1/R_k | H06-28, H06 §6.6.2 |
| Laser links | ≈ 1 µm × 15 µm; 10–15 µm clearance; cuts 10–30 mm/s | H06-29 |
| Dielectric strength (MV/cm) | Dry SiO₂ 11; PECVD oxide 3–6; TEOS 10; LPCVD nitride 10; PECVD nitride 5; V_max = t·E_max (200 Å at 5 MV/cm → 10 V) | H06-32 |
| TDDB budget | ≈ 10 FIT total; 1 FIT = 1 failure per 10⁹ h | H06-33 |
| Capacitor process variation | Gate oxide ±20% (some ±10%); ONO ≥ ±20%; junction plate ≥ ±20%; comb ≥ ±30% | H06 §7.1.1 |
| Capacitor density (fF/µm², ±%, V) | B-E 0.8/50/5; emitter oxide 0.07/30/40; GOX 0.86/20/15; poly-poly 1.3/25/15; stack 0.05/30/40; thick GOX 2.2/20/7; thin GOX 4.3/20/3.6; TiN 3.1/20/7; lateral flux 0.07/30/40 | H06-42, H06 §7.1.3 |
| Capacitor TC/VC | Thin oxide +20 ppm/°C; nitride +12; poly-poly < 50 (or < 250) ppm/°C, ≈ 150 ppm/V; silicide–oxide–metal 2–4 ppm/V; 600 Å nitride MIM −13 ppm/V, −4 ppm/V², 39 ppm/°C; antiparallel poly-poly ≈ 2 ppm/V; junction +500…+700 ppm/°C | H06-42, H06-36 |
| Fringe | 5 µm square over 200 Å: 43.1 fF + 2.6 fF; 1 µm × 5 kÅ line over 8 kÅ: fringe ≈ 60% (eq gives ≈ 69%) | H06-30, H06-31 |
| Stack density | ≈ 30,000 µm² of 10 kÅ ILO per pF; poly–M1–M2 ≈ 2× | H06-43 |
| MOS cap bias | ≥ 0.5 V beyond V_t or V_FB (C may still be 10% short); ≥ 1 V preferred; gate R lumps to 1/3 (one end) or 1/12 (both ends) | H06-37, H06-39 |
| Junction cap | ≤ 75% BV; forward ≤ 400 mV at RT, 200 mV at 125 °C; C 75% at −1 V, 50% at −5 V; comb if S_f < 2c_p/c_a | H06-35, H06-34 |
| ONO | Breakdown up to 50% lower with lower plate negative; absorption at ≥ 10 MHz; relaxes in ≈ µs | H06-44, H08-58 |
| Precision classes | MIN ±1–5%; MOD ±0.1–1%; EXC ±0.01–0.1%; six-sigma, 10 y, −40…125 °C, plastic | H08-02 |
| Random share | ≤ 75% MIN, ≤ 50% MOD, ≤ 25% EXC | H08-03 |
| Area for k_A = 2%·µm | 500 µm² (±1%), 0.12 mm² (±0.1%), 46 mm² (±0.01%); thin film 2.9 mm² (±0.01%) | H08-03 |
| Poly k_R | k_R = d_g·√(η·R_s), η ≈ 2, d_g < 0.1 µm; valid to somewhat below 0.3 µm | H08-08 |
| Segment width floors | 150% / 200% / 400% of minimum (MIN/MOD/EXC); segmenting text: ≥ 150%, ≥ 300% accurate | H08-41 |
| Segment length floors | 3× / 5× / 10× minimum; EXC poly series path ≥ 1000·d_g or 200 µm | H08-41 |
| Capacitor units | Squares 20×20 to 50×50 µm for EXC; 2:1–3:1 rectangles MOD | H08-57 |
| Iso-dense bias | Negligible above w_min = λ/NA (≈ 0.6 µm i-line, NA 0.6); significant below ≈ 2× minimum feature | H08-12 |
| Etch microloading | Penetrates ≥ 10 µm; 0.02 µm/side on 1 µm resistors = 2%; dummies span ≥ 10 µm for best matching | H08-13 |
| Resistor dummies | MIN deposited: 1 min-width per end; MOD: 1 full-width per end; EXC: span 5–10 µm; body extension ≥ 3× (MOD), 5× (EXC) min width; diffused dummies 3–4× x_j | H08-13 |
| Capacitor dummies | MOD under shield ≤ 3× min width; unshielded fringing reaches 10–30 µm; EXC exact copies; Fig 8.5: 6 active, 14 dummies | H08-14, ref-99 §2.8 |
| Topography | Poly-poly caps perturb MOS matching within ≈ 30 µm; poly R spacing ≥ poly-2 width along the joining line | H08-17 |
| Well proximity | Matched diffusions ≥ 2–3 µm inside well edges | H08-19 |
| Moat setback | Resistors ≥ 5 µm; MOD capacitors ≥ 5–10 µm | H08-24 |
| Foreign diffusions | ≥ 150% of minimum spacing (MOD/EXC); block-mask enclosure 150% | H08-18, H08-60 |
| Jumper resistance | 100 µm × 0.5 µm × 50 mΩ/□ = 1% of 1 kΩ; watch segments ≤ 1 kΩ | H08-20 |
| Lead capacitance | ≈ 0.13 fF/µm (7.5 kÅ, 1 µm metal over 10 kÅ FOX); 100 µm = 13 fF = 1.3% of 1 pF; lead spacing ≥ 2 µm or 2–3× ILD | H08-22 |
| Shields | Capacitor shield overhang 3–5 µm, digital ≥ 3–4 µm clear; resistor Faraday shield overhang 2–3 µm; per-segment above ≈ 500 Ω/□ or a few volts; EXC hydrogen plate ≥ 10 µm overhang | H08-23, H08-29, H08-27 |
| Thermal | ΔR = α·R·d_CC·∇T; κ Si 1.56, SiO₂ 0.013, mold 0.008–0.021 W/cm·°C; most resistors TC ≥ 1000 ppm/°C; power devices tens of °C across a die | H08-32, H08 §8.2.7 |
| Power placement | Separation ≥ 1 µm/mW (½–¼ acceptable) for MOD near small sources; two sources ≥ 0.5 mm; EXC TC < 500 ppm/°C near power; EXC dissipation ≤ 1–2 µW/µm² | H08-47, H08-55 |
| R7 CC triggers | Die > 15 mm²; P > 250 mW; heat-sunk; solder/sinter/eutectic; within 250 µm of edge or heat source | H08-47 |
| Thermoelectric | S = 50–500 µV/°C; 1 °C × 100 µV/°C = 0.1 mV = 0.4% bipolar mirror error | H08-45 |
| Stress | Edge anomalies within ≈ 1 die thickness (250 µm; 50–75 µm thin dice); centre −100…−200 MPa (older compounds); poly at −110 MPa shifts 4%; filler σ up to 2% (pre-2000), 2–3× better now | H08-48, H08-51, H08-54 |
| Die location | MIN ≥ 50–100 µm from edges, no corners; MOD inset 100–250 µm; EXC near centre; EXC caps several hundred µm from sides | H08-48 |
| Die aspect | Plastic 1.5:1 preferred, 3:1 (≤ 10 mm²) or 2:1 max; solder/eutectic > 10 mm² 1.5:1 max; > 1.5:1 risky over 3–4 mm | H08-50 |
| Piezocapacitance | ξ ≈ 5.8e-13 Pa⁻¹ (100), 4.6e-13 (111) | H08-53 |
| Field plates | MOD diffused ≥ 500 Ω/□ and all EXC; split plates for > 1 kΩ/□ matching better than ±0.5%; dielectric absorption 0.1% in 2 kΩ/□ HSR under BPSG | H08-30 |
| Filler averaging | Copies ≥ 50–100 µm apart; overcoat 10–30 µm gives 3× less shift | H08-54 |
| Validation sample | ≥ 30 units, several wafers, boat front/middle/back, random die, not extreme edge | H08-01, H08-62 |
| Inductors | Useless < 100 MHz; width 10–15 µm at 1–3 GHz; d_i ≥ 5w and ≥ d_o/3; ≤ 1–2 layers; keep-out ≥ ½ width; substrate > 10 Ω·cm (prefer > 100); skin depth ≈ 2.3 µm Al at 1 GHz; Q 1–40; bondwire 25 µm in 1 mm loop ≈ 5.6 nH | H06-47, H06-48, H06-50, H06-51, H06-53, H06-54 |

---

## 3. Conflicts and trade-offs

- **Dispersion vs random area.** More CC sub-arrays cut the gradient residue but the gaps cut active area and raise the random term; the optimum is rarely computable by hand, and with negligible gradients plain ABBA is as good as CC (H08-38, H08-36).
- **Segment length vs jumper share vs gradient.** Longer segments shrink the jumper-resistance fraction but make spindly, gradient-sensitive arrays; short segments raise head/contact share (2(N−1) extra heads) and need the H06-02 correction (H08-42, H08-20, H06-02).
- **Dogbone vs strip heads vs parallel segments.** Dogbones fit contacts on narrow bodies but pack worse, add a −0.3 to −0.7 □ term and force embedded dummies in mixed-length arrays; parallel longer segments are preferred for < 10 □ (H06-06, H06-02, H08-43).
- **Serpentine vs strip array.** A serpentine is compact and jumper-free but carries turn terms, turn-spacing bias and head-misalignment Seebeck error; acceptable only for MIN (H06-05, H08-46, H08-18, H08-45).
- **Folded-in vs folded-out jumpers.** Folded-in is compact but getters hydrogen over the bodies; folded-out matches better; a met1 plate restores the folded-in option at the cost of a layer (H08-27).
- **Shield vs capacitance.** A top shield raises lead capacitance but makes it predictable, allows leads over the array and shrinks the dummy ring; without it dummies must cover 10–30 µm of fringing (H08-23, H08-14).
- **Common vs per-segment plates.** One plate is cheap but modulates segments above ≈ 500 Ω/□ or when several volts span the array; per-segment plates fix that at routing cost (H08-29, H08-30).
- **Heat vs stress location.** The thermal optimum is far from the power device, the stress optimum is the die centre; the compromise is on the axis about halfway (¾ for EXC), better found by minimizing both terms (H08-49, H08-47, H08-48).
- **Width.** Wider segments cut random mismatch (1/W) and self-heating (1/W²) but cost area and parasitic capacitance (0.05 fF/µm²) (H08-06, H06-12, H06-17).
- **Capacitor unit size.** Small units suffer periphery effects and OPC corners; large units suffer gradients; optimum 20–50 µm squares (H08-57).
- **Capacitor style.** MIM has the best VCR/TC but needs masks; lateral-flux MOM wins only when spacing < ILO; stack caps are low density but free; fractal trades series R for density; MIM over circuitry saves area but couples to nets beneath (H06-43, H06-42, H06-45).
- **Poly-poly dielectric.** ONO is denser but absorbs above 10 MHz, breaks down asymmetrically and relaxes in µs; single-layer oxide is preferred for SC and charge-redistribution paths (H06-44, H08-58).
- **MOS capacitor geometry.** Lower gate resistance wants short, wide gates; lower channel resistance wants the opposite; split sections balance the two (H06-39).
- **Antiparallel split.** Cancels odd-order VCR and handles polarity reversal but puts one high-parasitic plate on each net (H06-36, H06-38, H06-40).
- **Non-integer ratios.** Integer units are safest; per-resistor lengths need embedded dummies with dogbones; partial segments are last resort; the Fig 8.20 optima are not golden for Eq 8.29 (H08-43, H08-44, ref-99 §2.1).
- **Small-member parallelism.** Making the small resistor from parallel units halves mismatch at 4× area (9:1 as 2×2), far cheaper than widening both (H08-07).
- **Voltage-dependent spacing.** Per-pair spacing saves area over a global worst case but needs node voltages (H06-22).
- **Trim methods.** Discrete fuse ladders stop at ±0.05%; electrical poly trim shifts TC; laser trim reaches ±0.01% on thin film but costs test time; remote switches beside the resistor avoid long noisy trim leads (H06-27, H06-28, H06-29, H06-56).
- **Higher-order arrays.** Octagonal arrays cancel quadratic terms but need non-Manhattan shapes and area; 2-D CC suffices in practice (H08-40).
- **Topography keep-away vs compaction.** Matched poly must stay a poly-2 width from poly-poly caps and MOS 30 µm, which conflicts with dense packing (H08-17).
- **Tank grouping.** Per-group tanks remove body-modulation mismatch at the cost of well area; shared tanks are fine only for low-sheet material (H08-31).
- **EXC resistors.** Untrimmed EXC needs tens of mm² at k_A = 2%·µm, so EXC almost always means thin film plus trim and is rarely compatible with power devices (H08-03, H08 §8.3.1).
- **Field plate thresholds.** Chapter 6 says plate above 2/3 of the thick-field threshold; rule R20 says 50% for matched diffused resistors; use the stricter for matched sets (H06-14, H08-30).
- **Inductors.** Wider turns lower DC resistance but lower f_crit; more turns or layers raise L but lower SRF; a patterned shield raises Q but adds capacitance (H06-48, H06-50, H06-52).

---

## 4. Precision classes

### 4.1 Resistors

**All classes (MIN and above)**
- Same material and model for every member (R1) (H08-56).
- Identical segment width; wider members built from parallel segments (R2) (H08-09).
- Random share of the budget ≤ 75% MIN / 50% MOD / 25% EXC, area sized by MAT-01 (R3) (H08-03).
- Width ≥ 150% of minimum (R4) (H08-41); length ≥ 3× minimum (R10) (H08-41).
- Equal segment lengths, straight rectangular strips (R5) (H08-10, H08-46).
- One orientation for all members, adjacent or not (R6) (H08-16).
- Series segments alternate direction, even count where possible (R11) (H08-45).
- Out of die corners and ≥ 50–100 µm from edges (R12) (H08-48).
- CC required if any R7 trigger holds: die > 15 mm², P > 250 mW, heat-sunk, solder/sinter/eutectic, within 250 µm of edge or heat source (H08-47).
- Deposited resistors: one minimum-width dummy per end; diffused: none (R9) (H08-13).
- No switching-digital or HF-analog leads over any matched resistor; MIN ≤ 500 Ω/□ may carry unconnected LF-analog or static-digital leads (R21) (H08-28).
- Serpentines allowed at MIN only (R16) (H08-46).
- Dummies connected to a quiet node, never floating or noisy (H08-15).

**MOD adds**
- Width ≥ 200% of minimum; length ≥ 5× minimum (R4, R10) (H08-41).
- Different segment lengths only with the Eq 8.27/8.29 sensitivity calculation (R5) (H08-43, H08-44).
- One full-width dummy per end at the active pitch; body ends extend ≥ 3× minimum width past the active region (R9) (H08-13).
- Arrays (not serpentines) even of dogbone segments (R16) (H08-46).
- Interior of the die, or middle of a side inset 100–250 µm; monocrystalline MOD on a die axis (R12, R14) (H08-48).
- Opposite side of the die from multi-watt devices, interdigitated, on the power device's axis; near small sources: array width ≤ device width, separation ≥ 1 µm/mW (R13) (H08-47).
- Same underlayer, ≥ 5 µm from moat edges (R17) (H08-24).
- NBL shadow must not intersect MOD HSR (R18) (H08-26).
- Gate-doping block enclosure 150% of the rule (R19) (H08-60).
- Field plate for diffused resistors ≥ 500 Ω/□ or above 50% of the thick-field threshold, per segment, tied to the tank (R20) (H08-30).
- No unconnected leads over bodies; connected only if unavoidable, unless a low-impedance plate intervenes; plate over as much body as possible or all metal (including fill) blocked (R21) (H08-27, H08-28).
- Foreign diffusions ≥ 150% of minimum spacing; sinkers behind heads (R24) (H08-18).
- Spacing from poly-poly capacitors ≥ poly-2 width along the joining line (R25) (H08-17).
- Jumpers ratiometric (same construction, several vias) (H08-20).
- Equal resistor-to-body voltage; per-group tanks with common-factor G (H08-31).

**EXC adds**
- Width ≥ 400% of minimum; length ≥ 10× minimum; poly series path ≥ 1000·d_g or 200 µm (R4, R10) (H08-41).
- All four CC rules; dispersion into sub-arrays; partial segments inset equally and aligned (R8) (H08-35, H08-44).
- Multiple dummies spanning 5–10 µm (≥ 10 µm per §8.2.3); body extension ≥ 5× minimum width (R9) (H08-13).
- Near the die centre on a symmetry axis (R12, R14) (H08-48).
- TC < 500 ppm/°C near power; ≈ ¾ of the way from centre to far edge from a single heat source; rarely feasible with ≥ 1 W (R13) (H08-47).
- NBL shadow must not intersect any EXC diffused resistor (R18) (H08-26).
- Every diffused resistor field-plated; split plates for > 1 kΩ/□ (R20) (H08-30).
- Met2 plate ≥ 10 µm beyond the active region, no met1 across the active area beneath (R21) (H08-27).
- Dissipation ≤ 1–2 µW/µm² (R22) (H08-55).
- Each metal layer's and via type's contribution scales with the ratio; Kelvin connections (R23) (H08-20, H08-21).
- Thin film plus laser trim in practice; same material still mandatory (H08 §8.3.1, H08-56).
- Optional: split copies ≥ 50–100 µm apart against filler stress; H+V split arrays for orientation-sensitive materials (H08-54, H08-52).

### 4.2 Capacitors

**All classes (MIN and above)**
- Equal area/perimeter ratio across members; integer unit arrays where ratios allow; no series units unless antiparallel (C1) (H08-11).
- Bottom plate to a low-impedance node or an N-well shield beneath (C6) (H08-25).
- Electrodes oriented by net impedance; never interchanged (H06-40).
- Any arrayed matched capacitor gets dummies on all four sides at the active pitch, both electrodes tied (C7) (H08-14).
- CC for every matched capacitor set regardless of stress; cross-couple even two equal capacitors (C9) (H08-53, H08-57).
- Random share ≤ 75% MIN (C3) (H08-03); MIN gains nothing from division into units (C3) (H08-57).
- Junction capacitors unsuitable; MOS only several volts into accumulation or inversion (H08-58).
- No oxide step under poly-poly plates (H06-45).
- Voltage within V_max = t·E_max; TDDB FIT within budget (H06-32, H06-33).

**MOD adds**
- Unit arrays customary (C1) (H08-11); units square or 2:1–3:1 rectangles, no odd shapes (C2) (H08-57); random share ≤ 50% (C3) (H08-03).
- Compact rows and columns, equal row pitch and equal column pitch (C4) (H08-57).
- Same underlayer; ≥ 5–10 µm from moat edges (C5) (H08-24).
- Dummy width ≤ 3× minimum under a shield; much wider without one (C7) (H08-14).
- Electrostatic shield above and below, tied to analog ground, 3–5 µm overhang; no digital or noisy lines over; 3–4 µm clearance (C8) (H08-23).
- Equalized lead capacitance: same widths, jogs or stubs, all cross or none, ≥ 2 µm or 2–3× ILD spacing (C10) (H08-22).
- MOS capacitors biased several volts into the flat region (H08-58).
- Antiparallel halves for asymmetric-electrode types on linearity-critical nets (H06-36).

**EXC adds**
- Unit arrays required; square units 20–50 µm (C1, C2, C3) (H08-11, H08-57); random share ≤ 25% (C3) (H08-03).
- Dummies are exact copies of the unit on all sides (C7) (H08-14).
- Nothing routed beneath the array: lower metal and poly removed or made a contiguous solid plate (C5) (H08-24).
- Metallic or silicided electrodes with thick homogeneous grown or LPCVD oxide; no ONO/TEOS on charge-redistribution paths; no poly electrodes or MOS caps (C11) (H08-58).
- Near the die centre, several hundred µm from the sides (C12) (H08-48, H08-53).
- Away from power devices, especially poly-electrode types with TC 50–100 ppm/°C (C13) (H08-59).
- All four CC rules with dispersion; quadratic residue scored (H08-35, H08-36).
- Shield and lead parasitics verified by back-annotation (H08-22).
- Oxide capacitors in plastic can reach ±0.01%, perhaps ±0.001% (H08 §8.3.2).
