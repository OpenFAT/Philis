# Substrate-noise isolation and electromigration: technique cards

Sources, both read in full:
- Charbon, Gharpurey, Miliozzi, Meyer, Sangiovanni-Vincentelli, *Substrate Noise* (Kluwer 2001), digest `docs/plans/ref-charbon-substrate-noise.md`, items SUB-01…SUB-58.
- Lienig, Thiele, *Fundamentals of Electromigration-Aware IC Design* (Springer 2018), digest `docs/plans/ref-lienig-electromigration.md`, items EM-01…EM-48.

Card ids are SUBN-xx (substrate) and EMR-xx (electromigration and IR drop). The digests' own item ids SUB-xx and EM-xx are cited as sources; EMR rather than EM keeps card ids distinct from the Lienig item ids. Where a fact exists only in a digest's section-by-section part and has no item id, the citation names the digest section. Values marked "figure" are the digests' plot readings (±2 dB / ±10 %), not printed numbers.

Vocabulary used throughout: "epi" = low-resistivity substrate, ~10 µm of 10–15 Ω·cm epitaxy on a ~1 mΩ·cm p+ bulk; "bulk" = high-resistivity, uniformly doped 20–50 Ω·cm substrate (SUB-01). L_gnd = backplate (package) ground inductance; L_gr = guard-ring pin plus bond-wire inductance (SUB-01, SUB-33). V_inj1 = noise on the ring's own net (SUB-33). V0 = local substrate potential at a victim; V0_max = its budget (SUB-15).

---

## 1. Technique cards

### 1.1 Substrate noise

### SUBN-01 Substrate kind decides the remedy class
- `id`: SUBN-01
- `technique`: Read the deck's substrate stack (kind, layers with thickness and resistivity, backplate state, L_gnd, L_gr, deep n-well availability) before choosing any isolation measure. Epi-on-p+ and high-resistivity bulk have opposite remedy classes: bulk responds to distance and guard rings; epi responds mainly to backplate grounding (SUB-01, SUB-05, SUB-28).
- `realizes`: Selection of the isolation measures that the source shows effective for the given substrate; avoids area spent on rings that do nothing in epi (SUB-28).
- `applicability`: All substrate kinds, all net classes, all frequencies below ~5 GHz where the substrate is resistive (SUB-01, SUB-02).
- `parameters`: kind ∈ {bulk, epi_on_pplus, soi}; layer stack top-down (thickness, resistivity); backplate ∈ {grounded, floating}; L_gnd; L_gr; deep n-well available (SUB-01).
- `guarantees` / `does not guarantee`: Guarantees: the remedies chosen are the ones shown effective for that kind (rings in bulk; backplate ground in epi) (SUB-28, SUB-05). Does not guarantee: any isolation number without the pair macromodel (SUBN-20); isolation numbers are comparable only within one known cross-section (SUB-56).
- `pdk data needed`: substrate kind; per-layer thickness (µm) and resistivity (Ω·cm); backplate state; L_gnd (nH); L_gr (nH); deep n-well flag (SUB-01).
- `verification`: When the stack is missing, report isolation as "unknown" and make substrate rules cost-only pulls (SUB-01).
- `prefer over / degrade to`: Prefer over any substrate-agnostic ring or keep-out policy; the digest notes a bulk deck being scored with an epi plateau rule as the failure this prevents (SUB-32). Degrade to: no kind → distance as cost-only, no isolation rings emitted (SUB-01).
- `source`: SUB-01, SUB-02, SUB-05, SUB-28, SUB-32, SUB-56.

### SUBN-02 Separation on epi-on-p+ substrates (saturating distance law)
- `id`: SUBN-02
- `technique`: Place the victim at distance d from the injector. Isolation improves until a critical distance d_sat of 2.5–5× the epi thickness (Su et al. measured 4×) with a finite L_gnd of 1–5 nH at 1 GHz, then plateaus; spend no area on distance beyond d_sat (SUB-31, SUB-05).
- `realizes`: Aggressor–victim isolation up to the plateau level, which L_gnd sets (SUB-05).
- `applicability`: epi_on_pplus only; all injector and victim classes. The plateau exists for L_gnd > 0; with L_gnd = 0 isolation keeps improving with distance. A floating backplate makes isolation distance-independent (≈ −22 dB at 1 GHz with C_sub 0.3 pF, figure) (SUB-05, SUB-31).
- `parameters`: d; d_sat = k·t_epi with k default 4 (SUB-31); victim contact area (high-frequency isolation worsens ~linearly with contact area, so smaller victim junctions help) (SUB-31).
- `guarantees` / `does not guarantee`: Guarantees: monotone improvement up to d_sat (SUB-31). Does not guarantee: improvement past d_sat; plateau at 1 GHz with 50 µm contacts ≈ −47 dB at 5 nH and ≈ −62 dB at 1 nH, versus ≈ −112 dB at 400 µm only when L_gnd = 0 (figure) (SUB-05). Does not substitute for the backplate ground (SUBN-04).
- `pdk data needed`: t_epi (µm), ρ_epi (Ω·cm), ρ_bulk (Ω·cm), L_gnd (nH), k_sat (dimensionless, default 4), fitted constants for R_i0(area) and R12(d) (SUB-31, SUB-57).
- `verification`: Per-pair isolation from the two-port macromodel at the aggressor frequency with the deck's L_gnd; flag victims whose budget is met only at L_gnd = 0 as "needs backside or down-bond ground" (SUB-05).
- `prefer over / degrade to`: Prefer the backplate ground (SUBN-04) when the plateau is above budget; prefer reducing victim junction area over more distance past d_sat (SUB-31). Degrade to a fixed keep-out of 4× epi thickness when no fit constants exist (SUB-31).
- `source`: SUB-05, SUB-31, SUB-57.

### SUBN-03 Separation on high-resistivity bulk (monotone distance law)
- `id`: SUBN-03
- `technique`: Place victims away from injectors. A large fraction of the current flows at the surface, so isolation keeps improving with distance even with finite backplate inductance and depends only weakly on L_gnd. Example at 100 MHz, grounded backplate, C_sub 0.3 pF: ≈ −67 dB at 0 µm to ≈ −80 dB at 400 µm (figure) (SUB-32).
- `realizes`: A continuous distance-versus-isolation trade for every pair, with no plateau (SUB-32).
- `applicability`: bulk only; all classes; frequencies below ~5 GHz (SUB-01, SUB-32).
- `parameters`: d; victim distance to the die edge (R to backplate rises near the edge, so less shunting) (SUB-32).
- `guarantees` / `does not guarantee`: Guarantees: R12 rises monotonically with distance and the isolation cost keeps a slope at all distances (SUB-32). Does not guarantee: a dB number without calibrated constants (the source gives no fit constants) (SUB-32).
- `pdk data needed`: ρ_bulk (Ω·cm), surface layer thickness (µm) and resistivity (Ω·cm), fitted R12(d) constants (a + b·ln d or a + b·d) from deck calibration (SUB-32, SUB-57).
- `verification`: Two-port macromodel per pair; placement cost ∝ max(0, V_victim − V0_max), not a fixed keep-out (SUB-32).
- `prefer over / degrade to`: Prefer a guard ring (SUBN-05) when area for distance is short, since rings are very effective in bulk (SUB-28). Degrade to distance-only cost without a plateau (SUB-32).
- `source`: SUB-01, SUB-28, SUB-32, SUB-57.

### SUBN-04 Low-impedance backplate ground
- `id`: SUBN-04
- `technique`: Ground the backplate through the lowest-inductance path available (backside contact, down-bond). In epi substrates "the most effective way of improving isolation is to provide a very good ground contact to the backplate"; it is the only lever that keeps isolation improving with distance (SUB-05, SUB-31).
- `realizes`: The isolation floor for every pair in epi; in bulk a grounded backplate adds 5–10 dB while surface rings remain the primary ground path (SUB-05; Charbon digest §8.6.2, L3704–3708).
- `applicability`: epi_on_pplus primarily; every pair; this is a package or assembly decision, outside placement and routing (SUB-05).
- `parameters`: L_gnd: 0 nH keeps improving with distance; 1 nH plateau ≈ −62 dB; 5 nH plateau ≈ −47 dB at 1 GHz, 50 µm contacts in a 1000 µm die (figure) (SUB-05).
- `guarantees` / `does not guarantee`: Guarantees: the plateau level in epi follows Z′12(jω) = R10 + R20 − j·R10·R20/(ω·L_gnd), so lower L_gnd lowers the floor (SUB-04, SUB-05). Does not guarantee: anything the layout itself can do; the report must say which victims need a package action (SUB-05).
- `pdk data needed`: L_gnd (nH) for each package option; backplate state (SUB-01).
- `verification`: Two-port macromodel at the deck's L_gnd; flag "needs backside ground" when the budget is met only at L_gnd = 0 (SUB-05).
- `prefer over / degrade to`: Prefer over guard rings in epi, which give only 7–10 dB (SUB-28). Degrade to reporting the required L_gnd to the package designer (SUB-05).
- `source`: SUB-04, SUB-05, SUB-28, SUB-31; Charbon digest §8.6.2.

### SUBN-05 p+ guard ring: minimum width, very good ground
- `id`: SUBN-05
- `technique`: Surround the injector or the victim with a heavily doped majority-carrier surface ring (p+ in a p substrate) acting as a current sink and Faraday shield. Draw it at minimum width and give it a very good ground connection: "the best rule for sizing guard rings is to use minimum width rings and ensure a very good ground connection to the ring" (SUB-29; Charbon digest §8.5).
- `realizes`: Raises the effective injector–victim resistance R12 and adds a sink node GR with resistances R1gr, R2gr to the two contacts (SUB-33).
- `applicability`: bulk substrates, where rings are very effective (SUB-28). epi: only 7–10 dB, a weak function of L_gr, so do not use rings for isolation there (SUB-28). Single-ended and differential circuits (SUB-29, SUB-39). Useful only below ω ≈ R1gr·R2gr/(L_gr·(R1gr + R2gr)) (SUB-33).
- `parameters`: width w: deck minimum; widening from 2 to 34 µm helps only at L_gr = 0 and worsens isolation at ~1 GHz otherwise; at 100 MHz a shallow optimum near 15 µm at 2 nH (SUB-29). Gap d to the enclosed circuit: small d raises R12 area-efficiently but lowers R2gr and so couples more ring-net noise; if V_inj1 is significant both w and d must be large (SUB-36). Cut rows: add only until the ring resistance budget is met (SUB-29). L_gr (SUB-33).
- `guarantees` / `does not guarantee`: Guarantees in bulk: large improvement when L_gr is small (1 GHz ≈ −45 dB at L_gr 5 nH versus ≈ −65 dB at 0 nH; 100 MHz ≈ −95 versus ≈ −103 dB, figure) (SUB-28). Does not guarantee: improvement with a large L_gr on an internal ground, which is worse than no ring (SUB-30); more than 7–10 dB in epi (SUB-28); any effect from minority-carrier (n-type) rings in epi, where the current flows in the p+ bulk (SUB-28).
- `pdk data needed`: minimum ring width (nm), ring-to-device clearance (nm), contact/cut resistance (Ω per cut), maximum ring resistance (mΩ), ring diffusion sheet resistance (Ω/sq), L_gr (nH) (SUB-29, SUB-33, SUB-36).
- `verification`: Ring macromodel (SUBN-21): report the "useful up to" frequency R1gr·R2gr/(2π·L_gr·(R1gr + R2gr)) and flag rings whose useful frequency is below the aggressor clock (SUB-33). Ring resistance ≤ budget (SUB-29). Connection-net check (SUBN-06) (SUB-30).
- `prefer over / degrade to`: Prefer distance when area is available in bulk; the annealer may hold both implementations in its search space (SUB-25). Prefer a ring when the per-pair minimum distance is unaffordable (SUB-16). Degrade to taps near the victim (SUBN-10) when no dedicated pin is available (SUB-30). In epi degrade to latch-up rings only (SUB-28).
- `source`: SUB-16, SUB-25, SUB-28, SUB-29, SUB-30, SUB-33, SUB-36; Charbon digest §8.5.

### SUBN-06 Guard-ring connection: dedicated, quiet, correctly referenced
- `id`: SUBN-06
- `technique`: Connect the ring through an independent bond-wire or pin when the victim signal is referenced to an external ground. Connect to the internal ground only when the signal is measured against that same internal ground (then isolation is independent of L_gr). Never share the ring net with the substrate-bias pad, the ESD substrate node or the digital supply. Route ring nets as a star to their pin with no segment shared with signal ground (SUB-30).
- `realizes`: A ring that lowers rather than raises received noise; a tap or ring whose net bounces injects that bounce over its whole extent (SUB-30).
- `applicability`: Every isolation ring and tap ring, all substrate kinds (SUB-30).
- `parameters`: ring role ∈ {latch-up, victim, aggressor}; connection net: the victim block's own reference ground if the netlist distinguishes it, else a dedicated ring net exported as a pin; aggressor rings get their own pin (SUB-30).
- `guarantees` / `does not guarantee`: Guarantees: an internally referenced ring is L_gr-independent when the signal shares that reference (SUB-30). Does not guarantee: any improvement when the ring shares a node with ESD cells (measured consistently worse) or is tied to the substrate-bias pad (increases noise through the package inductance of that contact) (SUB-30).
- `pdk data needed`: none beyond the ring layer; needs netlist roles distinguishing analog and digital grounds (SUB-30).
- `verification`: Flag rings whose connection net is shared with a clock-class device's source, with ESD cells, or with more than N unrelated devices. DRC and LVS do not catch this (SUB-30, SUB-52).
- `prefer over / degrade to`: Prefer a ring on a dedicated pin. Degrade to no ring at all rather than a ring on a noisy shared net, because a ring can increase noise when V_inj1 is large (SUB-30, SUB-34).
- `source`: SUB-30, SUB-34, SUB-52.

### SUBN-07 Ring the injector rather than the receiver
- `id`: SUBN-07
- `technique`: Default to a ring around the noisy circuit (injector cluster), with its bond-wire placed among the analog bond-wires. Ring the victim instead when the aggressors are many and dispersed or the injector cluster is large (SUB-34).
- `realizes`: The same isolation from the injector as a receiver ring, and ~20 dB better isolation from the ring's own net noise V_inj1 (SUB-34).
- `applicability`: bulk substrates with compact injector clusters (SUB-28, SUB-34).
- `parameters`: Example geometry b = 250, w = 4, a = 50, d = 10 µm: ≈ −98 dB at 100 MHz and ≈ −47 dB at 1 GHz, versus no ring ≈ −60 / −32 dB (figure) (SUB-34).
- `guarantees` / `does not guarantee`: Guarantees: V_inj1 dominates only above 35 mA of adjacent bond-wire current with an injector ring versus 3.5 mA with a receiver ring (1 GHz, mutual inductance 1 nH; 1 mA gives V_inj1 = 6.3 mV, received 0.14 mV versus 1.41 mV) (SUB-34). Does not guarantee: better isolation from the injector itself than a receiver ring (they are nearly equal) (SUB-34).
- `pdk data needed`: L_gr (nH); bond-wire mutual inductance M (nH) if ring-net noise is modelled (SUB-34).
- `verification`: Ring macromodel with the V_inj1 term scaled by R_load/R2gr (SUB-33).
- `prefer over / degrade to`: Prefer dual rings when two pins are available (SUBN-08). Degrade to a receiver ring when the injector cluster is large or dispersed (SUB-34).
- `source`: SUB-28, SUB-33, SUB-34.

### SUBN-08 Dual guard rings on separate pins
- `id`: SUBN-08
- `technique`: Ring both the injector and the receiver, each ring on its own bond-wire or pin. Tied together they act as one large ring and lose the benefit (SUB-35).
- `realizes`: Large additional isolation: no ring −62 / −34 dB; receiver ring −99 / −45 dB; two rings −130 / −57 dB (100 MHz / 1 GHz; 50 µm contacts 300 µm apart, 10 µm rings, 5 nH each, C_sub 0.8 pF) (SUB-35).
- `applicability`: bulk; when the required isolation exceeds the single-ring estimate and two package pins are available (SUB-35).
- `parameters`: two distinct ring nets; width and gap per SUBN-05 (SUB-35, SUB-29).
- `guarantees` / `does not guarantee`: Guarantees: the Table 8.1 gains under that geometry (SUB-35). Does not guarantee: anything if the two nets merge in routing or ring merging (SUB-35).
- `pdk data needed`: as SUBN-05; pin budget from the flow configuration (SUB-35).
- `verification`: Ring merging must only merge same-net rings; macromodel with two GR nodes (SUB-33, SUB-35).
- `prefer over / degrade to`: Prefer over a buried shield when deep n-well is unavailable. Degrade to a single injector ring when only one pin exists (SUB-34, SUB-35).
- `source`: SUB-33, SUB-34, SUB-35.

### SUBN-09 Buried shields: deep n-well, p+ buried Faraday shield, SOI
- `id`: SUBN-09
- `technique`: Place a buried shield under the circuit with the smaller area (usually the analog section), so its capacitance is small relative to the ground-path impedance. Three kinds: (c) buried n+ well / deep n-well tied to the highest potential, which combines a depletion dielectric with a Faraday shield and is least sensitive to both non-idealities; (a) p+-in-p buried Faraday shield, near-ideal with an ideal ground but very sensitive to ground-path impedance; (b) SOI or dielectric, good at low frequency, degrading with frequency through its series capacitance, improved by thicker oxide. For sensitive NMOS groups this is a triple well: deep n-well under the group, the enclosing n-well ring tied to a quiet supply, an isolated p-well tap to the victim's ground (SUB-38).
- `realizes`: Isolation beyond what distance plus rings give (SUB-16, SUB-38).
- `applicability`: Decks offering deep n-well; hard to place under PMOS n-wells or npn buried collectors (depths up to several µm); applied to the smaller of victim and aggressor groups. Measured (Joardar): buried n+ better than SOI (SUB-38).
- `parameters`: shield type; tie potential (highest supply); enclosing n-well ring net; p-well tap net; area of the shielded group (SUB-38).
- `guarantees` / `does not guarantee`: Guarantees: isolation ∝ 1/(ground-path impedance); a high-conductivity n+ shield behaves as a T-attenuator, near-ideal at low frequency and degrading with frequency; a low-doped shield behaves as a series junction capacitance, better at high frequency (SUB-38). Does not guarantee: isolation with nonzero shield resistivity even with a perfect ground; a p+ shield floats with its bond-wire inductance at high frequency (SUB-38).
- `pdk data needed`: deep n-well availability and depth (µm); n-well and deep n-well junction capacitance per area (fF/µm²) and perimeter (fF/µm); deep n-well sheet resistance (Ω/sq); enclosure rules (SUB-07, SUB-38).
- `verification`: Macromodel with the shield capacitance and ground impedance; lateral conductivity variation (wells, deep n-well, trenches) needs the finite-difference extractor, not the layered Green's function (SUB-54).
- `prefer over / degrade to`: Prefer deep n-well over SOI (SUB-38); prefer a p+ Faraday shield only with a near-ideal ground (SUB-38). Degrade to dual rings (SUBN-08) when no deep n-well exists.
- `source`: SUB-07, SUB-16, SUB-38, SUB-54.
- Coverage note: trench isolation appears in neither source beyond SUB-54's remark that trenches are lateral resistivity variations a finite-difference mesh can model and the layered Green's function cannot. No construction rule or number is available from these two books.

### SUBN-10 Local substrate taps around sensitive devices
- `id`: SUBN-10
- `technique`: Surround sensitive devices and nodes with substrate contacts within a maximum distance d_tap. Taps pin the local potential, reducing junction-capacitance modulation ΔC_j = −(C_j/n)·(ΔV_bias/ψ0)/(1 + V_bias/ψ0) and bias shifts ΔI_d = g_mb·ΔV_bias; "surrounding sensitive nodes of an amplifier by substrate contacts reduces undesirable changes in bandwidth" (SUB-41, SUB-14).
- `realizes`: Bandwidth and gain stability of amplifiers; lower V0 at victims, especially in bulk (SUB-41).
- `applicability`: bulk (taps more efficient because current flows at the surface) and, less so, epi (SUB-41, SUB-28). All sensitive devices, including inner devices of merged or stacked cells far from a cell's tap strip (SUB-41).
- `parameters`: d_tap_sensitive, tighter than the latch-up rule (no value in the source; deck or configuration) (SUB-41); tap count versus size by substrate kind (SUBN-11) (SUB-42).
- `guarantees` / `does not guarantee`: Guarantees: local potential pinning "quite effectively in its immediate vicinity, especially in high-resistivity substrates" (SUB-41). Does not guarantee: isolation in epi by itself (SUB-28); taps on a bouncing net inject (SUB-30).
- `pdk data needed`: tap minimum size (nm), tap spacing rules, d_tap default (µm) (SUB-41).
- `verification`: Maximum distance from any point of an active device to a tap ≤ d_tap; tap net quietness per SUBN-06 (SUB-14, SUB-41).
- `prefer over / degrade to`: Prefer several distributed taps over one large tap in bulk (SUB-42). Degrade to the MOS cell's own tap strip (SUB-41).
- `source`: SUB-14, SUB-28, SUB-30, SUB-41, SUB-42.

### SUBN-11 Tap sizing and victim junction area by substrate kind
- `id`: SUBN-11
- `technique`: In bulk, add more distributed taps rather than bigger ones: tap resistance to ground depends only logarithmically on size (~600 Ω at 10 µm to ~90 Ω near 700 µm, figure). In epi, use large-area backplate-tied taps near the aggressor: resistance is nearly inverse with area (~3–5 kΩ at 10 µm to ~2 Ω at 1000 µm, figure). Keep victim junction areas small in epi, where high-frequency isolation worsens roughly linearly with victim contact area (SUB-42, SUB-31).
- `realizes`: Lowest tap resistance per unit area; lower substrate capacitance at victims (SUB-42, SUB-07).
- `applicability`: By substrate kind (SUB-42).
- `parameters`: tap area, count and placement; victim diffusion area and perimeter (SUB-07, SUB-42).
- `guarantees` / `does not guarantee`: Guarantees: the correct lever per kind (SUB-42). Does not guarantee: a resistance number without the fitted R(area) constants (SUB-57).
- `pdk data needed`: junction capacitance per area (fF/µm²) and perimeter (fF/µm) for diffusion and n-well (SUB-07); fitted R_i0(area) (SUB-31, SUB-32).
- `verification`: C_sub and R_i0 per contact enter the two-port model (SUB-04, SUB-07).
- `prefer over / degrade to`: Prefer over widening rings, which does not help unless L_gr = 0 (SUB-29). Degrade to deck minimum tap strips (SUB-41).
- `source`: SUB-04, SUB-07, SUB-29, SUB-31, SUB-32, SUB-41, SUB-42, SUB-57.

### SUBN-12 Low-impedance drain at strong injectors; AC-ground every well
- `id`: SUBN-12
- `technique`: Place a substrate tap ring or strip immediately adjacent to each vertical PNP and each device with large substrate current, tied to a dedicated quiet ground. Tie every n-well to a low-impedance, locally AC-grounded quiet supply through taps within d_tap and with well-tap-to-rail resistance ≤ R_max; sensitive-block wells go to a quiet supply, not the digital supply (SUB-37, SUB-14).
- `realizes`: The vertical PNP's collector current (the substrate is its collector) is drained locally; a PMOS n-well that is not AC-grounded otherwise becomes a large injector through its well–substrate capacitance (SUB-06, SUB-14, SUB-37).
- `applicability`: Vertical PNPs (SUB-06); NMOS with high V_ds and I_d, where impact ionization I_impact ≈ C1·(V_ds − V_dsat)·I_d·e^(−C2/(V_ds − V_dsat)) is the dominant substrate-noise mechanism to at least 100 MHz (SUB-08); every n-well (SUB-14).
- `parameters`: tap ring resistance budget R_max; d_tap; well tap net (SUB-14, SUB-37).
- `guarantees` / `does not guarantee`: Guarantees: the injector's current has a local low-impedance path (SUB-37). Does not guarantee: victim isolation; it reduces injection, not transfer (SUB-37).
- `pdk data needed`: contact resistance (Ω per cut), well sheet resistance (Ω/sq), well–substrate capacitance (fF/µm², fF/µm) (SUB-07, SUB-14).
- `verification`: Per well: resistance to rail ≤ R_max and maximum point-to-tap distance ≤ d_tap. Per strong injector: adjacent tap present (SUB-14, SUB-37).
- `prefer over / degrade to`: Prefer treating the injector (removes current at its source) before victim-side rings. Degrade to the existing per-MOS tap strip when no injector weight is available (SUB-37).
- `source`: SUB-06, SUB-07, SUB-08, SUB-14, SUB-37.

### SUBN-13 Isolated-plate decoupling at clocked blocks
- `id`: SUBN-13
- `technique`: Place substrate-isolated (MOM or MIM) decoupling capacitors between a clocked block's supply and ground, adjacent to the aggressor, so high-frequency harmonic energy is shunted to circuit ground instead of entering the substrate through junction and interconnect capacitances (SUB-11).
- `realizes`: A lower injected high-frequency spectrum; the n-th harmonic of a rectangular clock is 20·log(n) dB below the fundamental (a 10 MHz, 3 V clock: 1.91 V fundamental, ~GHz harmonic 19.1 mV, −40 dB) (SUB-11).
- `applicability`: Clocked or digital aggressors in mixed-signal tops (SUB-11).
- `parameters`: decap value; placement adjacency (SUB-11).
- `guarantees` / `does not guarantee`: Guarantees: capacitors electrically isolated from the substrate shunt HF energy to circuit ground (SUB-11). Does not guarantee: a quantified reduction (no number in the source).
- `pdk data needed`: MOM or MIM availability and density (fF/µm²) (SUB-11).
- `verification`: Aggressor spectrum lines at k·f_clk with amplitude V/k; a victim whose band contains a harmonic gets the tighter budget (SUB-11).
- `prefer over / degrade to`: Complement to distance and rings; degrade to none (SUB-11).
- `source`: SUB-11.

### SUBN-14 Per-pair minimum distance from the victim budget and the isolation factor
- `id`: SUBN-14
- `technique`: Give each victim a substrate-voltage bound V0_max from its performance share: ΔK_i = Sᵀ·ΔΠ ≤ ΔK̄_i, bounds chosen to maximize flexibility (SUB-15). For each aggressor compute the required isolation factor α_req = V0_max / (I_peak·R_ground) and invert the kind-specific transfer model (SUBN-02 or SUBN-03) into a minimum distance d_min(aggressor, victim). Where d_min exceeds d_sat (epi) or the area budget, emit a ring requirement instead and re-evaluate with the ring model (SUB-16).
- `realizes`: Separation sized to actual injection and tolerance per pair rather than a uniform keep-out, which wastes area on weak pairs and under-protects strong ones (SUB-16, SUB-50).
- `applicability`: All kinds; victims with a budget and aggressors with a peak current. Early floorplanning uses one cluster contact per block sized to its total injecting area (SUB-18, SUB-50).
- `parameters`: block budget ΔK̄; sensitivity ∂K/∂V0 (≈ g_mb/g_m × gain referral; the ring-VCO case measured ≈ 2/3, constant over 0–300 mV) (SUB-15, SUB-17); peak injector current (SUB-10); aggressor R_ground (SUB-16); d_sat (SUB-31).
- `guarantees` / `does not guarantee`: Guarantees: constraints are process-independent and generated once per circuit; enforcement is process-dependent (SUB-15). Does not guarantee: feasibility; infeasible pairs become ring or buried-shield requirements (SUB-16, SUB-38).
- `pdk data needed`: transfer-model constants (SUBN-02, SUBN-03); L_gnd (SUB-16).
- `verification`: Peak victim voltage by DC analysis on the positive and negative peak injector currents (resistive substrate, valid to 2–5 GHz), compared with V0_max; per-pair report (SUB-02, SUB-56).
- `prefer over / degrade to`: Prefer over a fixed 4× epi keep-out (SUB-16). Degrade to a uniform keep-out when no budget or current exists (SUB-16).
- `source`: SUB-02, SUB-10, SUB-15, SUB-16, SUB-17, SUB-18, SUB-31, SUB-38, SUB-50, SUB-56.

### SUBN-15 Floorplan: isolated areas, aggressors clustered, victims opposite
- `id`: SUBN-15
- `technique`: Allocate well-isolated areas to noisy circuits; seed aggressor and victim clusters on opposite sides of the region (opposite die ends at top level) using the per-pair distances as soft separation; compute minimum distances from the aggressor's noise spectral energy against the victim's tolerated spurious energy; where space is lacking, design rings for the problematic part of the spectrum (SUB-50).
- `realizes`: Coarse separation before detailed placement (SUB-50).
- `applicability`: Global placement and floorplan, all kinds (SUB-50).
- `parameters`: one cluster contact per block with area equal to its total injecting area; rough maximum and minimum substrate conductances from two contacts at the chip edges or in close proximity (SUB-18).
- `guarantees` / `does not guarantee`: Guarantees: nothing numeric by itself. The PLL case: substrate-plus-parasitic-aware placement predicted jitter 0.005 against a spec of 0.007, where parasitic-only placement predicted 0.1, at 1.28× the area (SUB-23). Does not guarantee: per-pair compliance without the pair evaluation (SUB-56).
- `pdk data needed`: as SUBN-14.
- `verification`: Per-pair report (SUB-56).
- `prefer over / degrade to`: Prefer numbers over "digital = bad". Degrade to the early practice of large contact-dominated separation areas (SUB-50).
- `source`: SUB-18, SUB-23, SUB-50, SUB-56.

### SUBN-16 Substrate-aware placement cost: exact per epoch, estimated per move
- `id`: SUBN-16
- `technique`: In annealing, evaluate the contact network exactly once per temperature using Sherman–Morrison rank-1 updates (O(N²) per moved contact), estimate per move with a gradient linearization of the conductance matrix (accurate to 1 % within 5 grid steps and 10 % up to 1/10 of the chip size), and re-evaluate exactly only when a critical node reaches its bound. Cost = Σ over victims of max(0, V_v − V0_max)² × weight, zero when all are under bound (SUB-21, SUB-22, SUB-23).
- `realizes`: Enforcement of the per-pair budgets during placement at affordable cost (SUB-23).
- `applicability`: Analog blocks with up to a few hundred contacts (SUB-21). Alternative implementations (with and without ring) may sit in the search space without losing convergence (SUB-25).
- `parameters`: refresh thresholds (5 grid steps); contact granularity (one per cell early, per device junction for signoff) (SUB-18, SUB-22).
- `guarantees` / `does not guarantee`: Guarantees: simulated-annealing convergence is retained, also with alternative guard-ring implementations (SUB-25). Does not guarantee: accuracy beyond the linearization range (SUB-22).
- `pdk data needed`: transfer-model constants or the Green's function table of the stack (SUB-53).
- `verification`: Exact solve at the end of each epoch; signoff extraction (SUBN-22) (SUB-23, SUB-56).
- `prefer over / degrade to`: Prefer over edge-gap-only isolation scores. Degrade to the geometric d_min cost (SUB-16).
- `source`: SUB-16, SUB-18, SUB-21, SUB-22, SUB-23, SUB-25, SUB-53, SUB-56.

### SUBN-17 Orient differential receivers with the injector on the symmetry axis
- `id`: SUBN-17
- `technique`: For each matched pair, position and orient it so the dominant aggressor lies on the pair's perpendicular bisector; the worst case is the aggressor on the line through both devices (SUB-43).
- `realizes`: Common-mode conversion of substrate noise; with identical receivers and the injector on the axis, differential isolation is infinite in the model (SUB-43).
- `applicability`: Matched pairs with a symmetry axis, both kinds. In epi, differential isolation falls rapidly with distance and is backplate-insensitive (1 GHz ≈ −35 dB near to ≈ −120 dB at 400 µm, figure). In bulk the differential gain is smaller because the nearer contact shields the farther one (SUB-43, SUB-46).
- `parameters`: cost |Z_ka − Z_kb|·I_k from the contact network, or the geometric proxy |d(k,a) − d(k,b)| / (d(k,a) + d(k,b)) (SUB-43).
- `guarantees` / `does not guarantee`: Guarantees: symmetric exposure for identical receivers (SUB-43). Does not guarantee: anything under mismatch; 5 % substrate-capacitance mismatch collapses differential isolation (SUB-44).
- `pdk data needed`: none beyond the transfer model.
- `verification`: Differential-mode and common-mode isolation reported per pair and aggressor (SUB-46).
- `prefer over / degrade to`: Prefer together with a shared thin ring (SUBN-19). Degrade to mirror placement only (SUB-43).
- `source`: SUB-43, SUB-44, SUB-46.

### SUBN-18 Equal substrate exposure inside matched pairs
- `id`: SUBN-18
- `technique`: Require both halves of a matched pair to have equal junction area and perimeter to substrate, equal distance to the nearest tap or ring edge, equal well-edge distance, and equal exposure to each aggressor (SUB-44).
- `realizes`: Preserved differential isolation. A 5 % mismatch of the receiver pair's substrate capacitances drops 1 GHz isolation from ≈ −120 dB to ≈ −80 dB and 100 MHz from ≈ −150 to ≈ −123 dB (figure); mismatches turn differential circuits single-ended (SUB-44).
- `applicability`: Every matched pair, both kinds; also adjacent differential AC-coupling capacitors (SUB-44, SUB-47).
- `parameters`: |Δd_tap| ≤ grid; mismatch budget a fraction of the 5 % shown damaging (exact limit not given) (SUB-44).
- `guarantees` / `does not guarantee`: Guarantees: geometric equality (SUB-44). Does not guarantee: process mismatch of the capacitances (SUB-44).
- `pdk data needed`: junction capacitance per area and perimeter (SUB-07).
- `verification`: Compute C_sub and resistance to ring per half from drawn geometry and report the relative mismatch (SUB-44).
- `prefer over / degrade to`: Prefer over same-variant-only requirements. Degrade to same cell variant plus dummies (SUB-44).
- `source`: SUB-07, SUB-44, SUB-47.

### SUBN-19 One shared thin ring around a differential pair
- `id`: SUBN-19
- `technique`: Enclose both halves of a matched pair in a single minimum-width ring rather than one ring per device; the ring is a nearby equipotential plane that makes coupled noise common mode (SUB-39).
- `realizes`: Differential-mode isolation ≈ −92…−100 dB at 1 GHz and ≈ −130…−140 dB at 100 MHz in bulk (figure), improving slightly with width and independent of L_gr (SUB-39).
- `applicability`: bulk; differential pairs. Common-mode isolation worsens with width when L_gr > 0, so the ring stays thin (SUB-39, SUB-29).
- `parameters`: width (minimum); a single loop, no separate halves; connection per SUBN-06 (SUB-30, SUB-39).
- `guarantees` / `does not guarantee`: Guarantees: symmetric distance of both halves to the ring (SUB-39). Does not guarantee: survival of mismatch (SUB-44).
- `pdk data needed`: ring width and clearance rules (SUB-29).
- `verification`: The pair's ring is one loop; differential and common-mode report (SUB-39, SUB-46).
- `prefer over / degrade to`: Prefer over per-device rings. Degrade to shared taps (SUB-39).
- `source`: SUB-29, SUB-30, SUB-39, SUB-44, SUB-46.

### SUBN-20 Two-port (three-node) isolation macromodel per aggressor–victim pair
- `id`: SUBN-20
- `technique`: Isolation I = 20·log(V_rcv/V_inj) evaluated on the circuit: V_inj → C_sub1 → node 1; R10 and R20 from the two substrate nodes to the backplate node; backplate to ground through L_gnd; R12 between the nodes; node 2 → C_sub2 → victim node loaded by R_load. With small substrate-capacitance reactance, small load and R12 ignored, Z′12(jω) = R10 + R20 − j·R10·R20/(ω·L_gnd) (SUB-04). The substrate is quasi-static and resistive to 2–5 GHz; contact impedances are real to 4–5 GHz (SUB-02). The general N-contact form is the conductance network with Y_ii = Σ_j c_ij, Y_ij = −c_ij and the backplate as node 0 (SUB-03).
- `realizes`: Quantitative isolation in dB and margin against V0_max per pair, frequency- and package-aware, at O(1) complex arithmetic per pair (SUB-04).
- `applicability`: All kinds; exact below 2 GHz, "approximate" above (SUB-02).
- `parameters` (what the model needs): R10(area 1), R20(area 2) from the contact areas; R12(d) from the kind-specific distance law; C_sub per contact = C_j,area·A + C_j,sidewall·P from drawn diffusion or well geometry (SUB-07); R_load = impedance looking into the victim device from the substrate including feedback (SUB-12); L_gnd; aggressor frequency and peak current (SUB-10) (SUB-04).
- `guarantees` / `does not guarantee`: Guarantees: isolation scales linearly with R_load (+20 dB per decade) while the load pole 1/(2π·R_load·C_sub) lies above the noise frequency (1 kΩ with 0.8 pF gives ≈ 200 MHz; 200 Ω gives 1 GHz); at 100 MHz, 90 µm spacing, 3 nH, 0.8 pF: ≈ −103 dB at 10 Ω to ≈ −67 dB at 1 kΩ (figure) (SUB-04, SUB-12). Does not guarantee: lateral resistivity variations (wells, deep n-well, trenches), which need finite-difference extraction (SUB-54); comparability of numbers across cross-sections (SUB-56).
- `pdk data needed`: substrate stack, L_gnd (nH), junction capacitance per area and perimeter, fit constants per kind (SUB-01, SUB-07, SUB-57).
- `verification`: Calibrate the fit per deck against the exact extractor on two-contact sweeps (sizes 10–1000 µm, distances 3–1000 µm); accept when within 15 % for coupling ratios above 0.1 (SUB-57).
- `prefer over / degrade to`: Prefer over edge-gap proxies. Degrade to the geometric d_min rule (SUB-16).
- `source`: SUB-02, SUB-03, SUB-04, SUB-07, SUB-10, SUB-12, SUB-16, SUB-54, SUB-56, SUB-57.

### SUBN-21 Guard-ring macromodel and useful-frequency check
- `id`: SUBN-21
- `technique`: Add a ring node GR with resistances R1gr and R2gr to injector and victim and inductance L_gr to ground. Received voltage V2 = R_load·{(1/R12 + jωL/(R1gr·R2gr + jωL·(R1gr + R2gr)))·V_inj + (R1gr/(R1gr·R2gr + jωL·(R1gr + R2gr)))·V_inj1} (SUB-33).
- `realizes`: A per-ring verdict, per pair and per frequency: helps, neutral or hurts (SUB-33).
- `applicability`: Any ring enclosing an injector or victim, both kinds (SUB-33).
- `parameters`: wider w raises R12 and lowers R1gr and R2gr; low-frequency isolation is set by R12 (wide is good), high-frequency by R1gr + R2gr; the ring loses its advantage above ω ≈ R1gr·R2gr/(L_gr·(R1gr + R2gr)); the ring-net noise term scales with R_load/R2gr (SUB-33).
- `guarantees` / `does not guarantee`: Guarantees: the frequency above which the ring stops helping (SUB-33). Does not guarantee: the V_inj1 contribution unless the ring net's noise is known (SUB-34).
- `pdk data needed`: L_gr (nH), ring layer resistances, the same fit constants (the ring is treated as a contact of its drawn area at the shortest distance to each contact) (SUB-33).
- `verification`: Flag rings whose useful frequency is below the aggressor clock (SUB-33).
- `prefer over / degrade to`: Prefer over width heuristics. Degrade to the "minimum width, good ground" rule (SUB-29).
- `source`: SUB-29, SUB-33, SUB-34.

### SUBN-22 Signoff substrate extraction, calibration and report
- `id`: SUBN-22
- `technique`: Extract the full contact conductance matrix either with the layered-media Green's function (a 2-D DCT computed once per substrate stack at O(P̃Q̃ log P̃Q̃); only the P matrix of 50–5000 contacts is recomputed and inverted when contacts move; non-abrupt profiles by z-discretization) or with a 3-D finite-difference resistive mesh solved by preconditioned CG (fine grid near contacts, coarse far away; handles wells, deep n-well, trenches). Report per aggressor–victim pair: distance, isolation dB, V_peak against V0_max, differential and common-mode values for pairs, ring role and ring-net check, and the suggested remedy (SUB-53, SUB-54, SUB-56).
- `realizes`: A machine-checked substrate signoff and the calibration constants for the cheap models (SUB-56, SUB-57).
- `applicability`: All kinds; hundreds of contacts for analog blocks; thousands need accelerated matrix–vector products (SUB-55).
- `parameters`: contacts equipotential, subdivided if not; one admittance column per contact set to 1 V (SUB-53); grid spacing proposal 2 µm near contacts growing to 50 µm, 5–10 z layers (digest proposal, not a source value) (SUB-54).
- `guarantees` / `does not guarantee`: Guarantees: measured-level accuracy, within 15 % for voltage ratios above 0.5, good above 0.1, degrading below 0.1 (SUB-57). Does not guarantee: package inductance effects from on-wafer data (SUB-57).
- `pdk data needed`: full stack (thickness µm, resistivity Ω·cm per layer), contact depth (µm), well and deep n-well conductivity regions (SUB-53, SUB-54).
- `verification`: This is the verification; the count of failing pairs feeds the epoch residual (SUB-56).
- `prefer over / degrade to`: Prefer the Green's function for layered stacks and the finite-difference mesh when lateral variation matters (SUB-54). Degrade to the two-port macromodel alone (SUBN-20).
- `source`: SUB-53, SUB-54, SUB-55, SUB-56, SUB-57.

### SUBN-23 Injector and victim characterization (inputs the macromodel needs)
- `id`: SUBN-23
- `technique`: Weight every device as injector and victim from its type and operating point. Injectors: vertical npn through the collector–substrate capacitance C_js and a parasitic pnp in saturation; lateral pnp through its base–substrate capacitance; vertical pnp large because the substrate is its collector; NMOS through source/drain junction capacitance and impact ionization, dominant to at least 100 MHz; PMOS small unless its well is not AC-grounded; diffused resistors far above poly; p+ taps and rings on a bouncing net; reverse-biased junction leakage as DC drift (SUB-06, SUB-08). Switching devices: a single-gate pulse of 1–100 µA, 0.5–5× the gate delay wide, fitted as i(t) = A·e^(−t/τ)·cos(ωt − φ) with the example A = 100 µA, τ = 6.3 ns; the spectrum concentrates near 1/(average gate delay) with a DC part from impact ionization (SUB-10). Victims: body-effect gain g_mb/g_m typically 0.1–0.3, so the body-to-drain gain is only 14–20 dB below gate-to-drain; body effect dominates at low and medium frequency, capacitive pickup above ~1 MHz (SUB-09); victim weight ∝ |Z_node|, so high-impedance gain nodes, bias rails and lateral-pnp bases rank first (SUB-12). Spur classes that pair aggressors with victims: PA to LNA (power ratio above 100 dB, substrate not frequency selective), in-band digital harmonics, low-frequency noise upconverted by second-order distortion, beat products, DC offsets from clock or LO leakage into samplers and mixers, PA pulling of oscillators, loops across high-gain chains (80–100 dB in direct conversion) (SUB-49).
- `realizes`: Aggressor magnitude and victim sensitivity for the per-pair budget (SUB-15).
- `applicability`: Every device with op-point data I_d, V_ds, V_dsat, g_m, g_mb (SUB-08, SUB-09).
- `parameters`: impact-ionization constants C1, C2 (not given; when absent, rank by I_d·(V_ds − V_dsat) and mark relative); clock frequency; default pulse 100 µA (SUB-08, SUB-10).
- `guarantees` / `does not guarantee`: Guarantees: exhaustive classification, catching analog switching devices (latches, choppers, sampling switches) and high-V_ds NMOS that humans miss (SUB-06, SUB-08). Does not guarantee: absolute injection without C1 and C2 (SUB-08); white, pink or clock-synchronous single-source models underestimate harmful components (SUB-10).
- `pdk data needed`: impact-ionization constants (BSIM alpha0 and beta0 equivalents); junction capacitance per area and perimeter; N_A and C_ox or simulator g_mb (SUB-07, SUB-08, SUB-09).
- `verification`: The worst case is impulsive injection synchronous with the victim's switching (ring-VCO jitter peaks when the generator delay equals one inverter delay); evaluate worst-case and nominal activity and report both (SUB-17, SUB-58).
- `prefer over / degrade to`: Prefer op-point weights over "device on a clock net". Degrade to clock-net membership (SUB-06).
- `source`: SUB-06, SUB-07, SUB-08, SUB-09, SUB-10, SUB-12, SUB-15, SUB-17, SUB-49, SUB-58.

### SUBN-24 Break substrate feedback inside high-gain stages
- `id`: SUBN-24
- `technique`: For stages whose gain exceeds a threshold, space the output device from the input device, ring or tap the output device (bulk), ground the backplate, or implement differentially with close devices (SUB-13).
- `realizes`: A bounded Miller capacitance through the substrate: collector–substrate capacitances C_s1 = C_s2 = 0.2 pF coupled through R_s12 with a second-stage gain of 10 act as 1 pF at the first stage's output; measured bandwidth fell from 1.9 to 1.8 GHz (SUB-13).
- `applicability`: Amplifiers with large substrate capacitance, close devices and few or small substrate contacts, especially in bulk. In RF stages the path raises |S12| and lowers the maximum stable gain, and can satisfy Barkhausen (oscillation) (SUB-13).
- `parameters`: Miller budget = (C_s,out in series with C_s,in)·|A| through the substrate path; spacing; a ring on the output device (SUB-13).
- `guarantees` / `does not guarantee`: Guarantees: a computed bound on the feedback capacitance (SUB-13). Does not guarantee: visibility in schematic simulation; the loop is invisible without a substrate model (SUB-13).
- `pdk data needed`: junction capacitance (SUB-07); transfer-model constants.
- `verification`: Effective Miller capacitance from the contact network (SUB-03, SUB-13).
- `prefer over / degrade to`: Prefer taps or a ring near the output device in bulk. Degrade to spacing (SUB-13).
- `source`: SUB-03, SUB-07, SUB-13.

### SUBN-25 Capacitor plate orientation and adjacent differential AC-coupling capacitors
- `id`: SUBN-25
- `technique`: When one side of a capacitor is a reference node, connect the bottom plate to the reference (lower-impedance) node and the top plate to the active node. For two adjacent differential AC-coupling capacitors, model the substrate resistance R12 between their bottom plates: the output is attenuated by 1/(1 + r), r = bottom-plate parasitic over main capacitance (r = 0.2 gives 1.6 dB loss not predicted by single-ended models) (SUB-47).
- `realizes`: No substrate current through the capacitor, preserved Q, correct differential gain (SUB-47).
- `applicability`: LC tanks, bypass, AC coupling; all kinds (SUB-47).
- `parameters`: plate assignment; coupling distance between adjacent capacitors (SUB-47).
- `guarantees` / `does not guarantee`: Guarantees: avoidance of substrate current for reference-tied capacitors (SUB-47). Does not guarantee: benefit for single-ended AC coupling, where the bottom-plate substrate resistance can help (SUB-47).
- `pdk data needed`: bottom-plate parasitic capacitance ratio r (dimensionless) (SUB-47).
- `verification`: Include R12 between bottom-plate substrate nodes in the extracted netlist and report 1/(1 + r) (SUB-47).
- `prefer over / degrade to`: Degrade to orientation only (SUB-47).
- `source`: SUB-47.

### SUBN-26 Shield for low-noise input pads
- `id`: SUBN-26
- `technique`: Shield each low-noise input pad with a low-impedance shield: a buried n-well tied to supply, or a lower metal layer (SUB-40).
- `realizes`: Removal of substrate thermal noise under the pad (bond-pad capacitance 0.5–1 pF, substrate resistance several Ω to several kΩ) that the input device would amplify; the output noise peaks when the pad network's reactive and resistive parts are similar (SUB-40).
- `applicability`: Pads on nets classified as low-noise inputs (SUB-40).
- `parameters`: shield layer; shield net (supply or ground) (SUB-40).
- `guarantees` / `does not guarantee`: Guarantees: the pad-network thermal noise is shunted (SUB-40). Does not guarantee: pad-to-pad coupling or matching loss, which the substrate's series R–C at the node sets through |S11| (SUB-48).
- `pdk data needed`: n-well under pad allowed; lower-metal shield rules (SUB-40).
- `verification`: Substrate R–C under the pad from the network and the implied |S11| (SUB-48).
- `prefer over / degrade to`: Prefer the n-well shield when the deck allows it. Degrade to a lower-metal shield (SUB-40).
- `source`: SUB-40, SUB-48.

### SUBN-27 Sensitive routes: run-length-aware spacing and quiet-well or shield underlay
- `id`: SUBN-27
- `technique`: Score victim–aggressor route coupling with the empirical delay model ΔD ∝ L^α/S^β, α ≈ 2, β ≈ 1 (parallel run length squared over spacing) rather than a spacing threshold; treat the substrate under a victim wire as an aggressor with coupling capacitance C_c = C_s; route sensitive wires over a quiet well or shield (SUB-51).
- `realizes`: Crosstalk exclusion sized to run length (SUB-51).
- `applicability`: Sensitive-class routes, all kinds (SUB-51).
- `parameters`: parallel run length L, spacing S, shield or well underlay (SUB-51).
- `guarantees` / `does not guarantee`: Guarantees: a cost that grows as L²/S; the closed-form peak response (eq 2.11) is available for verification (SUB-51). Does not guarantee: absolute dB without the coupling capacitances.
- `pdk data needed`: interconnect capacitance per unit length (fF/µm), substrate capacitance under wires (SUB-51).
- `verification`: Summed coupling per victim against its budget (SUB-51).
- `prefer over / degrade to`: Prefer over fixed spacing. Degrade to spacing (SUB-51).
- `source`: SUB-51.

### SUBN-28 ESD cells: no added isolation structures, substrate node treated as noisy
- `id`: SUBN-28
- `technique`: Do not add taps or rings inside or adjacent to grounded-gate NMOS ESD cells beyond the deck's ESD rule; exclude them from ring merging; classify their substrate node as shared and noisy (SUB-52, SUB-30).
- `realizes`: Unchanged ESD turn-on, which depends on substrate resistance forward-biasing the parasitic npn (nonlinear through conductivity modulation); rings sharing that node measured consistently worse (SUB-52, SUB-30).
- `applicability`: All ESD NMOS and pad rings (SUB-52).
- `parameters`: an ESD cell class flag (SUB-52).
- `guarantees` / `does not guarantee`: Guarantees: the ESD substrate resistance is untouched (SUB-52). Does not guarantee: ESD performance itself (SUB-52).
- `pdk data needed`: ESD rule deck (SUB-52).
- `verification`: Ring-net sharing check (SUB-30).
- `prefer over / degrade to`: none.
- `source`: SUB-30, SUB-52.

### SUBN-29 Mirror-symmetric, fully differential, abutted floorplan for repeated stages
- `id`: SUBN-29
- `technique`: For ring oscillators and delay lines: abut all delay cells (minimum RC), fold the ring several times, keep the cell aspect ratio low, make each cell fully differential and mirror-symmetric, and share one symmetry axis across all stages (SUB-45).
- `realizes`: Capacitively coupled noise becomes common mode; thermal and substrate effects are balanced between branches; both differential outputs share one constraint (SUB-45).
- `applicability`: Repeated identical stages (SUB-45).
- `parameters`: fold count; row template (SUB-45).
- `guarantees` / `does not guarantee`: Guarantees: codified expert practice; no numbers (SUB-45). Does not guarantee: isolation from external aggressors (SUBN-14).
- `pdk data needed`: none.
- `verification`: Symmetry and matching rules (SUB-45).
- `prefer over / degrade to`: Degrade to per-stage symmetry (SUB-45).
- `source`: SUB-45.

Items from the Charbon digest deliberately not carded, with the card that absorbs them: SUB-17 worst-case sensitivity alignment (SUBN-23 verification); SUB-19 wire R/C loop (EMR-09); SUB-20 sizing feedback (outside layout); SUB-24 placement problem form (baseline, no technique); SUB-26 template reuse and SUB-55 accelerated matvecs (SUBN-22 scale-up); SUB-27 cross-PDK trend metric; SUB-48 RF matching loss (SUBN-26 verification); SUB-58 stimulus dependence (SUBN-23 verification).

### 1.2 Electromigration and IR drop

### EMR-01 Nominal wire width from current
- `id`: EMR-01
- `technique`: w_nom(T_eff) = max{ i_w,eq / (j_eff,eq(T_eff)·h_nom), i_w,peak / (j_eff,peak(T_eff)·h_nom), w_min,process }, with J = I/A and A = W·T; a deck limit given in µA per µm of width already folds the thickness in. i_w,eq is the worst-case RMS or average segment current (EM-12, EM-01, EM-06, EM-09).
- `realizes`: The electromigration current-density limit on every segment (EM-12).
- `applicability`: Every EM-critical segment (EMR-19), all net classes; i_eq is chosen by class (EMR-05) (EM-12, EM-20).
- `parameters`: segment current bounds (EMR-07); layer limit (µA/µm, or MA/cm² with h_nom); T_eff (EM-01, EM-12).
- `guarantees` / `does not guarantee`: Guarantees: J ≤ j_eff on straight homogeneous segments (EM-12). Does not guarantee: crowding at bends and vias (EMR-14, EMR-22), process margins (EMR-02); Black's equation behind the limit holds only for straight lines without bends, layer changes or material transitions and not under steep current rise (EM-02).
- `pdk data needed`: per layer: j_limit for average, RMS and peak (µA/µm at the reference temperature, or MA/cm² plus h_nom in µm); w_min (nm) (EM-01, EM-06, EM-12).
- `verification`: Fast method: compare the required cross-section per segment and via with the drawn one (EM-20). Report per segment the lifetime margin MTF/MTF_limit = (j_limit/j)^n · exp[(Ea/k)·(1/T − 1/T_ref)], minimum over segments (EM-02).
- `prefer over / degrade to`: Prefer over blanket widths. Degrade to the DC equivalent term only when no transient exists (EM-12).
- `source`: EM-01, EM-02, EM-06, EM-09, EM-12, EM-20.

### EMR-02 Effective width with process margins
- `id`: EMR-02
- `technique`: w_eff(T_eff) = w_nom(T_eff)·(h_nom/h_min) + Δw + w_etch: thickness spread, width variation and etch loss (EM-13).
- `realizes`: The EM limit at the thinnest, narrowest manufactured instance (EM-13).
- `applicability`: Every sized segment (EM-13).
- `parameters`: h_nom/h_min, Δw, w_etch (EM-13).
- `guarantees` / `does not guarantee`: Guarantees: margins are applied uniformly and cannot be forgotten (EM-13). Does not guarantee: replacement of the deck's own corner limits (EM-13).
- `pdk data needed`: per metal: h_nom (µm), h_min (µm), Δw (nm), w_etch (nm) (EM-13).
- `verification`: Emit a "no process margin" note when the fields are missing (EM-13).
- `prefer over / degrade to`: Degrade to h_nom/h_min = 1 and zero margins (EM-13).
- `source`: EM-13.

### EMR-03 Via-array cut count from current, temperature and crowding
- `id`: EMR-03
- `technique`: n_via = ceil( i_w,eq · g(H) / (i_single_via(T_ref) · f(T_eff)) ), where f(T_eff) = exp(−(Ea/(n·k·T_ref))·(1 − T_ref/T_eff)) with 1 ≤ n ≤ 2 and g(H) = 1 for homogeneous current flow, > 1 otherwise. The book's Eq. 3.25 multiplies by f, which would give a hotter via fewer cuts; f is the allowed-current factor and must divide (EM-14). Size the array to the full segment current, never to the line's current density times via area (EM-29).
- `realizes`: Via ampacity at every layer change (EM-14, EM-29).
- `applicability`: Every via stack of a critical net. Electron flow line → via causes via depletion; a larger line-to-via width ratio loads the via more (EM-29).
- `parameters`: i_single_via at T_ref; Ea; n; T_eff; g(H) by array geometry class (EMR-14) (EM-14).
- `guarantees` / `does not guarantee`: Guarantees: cut count follows current and temperature (EM-14). Does not guarantee: an even current split; parallel vias give a 2–4× TTF increase only when the current splits evenly (EM-36).
- `pdk data needed`: per via layer: current per cut (µA at T_ref); Ea (eV); n; g(H) factors per pattern (EM-14, EM-42).
- `verification`: cut count × I_cut ≥ i_W for every via stack of a critical net (EM-29).
- `prefer over / degrade to`: Prefer in-line arrays with g = 1 over L-turn arrays (EMR-14). Degrade to g = 1 when no characterization exists (EM-14).
- `source`: EM-14, EM-29, EM-36, EM-42.

### EMR-04 Temperature derating of limits, per segment from the thermal map
- `id`: EMR-04
- `technique`: At constant lifetime, j_max(T)/j_max(T_ref) = exp[(Ea/(n·k))·(1/T − 1/T_ref)]. Evaluate each segment's temperature as T_seg = T_die + ΔT_map from the device-power thermal map at the segment midpoint and derate that segment's layer and cut limits before sizing (EM-03, EM-04).
- `realizes`: Lifetime at the mission temperature instead of at the characterization temperature (EM-03).
- `applicability`: All metals and vias. Cu: +10 K requires more than a 50 % current cut. Al: −5 K allows about +25 % current; 25 → 125 °C requires about a 90 % cut. Automotive ambient reaches 175 °C (EM-03).
- `parameters`: Ea and n per layer; T_ref; thermal map (EM-03, EM-04).
- `guarantees` / `does not guarantee`: Guarantees: constant MTF at the hotter temperature (EM-03). Does not guarantee: Joule self-heating unless RMS sizing is used (EMR-05) or ΔT_self = I_rms²·R_seg·θ_seg is fed back and iterated; self-heating often limits the top metals, which conduct heat poorly (EM-05). Do not credit temperatures below T_ref without deck consent (EM-03).
- `pdk data needed`: per layer Ea (eV), n, T_ref (K). Fallback when the deck gives a limit and T_ref but no Ea or n: Cu Ea 0.9 eV, n 1.2 (midpoint of 1.1–1.3); Al Ea 0.7 eV, n 2; W contacts per deck only; emit a provenance note (EM-42).
- `verification`: Per-segment lifetime margin (EM-02).
- `prefer over / degrade to`: Prefer the per-segment map over a single die temperature (EM-04). Degrade to one die temperature; with no T_ref at all, no derating and a warning (EM-42).
- `source`: EM-02, EM-03, EM-04, EM-05, EM-42.

### EMR-05 Three current types and net classes with separate limits
- `id`: EMR-05
- `technique`: Carry per segment i_rms (governs for f < 1 Hz, analog DC nets and reliability-critical long pulses; conservative, no self-healing; includes Joule heating), i_avg (f > 1 Hz, digital and analog signal nets; includes self-healing; basis of Black-law lifetime) and i_peak (single events: ESD, short power-stage pulses; electrical-overstress rules). Size with whichever type needs the widest wire or the most cuts for the layer and temperature (EM-06). Use at least two limits per layer: one for DC or f < 10 kHz and one for AC above, with optional steps between 10 Hz and 10 kHz; one global limit wastes routing resources (EM-22). AC credit: j_eq = r·j⁺ − γ·(1 − r)·j⁻ with duty factor r and empirical γ; in Cu MTF_AC/MTF_DC rises up to ×500 over 10–10⁴ Hz; for zero-DC asymmetric pulses the minimum lifetime is at r ≈ 0.4 (EM-23).
- `realizes`: The correct limit per net class; AC signal nets are not over-widened, freeing area and capacitance for matching-critical routes (EM-22).
- `applicability`: Power, supply and bias nets are the DC class (self-healing negligible, most sensitive); clock nets carry near-symmetric AC with high RMS from fan-out; signal nets carry asymmetric pulsed AC with low RMS (EM-22, EM-23). In analog, power and signal currents are of the same order, so classification is by current, never by role alone (EM-20).
- `parameters`: r and γ per net (deck γ; else γ = 0 or use the average current) (EM-23); self-heating limit for top metals (EM-05).
- `guarantees` / `does not guarantee`: Guarantees: the max-of-three catches RMS-dominated clock-like nets and ESD paths (EM-06). Does not guarantee: skin-effect correction, which matters only for analog HF at ≥ 45 GHz (δ = √(2ρ/(ωμ)); Cu δ ≈ 9.4 mm at 50 Hz scaling as 1/√f; critical 90 GHz at W = t = 0.45 µm) (EM-24).
- `pdk data needed`: per layer j_avg, j_rms, j_peak (µA/µm); γ (dimensionless); self-heating limit (µA/µm RMS) or thermal resistance to substrate (K/W); ESD current (A) for clamp and pad nets (EM-05, EM-06, EM-23).
- `verification`: All three types evaluated per segment, layer and temperature (EM-06).
- `prefer over / degrade to`: Prefer two-class limits over one global limit (EM-22). Degrade to DC average only (EM-06).
- `source`: EM-05, EM-06, EM-20, EM-22, EM-23, EM-24.

### EMR-06 Terminal current bounds per type and operating phase
- `id`: EMR-06
- `technique`: Store per terminal lower and upper bounds (avg⁻/avg⁺, RMS⁻/RMS⁺, peak⁻/peak⁺) per operating phase as matrices L_n and U_n of O types × P phases (Model 3), 2·N·O·P values for an N-terminal net, sparse storage possible. RMS⁻ is computed from the zero and negative parts of the waveform and multiplied by −1 (EM-07, EM-08).
- `realizes`: Safe segment currents at Steiner points where two terminals have reversed worst cases, which a single max-|equivalent| per terminal misses (EM-07).
- `applicability`: Every net; a DC-only flow sets lower = upper = i_dc (EM-07).
- `parameters`: O = 3 types; P from testbench operating phases (normal, standby, hot) (EM-08).
- `guarantees` / `does not guarantee`: Guarantees: no undersizing from a single equivalent value (EM-07). Does not guarantee: KCL-consistent data, which is not required (EM-09).
- `pdk data needed`: none; needs a transient bench per phase (EM-06).
- `verification`: none (data model).
- `prefer over / degrade to`: Prefer Model 3 over Model 1 (KCL-consistent snapshot vectors; stores all waveforms and over-sizes) and Model 2 (time slots) (EM-08). Degrade to one DC value per pin (EM-07).
- `source`: EM-06, EM-07, EM-08, EM-09.

### EMR-07 Worst-case segment current in trees (bound sums on each side of a cut)
- `id`: EMR-07
- `technique`: Cutting segment s splits the terminals into LHS and RHS sets; sum the lower and upper bounds on each side; i_W,op = max{ min(|i_L,LHS|, |i_U,RHS|), min(|i_L,RHS|, |i_U,LHS|) }; i_W = max over operating phases. Example: terminals in [−1, +2] mA and [−3, 0] mA give i_W = 2 mA (EM-09).
- `realizes`: The exact dimensioning current for every tree edge in every mode (EM-09).
- `applicability`: Tree-topology nets; O(edges·O·P) per net (EM-09).
- `parameters`: terminal bounds (EMR-06) (EM-09).
- `guarantees` / `does not guarantee`: Guarantees: safety even for non-KCL terminal data, because summing annotated equivalents is always ≥ the equivalent of the vector sum (EM-09). Does not guarantee: loops; a cut through a mesh does not partition the terminals (EMR-08).
- `pdk data needed`: none.
- `verification`: Compare each edge's drawn cross-section with the requirement from i_W (EM-20).
- `prefer over / degrade to`: Prefer over trunk-only checks. Degrade to DC KCL subtree sums (EM-09).
- `source`: EM-09, EM-20.

### EMR-08 Current distribution in meshes (loops, parallel straps)
- `id`: EMR-08
- `technique`: For nets with loops (supply and ground meshes, parallel straps) solve a lumped resistive network with one element per segment for the segment currents; cutting the loops to a spanning tree misestimates the current in parallel straps. Meshed field methods are affordable for power and ground nets when hotspots must be located (EM-47; the spanning-tree remark is the digest's status note under EM-47).
- `realizes`: Correct per-strap current for sizing and the static IR drop; via arrays in power nets lower J and static IR drop (EM-36, EM-47).
- `applicability`: Supply and ground meshes (EM-36, EM-47).
- `parameters`: per-segment resistance from sheet resistance and geometry; via resistance (EM-47).
- `guarantees` / `does not guarantee`: Guarantees: a KCL-consistent split in the lumped model (EM-47). Does not guarantee: spatial resolution of crowding, which the lumped model lacks (EM-47). Derived, not stated in the source: the EMR-07 bound-sum rule does not apply to loop edges.
- `pdk data needed`: sheet resistance per layer (Ω/sq); via resistance (Ω per cut) (EM-47).
- `verification`: Optional 2-D finite-difference solve on the net's polygons; ignore single-cell corner singularities (EM-47).
- `prefer over / degrade to`: Prefer the lumped solve for sizing and the field solve for residual hotspots (EM-47). Degrade to a spanning tree, knowing it mis-sizes parallel straps.
- `source`: EM-36, EM-47.

### EMR-09 Wire width from a resistance or IR-drop bound
- `id`: EMR-09
- `technique`: With R = ρ·L/W and C = C0·W·L: start at W = W_min, L = L_min; while R > R_max increase W by one grid step while C < C_max; if C > C_max or R > R_max the constraint is infeasible, else enforced (SUB-19). Tie EM to the IR-drop budget: allowed ΔR_fail per segment = (max_drop − present drop)/i_W, so nets with IR headroom get a void-saturation credit and tight nets get none (EM-45).
- `realizes`: The IR-drop constraint on supply and bias nets; the minimal width meeting both an R bound and a C bound (SUB-19, EM-45).
- `applicability`: Nets carrying an R bound, a C bound or an IR-drop constraint (SUB-19, EM-45).
- `parameters`: R_max, C_max, grid step ΔW (SUB-19); IR-drop headroom (EM-45).
- `guarantees` / `does not guarantee`: Guarantees: the minimal width meeting both bounds, or an infeasibility report (SUB-19). Does not guarantee: via resistance unless it is added to R; the EM width (EMR-01) still applies and the larger of the two wins.
- `pdk data needed`: sheet resistance per layer (Ω/sq); capacitance per area (fF/µm²); via resistance (Ω per cut) (SUB-19).
- `verification`: Extracted resistance and IR drop against the bound (EM-45).
- `prefer over / degrade to`: Prefer the maximum of the EM width and the IR width. Degrade to the EM width only.
- `source`: SUB-19, EM-45.

### EMR-10 Blech-length immortality per segment
- `id`: EMR-10
- `technique`: A segment is the conductor between two vias or contacts or between in-plane branches, with no branch inside: a graph edge with an anode and a cathode end. If j·L < (jL)_Blech = Ω·Δσ/(e·z*·ρ), the stress gradient balances the electron wind and no void forms; the edge may then be sized to the Blech bound instead of I/J. Unit bridge: (I/w)·L in µA ≤ 100·(jL)_B in A/cm × t in µm, since 1 A/cm = 100 µA/µm (EM-30).
- `realizes`: Credit for short segments and a segment-length cap on critical nets (EM-30, EM-33).
- `applicability`: DC-dominant segments. Measured jL: Cu 375–3700 A/cm, Al 420–3800 A/cm, about 5–100 µm for Cu at typical densities (EM-30). Separate values per via configuration (EMR-11) (EM-31). Low-k dielectrics shorten the effective Blech length (EM-41). The lowest 4–6 thin layers are the critical ones (Lienig digest §4.3.2).
- `parameters`: (jL)_B per layer and via configuration; edge length L; thickness t (EM-30).
- `guarantees` / `does not guarantee`: Guarantees: immortality for isolated two-terminal segments (EM-30). Does not guarantee: anything inside a tree, where neighbouring segments act as reservoirs and emit flux, so isolated-segment conclusions can be reversed (EM-34); immunity to void-growth saturation unless the jL² criterion is checked (EMR-21) (EM-32). The length rules I_max(5 ≤ L ≤ 10) ~ (W/L)·S, I_max(L < 5) ~ (W/5)·S, I_max(2 < W < 20) ~ W·√W·S, I_max(W ≥ 20) ~ W·S are proportionalities with units not given and valid only for simple two-terminal segments; use them only when the deck states them (EM-33).
- `pdk data needed`: per layer (jL)_B for via-above and via-below (A/cm); thickness t (µm); critical stress σ_crit (MPa; 100 MPa when absent, the Fig. 5.1 assumption); low_k flag (EM-30, EM-31, EM-34, EM-41).
- `verification`: Tree-level steady-state stress solve derived from the Korhonen steady state: along each edge dσ/dx = z*·e·ρ·j/Ω, stress continuous at nodes, volume conserved with blocking boundaries at vias; one unknown offset per tree; pass if the maximum tensile stress is below σ_crit; the initial offset is the thermomechanical stress of about 140 MPa tensile from a 200 K cool-down (EM-34, EM-40, EM-44). Otherwise the conservative whole-run domain (EM-30).
- `prefer over / degrade to`: Prefer the per-edge domain with a via-configuration split; cap edge length on critical nets at L_B(j) and insert a jog or layer change when the via cost is lower (EM-33). Degrade to the whole same-layer run as the Blech domain (conservative) (EM-30).
- `source`: EM-30, EM-31, EM-32, EM-33, EM-34, EM-40, EM-41, EM-44; Lienig digest §4.3.2.

### EMR-11 Via-below (upstream) configuration for critical segments
- `id`: EMR-11
- `technique`: Know each edge's electron-flow direction (opposite to conventional current). Label the cathode (electron-entry) end "via-above" if it lands on a cut above the layer and "via-below" if on a cut below. In layer assignment for critical DC nets prefer stacks where the high-j segment is entered from below; apply separate jL values per configuration and restrict jL more on via-above segments; for via-above, overlap the metal liners at the via and the line beneath (EM-31).
- `realizes`: Up to ×10 Blech length under growth saturation: 375 A/cm via-above versus up to 3700 A/cm via-below, i.e. 7.5 µm versus 74 µm at 5·10⁵ A/cm² (EM-31).
- `applicability`: Critical DC segments in Cu, where CMP weakens the top surface so voids form at the top, right under a via-above (EM-31).
- `parameters`: layer assignment; liner overlap at via-above (EM-31).
- `guarantees` / `does not guarantee`: Guarantees: more tolerated void volume for via-below (EM-31). Does not guarantee: benefit on AC segments where the direction reverses (EM-23).
- `pdk data needed`: blech_above and blech_below (A/cm) per layer; with one value, treat it as via-above, the conservative choice (EM-31).
- `verification`: Per-edge configuration label and jL check (EM-31).
- `prefer over / degrade to`: Prefer via-below. Degrade to a single conservative jL (EM-31).
- `source`: EM-23, EM-31.

### EMR-12 Reservoirs at the cathode end of DC segments
- `id`: EMR-12
- `technique`: Extend the metal past the cathode-end via (end-of-line reservoir: enlarged via overlap, on grid, double- and triple-patterning compatible), or add a side reservoir (a 2-D branch intercepting travelling voids, which may break double-patterning rules) (EM-35).
- `realizes`: On average ×5 permissible Blech length at the same j; TTF scales with reservoir area; a source at the cathode can make a line immortal (EM-35).
- `applicability`: Unidirectional current only: supply and bias nets. Never AC nets; avoid with low-k dielectrics. A sink at the anode lowers the stress build-up, enlarges the balanced void and raises failure probability (EM-35, EM-41).
- `parameters`: end-of-line length (an optimum exists; value not given); reservoir area (EM-35).
- `guarantees` / `does not guarantee`: Guarantees: source behaviour only for unidirectional current (EM-35). Does not guarantee: benefit on AC or low-k, where it can hurt (EM-35, EM-41).
- `pdk data needed`: end-of-line reservoir length (nm); jL credit factor (dimensionless); low_k flag (EM-35, EM-41).
- `verification`: Credit jL by the deck factor; verify the electron direction of the edge (EM-35).
- `prefer over / degrade to`: Prefer the via-below configuration first (EMR-11). Degrade to zero reservoir length when the deck gives none (EM-35).
- `source`: EM-35, EM-41.

### EMR-13 Redundant vias and via arrays
- `id`: EMR-13
- `technique`: After EM sizing, add a second cut to every single-cut via whose overlap allows it without area growth or topology change (yield redundancy against mask shift, missing polygons, particles, shallow etch, incomplete fill). Via arrays lower J and static IR drop in power nets; they are distinct from redundancy (EM-36).
- `realizes`: 2–4× TTF when the current splits evenly; yield (EM-36).
- `applicability`: All nets, except AC nets on low-k, where the gaps between vias form reservoirs that hurt (EM-36).
- `parameters`: cut count; pitch (a larger pitch gives bigger reservoirs but lower dielectric rigidity and worse distribution) (EM-37).
- `guarantees` / `does not guarantee`: Guarantees: redundancy (EM-36). Does not guarantee: an even current split (EMR-14) (EM-36).
- `pdk data needed`: via pitch and enclosure rules; low_k flag (EM-36).
- `verification`: cut count ≥ the EMR-03 requirement (EM-29).
- `prefer over / degrade to`: Prefer arrays in line with the current (EMR-14). Degrade to EM-required cuts only.
- `source`: EM-29, EM-36, EM-37.

### EMR-14 Via-array geometry aligned with the current
- `id`: EMR-14
- `technique`: At a direction change the inside-curve vias are overloaded, and in a series of vias along the wire the one on the shortest path carries the most current. Place vias at minimum spacing on the perpendicular to the bisector between the connected wires, or route one wire around the corner so both share an orientation and the array lies in line with the current, giving equal path lengths (EM-37).
- `realizes`: Uniform via stress and g(H) → 1 in the cut count (EM-14, EM-37).
- `applicability`: Corner vias of critical nets; the in-line variant needs no strict preferred routing directions (EM-37).
- `parameters`: array orientation; pitch (EM-37).
- `guarantees` / `does not guarantee`: Guarantees: equal path lengths in the in-line variant (EM-37). Does not guarantee: single-via crowding.
- `pdk data needed`: g(H) per pattern class from characterization (EMR-22) (EM-14).
- `verification`: g(H) applied in the cut count (EM-14).
- `prefer over / degrade to`: Prefer the in-line array; then a diagonal row perpendicular to the bisector. Degrade to a full-overlap fill with g(H) > 1 (EM-37).
- `source`: EM-14, EM-37.

### EMR-15 No 90° corners on current-critical wires
- `id`: EMR-15
- `technique`: Replace each L-corner of a critical segment with a 45° chamfer (two 135° bends; 135° is markedly better than 90°), leg about w, clipped to DRC. Where non-Manhattan shapes are forbidden, add an inner-corner support polygon (right triangle or square of about w_eff; the book gives no dimension). At terminals add a flared pad from wire width to pin-region width where widening is impossible (EM-25, EM-27).
- `realizes`: Lower current crowding at direction changes; damage concentrates at line ends, direction changes, layer changes, cross-section changes, material changes, existing damage, temperature and stress gradients (EM-25, EM-26).
- `applicability`: EM-critical segments, DC and AC (EM-25).
- `parameters`: chamfer leg; fill size (EM-27).
- `guarantees` / `does not guarantee`: Guarantees: fewer and milder crowding events (EM-25). Does not guarantee: a number without a crowding factor (EMR-22) (EM-46).
- `pdk data needed`: diagonal-edge legality; minimum notch and fill rules (EM-25).
- `verification`: Per-net counts of bends, vias and width steps, with crowding factors; merge collinear width steps (EM-26, EM-46).
- `prefer over / degrade to`: Prefer the chamfer; degrade to a support polygon; then to a width multiplier on corners (EM-25, EM-27).
- `source`: EM-25, EM-26, EM-27, EM-46.

### EMR-16 Current-driven Steiner topology minimizing routing area
- `id`: EMR-16
- `technique`: Topology and segment currents depend on each other. Build each critical net's tree by minimizing Σ over edges of L_e·w(i_W,e) rather than Σ L_e: start from the rectilinear Steiner minimal tree, move or merge Steiner points and re-attach terminals, recompute edge currents (EMR-07), accept area-lowering moves; join terminals with large opposite currents (source and sink) by the shortest path and branch small loads off it; hand per-edge widths to detailed routing, which becomes point-to-point with known widths (EM-10).
- `realizes`: Current-density-correct topology by construction with minimum interconnect area (EM-10).
- `applicability`: Nets above the criticality threshold with known currents (EM-10, EM-20).
- `parameters`: the width function w() from EMR-01 (EM-10).
- `guarantees` / `does not guarantee`: Guarantees: correct-by-construction current-density terminal connections and a planned topology (EM-10). Does not guarantee: global optimality (heuristic).
- `pdk data needed`: as EMR-01.
- `verification`: EMR-07 per edge after routing (EM-09).
- `prefer over / degrade to`: Prefer over length-minimal trees. Degrade to pricing each path step by current × layer resistance (EM-10).
- `source`: EM-09, EM-10, EM-20.

### EMR-17 Current-aware placement
- `id`: EMR-17
- `technique`: Add Σ over nets of I_net·HPWL_net as a weighted wirelength term normalized by the heaviest net, and keep high-current conductors away from hot devices: "place the function blocks at locations so as to minimize current flows" (EM-11, EM-48).
- `realizes`: Short high-current nets (output stages, mirrors feeding large loads, supply taps), so fewer segments need widening or Blech credit (EM-11).
- `applicability`: Global and detailed placement with op-point currents (EM-11).
- `parameters`: weight per net (EM-11).
- `guarantees` / `does not guarantee`: Guarantees: shorter heavy nets (EM-11). Does not guarantee: numbers; none given (EM-11).
- `pdk data needed`: none.
- `verification`: Downstream EM sizing (EMR-01).
- `prefer over / degrade to`: Degrade to plain wirelength.
- `source`: EM-11, EM-48.

### EMR-18 Pin-region ampacity and access-point filtering
- `id`: EMR-18
- `technique`: Export each pin rectangle split into regions, each with ampacity = region width × layer limit plus cut count × cut limit of contacts under the region. Exclude regions whose ampacity is below the arriving segment current as access points; if none remain, flag the pin (the cell must widen its strap) or split the connection into several points (EM-19).
- `realizes`: Current-density-correct terminal connections (Fig. 3.15: a U-shaped pin with regions carrying 3, 2, 1 and 0.5 mA) (EM-10, EM-19).
- `applicability`: Analog pins of varying shape, including inner fingers of wide multi-finger MOSFET straps (EM-19).
- `parameters`: region partition; arriving i_W (EM-19).
- `guarantees` / `does not guarantee`: Guarantees: no pin region is overloaded (EM-19). Does not guarantee: the pin's own sizing (EM-19).
- `pdk data needed`: layer and cut limits (EMR-01, EMR-03).
- `verification`: Per-pin check (EM-19).
- `prefer over / degrade to`: Prefer a multi-point connection when no single region carries the current. Degrade to a flag (EM-19).
- `source`: EM-10, EM-19.

### EMR-19 Criticality filter on current, never on net role
- `id`: EMR-19
- `technique`: Mark a net EM-critical when the sum of worst-case terminal currents exceeds the maximum permitted current of the minimum-size wire on the minimum layer. Before layout, bound i_W,max ≤ min(Σ|positive bounds|, Σ|negative bounds|) over all terminals (derived from the cut rule). Only critical nets receive EM constraints in routing (EM-20, EM-21).
- `realizes`: An exhaustive and provably safe classification; the non-critical majority costs nothing (EM-20, EM-21).
- `applicability`: All nets. Analog power and signal currents are of the same order, so filtering by role would drop critical nets (EM-20).
- `parameters`: I_max of the minimum wire on the minimum layer (EM-20).
- `guarantees` / `does not guarantee`: Guarantees: no critical net is dropped (EM-21). Does not guarantee: terminal correctness, which is step 1 of verification regardless (EM-20).
- `pdk data needed`: w_min (nm) and j_limit of the lowest layer (EM-20).
- `verification`: The flow: verify terminals; de-select nets below the threshold; compute J against the layer limit; drop dummy errors such as corner singularities (EM-20).
- `prefer over / degrade to`: Prefer over role-based filtering. Degrade to "unknown" without op-point currents (EM-20).
- `source`: EM-20, EM-21.

### EMR-20 Current-driven post-route decompaction (local repair)
- `id`: EMR-20
- `technique`: On an underwidth or cut violation: decompose the net into segments whose ends are sources or sinks; size wires and via arrays from the located currents; add support polygons where widening is impossible or insufficient; decompact (push neighbouring wires) while preserving topology (EM-28).
- `realizes`: Repair without another place-and-route cycle (EM-28).
- `applicability`: After detailed routing (EM-28).
- `parameters`: none beyond EMR-01 and EMR-03.
- `guarantees` / `does not guarantee`: Guarantees: topology preservation (EM-28). Does not guarantee: success when spacing blocks the shove; then feed back to the outer loop (EM-28).
- `pdk data needed`: spacing rules.
- `verification`: Re-run the width and cut checks (EM-20).
- `prefer over / degrade to`: Prefer local repair. Degrade to the outer place-and-route loop (EM-28).
- `source`: EM-20, EM-28.

### EMR-21 Void-growth saturation and the jL² / ΔR criterion tied to IR drop
- `id`: EMR-21
- `technique`: Saturated void volume V_sat = e·z*·ρ·A_Cu·j·L²/(2·Ω·B); resistance rise ΔR_sat = V_sat·ρ_b/(A_Cu·A_b) through the liner; a void spans the full cross-section at V = H²·W. A segment passes if V_sat stays below H²·W, or if ΔR_sat·I stays within the net's IR-drop headroom (EM-32, EM-45).
- `realizes`: EM margin tied to the circuit's real tolerance (absolute ΔR, i.e. IR drop) rather than relative ΔR, because transistors fail when an IR drop is exceeded (EM-45).
- `applicability`: Segments shorter than about 20 µm benefit most; nets with an IR-drop constraint (EM-32, EM-45).
- `parameters`: bulk modulus B, atomic volume Ω, effective charge z*, liner resistivity ρ_b, liner area A_b, ΔR_fail (EM-32).
- `guarantees` / `does not guarantee`: Guarantees: less restriction than the Blech criterion for short segments (EM-32). Does not guarantee: anything without deck physics; none of these values is in the book (EM-32).
- `pdk data needed`: B (GPa), Ω (m³), z*, ρ_b (µΩ·cm), A_b (µm²), H (µm) per layer (EM-32).
- `verification`: ΔR_sat·I against the IR-drop headroom (EM-45).
- `prefer over / degrade to`: Prefer when the deck gives the physics. Degrade to the Blech criterion (EMR-10) (EM-32).
- `source`: EM-32, EM-45.

### EMR-22 Pre-characterized routing patterns with crowding factors
- `id`: EMR-22
- `technique`: Once per PDK, solve the 2-D Laplace problem (j = σ·E, E = −∇Φ; fixed potential on one face, fixed j on another) on each routing primitive (straight run, L-corner, T-junction, cut stack, cut array) and store the peak-to-average current-density ratio per primitive and aspect ratio. Use it as g(H) for vias and as a width multiplier on corners; verification multiplies per-segment j by the pattern factor with no full-layout field solve. Partition at homogeneous-current cuts (straight wire away from vias and branches); via-only patterns need wire appendices; diffusion barriers are natural flux cut points (EM-46, EM-47).
- `realizes`: Quantified crowding on every corner and via; constraint-driven routing that uses only EM-robust elements (EM-46).
- `applicability`: All critical nets (EM-46).
- `parameters`: primitive set; aspect ratios (EM-46).
- `guarantees` / `does not guarantee`: Guarantees: at most 3 % error at the cut interface; one library characterization can be faster than one full FEM run (EM-46). Does not guarantee: microstructure effects, which need probabilistic treatment (EM-47).
- `pdk data needed`: a crowding-factor table per primitive (dimensionless) (EM-46).
- `verification`: This is the verification; discard corner-singularity dummy errors (EM-47).
- `prefer over / degrade to`: Prefer over full FEM of the layout. Degrade to g(H) = 1 plus the chamfer rule (EM-14, EM-25).
- `source`: EM-14, EM-25, EM-46, EM-47.

### EMR-23 Mission-profile limits: characteristic → reference → effective
- `id`: EMR-23
- `technique`: t_life,ref = AF_TF·AF_T·AF_q·t50,char with AF_TF = target lifetime / 10 years (15 years gives 1.5; reference 10 years at T_ref = 378 K), AF_T = exp[(Ea/(n·k))·(1/T_ref − 1/T_char)], AF_q = S1/exp[norminv(q_ref)·σ] with 1 < S1 ≤ 10; j_ref = j_char/(AF_T·AF_q·AF_TF); j_eff = j_ref/(AF_T(T_ref, T_eff)·AF_q(q_ref, q_eff)·AF_TF). Effective temperature of a multi-phase profile: 1/T_eff = 1/T_ref − (k/Ea)·ln(t_life,eff/t_life,ref) with t_life,eff = Σ over phases of t_s·exp[(Ea/(n·k))·(1/T_ref − 1/T_s)], computed per layer (Ea and n are layer-specific) with the highest T_eff used. Error quantile q_eff ≪ 1/m, m = number of routed segments. Re-verify whenever the temperature profile or phase durations change, even with an identical layout (EM-15, EM-16, EM-17, EM-18).
- `realizes`: The same layout re-qualified for consumer (10 years at 105 °C) and automotive profiles by recomputation (EM-15).
- `applicability`: All layers; characterization runs above 473 K; self-heating assumed negligible (EM-15, EM-16).
- `parameters`: mission {years, phases (hours, T), q_target}; S1; lognormal σ (EM-15).
- `guarantees` / `does not guarantee`: Guarantees: a design-aware quantile (more segments → stricter j) (EM-17). Does not guarantee: the printed equations' direction; as printed they mix time and current-density factors, so derive from Black's law: j_ref = j_char·exp[(Ea/(n·k))·(1/T_ref − 1/T_char)]·(t50,char/(AF_q·t_life,ref))^(1/n), and check against [JK14] (EM-15).
- `pdk data needed`: j_char at T_char (MA/cm² or µA/µm), T_char (K), Ea (eV), n, lognormal σ per layer (EM-15).
- `verification`: Persist the per-segment table (layer, width, length, current bounds) next to the GDS; a mission change re-runs only the limit scaling and the check (EM-18).
- `prefer over / degrade to`: Prefer over the deck's single reference. Degrade to the deck reference with no scaling (EM-15).
- `source`: EM-15, EM-16, EM-17, EM-18.

### EMR-24 Slotting wide high-current straps
- `id`: EMR-24
- `technique`: When the EM-sized width exceeds the deck's maximum metal width, split the strap into parallel slotted strips (rectangular or octagonal cutouts) per the deck's slot rules and recompute the width from the effective non-slot metal (EM-38).
- `realizes`: CMP maximum-width and density compliance; the strips are bamboo-like, which helps strongly in Al (below 2 µm) and weakly in Cu unless surface diffusion is suppressed; the effect needs w/D50 < 0.5 (EM-38, EM-39).
- `applicability`: Wide power straps (EM-38).
- `parameters`: slot width and pitch; effective metal width (EM-38).
- `guarantees` / `does not guarantee`: Guarantees: DRC maximum-width compliance (EM-38). Does not guarantee: a bamboo credit in Cu; resistance rises (EM-38); MTF against width is non-monotonic and a self-heating width limit bounds it from below (EM-39).
- `pdk data needed`: maximum metal width (µm); slot rules; a width-dependent limit table if the deck publishes one (EM-38, EM-39).
- `verification`: EMR-01 with the effective width (EM-38).
- `prefer over / degrade to`: Prefer parallel strips on two layers when slotting is forbidden. Degrade to DRC-driven slotting after the fact.
- `source`: EM-38, EM-39.

Items from the Lienig digest deliberately not carded, with the card that absorbs them: EM-02 lifetime metric (EMR-01 and EMR-04 verification); EM-40 thermomechanical stress (initial offset in EMR-10's tree solve); EM-43 and EM-44 flux and Korhonen theory (EMR-10 verification); EM-48 lever priority (decision procedure).

---

## 2. Numeric rules of thumb

### 2.1 Substrate noise

| Rule | Value | Source |
|---|---|---|
| High-resistivity bulk stack | uniform 20–50 Ω·cm, 400 µm, 1 µm surface layer of 0.1 Ω·cm | SUB-01 |
| Low-resistivity epi stack | ~10 µm epi of 10–15 Ω·cm on ~1 mΩ·cm p+ bulk, 300 µm, 1 µm surface layer of 1 Ω·cm | SUB-01 |
| Substrate is resistive (quasi-static valid) | below ~5 GHz; model valid to 2–5 GHz; impedances real to 4–5 GHz | SUB-01, SUB-02 |
| Epi saturation distance | 2.5–5× epi thickness (Su: 4×) for L_gnd 1–5 nH at 1 GHz; none for L_gnd = 0 | SUB-31 |
| Epi plateau at 1 GHz, 50 µm contacts | ≈ −47 dB at 5 nH; ≈ −62 dB at 1 nH; ≈ −112 dB at 400 µm only for 0 nH (figure) | SUB-05 |
| Epi, floating backplate | ≈ −22 dB at 1 GHz, independent of distance (C_sub 0.3 pF, figure) | SUB-05 |
| Bulk distance law, 100 MHz grounded | ≈ −67 dB at 0 µm → ≈ −80 dB at 400 µm, keeps improving (figure) | SUB-32 |
| Guard ring in epi | 7–10 dB only; Su: ~20 % with a dedicated ground near the sensor, distant ring none, n-type ring none | SUB-28 |
| Guard ring in bulk, 1 GHz | ≈ −45 dB at L_gr 5 nH vs ≈ −65 dB at 0 nH; 100 MHz ≈ −95 vs ≈ −103 dB (figure) | SUB-28 |
| Grounded backplate in bulk | adds 5–10 dB | Charbon digest §8.6.2 |
| Ring width | minimum width; 2→34 µm helps only at L_gr = 0; shallow optimum ~15 µm at 100 MHz with 2 nH | SUB-29 |
| Ring useful frequency | ω ≈ R1gr·R2gr / (L_gr·(R1gr + R2gr)) | SUB-33 |
| Ring on internal ground with large L_gr | worse than no ring | SUB-30 |
| Dual rings (100 MHz / 1 GHz) | none −62/−34 dB; receiver ring −99/−45 dB; two rings −130/−57 dB | SUB-35 |
| Injector ring vs receiver ring | same isolation from injector; ~20 dB better vs ring-net noise; V_inj1 dominates above 35 mA vs 3.5 mA adjacent bond-wire current (1 GHz, M = 1 nH) | SUB-34 |
| Victim load pole | 1/(2π·R_load·C_sub): 1 kΩ with 0.8 pF ≈ 200 MHz; 200 Ω ≈ 1 GHz; isolation +20 dB per decade of R_load below the pole | SUB-04, SUB-12 |
| Isolation vs load, 100 MHz, 90 µm, 3 nH, 0.8 pF | ≈ −103 dB at 10 Ω → ≈ −67 dB at 1 kΩ (figure) | SUB-12 |
| Differential pair, epi, 1 GHz | ≈ −35 dB near → ≈ −120 dB at 400 µm; 100 MHz ≈ −153 dB (figure) | SUB-43, SUB-46 |
| 5 % substrate-C mismatch in a pair | 1 GHz ≈ −120 → ≈ −80 dB; 100 MHz ≈ −150 → ≈ −123 dB (figure) | SUB-44 |
| Shared ring around a pair, bulk | DM ≈ −92…−100 dB at 1 GHz, ≈ −130…−140 dB at 100 MHz (figure) | SUB-39 |
| Body-effect reception | g_mb/g_m 0.1–0.3 → body-to-drain gain only 14–20 dB below gate-to-drain; capacitive pickup above ~1 MHz | SUB-09 |
| NMOS injection | impact ionization dominant to ≥ 100 MHz; PMOS ionization small | SUB-08 |
| Switching gate pulse | 1–100 µA, 0.5–5× gate delay; fitted A = 100 µA, τ = 6.3 ns | SUB-10 |
| Clock harmonics | n-th harmonic 20·log(n) dB below fundamental; 10 MHz 3 V clock: 1.91 V fundamental, ~GHz harmonic 19.1 mV (−40 dB) | SUB-11 |
| Substrate Miller feedback | C_s 0.2 pF each, gain 10 → 1 pF; bandwidth 1.9 → 1.8 GHz | SUB-13 |
| Bond pad to substrate | C 0.5–1 pF; R several Ω to several kΩ | SUB-40 |
| Differential AC-coupling bottom plates | loss 1/(1 + r); r = 0.2 → 1.6 dB | SUB-47 |
| Single-contact resistance, bulk | ~600 Ω at 10 µm → ~90 Ω near 700 µm, logarithmic (figure) | SUB-42 |
| Single-contact resistance, epi | ~3–5 kΩ at 10 µm → ~2 Ω at 1000 µm, inverse with area (figure) | SUB-42 |
| Gradient move estimate | 1 % within 5 grid steps; 10 % up to 1/10 of chip size | SUB-22 |
| Sherman–Morrison update | O(N²) per moved contact | SUB-21 |
| Calibration accuracy to measurement | < 15 % for voltage ratios > 0.5; good > 0.1; systematic error below 0.1 | SUB-57 |
| VCO sensitivity to V0 | ≈ 2/3, constant over 0–300 mV; jitter worst when generator delay = one inverter delay | SUB-17 |
| PLL case result | parasitic-only placement jitter 0.1; substrate-aware 0.005; spec ≤ 0.007; area ×1.28, CPU 885 s vs 407 s | SUB-23 |
| Crosstalk delay shape | ΔD ∝ L^α/S^β, α ≈ 2, β ≈ 1 | SUB-51 |
| Spur budgets | PA→LNA power ratio > 100 dB; direct-conversion loop 80–100 dB at one frequency | SUB-49 |
| Buried shield placement | around the smaller-area circuit; buried n+ better than SOI (Joardar) | SUB-38 |
| Ring measurement limits | ring wiring inductance kills isolation above 10 GHz; probe floor −40…−45 dB | SUB-57 |

### 2.2 Electromigration and IR drop

| Rule | Value | Source |
|---|---|---|
| Black's equation parameters | Al Ea ≈ 0.7 eV, n = 2; Cu Ea 0.9 eV (surface), n = 1.1–1.3; n = 1 void-growth-limited, n = 2 nucleation-limited | EM-02, EM-42 |
| Temperature sensitivity, Cu | +10 K → more than 50 % current cut at equal MTF | EM-03 |
| Temperature sensitivity, Al | −5 K → about +25 % j; 25 → 125 °C → about 90 % cut | EM-03 |
| Automotive ambient | up to 175 °C | EM-03 |
| Reference lifetime | 10 years at 378 K (105 °C); 15 years → AF_TF 1.5; characterization above 473 K | EM-15 |
| Error quantile | q_eff ≪ 1/m, m = routed segments | EM-17 |
| Current type by frequency | RMS for f < 1 Hz and analog DC; average for f > 1 Hz; peak for single events | EM-06 |
| Net-class limits | at least two: DC or f < 10 kHz, and AC above; optional steps between 10 Hz and 10 kHz | EM-22 |
| AC self-healing, Cu | MTF_AC/MTF_DC up to ×500 over 10–10⁴ Hz; minimum lifetime at duty factor ≈ 0.4 for zero-DC asymmetric pulses | EM-23 |
| Skin effect | Cu δ ≈ 9.4 mm at 50 Hz, ∝ 1/√f; critical 90 GHz at W = t = 0.45 µm, ~35 THz at 22 nm; matters for analog HF ≥ 45 GHz | EM-24 |
| Blech product, measured | Cu 375–3700 A/cm; Al 420–3800 A/cm; Cu ≈ 5–100 µm at typical j | EM-30 |
| Unit bridge | 1 A/cm = 100 µA/µm | EM-30 |
| Via-above vs via-below | 375 vs up to 3700 A/cm; 7.5 vs 74 µm at 5·10⁵ A/cm²; ×10 under growth saturation | EM-31 |
| Reservoir credit | average ×5 permissible Blech length at the same j; DC only | EM-35 |
| Parallel vias | 2–4× TTF with an even split | EM-36 |
| Bends | avoid 90°; 135° markedly better | EM-25 |
| jL² criterion | less restrictive than Blech for L < 20 µm; scales with absolute ΔR (IR drop) | EM-32 |
| Pattern-library accuracy | ≤ 3 % error at the cut interface | EM-46 |
| Bamboo regime | Al < 2 µm; Cu < 1 µm after 3 h at 400 °C; needs w/D50 < 0.5; weak in Cu | EM-38, EM-39 |
| Thermomechanical stress | ≈ 140 MPa tensile for ΔT = 200 K; stress-free T ≈ 250 °C; σ_crit assumption 100 MPa | EM-40, EM-30 |
| Material Ea (eV): bulk / grain boundary / surface | Al 1.2 / 0.7 / 0.8; Cu 2.3 / 1.2 / 0.8 | EM-42 |
| Void-migration Ea (eV) and resistivity (µΩ·cm) | Al 0.61 / 2.44; Ag 0.66 / 1.47; Cu 0.70 / 1.54; Au 0.75 / 2.03; W 1.89 / 4.84 | EM-42 |
| Cu/barrier interface Ea (eV) | Ta 2.1; Ta/TaN 1.4; SiN 0.7–1.1; SiN on Cu(Ti) 1.3; CoWP 1.9–2.4; SiCxHy 0.9 | EM-42 |
| Combined layout measures | about ×10 permissible J; thermal vias and wires lower T by only a few K | EM-48 |
| Critical layers | the lowest 4–6 thin layers; about 5 % of routing exceeds Blech by 2026 | Lienig digest §4.3.2, §5.2 |
| ITRS Cu at 105 °C, 2016 → 2028 | tolerable J 3.0 → 0.2 MA/cm²; required J 1.81 → 5.35 MA/cm² | Lienig digest §1.1 (Table 1.1) |
| Solid-state EM threshold | j > 10⁴ A/cm² | Lienig digest §2.1 |
| FIT | one failure in 10⁹ device-hours | Lienig digest preface |

---

## 3. Decision procedures

### 3.1 Isolation: given injectors, victims and the substrate kind

1. Read the substrate kind, layer stack, backplate state, L_gnd, L_gr and deep n-well availability from the deck. With no stack, make every substrate rule cost-only and report "unknown" (SUB-01).
2. Characterize every device: injector weight and peak current from type and operating point (vertical PNP, high-V_ds NMOS, clocked devices, un-grounded wells), aggressor frequency and harmonics; victim weight from body-effect gain, node impedance and block role; spur-class pairings (SUB-06, SUB-08, SUB-09, SUB-10, SUB-11, SUB-12, SUB-49).
3. Set per-victim budgets V0_max from the block's performance share and sensitivity (SUB-15, SUB-17).
4. Apply the unconditional local rules before any pair evaluation: adjacent drain tap at every vertical PNP and strong injector (SUB-37); every n-well AC-grounded to a quiet supply within d_tap and R_max (SUB-14); taps around sensitive devices within d_tap_sensitive (SUB-41); no isolation structures at ESD cells and their substrate node marked noisy (SUB-52); capacitor bottom plates to the reference node (SUB-47); isolated-plate decoupling at clocked blocks (SUB-11).
5. For every aggressor–victim pair compute α_req = V0_max/(I_peak·R_ground) and invert the kind-specific distance law into d_min; in epi clamp to d_sat (SUB-16, SUB-31, SUB-32).
6. Floorplan and place: aggressor clusters and victim clusters on opposite sides with d_min as soft separation (SUB-50, SUB-18); matched pairs oriented with the dominant aggressor on their symmetry axis and with equal substrate exposure (SUB-43, SUB-44); repeated stages abutted, folded and mirror-symmetric (SUB-45); annealing cost exact per temperature, estimated per move, exact on violation (SUB-21, SUB-22, SUB-23).
7. Where d_min is unaffordable or beyond d_sat, branch on kind:
   - epi: do not add isolation rings (7–10 dB) and never n-type rings (SUB-28); report the L_gnd needed and flag the victim as "needs backside or down-bond ground" (SUB-05); shrink victim junction areas and use large backplate-tied taps at the aggressor (SUB-31, SUB-42); then deep n-well around the smaller group if the deck has it (SUB-38).
   - bulk: minimum-width p+ ring around the injector cluster on a dedicated quiet pin, star-routed, never on the ESD or digital node (SUB-34, SUB-29, SUB-30); one shared thin ring per differential pair (SUB-39); check the ring's useful frequency against the aggressor clock (SUB-33); if still short and a second pin exists, dual rings (SUB-35); if still short, deep n-well or buried shield around the smaller group, preferring buried n+ over SOI (SUB-38). Let the annealer trade ring area against distance (SUB-25).
8. Check substrate feedback in every high-gain stage; add spacing or a ring on the output device when the substrate Miller capacitance exceeds budget (SUB-13).
9. Sensitive routes: run-length-aware spacing (L²/S) and quiet-well or shield underlay (SUB-51); low-noise input pads shielded (SUB-40).
10. Sign off: two-port and ring macromodels per pair at the aggressor frequency with deck L_gnd and L_gr (SUB-04, SUB-33); exact extraction for calibration and final numbers (SUB-53, SUB-54, SUB-57); report distance, isolation dB, V_peak against V0_max, DM/CM per pair, ring role and ring-net check, suggested remedy; failing pairs count toward the epoch residual (SUB-46, SUB-56).

### 3.2 Sizing: given a net's currents and layers

1. Build per-terminal bounds (avg, RMS, peak; lower and upper) per operating phase from transient benches; DC-only flows set lower = upper (EM-06, EM-07, EM-08).
2. Filter on current: critical when the terminal sum exceeds the minimum wire's ampacity on the minimum layer, with the pre-layout bound min(Σ|positive|, Σ|negative|); non-critical nets get minimum width and skip the rest (EM-20, EM-21).
3. Classify DC versus AC by waveform, not by role; compute the equivalent current per class (RMS or average with self-healing γ; peak for single events) (EM-22, EM-23, EM-06).
4. Place with a current-weighted wirelength term and keep heavy conductors away from hot devices (EM-11, EM-48).
5. Plan each critical net's topology by minimizing Σ L_e·w(i_e), joining source and sink by the shortest path and branching small loads off (EM-10).
6. Compute per-edge currents: trees by the LHS/RHS bound-sum rule and the maximum over phases (EM-09); loops by a lumped resistive solve (EM-47).
7. Derive the limits: mission profile to T_eff per layer and the quantile from the segment count (EM-15, EM-16, EM-17); per-segment temperature from the thermal map; derate each layer and cut limit with Ea and n, falling back to Cu 0.9 eV / 1.2 or Al 0.7 eV / 2 with a provenance note (EM-04, EM-03, EM-42).
8. Width: w_nom as the maximum of the equivalent, peak and minimum terms (EM-12); the resistance or IR-drop width where a bound exists, taking the larger (SUB-19, EM-45); process margins to w_eff (EM-13); slot when above the maximum metal width (EM-38).
9. Blech credit per edge: label the cathode end via-above or via-below from electron direction and use the matching jL (EM-31); credit edges below (jL)_B (EM-30); confirm with the tree-level steady-state stress solve against σ_crit (EM-34, EM-44); with deck physics and an IR budget, apply the jL² / ΔR criterion (EM-32, EM-45); cap edge length on critical nets and insert a jog or layer change when the via cost is lower (EM-33).
10. Vias: cut count with the temperature factor dividing and g(H) multiplying; size to the full segment current (EM-14, EM-29); arrays in line with the current or on the bisector perpendicular (EM-37); yield redundancy where free (EM-36); cathode-end reservoirs on DC nets only and never on low-k (EM-35, EM-41).
11. Corners and pins: chamfer or support polygons on critical L-corners; flared terminals (EM-25, EM-27); access points filtered by pin-region ampacity (EM-19).
12. Verify and repair: required versus drawn cross-section per segment and via with pattern crowding factors (EM-20, EM-46); lifetime margin report (EM-02); local current-driven decompaction on violations before any outer loop (EM-28); persist the per-segment table for re-verification on mission change (EM-18).

---

## 4. What the deck must carry

### 4.1 Substrate

| Datum | Unit | Used by | Source |
|---|---|---|---|
| substrate kind {bulk, epi_on_pplus, soi} | enum | SUBN-01 and every substrate card | SUB-01 |
| layer stack top-down: thickness, resistivity | µm, Ω·cm | SUBN-02, SUBN-03, SUBN-20, SUBN-22 | SUB-01, SUB-53 |
| backplate state {grounded, floating} | enum | SUBN-02, SUBN-04 | SUB-01, SUB-05 |
| L_gnd backplate ground inductance per package option | nH | SUBN-02, SUBN-04, SUBN-20 | SUB-01, SUB-05 |
| L_gr ring pin plus bond-wire inductance | nH | SUBN-05, SUBN-21 | SUB-01, SUB-33 |
| bond-wire mutual inductance M | nH | SUBN-07 (ring-net noise) | SUB-34 |
| deep n-well availability, depth, sheet resistance, enclosure rules | bool, µm, Ω/sq, nm | SUBN-09 | SUB-38 |
| junction capacitance per area and sidewall for n+/p+ diffusion, n-well, deep n-well | fF/µm², fF/µm | SUBN-11, SUBN-18, SUBN-20, SUBN-23 | SUB-07 |
| impact-ionization constants C1, C2 (BSIM alpha0, beta0 equivalents) | 1/V, V | SUBN-23 | SUB-08 |
| N_A and C_ox, or simulator g_mb | cm⁻³, fF/µm² | SUBN-23 | SUB-09 |
| transfer-model fit: R_i0(area), R12(distance) constants, k_sat | Ω vs µm², Ω vs µm, dimensionless | SUBN-02, SUBN-03, SUBN-20 | SUB-31, SUB-32, SUB-57 |
| contact depth for extraction | µm | SUBN-22 | SUB-53 |
| ring minimum width, ring-to-device clearance | nm | SUBN-05, SUBN-19 | SUB-29, SUB-36 |
| contact/cut resistance, maximum ring resistance, ring diffusion sheet resistance | Ω per cut, mΩ, Ω/sq | SUBN-05, SUBN-12 | SUB-29 |
| d_tap (latch-up) and d_tap_sensitive | µm | SUBN-10, SUBN-12 | SUB-14, SUB-41 |
| well tap resistance budget R_max, well sheet resistance | Ω, Ω/sq | SUBN-12 | SUB-14 |
| resistor layers: sheet resistance and capacitance to substrate | Ω/sq, fF/µm² | SUBN-23 | SUB-07 |
| MOM or MIM availability and density | fF/µm² | SUBN-13 | SUB-11 |
| capacitor bottom-plate parasitic ratio r | dimensionless | SUBN-25 | SUB-47 |
| interconnect capacitance per length, substrate capacitance under wires | fF/µm | SUBN-27 | SUB-51 |
| pad shield options (n-well under pad, lower-metal shield rules) | flags | SUBN-26 | SUB-40 |
| ESD cell class and rules | flags | SUBN-28 | SUB-52 |
| default aggressor clock frequency when the netlist has none | MHz | SUBN-20 | SUB-04 |

### 4.2 Electromigration and IR drop

| Datum | Unit | Used by | Source |
|---|---|---|---|
| per metal: current limit for average, RMS and peak at T_ref (or J with h_nom) | µA/µm (or MA/cm² plus µm) | EMR-01, EMR-05 | EM-01, EM-06, EM-12 |
| per via layer: current per cut at T_ref | µA | EMR-03 | EM-14 |
| T_ref, Ea, n per layer (fallback per metal family) | K, eV, dimensionless | EMR-04, EMR-23 | EM-03, EM-42 |
| h_nom, h_min, Δw, w_etch per metal | µm, µm, nm, nm | EMR-02 | EM-13 |
| w_min, maximum metal width, slot rules | nm, µm, rules | EMR-01, EMR-24 | EM-12, EM-38 |
| sheet resistance per layer, via resistance per cut, capacitance per area | Ω/sq, Ω, fF/µm² | EMR-08, EMR-09 | SUB-19, EM-47 |
| (jL)_B for via-above and via-below per layer | A/cm | EMR-10, EMR-11 | EM-30, EM-31 |
| critical stress σ_crit (100 MPa default), α, E, stress-free temperature | MPa, 1/K, GPa, °C | EMR-10 verification | EM-30, EM-34, EM-40 |
| end-of-line reservoir length, reservoir jL credit factor | nm, dimensionless | EMR-12 | EM-35 |
| low_k flag | bool | EMR-10, EMR-12, EMR-13 | EM-41 |
| self-healing factor γ | dimensionless | EMR-05 | EM-23 |
| self-heating limit or thermal resistance to substrate per layer | µA/µm RMS, K/W | EMR-04, EMR-05 | EM-05 |
| ESD current for pad and clamp nets | A | EMR-05 | EM-06 |
| g(H) crowding factor per via-array pattern and per routing primitive | dimensionless | EMR-03, EMR-14, EMR-22 | EM-14, EM-46 |
| j_char at T_char, lognormal σ, S1 per layer | MA/cm² or µA/µm, K, dimensionless | EMR-23 | EM-15 |
| B, Ω, z*, ρ_b, A_b, H per layer | GPa, m³, dimensionless, µΩ·cm, µm², µm | EMR-21 | EM-32 |
| diagonal-edge legality, notch and fill rules | rules | EMR-15 | EM-25, EM-27 |
| via pitch and enclosure rules | nm | EMR-13, EMR-14 | EM-36, EM-37 |
| width-dependent limit table, if published | µA/µm vs µm | EMR-24 | EM-39 |
