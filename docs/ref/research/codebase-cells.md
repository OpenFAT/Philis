# Philis device generators and visualizer: inventory for a Substrate2 embedding

Read in full: `kernel/cells/src/*.rs`, `kernel/cells/tests/cell_selfcheck.rs`, `kernel/macroMaster/src/{pdk/mod.rs,legacy/mod.rs,legacy/adapter.rs,build.rs}`, `kernel/visualizer/src/{lib.rs,main.rs}`; plus `kernel/core/src/{macro.rs,units.rs,process.rs,geom.rs}`, `backend/verify/src/pdk.rs:1160-1470`, `kernel/analog/src/{cell.rs,matching/class.rs}`, the `cell` section of the four decks, and the requested parts of plan-03, audit-03 and ref-00.

Path legend: a bare `mosfet.rs` is `kernel/cells/src/mosfet.rs` (same for every cells file); `builder.rs` is `kernel/cells/src/builder.rs`; `pdk/mod.rs`, `legacy/mod.rs`, `adapter.rs` are under `kernel/macroMaster/src/`; `visualizer` is `kernel/visualizer/src/lib.rs`; `core/*` is `kernel/core/src/*`; `pdk.rs` is `backend/verify/src/pdk.rs`; `selfcheck` is `kernel/cells/tests/cell_selfcheck.rs`.

## 0. Shared contract every generator obeys

| Item | Where | Contract |
|---|---|---|
| `trait Cell: Clone` | `lib.rs:46-53` | `enumerate(group, constraints, process) -> Vec<Self>` = every feasible variant, deterministic order, "never ranked: the placer picks" (`lib.rs:47-49`); `draw(&self, group, constraints, process) -> Macro`, pure and byte-deterministic (`lib.rs:8`). One exception to "never ranked": `CapArray::enumerate` sorts an Exceptional bank by (M_sys, INL, area) (`cap_array.rs:142-152`). |
| `Pattern` | `lib.rs:30-43` | `Single` (contiguous fingers), `Cc1d` (ABBA), `Interdig` (ABAB), `Chain` (series stack, odd nf per member). Used by `Mosfet.style` and `Resistor.pattern`; `cap_array` has its own `Pattern` (`cap_array.rs:40-58`). |
| Inputs | `builder.rs:248-280` | Sizing comes from the `Unitization` covering the group (subset match, `builder.rs:252`): `unit_w`, `unit_l`, per-member `dev_nf`, `device_type`, `series_parallel`, `dummy_required`, `route_matching_required`, `class`, `series` (`kernel/analog/src/cell.rs:52-76`). No unitization: one finger at the generator's deck default (`builder.rs:258-262`). `post_cell` additionally reads `Constraints::guard_rings` (`post_cell.rs:29-33`). |
| Process seam | `core/process.rs:7-101` | `layer(role)`, `rule(name, default)`, `grid()`, `sheet_ohm`, `cut_ohm`, `space` (widest of min, wide-metal, array), `min_space`, `eol_space`, `width`, `enclosure`, `endcap`, `extension`, `area`, `space_between`, `tier(key, MatchClass)`. Real impl `verify::Pdk` and its recipe `Overlay` (`pdk.rs:1165-1255`, `1457-1469`). |
| Mandatory role | `builder.rs:198-202` | `req(process, role)` panics when the deck lacks the role. |
| Sidecar vs deck | `builder.rs:208-225` | `dim(key)` = max(sidecar `cell.<key>`, deck-derived rule) for `contact`, `mcon_size`, `m1_enc`, `met1_space`, `poly_ext`, `poly_min_width`/`min_gate_l`, `min_finger_width`, `nwell_diff_enc`, `nwell_min_width`, `via_spacing`, `via_enclosure`. A sidecar key can only raise a value. |
| Pin naming | `lib.rs:10-12`, `builder.rs:230-234` | `d{i}:{T}`, `i` = member index in `group.devices`, `T` in `G S D B P N C`; the net is a placeholder `NetId(i*8 + t)` the caller rebinds by name. Non-member pins: `GND` (resistor dummies `resistor.rs:409`, general cap set dummy bus `cap_array.rs:638`) and `ring` (`post_cell.rs:208`), net `NetId(u16::MAX)` or the ring's `connection_net`. |
| Output `Macro` | `core/macro.rs:10-30` | `shapes` (rects only), `pins`, `bbox`, `units` (`core/units.rs:17-33`: owner, centre, weight, signed `phi`, `sa`/`sb`), `dummies` (`core/macro.rs:111-118`), `drawn` LVS cards (`48-70`, nodes `Pin`/`Internal`/`Unused` `74-78`), `keepouts` (`82-92`: `Gate`, `ResistorBody`, `CapPlate`), `figures` (`96-104`: per-owner S/D area+perimeter, gate ohms). |
| Builder | `builder.rs:9-148` | Grid snap with min one grid step (`59-67`); `cover_poly_cuts` draws the `npc` role over every licon on poly (`76-125`, no-op without the role); `finish` dedupes exact duplicate shapes and rounds the bbox corner to the 2-grid cut lattice and extents to 4 grid steps (`131-148`). Cut lattice = 2×grid, "magic grows an off-lattice cut" (`152-162`). |
| Placement stamp | `core/macro.rs:126-139` | `place_macro` shifts shapes, pins, keepouts; `units` stay local (world units come from `UnitLib`). |

## 1. Device block inventory

### 1.1 MOS planar: `mosfet::Mosfet`

**Variant struct** `mosfet.rs:46-58`

| Knob | Type | Meaning |
|---|---|---|
| `nf` | `u16` | Schematic fingers per device, never refolded (`mosfet.rs:95-98`). |
| `style` | `Pattern` | `Single` (blocks), `Cc1d` (diffusion-legal centroid row from `pattern::diffusion_cc_row`, `mosfet.rs:1157-1164`), `Chain` (series stack). `Interdig` is never offered: an ABAB boundary lands on a drain (`110-113`). |
| `dummies_per_edge` | `u8` | Dummy gates per diffusion end; 0 or the deck's `dummy_gates_per_end` (clamped 0..4) when `dummy_required` (`64-66`, `94`). Not a search axis: "a matched set drawn as separate cells must see one environment" (`60-63`). The drawn count is raised to the class reach (`449`). |
| `split_gates` | `bool` | Odd-indexed devices' gate bars above the diffusion, even below: two private gates for an ABBA pair (`23-27`, `539-542`). |
| `mirror_pins` | `bool` | Equal pairs only: fingers pair on shared drains in a swap-reverse-symmetric order so device 1's D and G pins are device 0's reflected about the cell centre (`29-35`, `1114-1152`). Brute force over 2^(nf/2) halves, capped at nf=32 (`1127-1140`). |
| `rows` | `u16` | 1, 2 or 4. Two rows share one tap strip, second row relabelled (pair) or reversed (`39-42`, `312-333`, `216-223`); four rows = MAT-15 order-3 grid `pattern::nth_order_rows(3, ·)` stacked pad to pad (`43-45`, `315-318`, `232-240`). Three rows is an assertion failure (`314`). |
| `double_gate` | `bool` | Every finger contacted at both ends with a strap each; offered only when a finger's poly R exceeds its gate cut R (`54-57`, `81-86`, `163`, `174`). |

**Enumerate** `mosfet.rs:89-183`: empty group → `[]`. Series unitization → exactly one `Chain` variant (even nf anywhere → `[]`), split only if bars do not fit (`101-109`). Else: `Single` if every boundary can sit on a source (`114-118`); `Cc1d` if a diffusion-legal centroid row exists (`119-120`); for an equal pair additionally `(Cc1d, split)`, `(Cc1d, split, mirror)`, `(Cc1d, mirror)`, and a route-matched pair keeps only mirror orders (`124-136`); `(Single, split)` for multi-device blocks (`141-143`); each style × rows {1, 2 if every member halves legally, 4 if equal `Cc1d` pair with nf % 8 == 0} × double_gate {false, true if wide and unsplit} (`148-178`); filtered by `bars_fit` and `grid_legal` (`181`).

**Draw** `mosfet.rs:185-211`: a row that cannot drop a boundary cut redraws split (`189-191`); rows drawn by `draw_row` and merged (`192-199`); `figures.gate_ohm` per owner = (R□·W/(k·L) + R_cut)/N_f, k = 3 or 12 (`203-209`).

`draw_row` (`343-873`) draws, in order: one continuous diff row with end regions and LOD moat (`436-463`); gates as poly stripes with stubs to a bar row below (or above for split/double) (`552-562`); one `Unit` per finger with signed `phi` and SA/SB (`564-572`); gate cuts and per-device bars with li straps (`575-623`), EXC class per-finger isolated pads instead (`357`, `583-586`, `614`); S/D cut columns at the cut pitch, li columns, pins, junction figures (`628-679`); end dummies with cuts and a poly skirt plus li risers to the tap rail (`683-763`, `Dummy` records `724-730`); a bulk tap strip above the row with its implant, cuts, li rail and one `B` pin per device (`765-822`); channel implant of the device polarity (`785-792`); `gate_marker{i}` recogniser layers over every gate (`824-845`); PMOS nwell inflated by the class WPE halo (`848-867`); `cover_poly_cuts` (`869`).

**Pins** (all on `li`, `mosfet.rs:601, 621, 674, 677, 821`): `d{i}:G` one per device at its first cut finger (mirror pins: device 1 at its last), a second `d{i}:G` on the top strap when `double_gate`; `d{i}:S`/`d{i}:D` one per S/D region, a shared region carries both neighbours' pins at the same rect (`670-678`); `d{i}:B` one per device on the tap rail.

**Invariants held by construction (tests)**: every inter-device boundary on a source, so no drain short on shared diffusion (`legal_row` `1095-1097`; tests `1606-1621`, `1643-1652`); centroid of a `Cc1d` row exact, ratioed `[2,4]` merges as A BBBB A (`1657-1670`); two rows cross-couple a blocked pair (`1841-1852`); four rows cancel moments through order 3 (`1290-1320`); split gates keep private poly islands, shared-strap ABBA shorts them (`1857-1919`); mirror pins reflect D and G pin sets within one lattice step (`1944-1975`); uniform poly pitch with dummies (`1820-1836`); contacted pitch = deck minimum, 430 nm on sky130 at L = 150 (`885-899`, `1980-1983`); gate bars are notch-free (`1990-2036`); class scales moat, dummy reach, gate overhang, WPE clearance, EXC isolated pads at 490 nm, MOD bar ≥ 1 µm off (`1410-1584`); dummy L capped at `dummy_max_l_nm` (`382-385`, `1686-1730`); every diffusion point within `tie_max_dist_nm` of its own tap on sky130/gf180/ihp (`908-994`, `2111-2142`); each finger extracts as one MOS, dummies included (`1348-1389`, `1802-1815`); S/D junction figures add up and split half/half on shared regions (`2155-2211`); gate ohm formula (`2215-2226`); every variant DRC/ERC clean on sky130 for 1/2/4 devices, ratioed, every class, long L, wide W (`1245-1284`).

**Roles read**: required `diff`, `poly`, `li`, `tap`, tap implant `nsdm`/`psdm` (`349-351`, `770-772`); optional `licon` (falls back to the poly layer, `352`), channel implant (`785`), `nwell` (`849`), `npc` (via builder), `gate_marker{i}` (`829-831`), `mcon`/`met1` for the met1 landing pitch (`999-1003`). **Rules read**: `dummy_gates_per_end`, `dummy_max_l_nm`, `licon_poly_enc`, `poly_encloses_licon_one_side`, `polycon_to_diff_spacing`, `polycon_to_pdiff_spacing`, `li_encloses_licon`, `li_encloses_licon_one_side`, `li_min_spacing`, `diff_encloses_licon`, `licon_to_gate_spacing`, `tap_encloses_licon_one_side`, `poly_min_spacing`, `li_min_area`, `guard_licon_pitch`, `sd_width`, `tie_max_dist_nm` (`365-389`, `524`, `794`, `892`, `943`); `dim` keys `contact`, `poly_ext`, `min_finger_width`, `min_gate_l`, `nwell_diff_enc`, `nwell_min_width`; deck queries enclosure/endcap/space/space_between/area/width on those roles; `sheet_ohm("poly")`, `cut_ohm("licon","poly")` (`82`, `203`); class tiers `dummy_reach_nm`, `lod_moat_ext_nm`, `wpe_clearance_nm`, `gate_ext_extra_nm` through `mos_env` (`kernel/analog/src/matching/class.rs:63-76`).

**Public helpers**: `sd_and_pitch` (`885`), `tap_reach` (`908`), `taps_in_reach` (`938`), `max_finger_for_taps` (`956`, probe sweep), `gate_islands` (`1041`), `poly_islands_at` (`1055`), `cc_row_exists` (`1103`).

### 1.2 FinFET: `finfet::FinFet`

**Variant struct** `finfet.rs:21-25`: one knob, `shared: bool` (every member interleaved on one common-centroid active vs one active per member side by side).

**Enumerate** `finfet.rs:94-104`: `[]` unless the deck has a `fin` role; always `{shared: false}`; plus `{shared: true}` when `diffusion_cc_row(dev_nf, Outer::Drain)` exists for ≥ 2 members.

**Draw** `finfet.rs:106-294`: `nfin` = nearest fin count to `unit_w` (`125-126`, drawn width differs from the requested W); per active: one uncontacted edge gate per end, not a `Dummy` (`175-181`, `217-222`); fins at the fin pitch (`182-185`); per S/D gap a trench (`sdt`), LISD strip, V0, M1 strip; sources rise to one M1 source strap, drains land V1 pads on a per-member M2 track (`189-216`); gates with LIG pad and V0 on one M1 gate strap per member, tracks per member when shared (`223-240`); one `Unit` per gate with SA/SB to the active ends (`231`); device select and `gate_marker{i}` over the select (`244-255`); a finned tap strip above with its own select, trenches, rail and `B` pins (`259-286`); nwell for PMOS (`287-292`).

**Pins**: `d{k}:S` on the M1 source strap (`li` role) (`211`), `d{k}:D` on the drain V1 pad on the `met1` role (M2 on asap7) (`215`), `d{k}:G` on the M1 gate strap (`239`), `d{i}:B` all at the first tap cut (`284-286`).

**Invariants (tests, asap7 deck)**: fin rounding (`329-336`); one edge gate per end (`339-350`); SA+SB = active width and Σsa = Σsb per member (`353-372`); shared row common centroid and Φ = 0 per member (`377-396`); DRC/ERC clean with private D and G, release build only because asap7 ERC trips a GPurify debug assertion (`399-414`).

**Roles read** (`Rules::of` `57-90`, `111-114`): `fin`, `poly`, `diff`, `sdt`, `lisd`, `lig`, `licon` (V0), `li` (M1), `mcon` (V1), `met1` (M2), `nsdm`, `psdm`, `nwell`, `gate_marker{i}`; widths/spaces of each, extensions diff↔fin, diff↔poly, poly↔diff, select↔poly, enclosures/endcaps lisd↔licon, li↔licon, select↔diff, nwell↔diff, li↔mcon, met1↔mcon, `space_between` sdt↔poly, lig↔poly/diff, diff↔poly, areas li and lig, `eol_space` li/lig/met1, `min_gate_l`. No sidecar `rule()` keys except through `dim("min_gate_l")`.

Not drawn: dummies as `Dummy` records, split gates, mirror pins, rows, double gate, `figures`, keepouts.

### 1.3 Resistor: `resistor::Resistor`

**Variant struct** `resistor.rs:162-166`: `segments: u16` (series segments per string, 1 or even, `562-584`), `pattern: Pattern` (`Single` = each string's segments adjacent; `Interdig` = common-centroid segment row `pattern::segment_row`, `551-556`).

**Value model** `ResModel` (`18-78`): R = sheet·(L+dl)/weff + head/(weff+head_dw) from recipe keys `res_sheet_mohm`, `res_head_mohm_um`, `res_dl_nm`, `res_dw_nm`, `res_head_dw_nm`, `res_narrow_permille`, `res_knee_nm`; `seg_len` inverts it so n segments keep the device value within `res_value_tol_ppm` (`71-78`, `84-86`). Self-heating helpers `self_heating_min_width_nm`/`self_heating_rise_k` (`95-111`, Hastings eq. 5.8) are exported for REL, not used in `draw`.

**Enumerate** `169-200`: `[]` without the `rpoly` role. Series unitization with per-member `series` counts (EXT-15): `Single` exact blocks, plus `Interdig` when one unit length keeps every member's value (`178-184`, `137-151`). Otherwise `feasible_segments` (the 8 squarest value-preserving counts, `562-584`) × {`Single`, `Interdig` if ≥ 2 devices}, dropping combinations whose non-adjacent joins exceed the met1 jumper tracks (`193-199`).

**Draw** `202-467`: each string column = poly strip with contacted heads (square or the deck's precision slot `res_contact_w/h`, `510-516`), body marked `rpoly` (+`rpoly_b`), `ResistorBody` keep-out (`314`), one `Unit` per segment with signed vertical `phi` and weight scaled by 1/m² for m parallel strings (`318-326`), serpentine joins on li when adjacent else met1 jumpers on a per-string track with two mcon (`335-354`), one `Drawn` resistor card per segment with `Internal` nodes (`357-362`); end dummies on a matched array (≥ 2 devices or `dummy_required`): plain poly columns at the segment pitch, contacted, pinned `GND`, count from `resistor_env` (`369-411`); `rpm` + `npc` region, `res_implant` with `res_keepout`, `res_block` salicide block (`415-463`); `cover_poly_cuts` (`465`).

**Pins** (li): `d{i}:P` at the string's entry head, `d{i}:N` at its exit (`337`, `365`); m parallel strings give m pins of the same name (`807-820`); `GND` on every dummy column (`409`).

**Invariants (tests)**: Σ segment values within 0.5 % of the model (`785-797`); n = 4 not offered under `res_min_segment` (`800-804`); m strings draw 2m cards and extract one R per segment (`807-833`); ratio arrays common centroid (`672-683`); high_po ratios fall back to exact blocks (`686-700`); dummies at the segment pitch, EXC spans 10 µm (`703-723`); every join has the same via count (`726-738`); per-member current directions cancel (`741-747`); DRC/ERC clean for 1/2/3 members, 10 and 40 µm, high_po and generic_po recipes, series arrays MOD/EXC (`606-648`).

**Roles read**: required `poly`, `rpoly`, `li`, `licon`, `mcon`, `met1` (`208-212`); optional `rpoly_b`, `rpm`, `npc`, `res_implant`, `res_block` (`311`, `422-428`, `442`). **Rules**: the `ResModel` keys, `res_value_tol_ppm`, `res_min_width`, `res_min_segment`, `res_seg_gap`, `res_head`, `res_contact_w/h`, `res_keepout`, `rpm_encloses_poly`, `poly_encloses_licon_one_side`, `li_encloses_licon` (`139`, `222-226`, `419`, `430`, `504`, `538`, `544`); `dim` keys `contact`, `mcon_size`, `m1_enc`, `met1_space`; tier `res_dummy_span_nm` via `resistor_env` (`class.rs:94-102`); deck enclosure/endcap/space/space_between/width/area/extension on those roles.

### 1.4 Capacitor (single merged plate): `capacitor::Capacitor`

**Variant struct** `capacitor.rs:39-42`: `units_x: u16` (column count of the unit grid, picks aspect only: a device's units merge into one plate `179-185`), `kind: Kind` (`28-32`): `VerticalInOneLayer` lateral comb on met1, `HorizontalAcrossLayers` met1/met2 plates, `VerticalAcrossLayers` met1/met3 bottom with met2 top strapped in a side column.

**Enumerate** `48-68`: every divisor of the total unit count × every feasible kind (`feasible_kinds` by metal/via stack depth `300-313`), truncated to 16 (`45`, `66`). **Draw** `70-89`: one plate per device left to right, `device_gap` apart. No `Unit`, `Drawn`, keep-out or LVS marker: "capacitors are an LVS reference skip" and "nothing may be drawn between stacked plates; the router is not told this" (`23-26`).

**Pins**: `d{i}:N` bottom (exposed border strip or the strap rail) on the bottom metal, `d{i}:P` top plate on the top metal (same metal for the comb) (`83-84`, `209`, `235`, `283`).

**Invariants (tests)**: comb electrodes never connect (`414-448`); plate area = n·W·L in every aspect (`452-460`); kinds occupy distinct stacks (`465-480`); bounded enumeration covering every kind (`484-492`); a one-metal deck degrades to the comb (`497-504`); DRC/ERC clean for 1 and 2 devices of 4 units (`332-344`).

**Roles read**: `met1..met5`, `via1..via4` by position (`289-296`). **Rules**: `contact`, `via_enclosure`, `via_spacing` (dim), `plate_spacing`, `device_gap`, `mom_finger_width`, `mom_finger_space`, `cap_unit_side` (`126-156`, `317`); deck width/space of via1, via2, met1..met3.

### 1.5 Capacitor array: `cap_array::CapArray`

**Variant struct** `cap_array.rs:65-69`: `pattern: Pattern` (`40-58`): `Spiral`, `Chessboard`, `BlockChessboard { k, bs }` (DACP Algorithm 1), `Split { lsb, bridge_top_lsb }` (split DAC, never enumerated, built by `CapArray::split` `282-294`); `tall: bool` (rows > columns, odd bit counts only).

**Enumerate** `107-154`: `[]` without a plate stack (`184-195`: `met1`, `met2`, `met3`, `via1`, `via2` must all exist, recipe roles `bottom`/`plate`/`top_contact`/`top`/`strap`/`bottom_contact` override the MOM defaults). A binary bank `[1, 1, 2, …, 2^(N-1)]`, 2 ≤ N ≤ 8 (`163-168`): Spiral, Chessboard, BlockChessboard for every nesting k with two block sizes, × tall, deduplicated by assignment (`120-140`); Exceptional class sorts by (M_sys, INL, area) at `RANK_G_PER_UM` (`142-152`). Any other set of ≥ 2 devices (or a single MIM) with ≤ 256 units: Spiral (`Fill::Compact`) and Chessboard (`Fill::Dispersed`) over `pattern::centro_assign` (`111-119`, `398-402`).

**Draw** `156-158` → `build` (`375-724`): unit grid with a dummy ring (MOM always; MIM only when `dummy_required`, `495`); MOM unit = met1 BOT, inset met2 TOP, centre via2, dummies with shorted plates (`562-571`); MIM unit = `bottom` plate enclosing the netlist's W×L `plate`, `top_contact` array, `Drawn` capacitor card, `Unit`, `CapPlate` keep-out (`342-372`); bottom plates routed inside the macro: met1 stub, via1, met2 branch across the column channel, via1 onto the slot's met1 track, per-slot met2 bus below the array (`572-596`, `612-642`); TOP: met3 strap per column joined above (MOM), `top` strap over each column joined under the top dummy row with a met3 island hop (MIM), or per-bank lines in alternating row gaps with a spine each side (Split) (`643-722`). TOP metal never crosses a bottom-plate route (`16-17`). `metrics` (`299-332`) returns `ArrayMetrics` (`73-97`): worst centroid offset, second-moment residue, cancelled order, INL/DNL, M_sys, route spread, vias, area.

**Pins** (met1 leads): `d{s}:N` per slot at its bus end (`640`), `GND` for a general set's dummy bus (`637-638`); `d{i}:P` for every member on the one TOP lead, two leads for Split (`719-721`).

**Invariants (tests)**: exact unit counts and point symmetry of every even slot (`783-799`); spiral centres C0/C1, chessboard colours the MSB (`804-812`); block chessboard keeps the core (`816-821`); Exceptional ranking (`841-870`); spiral least wire, chessboard least INL (`876-883`); DRC/ERC clean for 4 and 5 bit banks and general sets `[8,1]`, `[2,2]`, `[3,5]`, `[4,4,4]` (`887-902`); MIM bank clean ringed or not, extracts one C per unit, unit value = model C_u 53 282 aF (`969-1010`); lone MIM is one unit (`1015-1029`); split bank point-symmetric with two top nets (`1046-1078`); bridge = attenuation cap (`1086-1107`); extracted bit ratios within 6 % per unit (`1113-1146`).

**Roles read**: `met1`, `met2`, `met3`, `via1`, `via2` (`408`) plus recipe roles (`187-193`). **Rules**: `plate_spacing`, `cap_unit_side`, `c_area_af_um2`, `c_perim_af_um`, `c_dw_nm` (`429`, `203-206`, `728`); deck space/eol_space of the metals and cuts, enclosures/endcaps of each cut in its metals and of plate↔bottom/top_contact, widths, areas of met2 and bottom, `space("plate")`.

### 1.6 BJT: `bjt::Bjt`

**Variant struct** `bjt.rs:25-28`: `rows`, `columns` of a unit grid; owners from `pattern::centro_assign(Fill::Balanced)` (`59`): every member point-symmetric when at most one count is odd, 1:8 is the 3×3 with the single unit centred (`17-23`).

**Enumerate** `31-51`: `[]` for an NPN unless the deck has `dnwell` and `npn_isolation` (`38-40`); multi-member: `pattern::grids(counts, 3.0)`; single: columns {1, n, ⌈√n⌉}.

**Draw** `53-70`, `Unit::new` `89-156`, `Unit::draw` `158-225`: per unit an emitter (diff + implant + licon array + li plate + `E` pin), a base `tap_ring` (PNP: n-tap in a filled nwell = the base; NPN: p-tap), a collector `tap_ring` (PNP: p-tap on the substrate; NPN: n-tap on the isolating nwell ring with a deep nwell), `pnp`/`npn` marker, one `Unit` and one `Drawn` card `[C, B, E]` (`168-177`). Emitter size = the recipe's `bjt_emitter_w/l` when set (fixed-geometry sky130 models), else `unit_w/l` floored at `bjt_min_emitter_side` (`93-105`). Neighbouring PNP units share their collector band; NPN units keep their own isolation at the deck spacings (`60-63`, `148-154`).

**Pins** (li): `d{i}:E` on the emitter plate (`199`); `d{i}:B` on each of the four base bands and `d{i}:C` on each of the four collector bands (`post_cell.rs:298` copies the pin per band).

**Invariants (tests)**: DRC/ERC clean for 1 and 2 members NPN/PNP, `[1,8]`, `[2,2]`, `[1,4]`, and the `pnp_3p40` recipe (`249-284`); fixed-geometry recipe sets every card to 3400×3400 (`289-308`); 9 drawn cards for `[1,8]` (`313-331`); every a:b PNP ratio with ≤ 1 odd count is common centroid within 1 nm (`337-367`).

**Roles read**: `diff`, `li`, `licon` (`164`), `tap`, `nsdm`, `psdm` (through `tap_ring`), optional `nwell`, `dnwell`, `pnp`, `npn` (`209`, `222`). **Rules**: `npn_isolation`, `bjt_min_emitter_side`, `bjt_emitter_w`, `bjt_emitter_l`, `min_guard_ring_width`, `diff_encloses_licon`, `li_encloses_licon`, `nwell_min_spacing`, `contact`, `nwell_diff_enc` (`38`, `93-107`, `124-127`, `185`, `196`); deck enclosure/endcap/space/space_between on implants, tap, diff, licon, nwell, dnwell.

### 1.7 Diode: `diode::Diode`

**Variant struct** `diode.rs:24-27`: `rows`, `cols`; owners from `centro_assign(Balanced)` (`94`). **Enumerate** `30-44`: same grid rule as the BJT. **Draw** `46-143`: per unit a cathode diffusion with implant, licon array and li plate (`101-112`), an anode tap pad of licon.7-legal width with a cut column (`114-125`), `diom` LVS marker over the unit (`132-135`), one `Unit` and one `Drawn` diode card `[P, N]` (`137-138`). A deck with a `diode_mk` role gets the well form instead: p+ anode under the marker, n-tap cathode, one nwell per unit (`79-83`, `127-131`); it is "drawn but unusable" because the recogniser's nwell terminal is not a conductor (`16-19`).

**Pins** (li): `d{i}:N` cathode, `d{i}:P` anode tap (swapped in the well form, `81`).

**Invariants (tests)**: DRC/ERC clean for 1, 2 devices and `[4]`, `[2,2]` (`156-173`); a multiplied diode draws n units with Σ area = n·W·L (`179-196`); anode tap ≥ 410 nm (`201-216`); matched sets share a centroid exactly (`221-244`).

**Roles read**: required `diff`, `tap`, `li`, `licon`, `nsdm`, `psdm` (`54-55`); optional `diom`, `diode_mk`, `nwell` (`78-80`). **Rules**: `diode_w`, `diode_l`, `diode_gap`, `diff_encloses_licon`, `li_encloses_licon`, `tap_encloses_licon_one_side`, `li_min_area`, `nwell_min_spacing`, `contact`, `nwell_diff_enc` (`57-58`, `66`, `74`, `80-85`, `147`).

### 1.8 Inductor: `inductor::Inductor`

`inductor.rs:18-28`: no knobs; `enumerate` returns `[]`, `draw` an empty macro. The former met1 spiral shorted every turn (AC-03) and no deck recognises an inductor, so signoff reports `cell/undrawable` (`1-8`). Nothing to embed.

### 1.9 Post-cell geometry: `post_cell`

Not a `Cell`. Functions over placed macros, drawn after placement and before routing (`post_cell.rs:1-5`):

| Function | Signature | What it draws |
|---|---|---|
| `guard_rings` | `(layout: &Layout, c: &Constraints, process, cut_ohm: f32) -> Vec<Macro>` (`26`) | One ring macro per cluster of `GuardRingRequirement`s; same-class shareable requesters within `guard_ring_merge_gap_nm` (default 2 µm, `17`, `39`) merge unless the hull would cover a foreign cell (`79-162`). Inner box = union of placed bboxes shrunk by `ring_halo` (`58-64`). |
| `draw_ring` (private) | `204-219` | Four contacted bands of tap + implant + li through `tap_ring`, no met1 band (`202-203`); a `Tub` adds a band nwell and a deep nwell (`212-218`). |
| `tap_ring` | `pub(crate)` (`240`) | Bands `gap` outside `inner`, `rows` cut rows at the densest legal pitch, a copy of `pin` on each band, well shape `None`/`Filled`/`Band` (`301-316`). Also used by the BJT. |
| `ring_halo` | `pub` (`325`) | Placement reservation: band + gap + outer clearance + implant growth. |
| `drawable` | `pub` (`427`) | `Tap` always; `Ecgr` needs `nwell` + `nsdm`; `Hcgr` needs `retrograde_pwell`; `Tub` needs `dnwell`, `nwell`, `nsdm` and `enclosure(dnwell, nwell)`. |
| `band` | private (`474-486`) | Band width and cut rows: rows added while `cut_ohm / cuts > max_ring_resistance_mohm` (ring R = parallel cut R only, `470-473`). |
| `ring_ohm` | `pub` (`490`) | A drawn ring's R = `cut_ohm / cuts`. |
| `may_share_well` | `pub` (`508`) | Hastings §14.1.5 predicate over `CellFlags` (`498-502`). |
| `well_bridges` | `pub` (`530`) | Fills the gap between facing nwells of same-bulk PMOS cells (≤ 2× well spacing), hull or overlap, blocked by foreign diff/poly/psdm or a third well; legality through `can_share` per merged well (`585-601`). |
| `implant_bridges` | `pub` (`1080`) | Fills sub-spacing gaps between same-type implant rects sharing a span, never over the opposite implant. |

**Pins**: `ring` on every band, net = `connection_net` (`208`, `298`). **Roles**: `li`, `licon`, `tap`, `nsdm`, `psdm`, optional `nwell`, `dnwell`. **Rules**: `guard_ring_merge_gap_nm`, `min_guard_ring_width`, `diff_encloses_licon`, `guard_licon_pitch`, `nwell_min_spacing`, `retrograde_pwell`, `contact`, `nwell_diff_enc`, `nwell_min_width` (`39`, `354`, `361`, `431`, `462`, `478`, `533-535`); deck spaces of implants/tap/diff/nwell/dnwell/licon, enclosures dnwell↔nwell, nwell↔dnwell, nwell↔tap, implant↔tap, `space_between(nwell, diff)`.

**Invariants (tests)**: same-bulk PMOS cells bridge into one clean well, different bulks or active in the gap do not (`617-641`); offset wells bridge over hull or overlap (`692-730`); forbidden pairs never bridged, directly or transitively (`735-751`); ECGR well is a four-rect band clear of the NMOS by difftap.9 and ≥ well width, clean (`876-911`); tub clean with one deep nwell, isolated bulk extracts on its own net, halo clears dnwell spacing (`931-993`); cluster policy table tests (`996-1060`).

## 2. Substrate embedding today

### 2.1 `MosTile<M>` on generated PDKs (`pdk/mod.rs`)

- **PDK types**: `PhilisPdk` (`54-74`) = Substrate layout + schematic schema with a `verify::Pdk` deck, GDS pair mapping both ways, a `mos` primitive constructor and `export_spice`. `MosModel` (`77-82`) names the deck recogniser model and polarity. `build.rs` generates `Layer` and `Model` enums for gf180mcu, ihp_sg13g2 and asap7 from each deck's drawn GDS pairs and recognised MOS models (`build.rs:9-10`, `20-45`); `define_pdk!` (`pdk/mod.rs:351-451`) emits the schemas, `Primitive<Model>` to SPICE conversion (`96-103`: an `M` card with `w` total, `l`, `nf`) and `DrawnPdk` (`158`, `426`).
- **Schematic block** `Mos<M>` (`108-154`): `MosIo` (`D G S B`), a `PrimitiveBinding` on the PDK primitive.
- **Layout block** `MosTile<M>(Mos<M>)` (`173`), only where `M::Pdk: DrawnPdk` (`184-237`). `draw()` calls `draw_mos` (`280-291`): finger width = `w / nf` (`283`), the model's recogniser overlay for flavour markers (`284-285`), `FinFet` on a `fin` deck else `Mosfet` picked by `m.nf == nf && m.rows == 1` through the legacy adapter, which synthesises a one-device `Unitization` (`adapter.rs:28-42`), keeps only accepted variants (all when none match) and returns the smallest-bbox one (`adapter.rs:43-49`). No dummies (`dummies = false`, `pdk/mod.rs:287-289`), no class, no `taps_in_reach` filter (that lives in `frontend/library/src/cellgen.rs:971`).
- **Ports**: `MosTileIo` (`162-168`) = `d`, `g`, `s`, `b` as `Array<Signal>`, one entry per region the drawing leaves unjoined. `regions()` (`241-276`) union-finds the macro's shapes per layer and groups pins by terminal letter and the island their rect touches, sorted `D G S B` then left to right; `io()` counts them by drawing the tile (`204-208`). Each region becomes a `PortGeometry` whose primary shape is the first pin rect and `unnamed_shapes` the rest (`225-235`).
- **Geometry**: every `Shape` goes through `layer_map` (`294-300`): deck `LayerId` → GDS pair (`Pdk::layer_gds`) → `P::from_gds`; a deck layer with no GDS pair is an error. `rect()` converts nm `Rect` to Substrate `Rect::from_sides` (`302-304`). `Layout::Data = ()` (`217`): units, dummies, drawn cards, keepouts and figures are dropped.
- **GDS out**: `to_gds` (`317-348`) maps a LayIR library to `GdsLayer`s and adds a text label per port shape at its centre on the conductor's label layer (`Pdk::label_gds`, `320-322`, `334-343`); `GDS_UNITS` = 1 nm (`311`).
- **Signoff**: `signoff::run` (DRC/ERC/LVS/PEX of a `Layout + Schematic` block), `run_layout` (DRC/ERC only, for a tile), `check` (an exported LayIR library plus SCIR netlist) (`kernel/macroMaster/src/signoff.rs:148`, `165`, `196`). Labels: one per shape of every top-cell port; the reference flattens SCIR to cards, a MOS as `nf·m` per-finger cards (`signoff.rs:19-24`).

### 2.2 What sky130 does instead

`impl PhilisPdk for sky130::Sky130` (`pdk/mod.rs:459-479`) uses Substrate2's own `Sky130` schema, `Sky130Layer` GDS mapping and `sky130::Primitive::Mos` with `MosKind` (`481-505` maps kinds to deck model names). sky130 is not `DrawnPdk`, so `MosTile` does not exist for it; "sky130 uses upstream's own tiles" (`19-20`, `157`). The cells generators are therefore unused in the Substrate path on the one deck they are self-checked against most.

### 2.3 Older composition API (`legacy/mod.rs`)

`DeviceGen::layout(&mut DeviceBuilder<P>)` draws raw rects and pins with an off-grid check (`93-102`, `301-331`); `Composition::build(&mut CompBuilder<P>)` instantiates, aligns (`AlignMode` `145-156`), places with overlap/duplicate/grid checks (`407-420`), mirrors a matched partner while refusing an odd-finger mirror that would flip Σφ (`380-393`), and declares nets by name (`429-431`); `build_with` resolves nets by union-find, qualifies pins `inst.pin`, and emits a `Netlist` when every instance declares `GenDevice`s (`462-540`). Shipped `variants`: `Mos` (adapter → `Mosfet`, pins lower-cased, dummies as extra off-transistor cards `626-639`), `MatchedPair` (two-member `Mosfet`, `d0:G→g1`, `d1:G→g2`, `B→b`, `690-727`), `Res` (`P→a`, `N→b`, `742-775`). `Instance::orient` turns units' `phi` with the geometry (`195-207`). `Macros` registry shadows auto-draw by instance name (`785-802`). This tier carries no layer typing and no Substrate IO.

### 2.4 What a full embedding of every generator needs

Common to all:

1. **Variant as block parameters.** Substrate caches by `Block: Hash + Eq`; the cells variant structs derive only `Clone` (`mosfet.rs:46`, `finfet.rs:21`, `resistor.rs:162`, `capacitor.rs:38`, `bjt.rs:24`, `diode.rs:23`) and `CapArray` `Clone + Debug` (`cap_array.rs:64`). The block must carry the variant plus the group sizing the adapter synthesises today (`adapter.rs:28-42`: `dev_nf`, `unit_w/l`, `series_parallel`, `dummy_required`, `route_matching_required`, `class`, `series`), and must name the variant instead of taking the smallest bbox (`adapter.rs:46-48`).
2. **Process behind the PDK.** Every generator takes `&dyn Process`; `P::deck()` supplies it (`pdk/mod.rs:190`). A Substrate PDK without a verify deck (roles in `cell.layers`, sidecar keys, recipes, recognisers) cannot drive any generator.
3. **Layer mapping** by GDS pair (`294-300`); computed deck layers and label layers need `label_gds` (`320-322`).
4. **Ports from `d{i}:T` pins.** `regions()` handles one device with `D G S B` (`182`, `241-276`); other generators need their terminal sets, per-member arrays (`d{i}`), and the non-member pins `GND` and `ring`.
5. **Schematic primitives.** Only `Primitive::Mos` exists (`86-89`). R, C, D, BJT need SCIR primitives, SPICE conversion, and deck recognisers so `signoff` can type them (`signoff.rs:20-24`); multi-card devices need the `Macro::drawn` list (segments with `Node::Internal`, MIM units, emitters) and `Macro::dummies` (off-transistor cards, as `legacy` does `626-639`) turned into schematic instances or the LVS reference will not match.
6. **Non-geometry payload.** `units`, `figures`, `keepouts`, `drawn`, `dummies` have no home in `Layout::Data = ()`; matching (`UnitLib`), routing keep-outs and PEX figures need a typed `Data`.
7. **Post-place passes.** `guard_rings`, `well_bridges`, `implant_bridges` need every placed macro (`post_cell.rs:26`, `530`, `1080`); `ring_halo` must inflate the reserved footprint before placement (`325`). These are tile-level or floorplan-level hooks, not device blocks. `tap_ring` is `pub(crate)` (`240`) and would need exporting to become a ring tile.

Per generator:

| Generator | Needed beyond the common list |
|---|---|
| `Mosfet` | Expose `style`, `split_gates`, `mirror_pins`, `rows`, `double_gate`, `dummies_per_edge` and the matched-set inputs (`class`, `dummy_required`, `route_matching_required`); multi-member groups (`dev_nf` per member, `Chain`) as one block with per-member port arrays (today one device, `rows == 1`, `pdk/mod.rs:289`); the `taps_in_reach` filter (`cellgen.rs:971`); second `G` region for `double_gate` (`mosfet.rs:620-622`) already falls out of `regions()`; `Figures` and `Unit`s as `Data`; flavour overlay already done (`284`). |
| `FinFet` | `shared`; schematic `w` from the drawn fin count (`finfet.rs:125-126`), not the request; D pins on the `met1` role (M2) while S/G/B are on `li` (M1) (`211-239`); edge gates are not `Dummy`s (no cards). |
| `Resistor` | Recipe overlay per model (`cellgen.rs:977-980`, `pdk.rs:1437-1452`); `segments`, `pattern`; repeated `d{i}:P/N` pins for parallel strings (`resistor.rs:814`) collapse into one port each; `GND` dummy port; series chain of R primitives from `Drawn` cards; `ResistorBody` keep-outs. |
| `Capacitor` | `units_x`, `kind`; no `Drawn`, no units, no recogniser for MOM (`capacitor.rs:23-26`): a schematic C has nothing to LVS against; the no-via zone between plates must become a keep-out or routing blockage. |
| `CapArray` | One block per bank: ports `n[i]` per slot, `p` (two for `Split`), `gnd` for a general set; MIM recipe overlay (`cellgen.rs:985`); `CapArray::split` constructor (`282-294`) outside `enumerate`; `metrics` as `Data`; `Drawn` capacitor cards only for MIM (`369`). |
| `Bjt` | `rows`, `columns`; `C B E` per member with four pins per ring band (`post_cell.rs:298`); a PNP's `C` is one substrate net for every member (`lib.rs:135-137`); recipe overlay for fixed emitters (`bjt.rs:97-104`); NPN needs `dnwell` and `npn_isolation`; `pnp`/`npn` markers by literal layer name. |
| `Diode` | `rows`, `cols`; `P N` per member, a substrate diode's `P` common (`lib.rs:138`); `diom` marker; the `diode_mk` well form is not extractable (`diode.rs:16-19`). |
| `Inductor` | Nothing: no construction exists. |
| `post_cell` | `GuardRingRequirement` inputs (`kernel/analog/src/cell.rs:8-21`), a placed `Layout` (`core/layout.rs`), `cut_ohm` from the deck's pex; `ring` port with the ring's net; `well_bridges` needs each cell's `B` pin net (`post_cell.rs:544-546`) and the `can_share` predicate from substrate tags. |

## 3. Self-check method

**Where**: a `testkit` in `lib.rs:57-170` (cfg(test)), a `tests` module per generator, and the integration test `kernel/cells/tests/cell_selfcheck.rs`. `verify` is a dev-dependency only, so there is no build cycle (`kernel/cells/Cargo.toml:10-14`).

**Decks**: sky130 from `pdks/sky130.json`, skipped (test returns Ok) when the file is absent, failing when present but broken (`lib.rs:63-67`, `selfcheck:30-35`); FinFET tests use `pdks/asap7.json` and panic when it is missing (`finfet.rs:309-313`); cross-deck tests load sky130, gf180mcu, ihp_sg13g2 and asap7 (`selfcheck:337`, `369`, `459`).

**Checks**:

| Check | How | Where |
|---|---|---|
| DRC + ERC in isolation | `verify::drc` and `verify::erc` on one drawn macro, pins labelled at their centres; findings on `*_density`, `floating_gate` and `soft_connection*` filtered (chip-level, no driver, taps meet through the substrate) | `lib.rs:110-117`, `selfcheck:528-533` |
| Label policy | Terminals in `common` named by the terminal alone (one net for all members), others `d{i}_{T}` private; default common = `G S B`, plus `C` for PNP, `P` for a substrate diode | `lib.rs:121-149`, `156-169`; `selfcheck:517-525` |
| Private-gate ERC | Only `S B` common, so a poly or strap joining two gates is a short; the shared-strap ABBA must trip it (control) | `mosfet.rs:1887-1919`; `finfet.rs:410` (`S B`); resistor/cap sets label nothing common (`resistor.rs:625`, `cap_array.rs:961`) |
| Sweep both dummy settings | `dirty` runs every variant with `dummy_required` false and true | `lib.rs:92-104` |
| Extraction count | `verify::extract_spice(..Detail::Schematic)` and count `M`/`R`/`C` lines against `units + dummies`, `drawn`, or 1 per unit | `mosfet.rs:1590-1601`, `resistor.rs:823-833`, `cap_array.rs:982-991`, `1072-1076` |
| Extraction does not abort | `verify::Checker::device_count(shapes).is_some()` (implantless or double-implanted channels abort) | `selfcheck:212-235` |
| Implant covers every channel, PMOS well encloses diffusion, bbox contains every shape, half-extent on the (hard-coded 10 nm) lattice | geometry predicates over `on_layer` by deck layer name | `selfcheck:167-209`, `238-259`, `262-282`, `285-300` |
| Pin shares | `pnr_core::pin_shares` sums to 1 per terminal and each pin touches 1 or 2 fingers | `selfcheck:123-144` |
| Axis coverage | labels must contain `dummies=0`, `dummies=1`, `Single`, `n=2 nf=2 dummies=1 Cc1d`, `n=4 nf=4 dummies=1 Cc1d` so the loops cannot pass vacuously | `selfcheck:151-164` |
| Well construction per bipolar kind | PNP nwell without dnwell; NPN nwell only with dnwell | `selfcheck:306-323` |
| Every deck, enumerate | each kind through the flow's generator choice must enumerate unless `UNDRAWABLE` lists `(deck, kind)`; a stale entry fails too; debug build | `selfcheck:400-481` |
| Every deck, DRC/ERC | MOS (planar or fin) at four sizes plus chain stacks; resistors through every recipe; cap, 5-bit bank, `[2,2]` set; `[1,8]` BJT; **release build only** ("asap7 ERC trips a GPurify debug assertion") | `selfcheck:332-394`, `486-562` |
| Matching claims | centroid, moments, Φ, pin reflection, pitch, class environment, value fidelity (listed per generator in §1) | per-generator tests |
| Coupling measurement | `verify::signoff_with_caps` extracts TOP-to-bit coupling per bank variant, 6 % tolerance | `cap_array.rs:1113-1146` |

**Gaps**:
- Cross-deck DRC/ERC and the FinFET DRC are `#[cfg(not(debug_assertions))]` (`selfcheck:332`, `363`; `finfet.rs:402`), so a debug `cargo test` never runs them.
- Missing sky130 deck skips silently with a `return` (`selfcheck:124-127` and every `let Some(pdk) = … else { return }`).
- `G S B` common labelling hides a gate short between members except in the dedicated split-gate test; `S` common hides source shorts everywhere.
- `cell_selfcheck` sweeps only `Mosfet` across sizes; BJT/diode/cap go through one size each; `FinFet` only on asap7.
- No LVS against a schematic card inside `cells` (counts only); no parameter comparison (plan-03 §5 Q2: GPurify measures W/L for MOS only).
- `Capacitor` (MOM) has no recogniser, so only DRC/ERC is checked (`capacitor.rs:23-26`); BJT has no sky130 recogniser (plan-03 §5 Q6), so extraction is unchecked.
- The diode well form is known unusable and untested (`diode.rs:16-19`).
- The cut-lattice test hard-codes 10 nm rather than `2·grid` (`selfcheck:291-295`).
- No golden geometry, no antenna, density or PEX checks at cell level; `metrics` tolerance 6 % (`cap_array.rs:1143`).

## 4. Visualizer API (`kernel/visualizer`)

Dependencies: `pnr_core`, `wgpu 24`, `winit 0.30`, `bytemuck`, `pollster`, `earcutr` (`kernel/visualizer/Cargo.toml:15-20`). No `verify` dependency: layer names come from the caller (`visualizer:54-56`). Re-exported as `library::visualizer` (`frontend/library/src/lib.rs:78`).

| Public item | Contract | Where |
|---|---|---|
| `struct Poly { layer: u16, datatype: u16, pts: Vec<[i32; 2]> }` | One polygon on a GDS pair, nm, y-up. | `38-42` |
| `struct TextEntry { x, y, text }` | One label at a point. | `45-49` |
| `type LayerMap = HashMap<(i32, i32), String>` | GDS pair → layer name. | `52` |
| `fn layer_names(pairs: &[(String, (u16, u16))]) -> LayerMap` | Build the map from a deck's `(name, gds)` pairs; last pair on a key wins. | `58-60` |
| `fn parse_gds(data: &[u8]) -> (Vec<Poly>, Vec<TextEntry>)` | Parse GDS bytes: BOUNDARY, BOX, PATH (rectified per segment), SREF, AREF, TEXT; flatten the top cell (first unreferenced, else any) through SREF/AREF transforms with a depth cap of 64. Malformed records stop the scan without error. | `403-413`, `149-342`, `352-400` |
| `fn export_svg(gds_bytes: &[u8], layer_names: &LayerMap) -> String` | Headless SVG: 800×800, viewBox padded 2.5 %, one `<g id=layer>` per GDS pair with the conventional fill/outline colours, y flipped, labels as upright monospace text; empty GDS gives an empty `<svg/>`. | `631-684` |
| `fn run_viewer(path: PathBuf, layer_names: LayerMap)` | Blocking wgpu window over a GDS file, reloaded when its mtime changes (500 ms poll); pan by drag/arrows, zoom by wheel/`+`/`-`, `r` refit+reload, Esc quits; exits the process without a display server. | `1116-1124`, `1025-1112` |
| `struct Probe` | A window on a background thread showing the latest polygons sent to it; a no-op without a display ("probe: no display", sends dropped). | `1142-1178` |
| `Probe::open(title: &str) -> Probe` | Spawn the window thread (X11/Wayland any-thread event loop on Linux). | `1149-1163`, `1127-1138` |
| `Probe::send(&self, polys: &[Poly], title: Option<&str>)` | Replace the shown geometry (the newest queued message wins, `1101`) and optionally retitle the window. Texts are not sent (`1105` passes none). | `1173-1177` |
| `Probe::wait(&mut self)` | Drop the sender and block until the window closes. | `1166-1171` |
| `fn polys_from_shapes(shapes: &[pnr_core::Shape], gds: &[(u16, u16)]) -> Vec<Poly>` | Rects → 4-point polys on `gds[LayerId]` (a missing row keeps the layer id with datatype 0). | `1184-1197` |
| `fn show_macro(mac: &Macro, title: &str, gds: &[(u16, u16)])` | `Probe::open` + `send` + `wait` for one macro. | `1201-1205` |

Rendering conventions (private but they define what a viewer shows): fixed colours by layer name for diff/tap/poly/cuts/li/met1..5/rpoly, outline-only for anything containing `well` or `sdm`, ending `_mk`, or in a marker list (`nplus pplus nsd psd npc hvtp lvtn rpm diom dualgate sab salblock`), hashed palette otherwise (`420-457`); polygons sorted by layer key before upload (`964`); fills triangulated with earcut, outlines as line lists, a screen-space legend with 5×7 stroke glyphs (`469-618`); camera fit only on the first non-empty `show` (`965-968`); a redraw is requested on every `about_to_wait` (`1109-1111`, not idle).

**Headless capabilities**: `parse_gds`, `export_svg`, `layer_names`, `polys_from_shapes` are pure and used headlessly by `benchmarks/src/bench.rs:421` (per-circuit SVG in `assets/`), `benchmarks/src/gds2svg.rs:18-19` and `frontend/library/src/gds.rs:208, 267` (GDS round-trip tests). There is no offscreen raster: the GPU path renders only to a window surface (`720-733`, `879-918`).

**Consumers today**: `examples/three_stage_opamp/src/main.rs:75-78` sends the final placed-and-routed geometry to a `Probe` and waits; the benchmarks write SVGs; `visualizer` binary views a file with empty layer names (`main.rs:1-10`).

**How a progress view of a multi-stage flow could use it**: open one `Probe` per run and, after each stage (variant choice, placement, rings and bridges, routing, repair), call `send(polys_from_shapes(&placed_shapes, &pdk.layer_gds()), Some("stage: …"))`; the latest message replaces the picture, so a tight loop can stream every accepted move. Highlights (a violation, a moving cell) can ride along as extra polys on an unused GDS pair. For CI or a web report, emit GDS per stage and call `export_svg`. Limits to design around: `Probe` builds its `App` with an empty `LayerMap` (`1160`), so colours hash and the legend reads `L{l}/{d}` (`64-66`); texts cannot be sent over the channel; the camera does not refit when the layout grows (`965-968`); window size is fixed at 1280×720 (`1011`); triangulation of the whole polygon list runs on the UI thread per message (`970-974`); there is no PNG/offscreen output and no diff between messages.

## 5. Known defects from audit-03 and their state in the tree

Audit `docs/plans/audit-03-cells-device-generators.md` §4 (lines 372-407), dated 2026-09-28. State judged from the code read above; plan-03 marks CELL-01..04, 06..10, 30 done (`docs/plans/plan-03-device-generators.md:112, 198, 227, 246, 265, 320, 341, 397, 426, 793`).

| ID | Sev | Finding (one line) | State now |
|---|---|---|---|
| AC-01 | critical | Capacitors drawn as met1/met2 plates, MIM never drawn, LVS skips them | Addressed for arrays: MIM recipe stack and `Drawn` cards (`cap_array.rs:184-195`, `369`, test `982-991`); open for `Capacitor` (no marker, `capacitor.rs:23-26`). |
| AC-02 | critical | Resistor and diode ignore `dev_nf` | Addressed (`resistor.rs:115-118`, `265-268`; `diode.rs:49`, `94-139`). |
| AC-03 | critical | Inductor turns merge into one conductor, tap floats | Addressed by drawing nothing (`inductor.rs:21-27`). |
| AC-04 | high | Per-finger gate pads force 730 nm pitch | Addressed: continuous bars, 430 nm (`mosfet.rs:610-623`, `1980-1983`). |
| AC-05 | high | Greedy centroid crosses drains; ratioed mirrors never merge | Addressed (`mosfet.rs:1095-1097`, `1157-1164`, `1657-1670`). |
| AC-06 | high | Collecting rings drawn as bulk rings; no minority rings or double rings | Partial: `Ecgr` band well (`post_cell.rs:308-314`, `360-363`), `Tub` (`212-218`); `Hcgr` undrawable on sky130 (`427-434`); no double rings. |
| AC-07 | high | One tap strip; `tie_max_dist_nm` unread | Addressed: 15 000 nm key read by `taps_in_reach` (`mosfet.rs:938-944`, `2111-2142`), filtered in `cellgen.rs:971`; second strip deferred (plan-03 appendix, CELL-13). |
| AC-08 | high | No BJT recogniser; pairs not cross-coupled; no units; per-NPN dnwell | Partial: `centro_assign`, `Unit`, `Drawn` (`bjt.rs:59`, `168-177`); per-unit NPN isolation remains (`140-154`); no sky130 recogniser (plan-03 §5 Q6). |
| AC-09 | high | FinFET single variant, no dummies, no CC, `sa = sb = 0`, no tests | Partial: shared CC variant, edge gates, SA/SB, tests (`finfet.rs:94-104`, `175-181`, `231`, `302-415`); no dummies/mirror/rows/double gate. |
| AC-10 | high | Empty enumeration becomes a silent empty macro | Addressed: `UNDRAWABLE` table and enumeration test (`selfcheck:400-481`), `cell/undrawable` row; a MOS whose every variant fails tap reach still gets an empty placeholder (`cellgen.rs:972-974`). |
| AC-11 | medium | Split gates give unequal poly overhang; dummy poly differs | Addressed (`mosfet.rs:488`, `714`, test `1439-1464`). |
| AC-12 | medium | Poly strap kept under the li strap (rule 22) | Partial: EXC isolates pads in metal (`mosfet.rs:357`, `583-586`, `614`); MIN/MOD keep the poly bar. |
| AC-13 | medium | One moat/WPE tier, one dummy per end | Addressed by class tiers (`class.rs:63-76`; `mosfet.rs:449`, `458`, `850-851`; tests `1410-1483`). |
| AC-14 | medium | Dummy pitch ≠ segment pitch | Addressed (`resistor.rs:502-506`, `703-723`). |
| AC-15 | medium | Resistors 1-D greedy only; no ratio by segment count; multi-segment LVS unverified | Partial: `segment_row` CC and series-unit arrays (`551-556`, `122-131`, `247-263`), one card per segment extracted (`823-833`); no 2-D array. |
| AC-16 | medium | No split DAC, bridge, shield; `metrics` unused; 6 % tolerance | Partial: `Split` + bridge (`cap_array.rs:57`, `213-222`, `282-294`), metrics rank Exceptional banks (`142-152`); shields deferred (CELL-24); 6 % remains (`1143`). |
| AC-17 | medium | Merged plate includes gaps; no-via zone not exported | Partial: plate = n·W·L (`capacitor.rs:179-185`); keep-outs exist only for MIM units (`cap_array.rs:370`). |
| AC-18 | medium | Diode one cut per terminal, no units, gf180 form unusable, anode 330 nm | Partial: cut arrays, units, licon.7 (`diode.rs:87-125`, `137`, `201-216`); gf180 form still unusable (`16-19`). |
| AC-19 | medium | Cross-deck tests release-only and vacuous; labels G/S/B common | Partial: empty enumeration is a finding and runs in debug (`selfcheck:439-481`, `505-507`); release-only and common labels remain (`332`, `363`, `520`). |
| AC-20 | medium | No in-cell S/D routing | Open, deferred (CELL-27). |
| AC-21 | medium | No per-variant figures for the placer | Partial: `Figures.sd`, `gate_ohm` (`core/macro.rs:96-104`; `mosfet.rs:203-209`, `639-656`), `ArrayMetrics` in tests and ranking only. |
| AC-22 | medium | No matching-grade input | Addressed: `Unitization.class` (`cell.rs:69`), `MatchClass` (`core/process.rs:107-112`). |
| AC-23 | low | Merged rings share one net; dead keys; ring R counts cuts only | Partial: dead keys removed (plan-03 CELL-05 status), roles never merge (`post_cell.rs:85`); ring R still cut-only (`470-473`). |
| AC-24 | low | Chain needs all-odd nf; CC for n > 2 needs nf % 4 == 0 | Open: even nf gives no chain variant (`mosfet.rs:102-104`); deferred in plan-03 appendix. |
| AC-25 | low | Literals, stale comments | Not re-verified here. |
| AC-26 | low | `Pattern::Interdig` has three meanings | Partial: `cap_array` has its own enum (`cap_array.rs:40`); resistor still reuses `Pattern::Interdig` for its CC row (`resistor.rs:551-556`) while MOS never offers it. |
| AC-27 | low | Docs stale against the code | Not checked here. |
| AC-28 | low | Wrong Hastings citation for gate R | Addressed (`mosfet.rs:54-56` cites Razavi §19.2.1). |
| AC-29 | low | Rectangles only, no 45° or octagonal shapes | Open (`builder.rs:27-30`). |

Audit §5 questions still open per plan-03 §5: BJT recogniser on sky130 (Q6), MOM capacitance uncalibrated (Q11), LVS `area` params deferred (appendix).

## 6. Portability notes

Sky130-specific by construction, or blocking a PDK-agnostic recipe:

1. **Role vocabulary is sky130's layer vocabulary.** `Pdk::layer(role)` resolves the `cell.layers` role table first, then `met{n}`/`via{n}` by stack position, then a deck layer of that literal name (`pdk.rs:1165-1175`). Roles the generators request outside the table resolve only by literal name: `npc`, `dnwell`, `diom`, `diode_mk`, `pnp`, `npn`, `rpm`, `rpoly_b`, `res_block`, `res_implant`, `capm`-style `plate`, `gate_marker{i}`, `fin`, `sdt`, `lisd`, `lig`. On gf180/ihp these quietly resolve to `None` (optional features vanish) unless the deck or a recipe names them (`pdks/*.json` `cell.layers`: sky130 maps 11 roles + `diom`; gf180mcu and ihp 11; asap7 adds `fin sdt lisd lig gcut`).
2. **The local-interconnect assumption.** Every pin sits on the `li` role with `licon` as its cut and `mcon` up to `met1` (`mosfet.rs:601`, `resistor.rs:337`, `bjt.rs:199`, `diode.rs:112`, `post_cell.rs:298`). On gf180/ihp `li` = metal1, `met1` = metal2 (`cell.layers`), so "met1" in the generators is the second metal and the capacitor stacks start one level up; on asap7 `li` = M1, `licon` = V0. A Substrate layer enum must carry this role indirection, not metal names.
3. **`tap` and `diff` on one layer** on gf180 (`comp`), ihp (`activ`) and asap7 (`ACTIVE`); `stack_on_tap` finds the strip as "the topmost shape on the tap layer" because of it (`mosfet.rs:217-220`), and `tap_ring` draws li+tap+implant per band (`post_cell.rs:270-272`). Decks with a distinct tap layer and decks without behave differently under DRC derived layers (`placement_space` note, `pdks/sky130.json`).
4. **Sidecar keys that exist only on sky130**: `licon_to_gate_spacing`, `polycon_to_pdiff_spacing`, `m1_enc` (also gf180), `min_finger_width` (also asap7), `npn_isolation` (NPN undrawable elsewhere, `selfcheck:400-406`); `polycon_to_diff_spacing` only on gf180/ihp (`mosfet.rs:369` reads both). All fall back to deck-derived rules or 0, so a missing key changes geometry silently.
5. **Policy tiers are not deck data.** `lod_moat_ext_nm`, `wpe_clearance_nm`, `dummy_reach_nm`, `gate_ext_extra_nm`, `res_dummy_span_nm`, `res_width_floor_permille`, `res_length_floor_x` are identical `[MIN, MOD, EXC]` arrays on all four decks, sourced from Hastings (`pdks/sky130.json` `*_source`); `dummy_max_l_nm` 3000 is "Philis policy: the lower end" and absent on asap7.
6. **Recipes drive passives.** Resistors need an `rpoly` role (asap7: undrawable) and a recipe with `res_sheet_mohm` etc. for any segmentation (`resistor.rs:171`, `565`); sky130's `high_po` adds `rpm`, `npc`, `res_implant`, slotted contacts; the MIM path needs `capacitors.recipes.*.layers` with six roles (`cap_array.rs:184-195`), only sky130 has one; BJT fixed emitters come from `bjts.recipes` (sky130 only). gf180/ihp/asap7 have no capacitor or BJT recipes.
7. **Cap arrays hard-wire the routing stack** to `met1` tracks, `met2` branches/buses, `met3` straps (`cap_array.rs:408`, `562-596`, `693-705`) and require all five layers even for a MIM on met3/met4 (`194`).
8. **MOS construction assumptions**: bulk p-substrate with no drawn pwell (only `nwell` for PMOS, `mosfet.rs:848-867`, `finfet.rs:287-292`); a single tap strip above the row (`765-822`); dummies tied to bulk through li risers (`741-763`); gate contacts as `licon` on poly (`575-592`); a channel implant pair `nsdm`/`psdm` that "is the only thing that tells the extractor NMOS from PMOS" (`783-792`); flavour markers from the recogniser's `and/not` expression (`pdk.rs:1401-1431`). A triple-well or SOI deck, or one contacting gates on a different cut, needs new construction, not new keys.
9. **Cut lattice = 2×grid because of magic** (`builder.rs:152-153`); `cell_selfcheck` hard-codes 10 nm (`selfcheck:291-295`).
10. **`cover_poly_cuts` is the sky130 `npc` rule** (`builder.rs:76-80`), a no-op elsewhere.
11. **Pin layer inconsistency**: planar MOS pins all on `li`, FinFET drain pins on the `met1` role (`finfet.rs:215`); `regions()` groups by the pin's own layer, so the two tiles expose different port layers.
12. **Rect-only geometry** (`builder.rs:27-30`); the visualizer accepts polygons but no generator emits one.
13. **Diode well form** (`diode_mk`, gf180 `pn_3p3`) is drawn but not extractable (`diode.rs:16-19`).
14. **Checker-specific filters** in the self-checks (`floating_gate`, `soft_connection`, `*_density`) and the asap7 debug-assertion workaround (`finfet.rs:399-402`) tie the test suite to GPurify's rule names.
15. **`Process::rule` names synthesised by `verify`** (`<layer>_min_width`, `<layer>_min_spacing`, `<role>_min_area`, audit-03 §1.1) are part of the de-facto contract: `poly_min_spacing`, `nwell_min_spacing`, `li_min_area` are read by name (`mosfet.rs:389`, `524`; `bjt.rs:127`; `post_cell.rs:354`; `diode.rs:85`).

## 7. Reference notes from ref-00 relevant here

- Pins and units: electrical unit centroids, not bounding boxes (NOTES-19, `docs/plans/ref-00-prior-notes-handbook.md:555-562`), implemented in `core/units.rs`; higher-order moment certificates still radial-only in cap_array (NOTES-20, `564-571`); nth-order recursive patterns partial (NOTES-21, `573-580`); signed Φ equality within a macro only (NOTES-22, `582-589`); qualified tied dummies for MOS and caps, resistor dummies now present (NOTES-24, `600-607`).
- Variant policy: pattern competition judged electrically, `Interdig` withheld because shared diffusion shorts drains (NOTES-25, `609-616`); contextual variant frontier (NOTES-40, `744-751`).
- Specialty devices need declared domain and exclusion shapes; the inductor has none (NOTES-59, `915-922`).
- No visualization convention is recorded in ref-00 beyond "56 SVGs rendered" for the handbook diagrams (`122`); the rendering conventions in force are the visualizer's own (`visualizer:420-457`).
