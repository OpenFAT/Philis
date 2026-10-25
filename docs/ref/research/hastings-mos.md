# Hastings ch.12–13: MOS construction and matching, as technique cards

Sources read in full: the Philis digests of Hastings, *The Art of Analog Layout* 3e, chapter 12 (`ref-hastings-12-ch12-field-effect-transistors.md`, items H12-01…H12-51), chapter 13 (`ref-hastings-13-ch13-mos-applications-matching.md`, items H13-01…H13-55) and the figure-description digest (`ref-hastings-99-figure-descriptions.md`, chapter 12 and 13 material only).

Citation key. `H12-nn` and `H13-nn` are item ids of the two chapter digests. `H12 §x` / `H13 §x` cite a digest's section-by-section part where no item id covers the claim. `F99 §n` cites the figure-description digest. Statements marked *(inference)* are conclusions the cited text supports but does not state.

Precision classes. Hastings names them minimal, moderate and exceptional (H13-42). This report writes MIN, MOD, EXC. The brief's "precise" is Hastings' "exceptional".

Constraint-to-card index:

| Constraint the engine must realize | Cards |
|---|---|
| Compact wide device, low drain C, gate RC | MOS-01, MOS-03, MOS-12, MOS-35 |
| Exact ratio, identical unit sections | MOS-02, MOS-05 |
| Area and node-capacitance saving between devices | MOS-04, MOS-05, MOS-10 |
| Backgate pinning, latch-up, snapback, SOA | MOS-06, MOS-07, MOS-08, MOS-09 |
| Device flavour, well class, body bias | MOS-10, MOS-14 |
| Long L, HV, stringer immunity | MOS-11, MOS-12 |
| Gate resistance, contact placement, corner rounding | MOS-13 |
| Orientation equality | MOS-15, MOS-34 |
| Equal etch / stress / doping environment | MOS-16, MOS-17, MOS-18, MOS-19, MOS-20, MOS-21 |
| Equal hydrogen and metal environment | MOS-22, MOS-23 |
| Centroid coincidence, symmetry, dispersion | MOS-24, MOS-25, MOS-26, MOS-27 |
| Current matching / voltage matching / equal V_DS | MOS-28, MOS-29, MOS-30 |
| Thermal and stress gradients at die level | MOS-31, MOS-32 |
| Sense ratio inside a power array | MOS-33 |

---

## 1. Technique cards

### MOS-01
- **id**: MOS-01
- **technique**: Finger folding. Split a wide device into N identical parallel sections (fingers) that share source/drain diffusions, notation N(W/L).
- **realizes**: compact aspect ratio; lower drain junction capacitance (shared S/D cuts junction capacitance by up to 50 %) (H12-27); gate RC inside a switching-delay budget (H13-11); the "identical sections" precondition of matching rule 1 (H13-22).
- **applicability**: any MOS with W/L above about 10; below that use one section (H12-27). An annotated N(W/L) must be laid out exactly as given; otherwise refold with N·W_f = W (1000/0.5 → 20(50/0.5)) and tell the designer (H12-27). ESD devices: robustness depends on per-finger width, 20/0.5 → 2(10/0.5) "might halve" it, so lock W_f (H12-27). Switching devices: W_f ≤ W_max from gate RC; contacting both gate ends doubles W_max; wider devices split into side-by-side banks; rarely switching devices are exempt; over-wide gates switch non-uniformly and heat locally (H13-11). Matched devices: all members share one W_f (MOS-02) (H12-28).
- **parameters**: N; W_f; outer electrode (MOS-03); gate contact ends (MOS-13); finger pitch = max of the silicon bound L_d + W_C + 2S_PC, the M1 bound W_C + 2E_MC + S_M1 and the via bound W_V + 2E_M1V + S_M1 (H13-03); metal stack over fingers (H13-04).
- **guarantees**: every section is identical, so the width bias W_b is common to all fingers (H12-02); S/D diffusions are shared between neighbours (H12-27).
- **does not guarantee**: ESD robustness after a refold (H12-27); threshold uniformity if W_f falls below "a few µm" (narrow-channel effect) (H12-07); matching to another device unless W_f is shared (H12-28); uniform finger current when a finger is longer than 1.8·d_pen (H13-05).
- **pdk data needed**: minimum matched finger width W_nce_min (Hastings says only "a few µm"; deck value) (H12-07); poly sheet resistance and t_ox for W_max (H13-11); contact, M1 and via pitch rules (H13-03); a maximum finger width.
- **verification**: LVS with the folded nf or m card (H12-27); W_max = √(t_ox·t_pmax/(2.6·R_S·ε_ox)) (H13-11); every refold logged for designer sign-off (H12-27).
- **prefer over / degrade to**: single section for W/L < 10 (H12-27). Serpentine (MOS-11) when L ≫ W. Annular (MOS-12) when drain capacitance must be minimal. Banks of fingers when W exceeds 2·W_max (H13-11).
- **source**: H12-27, H12-02, H12-07, H12-28, H13-03, H13-04, H13-05, H13-11, H13-22.

### MOS-02
- **id**: MOS-02
- **technique**: Unit-section unitization. Build every member of a matched or ratioed set from one unit section (W_u, L_u): p units in parallel, s units in series, or both. The ratio is a count ratio.
- **realizes**: matching rule 1, identical sections (H13-22); cancellation of the process biases W_b and L_b, which only cancel between sections of identical drawn size (H12-02, H12-28); exact N:1 ratios (H13-22).
- **applicability**: any matched set (mirror, differential pair, DAC branch) of equal L (H12-28). Example: M1 100/0.5 and M2 2(100/0.5) become 4(25/0.5) and 8(25/0.5) (H12-28). Large ratios: the smaller device is series units, the larger parallel units; 20:1 = 4 units in series against 5 in parallel (H13-22). The figure description of Fig 13.41 confirms M1A–M1D in series and M2A–M2E in parallel (F99 §2.8). Series units carry a small inherent mismatch, so not for EXC (H13-22). Members of unequal L cannot be unitized by parallel count alone (H12-28).
- **parameters**: (W_u, L_u); per-member (p_i, s_i); candidate W_u = gcd(W_i)/k, bounded below by W_nce_min and above by the finger width limit (H12-28).
- **guarantees**: ratio fixed by integers; W_b cancels exactly (H12-02); for non-identical devices the penalty is known: s_Vt = c_Vt·√(1/(2W₁L₁) + 1/(2W₂L₂)) (eq 13.49) (H13-20).
- **does not guarantee**: the random mismatch magnitude, which is set by area (H13-20); orientation equality (MOS-15); the series stack's inherent mismatch (H13-22).
- **pdk data needed**: finger width limits; c_Vt per polarity to score the unit (H13-20).
- **verification**: LVS: member_i = p_i × s_i units; eq 13.49 for any pair that is not identical (H13-20).
- **prefer over / degrade to**: parallel-only units (equal L) → series plus parallel (large ratios, MIN/MOD) → non-unit devices with the eq 13.49 penalty reported (H13-20).
- **source**: H13-22, H12-28, H12-02, H13-20, F99 §2.8.

### MOS-03
- **id**: MOS-03
- **technique**: Finger parity and outer-electrode choice. Choose N odd (equal S and D counts) or even (one extra outer electrode), and choose which electrode is outside ("minimize drain" or "minimize source").
- **realizes**: minimum junction capacitance on the critical node (H12-29); the precondition for butting taps (MOS-06) (H12-30); for non-self-aligned drain-extended devices, an even number of sections with sources at both array ends cancels poly/well misalignment (H13-18).
- **applicability**: all multi-finger devices. Odd N gives equal S and D finger counts; even N gives one extra S or D with more junction capacitance (H12-29). PCells offer minimize-drain and minimize-source; designers "generally prefer to minimize the drain" (H12-29). Very high-speed small transistors are split into two sections to cut drain capacitance (H12-29). Devices whose V_DS changes sign (pass gates, switches, bleeders) have no fixed drain: source and drain are defined by bias, so drain-side rules apply to both diffusions (H12-01).
- **parameters**: parity of N; outer electrode ∈ {S, D}.
- **guarantees**: the outer electrode carries the extra finger's capacitance; with the drain minimized and N even, butting taps fit at both ends (H12-30).
- **does not guarantee**: that the drain is the node that should be minimized; choosing by node impedance is the digest's recipe, not Hastings' text (H12-29) *(inference)*.
- **pdk data needed**: none.
- **verification**: S versus D finger count; extracted node capacitance (PEX).
- **prefer over / degrade to**: minimize drain (default) → minimize source for a high-impedance source node *(inference)* → odd N for equal counts.
- **source**: H12-29, H12-30, H12-01, H13-18.

### MOS-04
- **id**: MOS-04
- **technique**: Diffusion sharing between non-matched devices (merged devices). Same-polarity, same-bulk devices that share an S or D net are merged into one diffusion; unequal widths use a notched moat.
- **realizes**: area reduction and lower shared-node capacitance; one nearby backgate contact serves several devices (H12-31).
- **applicability**: non-matched devices (or members of one matched group, which already share) with a common S/D net, same polarity and same bulk net (H12-31, H12-27). Unequal W: a notched moat whose poly-to-moat spacing S_PM enlarges the shared region slightly; still smaller and lower in capacitance than two devices (H12-31).
- **parameters**: merge set (a diffusion-sharing, Euler-path style search) (H12-31); notch geometry.
- **guarantees**: shared electrode area and capacitance below those of two separate electrodes (H12-31).
- **does not guarantee**: an equal environment for matched devices. Rule 12 wants each active gate to face poly at the active pitch with equal W and extension (H13-26); a merged foreign neighbour of different W or L breaks that *(inference)*.
- **pdk data needed**: S_PM; implant oversize and clipping rules where NSD and PSD abut (H12-12).
- **verification**: LVS of shared nets; DRC notch rules; an implant-overlap self-check at abutments (H12-12).
- **prefer over / degrade to**: equal-W full-pitch sharing → notched moat → separate devices joined in metal.
- **source**: H12-31, H12-12, H12-27, H13-26.

### MOS-05
- **id**: MOS-05
- **technique**: Series stack as a contact-free chain. Series devices share the internal diffusion without contacts; gates may be brought closer with 90° or 45° jogs.
- **realizes**: minimum internal-node capacitance and area (H12-32); the series-unit member of a ratioed set (H13-22).
- **applicability**: series pairs whose internal net has no other connection (H12-32); the smaller device of a large ratio (H13-22); cascode stacks (MOS-30).
- **parameters**: jog angle; poly-to-poly spacing; number of series units s.
- **guarantees**: "the drain of M3 simultaneously acts as the source of M4", so the node needs no contact (H12-32).
- **does not guarantee**: exact equivalence of series and parallel units (small inherent mismatch) (H13-22). Gate jogs change the poly extension, which rules 21–22 want uniform for MOD/EXC (H13-54), so do not jog matched gates *(inference)*.
- **pdk data needed**: poly spacing and jog rules.
- **verification**: ERC: the internal node has no external connection; LVS.
- **prefer over / degrade to**: chain → contacted internal node joined in metal.
- **source**: H12-32, H13-22, H13-54.

### MOS-06
- **id**: MOS-06
- **technique**: Abutting (butting) backgate contact. A backgate-contact strip on the source net abuts the outer source finger.
- **realizes**: low backgate resistance (pins the backgate "more effectively" than a separate strip) and area (saves the NMoat–PMoat spacing) (H12-30).
- **applicability**: only when backgate net = source net (H12-30, H12-41). Odd N: one end. Even N with minimized drain: either or both ends. Minimized source: none (H12-30). Otherwise keep the NMoat–PMoat spacing (H12-30).
- **parameters**: ends ∈ {left, right, both}.
- **guarantees**: pinning and the spacing saving (H12-30).
- **does not guarantee**: coverage of the 25–250 µm maximum-distance rule across a wide device (MOS-07) (H12-43); symmetric end environments when only one end is butted, which rule 12's equal-environment requirement asks for (H13-26) *(inference)*.
- **pdk data needed**: butting-contact rules; implant derivation and clipping (PSD = PMoat oversized 1.0, clipped where common to NMoat, and the Active+Tap equivalents) (H12-12).
- **verification**: ERC bulk = source; DRC on the abutted implants (H12-12); distance check (H12-43).
- **prefer over / degrade to**: butting → separated tap strip at NMoat–PMoat spacing (H12-30) → ring.
- **source**: H12-30, H12-41, H12-12, H12-43, H13-26.

### MOS-07
- **id**: MOS-07
- **technique**: Backgate contact coverage. Choose among a side strip, two-sided strips, interdigitated strips, distributed plugs inside source fingers, finger-end taps, or sublayer contacts so that every point of the transistor lies within the allowed distance of a same-bulk contact.
- **realizes**: latch-up immunity through low backgate resistance R_B (H12-38); snapback margin, BV_DII ≥ maximum operating voltage for pass and power devices, which "requires backgate resistance be minimized" (H12-10); electrical and electrothermal safe operating area (H13-12).
- **applicability**: every MOS needs a backgate contact; otherwise the parasitic SCR can latch (H12 §12.2.9, H12-38). Without a sublayer (P-epi on P−, N-well without NBL, isolated tanks): contact every transistor; the maximum distance from any point in the transistor to the nearest contact "usually falls somewhere between 25 and 250 µm" by sheet resistance and expected current; the exercise uses 50 µm; exempt if the backgate touches NBL or the device cannot inject (always linear) (H12-41, H12-43). Over a heavily doped sublayer (P+ substrate, N-well touching NBL, about ≤ 10 Ω/□): vertical conduction dominates unless the device is very small, a large distant substrate-contact area beats a small adjacent one, contact the sublayer; a deep-N+ ring around PMOS over NBL also blocks hole injection; extra deep-N+ stripes for large currents; retrograde wells do not count as a sublayer (H12-41, H13-13). Interdigitated strips reduce distance but "substantially increase the size"; distributed plugs in source fingers (bulk = source only) add a little source resistance and "greatly reduce the area", on every source finger or every k-th; plugs must survive outdiffusion and misalignment (H12-44, H13-13). Power devices: contacts at the ends of each source finger remove field intensification there; distributed contacts minimize effective R_B against electrothermal runaway (H13-12). Isolated NMOS is immune to CMOS latch-up but can still suffer lateral NPN snapback at high voltage, which distributed contacts mitigate (H12 §12.2.9, H12-10).
- **parameters**: style; k (every k-th finger); plug size; W compensation for the added source resistance (H12-44); sublayer contact area.
- **guarantees**: the distance rule holds by construction for the chosen k (H12-44); R_B reduced (H12-38).
- **does not guarantee**: a numeric latch-up margin unless eq 12.33 is evaluated with V_trig at T_max (H12-38, H12-40); matched-device equivalence when plugs sit inside source fingers; Hastings says nothing on that *(inference)*.
- **pdk data needed**: maximum backgate-contact distance per well type (H12-43); substrate type, sublayer sheet resistance, N-well-touches-NBL flag (H12-41, H13-13); plug minimum size and enclosure (H12-44); JESD78 test current class (H12-38).
- **verification**: distance transform over the final GDS, every diffusion point within the limit of a same-bulk tap (H12-43); eq 12.33 margin > 0 at T_max (H12-38, H12-40); ERC: pass/power devices V_DS ≤ BV_DII (H12-10, H13-12).
- **prefer over / degrade to**: sublayer plus large distant contacts (when a sublayer exists) → butting (MOS-06) → distributed plugs (bulk = source) → interdigitated strips → side strip plus a second side when the limit is exceeded (H12-43).
- **source**: H12-38, H12-40, H12-41, H12-43, H12-44, H12-10, H13-12, H13-13.

### MOS-08
- **id**: MOS-08
- **technique**: Backgate pinning and guard rings between injector and victim. Majority-carrier contacts (pinning taps) on the victim's side facing an injector, across every direct path, as close to the victim as possible; minority-carrier guard rings only around injectors.
- **realizes**: latch-up immunity. Unconditional: β12·β21·(1−ηc12)(1−ηc21) < 1 (eq 12.32). Conditional: I_T·R_B2·(1−ηc12)·β12/(β12+1) < V_trig (eq 12.33) (H12-38). A ring of majority-carrier contacts satisfies eq 12.33; minority-carrier guard rings satisfy eq 12.32; guard rings reduce the η terms and backgate contacts reduce R_B (H12-38, H12-42).
- **applicability**: injectors are devices with a terminal on an external pin, bleeders whose backgate diode conducts more than "a few hundred µA", and switching drivers (H12-39). Guard rings "require so much room" that they go only around the few injecting devices; backgate contacts go on every device (H12-39).
- **parameters**: tap position (facing side, within a deck distance of the victim edge); partial or full ring; ring type (majority or minority carrier).
- **guarantees**: the eq 12.33 or eq 12.32 form, by ring type (H12-42).
- **does not guarantee**: margins at high temperature unless derated: V_trig ≈ 0.65 V at 25 °C and ≈ 0.4 V at 150 °C, and parasitic β rises with temperature (H12-40).
- **pdk data needed**: η per ring type (deck calibration); I_T (JESD78: ±100 mA to ±250 mA); backgate sheet resistance; V_trig(T) (H12-38, H12-40).
- **verification**: eq 12.33 margin per injector–victim pair (H12-38); geometric: the tap intersects every straight line between the two bounding boxes (H12-42).
- **prefer over / degrade to**: pinning taps on the facing side (cheap) → partial ring on facing sides → full minority-carrier guard ring on the injector (area).
- **source**: H12-38, H12-39, H12-40, H12-42.

### MOS-09
- **id**: MOS-09
- **technique**: Isolated-NMOS tank construction and tank grouping. An NMOS whose bulk is not the substrate sits in a P-tank (deep N-well to NBL, deep-N+ sinker, up-down isolation, or deep trench), and tanks are partitioned by injection source.
- **realizes**: isolated backgate (a five-terminal device) (H12-14); separation of noise injectors from sensitive devices; avoidance of parasitic rectification (H12-16, H12-17).
- **applicability**: NMOS bulk net ≠ substrate net (H12-14). Before layout, identify every isolated NMOS that can inject electrons; devices whose drains share one pin may share a tank; devices on different pins should not "unless one can demonstrate that this is not an issue"; never merge fast-switching ("noisy") isolated NMOS with sensitive analog, because parasitic rectification shifts DC points and bandgaps are "notoriously vulnerable" (H12-16, H12-17). The isolation terminal must be biased at or above the backgate (H12-14). Scatter numerous tank contacts with a maximum device-to-contact distance; designers often contact or ring every isolated transistor (H12-18).
- **parameters**: tank partition (bulk net × injection key); tank contact count and spacing; deep-N+ plug to cut the deep N-well's high vertical resistance (H12-14); isolation bias net (highest supply or the bulk) (H12-14).
- **guarantees**: tank membership derived from connectivity (H12-16).
- **does not guarantee**: immunity to lateral NPN snapback at high voltage (H12 §12.2.9, H12-10); well-to-isolation voltage limits: backgate-to-isolation ≤ 5–10 V, about 20 V with PBL; shallow N-well breakdown 20–30 V; deep isolation 40–80 V (H12-15).
- **pdk data needed**: isolation layers; breakdown values bv_well_iso, bv_nwell_sub, bv_iso_sub (H12-15); tank-contact maximum distance (H12-18).
- **verification**: ERC V(iso) ≥ V(bulk) across pin states (H12-14); |ΔV| against the breakdown limits (H12-15); contact distance (H12-18).
- **prefer over / degrade to**: one tank per (bulk net, pin group) → shared tank only for same-pin devices → never noisy plus sensitive (hard).
- **source**: H12-14, H12-15, H12-16, H12-17, H12-18, H12-10.

### MOS-10
- **id**: MOS-10
- **technique**: Well merging, well spacing by voltage class, and source-tied private wells. Same-bulk-net cells abut so their wells form one contiguous region; wells of different voltage classes keep class spacing; a matched PMOS whose source is not its bulk gets its own well tied to its source.
- **realizes**: area (no well-to-well spacing) (H12-33); correct spacing per well voltage (H12-13); zero and equal V_SB across matched members (H12-04).
- **applicability**: standard-cell style rows abut wells into one region; tapless libraries insert tap cells at intervals (H12-33). A generated P-well is the drawn N-well inverted and undersized by 2.0; each N-well voltage class gets its own spacing (NwellLV oversized 2.0, NwellHV oversized 3.0 in the example) (H12-13). Body bias raises |Vt| (NMOS 0.17, 0.43, 0.61, 0.89 V at V_BS 0, −1, −2, −4 V) and "increases the random variation of the threshold voltage" (H12-04). V_BS = 2 V on a 5 V device raises c_Vt about 30 %: avoid body bias on critical pairs (H13-20). Cascode backgates tie to their own sources (H13-34).
- **parameters**: merge set; tap-cell interval (MOS-07 distance); per-member private well.
- **guarantees**: merged wells remove the spacing (H12-33); a private source-tied well gives V_SB = 0 (H12-04).
- **does not guarantee**: WPE clearance: a merged well's edge must still be 3–5 µm or more from matched gates (H13-28, MOS-19); compactness when members sit in separate wells (see §3).
- **pdk data needed**: well spacing per voltage class (H12-13); well junction depth (H13-53).
- **verification**: well-net voltage class against spacing; V_SB per matched member from the operating point (H12-04).
- **prefer over / degrade to**: shared well (same bulk net) → separate wells at class spacing. For matched pairs with V_SB ≠ 0: private wells, accepting the WPE and compactness cost.
- **source**: H12-33, H12-13, H12-04, H13-20, H13-34, H13-28.

### MOS-11
- **id**: MOS-11
- **technique**: Serpentine long-channel MOS. A moat strip folded under one poly plate; each 90° bend adds W/2 to the drawn length, L = 2L_X + L_Y + W for the Fig 12.29 shape.
- **realizes**: compact very long L for trickle current sources (H12-35); high output resistance, since r_o scales about linearly with drawn L when backgate doping is uniform and analog lengths of 20–40 µm "are not uncommon" (H12-06).
- **applicability**: W/L ≪ 1. Serpentines "will not match very accurately unless they use identical layouts" (H12-35). Not useful in pocket-implanted processes, where r_o scales as √L (H12-25, H12-06).
- **parameters**: segment count and lengths; number of bends.
- **guarantees**: exact drawn L from the bend formula (H12-35).
- **does not guarantee**: matching to any non-identical layout (H12-35); Pelgrom scaling with pockets (H12-25).
- **pdk data needed**: poly and moat corner rules.
- **verification**: LVS W/L with the computed L (H12-35).
- **prefer over / degrade to**: straight long device → serpentine identical copies → series chain of unit devices (MOS-05) when the long-L device must match others (H13-22).
- **source**: H12-35, H12-06, H12-25, H13-22.

### MOS-12
- **id**: MOS-12
- **technique**: Annular (enclosed) MOS. A gate ring encloses the drain; circular, square, or elongated; variants with gate ends crossing onto field or a tic-tac-toe grid gate.
- **realizes**: minimum drain capacitance per unit width (interdigitation already halves C_D/W) (H12-36); no channel edge at the field oxide, which removes stringer and subthreshold-hump mismatch (H13-32); gate contacts off the active gate (H12-37, H13-50).
- **applicability**: drain on a high-speed or high-impedance node with the source on a low-impedance rail; high-voltage devices, circular preferred because square corners avalanche early; the elongated form is common for HV but is not a C_D/W gain (H12-36). Where contacts over active gate are banned: elongated gate crossing onto field with W = 2W_S (W_S between moat edges), or a grid gate with W ≈ 4W_S and four S/D regions (H12-37). Matching only between identical copies (H12-36, H12-37). Annular JFETs use the same equations (H12-49).
- **parameters**: shape; A, B (circular) or C, D (square); elongation U or V; gate-contact style.
- **guarantees**: closed forms: circular W = π(B−A)/ln(B/A), L = (B−A)/2, approximately W ≈ ½π(A+B); square W ≈ 2(C+D), L ≈ (D−C)/2; elongated circular W ≈ π(B−A)/ln(B/A) + 2U; elongated square W ≈ 2V + C + D (eqs 12.24–12.31) (H12-36).
- **does not guarantee**: matching between different shapes; HV robustness with square corners (H12-36).
- **pdk data needed**: contact-over-active-gate flag (H12-37); allowance for 45° or curved poly.
- **verification**: LVS W/L from the formulas (H12-36); contact ∩ active gate = ∅ (H13-50).
- **prefer over / degrade to**: for low drain capacitance: annular → two-section fingers (drain capacitance roughly halves, H12-29) → single section. For HV: circular → elongated circular → square.
- **source**: H12-36, H12-37, H12-29, H13-32, H13-50, H12-49.

### MOS-13
- **id**: MOS-13
- **technique**: Gate contacting and strapping. Contacts on poly pads beyond the diffusion, never on the active gate; every gate (dummies included) extended 1–2 µm beyond the overhang rule, all equal; fingers joined in metal rather than a poly comb; both gate ends contacted on wide switching fingers.
- **realizes**: rule 16, no contacts on the active gate (H13-50); rule 21, equal corner rounding (rounding reaches 1–2 µm) (H13-54, H13-26); rule 22, no poly-comb microloading (H13-54); gate RC: both-end contact doubles W_max (H13-11).
- **applicability**: rule 16 applies to all matched transistors in every class, even where DRC allows contacts on gate (H13-50). Rules 21–22 apply to MOD and EXC; MIN is exempt (H13-54). Poly combs are acceptable for all but the most accurate matching if the connecting poly lies 1–2 µm or more from the moat (H13-26, H13-54). Dummies get the same extension (H13-26).
- **parameters**: extra extension (1–2 µm, equal for all fingers including dummies); strap ∈ {metal with per-finger pads, poly comb ≥ 1–2 µm from moat, poly strap}; contacted ends ∈ {one, both}.
- **guarantees**: no gate extends further than its neighbours (H13-54); no contact over poly ∩ moat (H13-50).
- **does not guarantee**: anything about metal passing over the gate (MOS-22).
- **pdk data needed**: poly overhang rule; poly sheet resistance and t_ox (H13-11); contact rules.
- **verification**: contact ∩ (poly ∩ diff) = ∅ over matched devices (H13-50); all gate extensions equal (H13-54); t_p ≈ 2.6·W²·R_S·ε_ox/t_ox against the delay budget (H13-11).
- **prefer over / degrade to**: metal strap plus pads (MOD/EXC) → poly comb ≥ 1–2 µm from moat (all but EXC) → poly strap at minimum spacing (MIN).
- **source**: H13-50, H13-54, H13-26, H13-11.

### MOS-14
- **id**: MOS-14
- **technique**: Threshold-flavour and gate-oxide marker layers with same-flavour clustering. Draw the flavour marker (NatVt, LVT/HVT, Moat-2 for thick oxide) around every gate including dummies; cluster same-flavour devices so markers merge.
- **realizes**: the device flavour by construction; marker spacing rules paid once per cluster (H12-21, H12-23).
- **applicability**: natural, adjusted-threshold and thick-oxide devices (H12-21, H12-23). Natural (unadjusted) devices match better because dopant fluctuation dominates (H13-20). Members of a matched set share one flavour (rule 1, identical sections) (H13-22).
- **parameters**: marker enclosure around the gate, slightly overlapping the channel for misalignment and outdiffusion (H12-21); cluster set.
- **guarantees**: the flavour applies to every gate inside the marker (H12-21).
- **does not guarantee**: matching across flavours.
- **pdk data needed**: example rules: NATVT width 4 µm, overlap of GATE 2 µm, spacing to POLY 4 µm, NATVT-to-NATVT 2 µm, where GATE = POLY ∩ (NMOAT ∪ PMOAT) (H12-21); Moat-2 enclosure and spacing (H12-23); for dual-doped poly, PPoly/NPoly junctions "well away from gate regions" and every poly lead doped (H12-22).
- **verification**: DRC marker enclosure; LVS flavour; poly ⊆ NPoly ∪ PPoly where the deck has those layers (H12-22).
- **prefer over / degrade to**: clustered markers → isolated markers.
- **source**: H12-21, H12-22, H12-23, H13-20, H13-22.

### MOS-15
- **id**: MOS-15
- **technique**: Orientation lock. All matched devices drawn with parallel channels and equal orientation metric Φ, inside one cell; non-self-aligned devices superimposable by translation only; no 90° rotation of any device or block in directional-halo processes.
- **realizes**: rule 7, same orientation (H13-25); Table 13.2 rule 5, orientation (H13-40); integrity of directional pocket implants (H12-26).
- **applicability**: rotated devices mismatch by several % in current, up to 5 % on off-axis (tilted) wafers even for nominally equivalent axes (H13-25). Φ_i = +1 for current flowing to the right, −1 to the left; Φ = (1/N)·ΣΦ_i; equal sign and magnitude is required: (3−1)/4 = (9−3)/12 = ½ matches, but ½ ≠ −½; 2-D arrays need both Φ_H and Φ_V equal (H13-25). Equal Φ is required for EXC and for any class with non-self-aligned channels (asymmetric DEMOS); equal or nearly equal for MOD self-aligned devices (H13-25). Tilted S/D implants and non-self-aligned layers make left-drain and right-drain sections differ (H13-41). Rotating a cell that holds one member by 90° at top level silently creates a 90° mismatch, so keep matched devices in the same cell (H13-25). Directional pockets shot left and right only: horizontal-L devices get pockets and vertical-L devices do not; "a block can safely be reflected or rotated by 180°, but it cannot be rotated by 90°", and this applies to all devices (H12-26). No figure states the halo angle; Fig 13.43 shows device pairs at 0°, 45° and 90° (F99 §2.6, F99 §4).
- **parameters**: per-unit φ = (φ_x, φ_y); allowed transform set {translate, mirror, R180} or {translate} only.
- **guarantees**: no orientation mismatch when Φ is equal (H13-25).
- **does not guarantee**: equal stress sensitivity, which is axis dependent (PMOS least sensitive along ⟨100⟩ = 45°, NMOS along ⟨110⟩ = X/Y) (H13-38); Φ equality of separately placed members unless checked after top-level transforms (H13-25).
- **pdk data needed**: halo type and axis (none, blanket, directional) (H12-25, H12-26); wafer orientation.
- **verification**: Φ_H and Φ_V per member on the final layout, fail MOD/EXC on inequality (H13-25); no quarter-turn moves in directional-halo decks (H12-26).
- **prefer over / degrade to**: identical orientation with equal Φ (EXC, non-self-aligned) → parallel channels with nearly equal Φ (MOD) → parallel channels only (MIN, self-aligned).
- **source**: H13-25, H13-40, H13-41, H12-26, H12-25, H13-38, F99 §2.6, F99 §4.

### MOS-16
- **id**: MOS-16
- **technique**: Full dummy gates (rule 12). Tied-off poly gates at each end of the array, geometrically identical to the active gates, with their own outer S/D region.
- **realizes**: equal poly etch for end gates (microloading: end gates of an array erode more and come out shorter) (H13-26); equal LOD stress (H13-47, H13-29); well edges pushed away from end devices (H13-28).
- **applicability**: MIN optional ("good idea if space permits"); MOD generally; EXC always (H13-47). Microloading decays exponentially, seldom significant beyond 3–5 µm; very accurate matching considers up to 10 µm (H13-26). MOD with L below about 3 µm: one full dummy per end. EXC with L below about 10 µm: one or more full dummies so the outer poly edge of the outermost dummy is at least 10 µm from the nearest active device (H13-47). §13.2.2: if L ≥ 3–5 µm one dummy per end suffices, else two or more (H13-29). Dummy L = active L when active L is below the reach, otherwise the dummy L may be capped at the reach (H13-26). Dummy gates tied off, usually to the backgate (H13-26). LDMOS outer fingers differ by up to 10 % (first-finger effect), so matched or sense fingers are never outermost (H13-31).
- **parameters**: dummies per end; dummy L; tie net; outer S/D contacted or not.
- **guarantees**: geometric equivalence: dummy-to-active spacing equals active-to-active spacing; dummy width equals active width; same poly extension beyond the moat; off state (H13-26). Proper dummies also cancel LOD (H13-47).
- **does not guarantee**: the side-edge WPE random component (H13-28); gradients; DRC or LVS may flag an uncontacted outer S/D, in which case contact it (H13-26).
- **pdk data needed**: microloading reach by class (3–5 µm, 10 µm); poly minimum length; moat-over-poly overlap (H13-26, H13-47).
- **verification**: geometric comparison of dummy against active (pitch, W, extension); reach inequality measured from the outermost dummy's outer poly edge; LVS dummy handling (H13-26).
- **prefer over / degrade to**: full dummies by reach → half dummies (MOS-17) → moat extension only (MOS-18) → none (MIN).
- **source**: H13-26, H13-47, H13-29, H13-28, H13-31.

### MOS-17
- **id**: MOS-17
- **technique**: Half dummies. Dummy poly over the end of the moat with a single S/D termination and no outer diffusion.
- **realizes**: microloading shielding at lower area (H13-26); the dummy form used in the cross-coupled quad (H13-41).
- **applicability**: L above 3–5 µm and less accurate matching (H13-26). Rule 12: MOD with L above about 3 µm may use half dummies; the digest records the poly-width condition as "poly width ≤ 3 µm needed", while §13.2.2 puts the half-dummy minimum poly length at moat-over-poly overlap plus poly-over-moat overhang and "not narrower than 2–3 µm" (H13-47, H13-26). EXC with L above about 10 µm: half dummies with poly width ≥ 10 µm (H13-47).
- **parameters**: poly width.
- **guarantees**: shielding over its own width.
- **does not guarantee**: LOD and WPE push-out; half dummies "may let moat/well edges approach the active gate", so MOS-18 and MOS-19 must be applied separately (H13-26); not for the most accurate matching (H13-26).
- **pdk data needed**: poly minimum length rules.
- **verification**: as MOS-16.
- **prefer over / degrade to**: full dummy → half dummy → none.
- **source**: H13-26, H13-47, H13-41.

### MOS-18
- **id**: MOS-18
- **technique**: Active-area extension past the outermost gate (LOD and OD-to-OD stress). Stretch the end moats, or let the dummy's outer S/D supply the length, so STI stress at end devices equals that at inner devices.
- **realizes**: equal STI stress. The densification oxide (2.2× silicon volume) compresses channels: NMOS k down and PMOS k up by 10 % or more, Vt shifts of 10 mV or more; widening S/D reduces it (H13-29).
- **applicability**: STI processes. Less accurate matching: 2–3 µm; optimal: moat 3–5 µm or more beyond the last active transistor; one dummy per end suffices if L ≥ 3–5 µm, else two or more (H13-29). Rule 12 for STI: MOD moat at least 5 µm past the last active gate (a full dummy with a 2–3 µm channel may be the best choice); EXC at least 10 µm (a full dummy with an 8–10 µm gate may be best) (H13-47). OD-to-OD stress (trench width to neighbouring active) is fixed the same way (H13-29).
- **parameters**: moat extension per end (SA, SB); trench width to the neighbouring active.
- **guarantees**: end devices see the inner devices' transverse stress once the extension is reached (H13-29).
- **does not guarantee**: microloading, which needs poly (MOS-16); die-level stress gradients (MOS-32).
- **pdk data needed**: SA/SB stress model parameters; piezoresistance Table 13.1 in 10⁻¹¹ Pa⁻¹: ⟨100⟩ PMOS π_L −15, π_T 30, NMOS −90, 40; ⟨110⟩ PMOS 65, −50, NMOS −30, −20 (H13-29); class-tiered moat extension (H13-47).
- **verification**: SA/SB per unit; compare member means (H13-29).
- **prefer over / degrade to**: dummy whose S/D provides the length → bare moat extension → none (MIN).
- **source**: H13-29, H13-47.

### MOS-19
- **id**: MOS-19
- **technique**: Well-edge and deep-diffusion clearance (WPE, rule 19). Keep drawn well edges and deep diffusions a class-dependent distance from matched active gates; push end edges out with end dummies; widen devices or move side edges out.
- **realizes**: equal well-proximity doping. Retrograde MeV well implants scatter off resist edges and dope the silicon near well edges more heavily; the effect reaches 3–5 µm and is worst within 1–2 µm; 5 % current mismatch was measured at 1.8 µm gate-to-well-edge spacing and 25 % at 0.95 µm (H13-28). Deep diffusion tails (deep-N+ sinkers, wells) shift Vt and k (H13-53).
- **applicability**: EXC: well boundary to active gate at least 5–10 µm or at least twice the well junction depth, whichever is greater; the same for deep-N+ and other deep diffusions. MIN and MOD: layout rules, except in WPE-prone low-voltage processes: MOD at least 3 µm, MIN at least 2 µm from drawn well boundary to active gate (H13-53). §13.2.2: add "several microns" to minimum spacings; keep N-well away from matched NMOS; keep matched PMOS well inside their N-well; with several voltage rules add several µm to the lowest-voltage rule (H13-53). End dummies push well edges away and remove the end-versus-middle systematic term, but side-edge doping still adds random mismatch, so widen devices or move side well edges out (H13-28). These distances are body text; no figure description states them (F99 §2.4, F99 §2.5).
- **parameters**: clearance per side; end dummies; device width; side well-edge position; spacing to foreign wells for matched NMOS.
- **guarantees**: removal of the end-versus-middle systematic mismatch (H13-28).
- **does not guarantee**: the side-edge random component unless devices are widened or side edges moved out (H13-28).
- **pdk data needed**: well junction depth; WPE reach; class-tiered clearance (H13-53).
- **verification**: per-unit distance to the nearest well edge, own well and foreign wells (H13-28, H13-53).
- **prefer over / degrade to**: EXC clearance → MOD → MIN; widening the device is the alternative when area is constrained (H13-28).
- **source**: H13-28, H13-53, F99 §2.4, F99 §2.5.

### MOS-20
- **id**: MOS-20
- **technique**: Buried-layer shadow keep-out. Enlarge the well or NBL so the NBL pattern-shift shadow misses matched PMOS gates.
- **realizes**: rule 20; no shadow across the active area of MOD/EXC transistors (H13-30).
- **applicability**: non-STI (LOCOS) processes with NBL. Pattern shift is at most 150 % of the epi thickness; use 150 % of the maximum epi, on all sides if the shift direction is unknown (H13-30). STI/CMP processes have no shadow (H13-30).
- **parameters**: well or NBL enlargement.
- **guarantees**: shadow outside the active area (H13-30).
- **does not guarantee**: anything in STI decks (not applicable).
- **pdk data needed**: NBL shadow shift (absent for STI) (H13-30).
- **verification**: shadow region ∩ active = ∅.
- **prefer over / degrade to**: binary; no ladder.
- **source**: H13-30.

### MOS-21
- **id**: MOS-21
- **technique**: Extraneous-poly exclusion halo and poly-0 keepaway (rule 23). A class halo around the matched array in which no foreign or generated dummy poly may lie; a long-range keepaway from poly-0.
- **realizes**: equal poly etch environment beyond the dummies (H13-55, H13-26); resist-spin uniformity against poly-0 topography (H13-27).
- **applicability**: EXC: no unconnected poly within 5–10 µm; block generated dummy poly there (§13.2.2: within 10–15 µm for very accurate matching, with no poly other than active and dummy gates). MOD: the same out to 3–5 µm. MIN: arbitrary poly may be adjacent if end dummies exist; without dummies keep other poly 3–5 µm away (H13-55, H13-26). Poly-0 (double-poly capacitor bottom plate) disturbs resist spin: up to 12 % mismatch beyond 30 µm, so place matched transistors at least 150 µm from poly-0 (H13-27).
- **parameters**: halo by class; poly fill-block marker.
- **guarantees**: no foreign poly inside the halo (H13-55).
- **does not guarantee**: the dummies' own function (MOS-16).
- **pdk data needed**: poly fill-block layer; presence of a poly-0 layer (H13-27).
- **verification**: foreign poly ∩ halo = ∅; centroid distance to poly-0 ≥ 150 µm (H13-27). The rule 23 distances are body text only (F99 §2.4).
- **prefer over / degrade to**: EXC halo → MOD halo → MIN with dummies.
- **source**: H13-55, H13-26, H13-27, F99 §2.4.

### MOS-22
- **id**: MOS-22
- **technique**: Metal-over-active-gate policy (rule 17). Choose between no metal over the active gate, one metal field plate over all devices, or strictly identical metal crossings.
- **realizes**: equal hydrogen passivation. Hydrogen passivates interface traps; metal blocks hydrogen diffusion and TiSi/TiW getter it; a covered gate against an uncovered one shows up to 20 % systematic current mismatch, from any metal layer (H13-36).
- **applicability**: MIN: metal may cross only if the pattern over every section is identical; better, an M1 field plate over the whole active area of all devices at minimum spacing from the S/D metal, with upper layers free to cross the plate. MOD: no field plate and no leads crossing the active areas. EXC: no plate, no leads; minimize unnecessary metal within 5–10 µm of the active area; leads passing within 5–10 µm must present a similar adjacent pattern to every section (H13-51). Best: no metal over the active gates of critical pairs, minimal adjacent metal, identical surroundings; next best: a plate over all devices (H13-36). A full plate still leaves edge gradients and higher random mismatch (H13-36). Floating-gate devices take no metal at all, for retention and UV reasons (H12-46).
- **parameters**: policy ∈ {none, identical crossing, M1 plate}; halo 5–10 µm; mirrored routing within the halo.
- **guarantees**: equal coverage per section (H13-51).
- **does not guarantee**: the plate's edge gradient and random term (H13-36).
- **pdk data needed**: the metal layers under the overcoat (H13-52).
- **verification**: metal ∩ active gate = ∅ for MOD/EXC; per-member metal coverage per layer within a 10 µm halo, maximum pairwise difference (H13-36).
- **prefer over / degrade to**: no metal plus identical surroundings (EXC) → no metal (MOD) → M1 plate (MIN) → identical crossings (MIN).
- **source**: H13-36, H13-51, H13-52, H12-46.

### MOS-23
- **id**: MOS-23
- **technique**: Dummy-metal fill blocking with symmetric compensating fill (rule 18). Fill-block pseudolayers over the matched array plus halo on every metal, then hand-crafted identical fill around each member to meet density.
- **realizes**: no automatic fill on matched devices; equal surrounding fill. Different fill nearby, even with none over the devices, gives up to 1 % current mismatch (H13-36, H13-52).
- **applicability**: MIN with a field plate may take fill above the plate. Otherwise draw dummy-block layers over the whole array enclosing all active areas; for EXC the block extends 5–10 µm beyond the active gate area in all directions, on every metal under the overcoat (H13-52). Identical custom fill out to several µm and similar fill out to 10 µm or more for critical pairs (H13-36). Keep blocked regions few and small; cover less-critical pairs with plates instead (H13-52).
- **parameters**: halo (0, 0, 5–10 µm by class); fill pattern; density target (example: at least 25 % in any 250 µm window; walking-window DRC samples windows, so instances can pass or fail differently) (H13-52).
- **guarantees**: foundry fill honours the markers (H13-52).
- **does not guarantee**: density compliance, which custom fill must restore (H13-52).
- **pdk data needed**: fill-block layer per metal; density rule and window (H13-52).
- **verification**: fill ∩ (array plus halo) = ∅; density check; coverage similarity (H13-36).
- **prefer over / degrade to**: block plus symmetric fill (MOD/EXC) → plate with fill over it (MIN).
- **source**: H13-52, H13-36.

### MOS-24
- **id**: MOS-24
- **technique**: Adjacent placement. Matched devices as close as possible, same orientation, no interdigitation.
- **realizes**: rule 8 for MIN (H13-45). Gradient mismatch is ΔP = α·P₁₂·d_CC·∇T_CC, the gradient along the centroid axis times the centroid distance (eq 13.57), so distance is the only lever here (H13-37).
- **applicability**: MIN: bias mirrors at most 3–4 mirrors from the reference resistor (H13-42, H13-45).
- **parameters**: distance; orientation.
- **guarantees**: nothing beyond a small centroid distance (H13-37).
- **does not guarantee**: gradient cancellation; equal environment (needs MOS-16).
- **pdk data needed**: none.
- **verification**: centroid distance.
- **prefer over / degrade to**: → interdigitated (MOS-25) → common-centroid (MOS-26).
- **source**: H13-45, H13-42, H13-37.

### MOS-25
- **id**: MOS-25
- **technique**: Interdigitated 1-D row with S/D pattern strings. Alternate the fingers of A and B along one row following a pattern string whose subscripts fix the source and drain positions.
- **realizes**: rule 8 for MOD, "adjacent, preferably interdigitated so centroids approximately align" (H13-45); 1-D common centroid; equal Φ by pattern choice (H13-41, H13-25).
- **applicability**: devices with a common source: Table 13.3 patterns such as (ᴅAₛBᴅBₛA)ⁱᴅ and (ₛAᴅAₛBᴅBₛAᴅA)ⁱₛ (H13-41). Fig 13.57 is ᴅAₛBᴅBₛAᴅ; the figure description confirms gates A, B, B, A between dummies with the drain in the middle (F99 §2.8). Without a common source: insert gaps, or interdigitate along the width, which is natural for common-gate mirrors under one poly plate (H13-41). Left-drain and right-drain sections differ under tilted implants, so choose strings with equal Φ (H13-41).
- **parameters**: pattern string; repeat count i; end dummies (MOS-16); gap handling.
- **guarantees**: centroids coincide along the row for symmetric strings; the linear gradient is cancelled (common centroid cancels only the linear gradient) (H13-41).
- **does not guarantee**: the quadratic residue, which grows with array size (H13-41); the aspect limits of rule 9 when fingers are many (current matching MIN ≤ 10:1, MOD ≤ 3:1, EXC square; voltage matching MIN ≤ 3:1, MOD and EXC square) (H13-46).
- **pdk data needed**: none.
- **verification**: Table 13.2 checks: coincidence, symmetry, Φ (H13-40).
- **prefer over / degrade to**: 2-D AB/BA (MOS-26) when units are near square → 1-D → adjacency (MOS-24).
- **source**: H13-41, H13-45, H13-25, H13-40, H13-46, F99 §2.8.

### MOS-26
- **id**: MOS-26
- **technique**: 2-D common-centroid array and cross-coupled quad. Split each member into segments arranged on both axes (AB over BA), for example the cross-coupled pair ᴅAₛBᴅ over ᴅBₛAᴅ with half dummies.
- **realizes**: Table 13.2 rules 1, 2 and 5: coincidence, symmetry about both axes, orientation (H13-40); rule 8 for EXC (H13-45); rule 10, prefer 2-D (H13-41).
- **applicability**: EXC always; MOD when 2-D is natural (H13-45, H13-41). A 2-D AB/BA array has about 60 % of the residual gradient mismatch of a 1-D ABBA with the same elements (finite-element result), so square-ish units go 2-D (H13-41). The cross-coupled pair satisfies orientation and uses half dummies (H13-41); the Fig 13.59 description shows the left column A over B, the right column B over A, with gaps D, S, D (F99 §2.7). Cross-coupling only helps under a real gradient; test chips without heat sources show no difference (H13-41). Schematic annotation vocabulary from exercise 13.13: "C/C" means cross-couple, "~" means match as well as sizes allow, parentheses give priority groups; the Fig 13.60 description lists groups such as MP6-(MP7-MP8) C/C and MN4-MN5-(MN6-MN7 C/C) (H13 §13.5, F99 §4).
- **parameters**: rows × columns; unit shape; half dummies; S/D assignment of gaps.
- **guarantees**: coincidence and both-axis symmetry; equal Φ_H and Φ_V (H13-25, H13-40).
- **does not guarantee**: the quadratic residue (H13-41); easy wiring, since "elaborate patterns" are hard to wire (H13-41).
- **pdk data needed**: none.
- **verification**: the five Table 13.2 checks per group (H13-40).
- **prefer over / degrade to**: dispersed subarrays (MOS-27) for more than two segments or large arrays → single 2-D → 1-D (MOS-25).
- **source**: H13-40, H13-41, H13-45, H13-25, F99 §2.7, F99 §4, H13 §13.5.

### MOS-27
- **id**: MOS-27
- **technique**: Dispersion into compact subarrays. Split a large matched array into several small cross-coupled subarrays, each a common-centroid unit.
- **realizes**: Table 13.2 rule 3 (dispersion) and rule 4 (compactness) (H13-40); the residual grows with subarray size squared (H13-41); rule 9 counts only subarray aspect ratios (H13-46); rule 14 on power ICs (H13-49).
- **applicability**: more than two segments: two groups ₛAᴅAₛBᴅBₛ over ₛBᴅBₛAᴅAₛ while roughly square, else several small cross-coupled subarrays (H13-41). An elongated area can still match well when divided into compact subarrays (H13-46). EXC on power ICs: many small common-centroid subarrays (H13-49).
- **parameters**: subarray count and size; inter-subarray wiring.
- **guarantees**: only the subarrays need to satisfy coincidence and symmetry (H13-40).
- **does not guarantee**: simple wiring (H13-41).
- **pdk data needed**: none.
- **verification**: per-subarray Table 13.2 checks (H13-40).
- **prefer over / degrade to**: dispersed → single 2-D → 1-D.
- **source**: H13-40, H13-41, H13-46, H13-49.

### MOS-28
- **id**: MOS-28
- **technique**: Current-mirror construction (current matching). Long-L unit devices, equal V_GS from a shared gate, overdrive of 100 mV or more, no pocket implants, equalized V_DS.
- **realizes**: current matching. I_D2/I_D1 ≅ (1 − Δk/k)(1 + 2ΔV_t/V_gst) (eq 13.41); Vt-dominated s_I ≈ g_m·s_Vt (H13-19); s_I = c_Vt·√(2I_D·k′)/L_e, so "current matching depends upon length" and width does not help at fixed I_D (H13-21).
- **applicability**: Table 13.5 channel lengths at I_D = 1 µA for 12, 5, 3.3, 1.8 V devices: NMOS MIN 24, 15, 11, 8.1 µm; MOD 79, 48, 37, 27; EXC 240, 140, 110, 81; PMOS MIN 14, 8.6, 6.6, 4.9; MOD 47, 29, 22, 16; EXC 142, 86, 66, 49; scale as I_D^−½ (H13-44, H13-21). Rule 4: where stringers exist (STI subthreshold hump), current-matched devices above MIN must run at V_GS − V_t ≥ 100 mV in all corners, or use stringer-resistant layouts (poly fins across channel edges, annular gates); mirrors with stringers mismatch far more in subthreshold (H13-32). Pocket implants: current matching typically needs L > 5 µm, so use no-pocket devices; "accurate current mirrors cannot be constructed using pocket-implanted transistors without resorting to resistive degeneration" (H13-23, H12-25). Channel-length modulation: I_D2/I_D1 ≈ 1 + λ·ΔV_DS with λ ≈ 1/(B·L_e·√N_B), B ≈ 0.1–0.2 V·µm^½; mirror lengths fell from 15–30 µm (20 V) to 5–10 µm (2 V); limit cascaded mirrors; use cascodes to equalize V_DS; source degeneration is impractical for MOS (H13-33). Common-gate mirrors interdigitate along the width under one poly plate (H13-41). Bias mirrors at most 3–4 from the reference resistor default to MIN (H13-42). A differential pair's load mirror takes its class from its weight g_m3/g_m1 in the offset composition s_Vio = √(s_Vt12² + (s_Id12/g_m1)² + (g_m3/g_m1·s_Vt34)² + (s_Id34/g_m1)²) (eq 13.60) (H13-39).
- **parameters**: L from class and I_D; unit (W_u, L_u) and counts (MOS-02); V_gst; pattern (MOS-25, MOS-26); cascode (MOS-30).
- **guarantees**: the random current mismatch of eq 13.48 for the chosen L (H13-21).
- **does not guarantee**: the systematic ΔV_DS error without a cascode (H13-33); hot-carrier drift of up to several tens of mV between devices at different V_DS (H13-34); stringer immunity below 100 mV overdrive (H13-32).
- **pdk data needed**: c_Vt and c_k per polarity, k′, pocket flag and L_c, stringer-prone flag (H13-20, H13-24, H13-23, H13-32).
- **verification**: L ≥ L_req = 6·c_Vt·√(2k′/I_D)/ε_class (H13-21); V_gst ≥ 100 mV in all corners (H13-32); ε_CLM = |g_ds·ΔV_DS/I_D| from the operating point (H13-33).
- **prefer over / degrade to**: cascoded long-L unit mirror (EXC/MOD) → long-L unit mirror interdigitated (MOD) → adjacent unit mirror (MIN) → resistive degeneration as a design change in pocket processes (H12-25).
- **source**: H13-19, H13-21, H13-44, H13-32, H13-23, H12-25, H13-33, H13-34, H13-41, H13-42, H13-39, H13-20, H13-24.

### MOS-29
- **id**: MOS-29
- **technique**: Differential-pair construction (voltage matching). Large-area thin-oxide units at low overdrive, zero body bias, 2-D common centroid (cross-coupled quad).
- **realizes**: voltage matching. ΔV_GS ≅ ΔV_t − V_gst·Δk/(2k) (eq 13.40); s_V ≅ √(s_Vt² + (V_gst/2 · s_k/k)²) ≈ s_Vt (H13-19); s_Vt = c_Vt/√(W_e·L_e) with units assumed within ±6·s_Vt, so "voltage matching depends upon area" (H13-20, H13-21).
- **applicability**: MOD is the default for general-purpose op-amp and comparator inputs; EXC for high-accuracy inputs (H13-42). Table 13.4 required active area in µm² for 12, 5, 3.3, 1.8 V devices: MIN 423, 56, 19, 5.8; MOD 4,700, 630, 220, 64; EXC 42,000, 5,600, 1,900, 580, assuming c_Vt = 1 mV·µm per nm of oxide and E_max 3.5, 4.0, 4.5 MV/cm (H13-43). Thin oxide preferred (rule 6); quadrupling area halves mismatch (H13-43). Little to gain from V_gst below about 100 mV (subthreshold); short devices waste area on S/D terminations (H13-19). Body bias: V_BS = 2 V on a 5 V device raises c_Vt about 30 %, so avoid it on critical pairs (H13-20, MOS-10). Keep W and L at 1 µm or more (edge terms matter below about 1 µm; rule 11) (H13-23). Pockets: s_Vt = c_Vt/√(W_e·min(L_e, L_c)) with L_c at most a couple of µm, so use L ≤ L_c and widen (H13-23). Natural devices match better (H13-20). Bias-temperature instability: PMOS comparator inputs that sit at different V_GS drift apart with no drain current; use clamp diodes or an NMOS input pair (H13-35). PMOS often shows 30–50 % more gm mismatch, some processes the reverse, so use measured data (H13-24). Weak-inversion pairs: ΔI/I = ΔV_t/(nV_T), raise the tier (H12-08). Room-temperature trim removes only about two-thirds of the −40 to 125 °C drift and cannot fix thermal gradients (H13-42, H13-39).
- **parameters**: unit and counts per member (MOS-02); V_gst; polarity; pattern (MOS-26); well tie (MOS-10).
- **guarantees**: the random offset of eq 13.46 for the chosen area (H13-20).
- **does not guarantee**: the systematic terms (gradient, LOD, WPE, metal), which the environment cards handle; BTI and HCI drift (H13-35, H13-34).
- **pdk data needed**: c_Vt per polarity (fallback 1 mV·µm per nm of t_ox, flagged as an estimate); body-bias factor; t_ox; pocket flag and L_c (H13-20, H13-23, H13-43).
- **verification**: W·L·p ≥ A_req = (6·c_Vt/ΔV_class)² (H13-43); V_SB = 0 (H12-04); W, L ≥ 1 µm (H13-23); composed offset by eq 13.60 (H13-39).
- **prefer over / degrade to**: EXC cross-coupled 2-D common centroid with the full environment → MOD interdigitated ABBA with dummies → MIN adjacent.
- **source**: H13-19, H13-20, H13-21, H13-23, H13-24, H13-35, H13-39, H13-42, H13-43, H12-04, H12-08.

### MOS-30
- **id**: MOS-30
- **technique**: Cascode for V_DS equalization and equal stress. Cascode devices make matched devices see equal and lower V_DS; cascodes are sized in the same ratio; cascode backgates tie to their own sources.
- **realizes**: no channel-length-modulation systematic error (H13-33); no differential hot-carrier drift of Vt and gm, which otherwise accumulates "up to several tens of millivolts" between devices at different V_DS (H13-34); equal lifetime stress, since unequal |V_DS| and I_D mean unequal HCI ageing (H12-05) *(inference from the stated mechanism)*.
- **applicability**: mirrors and matched devices whose V_DS would otherwise differ (H13-33, H13-34). Ratio identity: (W₁/L₁)/(W₂/L₂) = (W₃/L₃)/(W₄/L₄) (eq 13.55); with L₁ = L₂ and L₃ = L₄, W₁/W₂ = W₃/W₄ (eq 13.56) (H13-34). Cascode backgates tied to their own sources so impact-ionization current cannot skew the ratio (H13-34). Layout as a chain (MOS-05).
- **parameters**: cascode sizes; bulk ties; chain layout.
- **guarantees**: V_DS1 = V_DS2 when eq 13.55 holds (H13-34).
- **does not guarantee**: random mismatch, still set by area and length (H13-20, H13-21); the cascode pair's own matching, which is a matched set of its own *(inference)*.
- **pdk data needed**: none beyond the matching data.
- **verification**: eq 13.55/13.56 identity on the netlist; cascode bulk = source (H13-34); operating-point |V_DS| equality across members (H13-33, H12-05).
- **prefer over / degrade to**: cascode → longer L and fewer cascaded mirrors (H13-33) → accept and report the ΔV_DS error.
- **source**: H13-33, H13-34, H12-05.

### MOS-31
- **id**: MOS-31
- **technique**: Thermal-gradient placement. Matched groups on a symmetry axis through the heat sources, at a class-dependent distance from power devices, as compact common-centroid subarrays.
- **realizes**: cancellation of thermal gradients: ΔP = α·P₁₂·d_CC·∇T_CC (eq 13.57) (H13-37). TC_Vt ≈ −2 mV/°C, bias independent (H13 §13.2.5); about −1.5 mV/°C at 100 Å and −2.5 mV/°C at 500 Å oxide; mobility ∝ T^−1.7 (electrons) and T^−1.5 (holes), with k at 150 °C about 60 % of k at 25 °C (H12-03).
- **applicability**: rule 14, where a power IC dissipates 1 W or more. EXC: power device at one die end on an axis, matched devices at the other end on an axis about three-quarters of the way from centre to the far edge (thermal against stress compromise); consider a 2:1 to 3:1 die; many small common-centroid subarrays. MOD: away from multiwatt devices (opposite end), interdigitated or 2-D common centroid. MIN: within a few hundred µm of multiwatt devices if interdigitated or cross-coupled. Avoid positions near power-device corners, where gradients are neither horizontal nor vertical. EXC with even a 250 mW device needs large separation. MOD near small power devices: common centroid and at least 1 µm per mW, with one-half to one-quarter of that usually sufficient. MIN adjacent to small power devices if properly common-centroid (H13-49). Keep matched MOS away from power devices and on the symmetry axis through heat sources (H13 §13.2.5). Switching devices dissipate P_avg = I_D·V_DS·t_s·f_s (10 A, 5 V, 10 ns, 1 MHz give 0.5 W), so they count as heat sources (H13-10). Weak-inversion pairs (exponential in ΔV_t) and GIDL-critical off devices belong in the cold region (H12-08, H12-09). Cross-coupling pays only under a real gradient (H13-41). Trim cannot fix thermal gradients (H13-42).
- **parameters**: distance per mW; axis alignment; subarray count; heat-source list from DC plus switching power.
- **guarantees**: the linear gradient is cancelled by common centroid; the residual grows with array size squared (H13-41).
- **does not guarantee**: the quadratic residue; agreement with the stress optimum, since the die centre is best for stress and the three-quarter rule is a compromise (H13-49, H13-48).
- **pdk data needed**: TC_Vt and mobility TC (H12-03); a thermal model.
- **verification**: ΔT between members against the budget derived from the offset spec (H12-03); distance ≥ k_class·P (H13-49).
- **prefer over / degrade to**: EXC opposite-end axis with many subarrays → MOD distance plus common centroid → MIN adjacent interdigitated.
- **source**: H13-37, H13-49, H12-03, H13-10, H12-08, H12-09, H13-41, H13-42, H13 §13.2.5.

### MOS-32
- **id**: MOS-32
- **technique**: Die stress zones, symmetry axes and crystal-axis choice (rules 13, 15, 24). Keep matched devices out of high-stress die zones, on die symmetry axes, Manhattan-oriented; diagonal PMOS only for EXC under heavy stress.
- **realizes**: minimal stress gradient. Stress barely moves Vt but moves k: Δk ≈ −k·(π_Lσ_L + π_Tσ_T + π_LTτ_LT), with π_LT ≈ 0 on the usual axes; higher V_GS reduces the NMOS ⟨100⟩ coefficients by up to 50 % (H13-38). Oxide-thickness gradients are negligible for proper layouts (5 % radial over a 200 mm wafer, about 0.5 ppm/µm) (H13-37).
- **applicability**: leaded plastic package: the lowest stress gradient lies from the die centre halfway to the edges. MIN: not in corners, not within 50–100 µm of die edges. MOD: interior, or near the centre of one side inset 100–250 µm or more. EXC: interior, on a die symmetry axis. Bumps or pillars: gradients peak at bump perimeters, minima at bump centres and midway between bumps; MOD well inside a bump footprint or midway between bumps on their axis; EXC on the axis between bumps (H13-48). NMOS is least stress-sensitive along ⟨110⟩, the layout X/Y axes; PMOS along ⟨100⟩, which is 45°, rarely supported by editors and rules, so diagonal is "not generally recommended"; rule 24 allows diagonal PMOS only for EXC under significant stress (large die, solder or eutectic die attach), never with directional pocket implants, and only if the process permits (H13-38). Avoid die edges and corners; the die centre has the lowest gradient; sit on a die symmetry axis; use common centroid (H13 §13.2.5).
- **parameters**: forbidden bands; axis attraction; bump map; device orientation.
- **guarantees**: zone rules only; no numeric gradient bound is given (H13-48).
- **does not guarantee**: the thermal optimum (MOS-31).
- **pdk data needed**: piezoresistance table (H13-29); die and bump context.
- **verification**: zone membership; distance to the die axis.
- **prefer over / degrade to**: EXC axis in the interior → MOD inset → MIN edge band; Manhattan orientation → diagonal PMOS (EXC with heavy stress only).
- **source**: H13-38, H13-37, H13-48, H13-29, H13 §13.2.5.

### MOS-33
- **id**: MOS-33
- **technique**: SenseFET. A small sense transistor sharing gate and source with a power FET, formed from interior sections of the same array.
- **realizes**: a current-sense ratio set by section count that replicates the main device's temperature and metal debiasing (H13-16).
- **applicability**: a full-finger sense in the middle finger bisects the hot and cool regions; 100 sections give 49:1. Partial-finger: moat openings segment a drain finger at regular spacing, about 490:1 with one-tenth of a finger; the moat-over-poly rule may block it. Series segments reach large ratios with more gate area and less random mismatch; five 10 % segments in series give 4850:1 with 100 sections; a single dummy gate on each side; the moat-hole version keeps the poly pitch uniform (H13-16). Any deviation from the regular gate pattern induces etch variation (H13-16). LDMOS outer fingers differ by up to 10 % (first-finger effect), so the sense is never an outer finger, dummy fingers are added, LDMOS senses need finger pairs, and finger spacing must stay uniform (H13-16, H13-31). The sense-resistor alternative needs at least a 10 mV drop, 15–50 mV typical (H13-16).
- **parameters**: ratio; segment positions; series count; dummy fingers.
- **guarantees**: uniform poly pitch when moat holes are used (H13-16).
- **does not guarantee**: ratio accuracy if the sense is an outer finger (H13-31).
- **pdk data needed**: moat-over-poly rule (H13-16).
- **verification**: sense finger index inside [1 + n_dummy, N − n_dummy] (H13-31); ratio by count.
- **prefer over / degrade to**: series segments (large ratio, more area) → partial finger → middle full finger → sense resistor.
- **source**: H13-16, H13-31.

### MOS-34
- **id**: MOS-34
- **technique**: Matched non-self-aligned (drain-extended) devices. Two gate fingers around one extended drain; an even number of sections with sources at both array ends; members placed by translation only.
- **realizes**: cancellation of poly-to-well misalignment (H13-18); the orientation rule for non-self-aligned channels (H13-25).
- **applicability**: DEMOS and LDMOS, where the poly-to-N-well spacing "must exactly equal" the device designer's value (too far: no conduction; too close: gate-oxide overstress), and the drawn L includes well outdiffusion (H13-18). Even superimposable devices still see a tilted-implant S/D shift (H13-25). Field-gap dimensions are exact as specified; use the PCell and explode it only to remove non-critical parts such as outer drain fingers under a hole-blocking ring (H13-18).
- **parameters**: section count (even); array ends = sources; translation-only placement.
- **guarantees**: misalignment cancels between the two fingers of each drain (H13-18).
- **does not guarantee**: anything if a member is rotated or reflected (H13-25).
- **pdk data needed**: exact PCell spacings (H13-18).
- **verification**: Φ equal and transform ∈ {translate} (H13-25).
- **prefer over / degrade to**: none; the only form for this device class.
- **source**: H13-18, H13-25.

### MOS-35
- **id**: MOS-35
- **technique**: Finger metallization for wide (power) devices. M1 strips per S/D finger, perpendicular M2 buses, vias over contacts (tungsten plugs) or staggered between contacts (aluminium), stacked finger metals, and a bus-termination topology.
- **realizes**: an R_DS(on) metal share and electromigration margin (H13-01, H13-05, H13-06, H13-08).
- **applicability**: finger resistance R_M = R_S·W/(3·W_M) (rule of one-third), accurate to ±1.5 % for W ≤ d_pen and ±10 % for W ≤ 1.8·d_pen with d_pen = R_Si·W_M/R_S; avoid fingers longer than 1.8·d_pen (H13-05). Two buses: R_B = 4NPR_S2/(3W); if the bus length N·P exceeds about 2·d_pen terminate the two buses at opposite ends; terminating both ends of each bus cuts bus resistance four times and doubles d_pen (H13-06). First guess: metal is one-third of R_DS(on), less for HV, more for low voltage (H13-08). Finger peak current ≈ I_D/N at the finger–bus junction for N > 10 (H13-08). Pitch bounds of eqs 13.7–13.9 (H13-03). Stacked M1‖M2 fingers: R_S12 = R_S1·R_S2/(R_S1 + R_S2) (H13-04). Waffle and bent-gate alternatives: waffle gain (2S_d + 0.55L_d)/(L_d + S_d), positive only if S_d > 0.45·L_d, halved when metal is half of R, with early avalanche at 90° corners and no backgate-contact provision; bent gate (135°) accepts distributed backgate contacts (H13-15).
- **parameters**: W_f, N, metal width, stack, bus exits.
- **guarantees**: closed-form resistance inside the validity range (H13-05).
- **does not guarantee**: accuracy beyond 1.8·d_pen (H13-05); area extrapolation of specific on-resistance beyond 2–3× (H13-02).
- **pdk data needed**: sheet resistance per layer at maximum temperature; electromigration limits; via rules (H13-08).
- **verification**: PEX resistance at T_max; electromigration at the two peak points (H13-08).
- **prefer over / degrade to**: opposite-end termination → same-end; stacked fingers → single layer.
- **source**: H13-01, H13-02, H13-03, H13-04, H13-05, H13-06, H13-08, H13-15.

---

## 2. Numeric rules of thumb

| Quantity | Value | Source |
|---|---|---|
| Fold threshold | one section below W/L ≈ 10; identical parallel sections above | H12-27 |
| Shared S/D junction capacitance saving | up to 50 % | H12-27 |
| Process biases W_b, L_b | "seldom exceed a few tenths of a micron" | H12-02 |
| Narrow-channel Vt rise | W below "a few µm"; short-channel below a few µm; DIBL/DITS mainly L < 2 µm | H12-07 |
| Analog channel lengths | 20–40 µm "not uncommon"; r_o ∝ L (uniform doping), ∝ √L with pockets | H12-06, H12-25 |
| PMOS vs NMOS size for equal k | PMOS needs about 3× the NMOS W/L | H12 §12.1.1 |
| Gate RC finger limit | W_max = √(t_ox·t_pmax/(2.6·R_S·ε_ox)), ε_ox = 3.5·10⁻¹⁷ F/µm; 90 Å, 10 Ω/□, 1 ns → 90 µm; both ends contacted → 2× | H13-11 |
| Rule of one-third validity | ±1.5 % for W ≤ d_pen, ±10 % for W ≤ 1.8·d_pen, d_pen = R_Si·W_M/R_S | H13-05 |
| Bus termination | opposite ends when bus length > ~2·d_pen; both ends → bus R ÷ 4, d_pen × 2 | H13-06 |
| Backgate contact maximum distance | 25–250 µm by sheet R and current; exercise value 50 µm | H12-43 |
| Sublayer that replaces per-device taps | about ≤ 10 Ω/□ | H13-13 |
| Latch-up trigger | V_trig ≈ 0.65 V (25 °C), ≈ 0.4 V (150 °C); JESD78 I_T ±100 to ±250 mA | H12-38, H12-40 |
| Guard rings needed on bleeders | diode current above "a few hundred µA" | H12-39 |
| Well spacing by voltage class (example) | LV oversized 2.0, HV 3.0; P-well = N-well inverted, undersized 2.0 | H12-13 |
| Isolation voltage limits | backgate-to-iso ≤ 5–10 V (≈ 20 V with PBL); shallow N-well 20–30 V; deep isolation 40–80 V | H12-15 |
| Thick-field threshold derate | ≥ 30 %; 40 V published → ≤ ~25 V between M1 and N-well; poly lowest of all conductors | H12-19, H12-20 |
| Natural-Vt marker example rules | width 4 µm, overlap of GATE 2 µm, spacing to POLY 4 µm, marker-to-marker 2 µm | H12-21 |
| Body bias | NMOS Vt 0.17, 0.43, 0.61, 0.89 V at V_BS 0, −1, −2, −4 V; c_Vt +≈ 30 % at V_BS = 2 V on a 5 V device | H12-04, H13-20 |
| Pelgrom coefficient | c_Vt ≈ 1 mV·µm per nm of t_ox (150 Å → ~15 mV·µm); design to ±6·s_Vt | H13-20 |
| Pelgrom validity | W and L ≥ 1 µm; pocket L_c ≤ a couple of µm; current matching needs L > 5 µm | H13-23 |
| Overdrive floor | V_gst ≥ 100 mV in all corners when stringers exist; little voltage-matching gain below ~100 mV | H13-32, H13-19 |
| Orientation mismatch | several % in current if rotated; up to 5 % on tilted wafers | H13-25 |
| Directional halo | reflect or rotate 180° allowed; 90° forbidden for the whole block | H12-26 |
| Microloading reach | 3–5 µm; up to 10 µm for very accurate matching | H13-26 |
| Dummies by class (rule 12) | MOD: L < ~3 µm → 1 full dummy per end; STI moat ≥ 5 µm; EXC: outermost dummy poly edge ≥ 10 µm from active, half dummies ≥ 10 µm wide, STI moat ≥ 10 µm | H13-47 |
| Dummy count by L (§13.2.2) | one per end if L ≥ 3–5 µm, else two or more | H13-29, H13-26 |
| Half-dummy minimum poly | 2–3 µm | H13-26 |
| Corner rounding and gate extension | rounding ≤ 1–2 µm; extend all gates 1–2 µm beyond the overhang rule; poly comb ≥ 1–2 µm from moat | H13-26, H13-54 |
| Extraneous poly exclusion (rule 23) | EXC 5–10 µm (10–15 µm "very accurate" in §13.2.2); MOD 3–5 µm; MIN 3–5 µm without dummies | H13-55, H13-26 |
| Poly-0 keepaway | ≥ 150 µm; up to 12 % mismatch beyond 30 µm | H13-27 |
| LOD stress | k shift ≥ 10 % (NMOS down, PMOS up), Vt ≥ 10 mV; moat 3–5 µm (2–3 µm less accurate) | H13-29 |
| Piezoresistance (10⁻¹¹ Pa⁻¹) | ⟨100⟩ PMOS π_L −15, π_T 30; NMOS −90, 40. ⟨110⟩ PMOS 65, −50; NMOS −30, −20 | H13-29 |
| WPE | reach 3–5 µm, worst within 1–2 µm; 5 % at 1.8 µm, 25 % at 0.95 µm | H13-28 |
| Well clearance (rule 19) | EXC ≥ 5–10 µm or ≥ 2× well depth; MOD ≥ 3 µm; MIN ≥ 2 µm (WPE-prone LV processes) | H13-53 |
| NBL shadow | ≤ 150 % of epi thickness; none in STI/CMP | H13-30 |
| First-finger effect (LDMOS) | outer fingers differ by up to 10 % | H13-31 |
| PMOS vs NMOS gm mismatch | PMOS often 30–50 % more; some processes reversed | H13-24 |
| Channel-length modulation | λ ≈ 1/(B·L_e·√N_B), B ≈ 0.1–0.2 V·µm^½; mirror L 15–30 µm (20 V) → 5–10 µm (2 V) | H13-33 |
| Hot-carrier drift between unequal V_DS | up to several tens of mV | H13-34 |
| Metal over gate (hydrogenation) | up to 20 % current mismatch; unequal nearby fill up to 1 %; identical surroundings to several µm, similar to ≥ 10 µm; halo 5–10 µm | H13-36, H13-51, H13-52 |
| Fill density example | ≥ 25 % in any 250 µm window (walking window) | H13-52 |
| Oxide-thickness gradient | ≈ 0.5 ppm/µm (5 % over 200 mm) | H13-37 |
| Threshold and mobility temperature coefficients | TC_Vt ≈ −2 mV/°C (−1.5 at 100 Å, −2.5 at 500 Å); µ ∝ T^−1.7 (e), T^−1.5 (h); k(150 °C) ≈ 0.6·k(25 °C) | H13 §13.2.5, H12-03 |
| 2-D vs 1-D common centroid | AB/BA residual ≈ 60 % of ABBA with the same elements | H13-41 |
| Precision classes | MIN ±10 mV / ±10 %; MOD ±3 mV / ±3 %; EXC ±1 mV / ±1 % (six sigma, 10 years, −40 to 125 °C, plastic) | H13-42 |
| Class defaults | MIN for bias mirrors ≤ 3–4 from the reference resistor; MOD for op-amp/comparator inputs; EXC for high-accuracy inputs | H13-42 |
| Trim | room-T trim removes ~⅔ of the −40 to 125 °C drift; cannot fix thermal gradients | H13-42, H13-39 |
| Area for voltage matching (µm², 12/5/3.3/1.8 V) | MIN 423/56/19/5.8; MOD 4,700/630/220/64; EXC 42,000/5,600/1,900/580 | H13-43 |
| Length for current matching at 1 µA (µm, 12/5/3.3/1.8 V) | NMOS MIN 24/15/11/8.1, MOD 79/48/37/27, EXC 240/140/110/81; PMOS MIN 14/8.6/6.6/4.9, MOD 47/29/22/16, EXC 142/86/66/49; scale I_D^−½ | H13-44 |
| Aspect ratio (rule 9) | current: MIN ≤ 10:1, MOD ≤ 3:1, EXC square; voltage: MIN ≤ 3:1, MOD/EXC square; subarrays count, not the whole | H13-46 |
| Die zones (rules 13, 15) | MIN: no corners, ≥ 50–100 µm from edges; MOD: interior or side centre inset ≥ 100–250 µm; EXC: interior on a die axis; bump minima at centres and midway | H13-48 |
| Power separation (rule 14) | power IC ≥ 1 W; EXC at the far end ≈ ¾ from centre to edge, 2:1–3:1 die; MOD ≥ 1 µm per mW (½–¼ usually enough); MIN within a few hundred µm if interdigitated; EXC needs large separation even from 250 mW | H13-49 |
| Switching power example | 10 A, 5 V, 10 ns, 1 MHz → 0.5 W | H13-10 |
| Ratio by units | 20:1 = 4 units in series vs 5 in parallel | H13-22 |
| SenseFET ratios | middle finger of 100 → 49:1; partial finger ~490:1; 5 × 10 % series → 4850:1; sense resistor ≥ 10 mV (15–50 mV typical) | H13-16 |
| Serpentine length | each 90° bend adds W/2 | H12-35 |
| Annular W/L | circular W = π(B−A)/ln(B/A), L = (B−A)/2; square W ≈ 2(C+D), L ≈ (D−C)/2; field-crossing gate W = 2W_S; grid gate W ≈ 4W_S | H12-36, H12-37 |
| Subthreshold slope | S ≥ 60 mV/dec, typically 80, ~90 at 0.15 µm; subthreshold circuits unreliable much above 70 °C | H12-08 |

Values the sources do not give (deck keys the engine must own): the narrow-channel minimum finger width (only "a few µm") (H12-07); the σ inflation for V_SB ≠ 0 beyond the single +30 % point (H12-04, H13-20); the directional-halo tilt angle (F99 §2.6); the exercise 13.7 backgate-contact distance and the k′ behind Table 13.5 (F99 §2.7); the HCI ΔV_DS tolerance (H12-05).

---

## 3. Conflicts and trade-offs

1. **Interdigitation vs 2-D common centroid vs dispersion.** 1-D ABBA aligns centroids cheaply and wires easily (H13-45); 2-D AB/BA cuts the residual to about 60 % but "elaborate patterns" are hard to wire (H13-41); dispersion into subarrays lowers the quadratic residue (∝ size²) at further wiring cost (H13-41, H13-40). Cross-coupling only pays under a real gradient (H13-41). Square units favour 2-D; long thin units stay 1-D (H13-41).
2. **Thermal vs stress optimum.** Stress wants the die centre and a symmetry axis (H13-48, H13 §13.2.5); heat wants distance from the power device, so EXC on power ICs sits about three-quarters of the way out on the axis, a stated compromise (H13-49).
3. **Dummy count vs area, and full vs half dummies.** EXC's 10 µm reach needs several dummies or long ones (H13-47); half dummies save area but let moat and well edges approach the gate, so LOD and WPE then need their own extension (H13-26). Series units are compact for large ratios but carry an inherent mismatch that excludes them from EXC (H13-22).
4. **Compactness vs halos.** Rule 9 wants square compact arrays (H13-46) while rules 17, 18, 19 and 23 add 5–10 µm metal, fill, well and poly halos around EXC arrays (H13-51, H13-52, H13-53, H13-55), and WPE pushes well edges out (H13-28). Dispersion resolves the aspect side but multiplies the halos.
5. **Zero body bias vs shared well.** A private source-tied well per member gives V_SB = 0 (H12-04, H13-20) but puts members in separate wells, each with edges near its gate (WPE, H13-28) and with well-to-well spacing (H12-33, H12-13).
6. **Butting tap and minimize-drain vs symmetry.** Butting taps need bulk = source and the right parity (H12-30); a tap at one end only gives the two ends different environments, against rule 12's equal-environment intent (H13-26) *(inference)*; even N with minimized drain allows both ends (H12-30).
7. **Distributed backgate plugs vs source resistance and finger identity.** Plugs greatly reduce area and raise source resistance slightly (H12-44); interdigitated strips cost area but leave fingers intact (H12-44); in a matched array plugs must be placed identically in every member *(inference)*.
8. **Field plate vs no metal.** The MIN plate lets routing and fill pass over the array (H13-51, H13-52) but leaves edge gradients and higher random mismatch (H13-36); MOD/EXC forbid both plate and crossings and so block routing resources (H13-51).
9. **Fill blocking vs density.** Blocks protect matching but break density; they must be few and small and the density restored with identical custom fill, under a walking-window check that can pass one instance and fail another (H13-52).
10. **Finger width pulls in four directions.** ESD wants wide fingers (H12-27); gate RC wants W_f ≤ W_max (H13-11); matching wants a shared unit width (H12-28) that is not below the narrow-channel limit (H12-07); very high-speed devices want two sections for low drain capacitance (H12-29).
11. **Current matching vs voltage matching vs overdrive.** Current matching wants high V_gst and long L (H13-19, H13-21); voltage matching wants low V_gst and area (H13-19); both hit the 100 mV stringer floor (H13-32) and the "little to gain below ~100 mV" limit (H13-19). Long L for current matching costs area and is lost in pocket processes (H13-44, H13-23).
12. **Cascode vs cascaded mirrors.** Cascodes equalize V_DS and cure HCI drift (H13-34, H13-33) at the cost of extra devices that are themselves a matched set *(inference)*; the alternative is longer L and fewer cascaded mirrors (H13-33).
13. **Guard rings everywhere vs area.** Rings only around injectors; backgate contacts on everything (H12-39); pinning taps on the victim's facing side are the cheap substitute (H12-42).
14. **Tank sharing vs injection separation.** Sharing a tank saves area but only same-pin devices may share, and noisy devices never share with sensitive ones (H12-16, H12-17).
15. **Well merging vs class spacing and WPE.** Merged wells save the well-to-well spacing (H12-33) but must respect voltage-class spacing (H12-13) and WPE clearance to matched gates (H13-53).
16. **Diagonal PMOS vs everything else.** PMOS is least stress-sensitive at 45° (H13-38) but editors and rules rarely support it, directional implants forbid it, and only EXC under heavy stress justifies it (H13-38).
17. **PMOS vs NMOS input pair.** PMOS often has 30–50 % more gm mismatch (H13-24) and suffers NBTI drift when comparator inputs sit unbalanced (H13-35); the fix is clamp diodes or an NMOS pair (H13-35).
18. **Trim vs layout.** Trim at one temperature removes only about two-thirds of the drift and nothing of a thermal gradient, so EXC must be met by layout (H13-42, H13-39).
19. **Refold vs schematic intent.** A refold can improve aspect and parasitics but an annotated N(W/L) must be kept, and every refold must be reported (H12-27).

---

## 4. Precision classes

Class definitions (six sigma, 10-year life, −40 to 125 °C, plastic package): MIN ±10 mV or ±10 %, bias mirrors at most 3–4 mirrors from the reference resistor; MOD ±3 mV or ±3 %, general-purpose op-amp and comparator input pairs; EXC ±1 mV or ±1 %, high-accuracy input stages, large common-centroid arrays or trim, where room-temperature trim removes about two-thirds of the drift and cannot fix thermal gradients (H13-42). Kind: voltage-matched (equal I_D, compare V_GS) or current-matched (equal V_GS, compare I_D) (H13-19, H13-42).

The 24 rules of §13.3 by class:

| Rule | MIN | MOD | EXC | Card | Source |
|---|---|---|---|---|---|
| 1 Identical unit sections | required | required | required; no series units | MOS-02 | H13-22 |
| 2 Area for voltage matching | Table 13.4 MIN row | MOD row | EXC row | MOS-29 | H13-43 |
| 3 Length for current matching | Table 13.5 MIN row | MOD row | EXC row | MOS-28 | H13-44 |
| 4 Overdrive ≥ 100 mV all corners if stringers (current matching) | not required | required | required | MOS-28 | H13-32 |
| 5 Pocket implants | avoid unless short | V: L ≤ L_c; I: no-pocket devices | same | MOS-28, MOS-29 | H13-23 |
| 6 Thin oxide | preferred | preferred | preferred | MOS-29 | H13-43 |
| 7 Same orientation, equal Φ | parallel channels | parallel, Φ equal or nearly | Φ equal (also any class if non-self-aligned) | MOS-15, MOS-34 | H13-25 |
| 8 Proximity | as close as possible | adjacent, preferably interdigitated | carefully constructed common centroid, always | MOS-24, MOS-25, MOS-26 | H13-45 |
| 9 Compactness | I ≤ 10:1, V ≤ 3:1 | I ≤ 3:1, V square | square (subarrays count) | MOS-25, MOS-27 | H13-46 |
| 10 Prefer 2-D common centroid | optional | when natural | yes, with dispersion | MOS-26, MOS-27 | H13-41 |
| 11 Avoid submicron W or L | advisory | W, L ≥ 1 µm | W, L ≥ 1 µm | MOS-28, MOS-29 | H13-23 |
| 12 Dummies | optional | one full dummy per end (L < ~3 µm) or half dummies; STI moat ≥ 5 µm | full dummies to 10 µm reach or half dummies ≥ 10 µm; STI moat ≥ 10 µm | MOS-16, MOS-17, MOS-18 | H13-47 |
| 13 Die stress zones | no corners; ≥ 50–100 µm from edges | interior or side centre inset ≥ 100–250 µm | interior | MOS-32 | H13-48 |
| 14 Distance from power devices | within a few hundred µm if interdigitated | opposite end; ≥ 1 µm per mW near small heaters | far end on axis, ≈ ¾ out; many subarrays | MOS-31 | H13-49 |
| 15 Die symmetry axes | not required | bump axis when bumped | on a die axis (or between bumps) | MOS-32 | H13-48 |
| 16 No contacts on active gate | required | required | required | MOS-13 | H13-50 |
| 17 Metal over active gate | identical crossings or M1 plate | none | none; minimize within 5–10 µm; similar leads per section | MOS-22 | H13-51 |
| 18 Dummy-metal blocking | fill over the plate allowed | block over the array | block + 5–10 µm halo, all metals; symmetric custom fill | MOS-23 | H13-52 |
| 19 Well and deep-diffusion spacing | rules (≥ 2 µm if WPE-prone) | rules (≥ 3 µm if WPE-prone) | ≥ 5–10 µm or 2× well depth | MOS-19 | H13-53 |
| 20 NBL shadow | not required | outside active | outside active | MOS-20 | H13-30 |
| 21 Gate extension +1–2 µm, all equal | exempt | required | required | MOS-13 | H13-54 |
| 22 Metal gate straps (no poly comb) | exempt | required (comb ≥ 1–2 µm from moat tolerated) | required | MOS-13 | H13-54 |
| 23 Extraneous poly exclusion | adjacent poly allowed with dummies, else 3–5 µm | 3–5 µm | 5–10 µm; block dummy poly | MOS-21 | H13-55 |
| 24 Diagonal PMOS | no | no | only under significant stress, no directional implants | MOS-32 | H13-38 |

Checklist per class:

**MIN (±10 mV / ±10 %)**
- Identical unit sections, parallel channels, same cell (H13-22, H13-25).
- Area or length from the MIN rows of Tables 13.4 / 13.5 (H13-43, H13-44).
- As close as possible; aspect ≤ 10:1 (current) or ≤ 3:1 (voltage) (H13-45, H13-46).
- Dummies if space permits (H13-47).
- No contacts on the active gate (H13-50).
- Metal over gates only as identical crossings or one M1 plate; fill may sit on the plate (H13-51, H13-52).
- Not in die corners, 50–100 µm from edges (H13-48); within a few hundred µm of a multiwatt device only if interdigitated or cross-coupled (H13-49).
- Well edge ≥ 2 µm from the gate in WPE-prone processes (H13-53); other poly ≥ 3–5 µm away unless end dummies exist (H13-55).

**MOD (±3 mV / ±3 %)**
- Everything in MIN, plus:
- Area or length from the MOD rows (H13-43, H13-44); W, L ≥ 1 µm (H13-23); V_gst ≥ 100 mV in all corners for current matching where stringers exist (H13-32).
- Adjacent, preferably interdigitated; Φ equal or nearly equal; aspect ≤ 3:1 (current) or square (voltage) (H13-45, H13-25, H13-46).
- One full dummy per end when L < ~3 µm, half dummies otherwise; STI moat ≥ 5 µm past the last gate (H13-47).
- No metal plate and no leads over the active area (H13-51); fill blocked over the array (H13-52).
- Gates extended 1–2 µm beyond the rule, all equal; fingers joined in metal (H13-54).
- Well edge ≥ 3 µm in WPE-prone processes (H13-53); NBL shadow outside the active area (H13-30); other poly ≥ 3–5 µm away (H13-55).
- Interior of the die, or near a side centre inset 100–250 µm; away from multiwatt devices; ≥ 1 µm per mW near small heaters (H13-48, H13-49).

**EXC (±1 mV / ±1 %)**
- Everything in MOD, plus:
- Area or length from the EXC rows (H13-43, H13-44); no series units (H13-22); no pocket devices for current matching (H13-23).
- Carefully constructed 2-D common centroid, always; Φ_H and Φ_V exactly equal; square subarrays; dispersion for large arrays (H13-45, H13-25, H13-46, H13-41).
- Full dummies until the outermost dummy's poly edge is ≥ 10 µm from any active device, or half dummies ≥ 10 µm wide; STI moat ≥ 10 µm (H13-47).
- No metal within 5–10 µm beyond what is necessary; leads within 5–10 µm present a similar pattern to every section; fill block extends 5–10 µm on every metal with symmetric custom fill (H13-51, H13-52).
- Well boundary ≥ 5–10 µm or ≥ 2× well depth from the gate; the same for deep diffusions (H13-53); no unconnected poly within 5–10 µm and generated dummy poly blocked (H13-55).
- Die interior on a symmetry axis (or the axis between bumps); on power ICs at the far end about three-quarters of the way out, as many small common-centroid subarrays; large separation even from a 250 mW device (H13-48, H13-49).
- Diagonal PMOS only under significant stress and never with directional implants (H13-38).
