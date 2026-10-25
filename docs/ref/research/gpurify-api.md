# GPurify as the only PDK input: API inventory for Philis 2.0

Paths: `GPurify/` is `/home/omare/Documents/Projects/EDA-Tools/Layout/GPurify`, `Philis/` is `/home/omare/Documents/Projects/EDA-Tools/Layout/Philis`. Line numbers are of the working trees read on 2026-10-25.

Snapshot. GPurify HEAD is `40ff66c` (2026-10-16). Philis pins GPurify at `be14a73f` (2026-10-06) through `Cargo.lock` (`Philis/Cargo.lock:1552`), 733 commits behind HEAD. Everything below describes HEAD unless it says "at the pin". The engine glue Philis compiled against has since changed shape (section 3.7).

## 1. Deck model

### 1.1 Grammar in brief

A deck is plain text, one statement per line, `#` to end of line is a comment, a parenthesised rule may continue over lines (`GPurify/docs/deck.md:65-67`). The first word of a line is the statement keyword; the parser dispatches on 26 keywords and refuses any other word with a located diagnostic (`GPurify/crates/ingest/src/deck/parse.rs:519-551`). Every parameter is required and named, there are no defaults, every number carries a unit, and a length must be a whole multiple of the deck's `grid` and of the layout grid it is parsed against (`docs/deck.md:7-11`, `:68-69`; `parse.rs:115-117`). Errors are collected, up to 50, and the deck is refused as `DeckError::Invalid` with line, column and caret (`parse.rs:17`, `:36-50`; `crates/ingest/src/deck/mod.rs:69-79`). Statements that parse but contradict each other are refused at lowering as `DeckError::Malformed` (`mod.rs:59-64`; `crates/ingest/src/deck/source.rs:234`).

Reuse: `let name = value | [list of values or tuples]` binds once; `for (a, b) in list { ... }` repeats statements with `{a}` spliced into rule ids; `if name { }` and `if not name { }` read a block only when a `let` bound the name to `true`/`false`, a deck switch (`docs/deck.md:308-335`). Loops run only over lists written in the deck, so loading always terminates (`docs/deck.md:321-322`).

### 1.2 Values and units

Each value has a dimension and converts to one engine unit. The unit table (`GPurify/crates/ingest/src/deck/kinds.rs:106-151`) and the engine unit per dimension (`kinds.rs:8-102`):

| Dimension | Deck units | Engine unit / storage |
|---|---|---|
| length | `nm`, `um`, `mm` | grid units, `Dbu` (exact or refused) |
| area | `nm2`, `um2`, `mm2` | square grid units, `DbuArea` |
| voltage | `mV`, `V` | mV, `f64` |
| current | `nA`, `uA`, `mA`, `A` | µA |
| current per width | `uA/um`, `mA/um`, `A/m` | A/m (numerically µA/µm) |
| frequency | `Hz`, `kHz`, `MHz`, `GHz` | MHz |
| resistance | `mohm`, `ohm`, `kohm`, `Mohm` | ohm |
| temperature | `K`, `C` | K (`C` converted by offset, `kinds.rs:105`) |
| energy | `eV` | eV |
| thermal conductivity | `W/mK` | W/(m·K) |
| time | `h` | h |
| electric field | `V/um`, `V/nm`, `MV/cm` | V/µm |
| capacitance per area | `aF/um2`, `fF/um2` | aF/µm² (PEX only) |
| capacitance per length | `aF/um`, `fF/um` | aF/µm (PEX only) |
| temperature coefficient | `/K`, `/K2` | 1/K, 1/K² (PEX only) |
| capacitance | `aF`, `fF`, `pF` | aF |
| inductance | `pH`, `nH`, `uH` | nH |
| fraction | `%` or bare 0..1 | 0..1 |
| angle | `deg` | whole degrees |
| scalar, count, bool | bare number, bare integer, `true`/`false` | `f64`, `u32`, `bool` |
| layer, model, names | identifier, `"quoted"` | `LayerId`, interned `StrId` |

Composite parameter shapes a kind may take: `a x b` length pairs, `[..]` lists of lengths, voltages, frequencies, fractions, counts, areas, angles, models, names, `[[..], ..]` rows of lengths or current densities, `(name: value, ..)` groups, `[(area, n), ..]` piecewise-linear tables, `[none, len, ..]` optional-length lists, and a word from a fixed set (`kinds.rs:55-101`). A parameter declared optional is written `none` and then has no row (`kinds.rs:159-160`; `mod.rs:869-871`).

### 1.3 Statements

| Keyword | Form | What it produces |
|---|---|---|
| `grid` | `grid 5nm`, once | the deck's manufacturing grid; every length must be a multiple (`docs/deck.md:68-69`; `parse.rs:553-560`) |
| `layer` | `layer name = gds(L, D)` or `layer name = <expr>` | a drawn layer with its stream pair, or a derived layer (section 1.4) (`docs/deck.md:98-121`) |
| `ignore` | `ignore gds(L, D)` | a stream pair whose shapes and texts are dropped instead of refusing the layout; never a declared pair (`docs/deck.md:103-110`) |
| `boundary` | `boundary <drawn layer>`, at most once | the die outline for densities, substrate solve and `lib` area (`docs/deck.md:112-116`) |
| `rule` | `rule <id> [warning] <kind>(layers; name: value, ..) [>= \| <= \| ==] <limit>` | one DRC or ERC rule row (`docs/deck.md:283-306`); `warning` makes its findings `Severity::Warning` (`:289-297`) |
| `let`, `for`, `if` | see 1.1 | expansion only; nothing survives into the tables except the rules written |
| `stage` | `stage <name>` | heads a stage; later stages report `skipped` once an earlier one finds an error (`docs/deck.md:354-375`) |
| `waive` | `waive <rule id> <layer>` | violations of that rule whose marker lies on a shape of the layer are reported as `waived` (`docs/deck.md:337-352`) |
| `connect` | `connect conductors [L, ..]`, `connect touch_within_layer`, `connect via CUT [A, B]`, `connect label T names L`, `connect global L` | net extraction: conductors, same-layer touching, cuts joining two layers, text layers naming a conductor, net-only global layers (the substrate) (`docs/deck.md:725-753`; `parse.rs:1249-1252` refuses other words) |
| `device` | `device <mos\|bjt\|resistor\|capacitor\|diode\|inductor> <marker> model "<name>" terminals [L, ..] [sheet R] [area_cap C] [fringe_cap C] [inductance L] [on_gate_min V \| on_gate_max V]` | a device recogniser (`docs/deck.md:755-825`; `parse.rs:1255-1345`) |
| `lvs` | 21 controls, section 1.7 | per-model comparison controls (`docs/deck.md:827-1069`) |
| `netlist_scale` | `netlist_scale 1um`, once | unit of a bare `w`/`l`/area on an `X` card calling a deck model (`docs/deck.md:1071-1078`) |
| `pex` | `pex <layer> thickness L height L sheet R [dielectric k] area_cap C fringe_cap C [tc1 /K tc2 /K2]` | one parasitic stack row (`docs/deck.md:1080-1113`; keys `kinds.rs:1954-1969`) |
| `temperature` | `temperature T nominal T0`, once | operating and nominal temperature; then every `pex` row needs `tc1`/`tc2` (`docs/deck.md:1115-1144`) |
| `corner` | `corner <name> <layer> <pex keys..>` | a process corner's override of one layer's `pex` values (`docs/deck.md:1146-1168`) |
| `extraction` | `extraction r\|c\|cc\|rc\|rcc [coupling_threshold C] [coupling_halo n]`, once | which parasitics PEX writes and how far lateral coupling is looked for (`docs/deck.md:1170-1192`; `parse.rs:2240-2300`) |
| `pex_fill` | `pex_fill <layer> floating\|grounded\|ignored` | metal-fill treatment (`docs/deck.md:1194-1213`) |
| `pex_tsv` | `pex_tsv <cut layer> length L liner L liner_k k` | through-silicon via liner model (`docs/deck.md:1215-1230`) |
| `pex_substrate` | `pex_substrate ports [L, ..] backside grounded\|floating profile [(thickness, resistivity Ω·cm), ..]`, once | substrate network (`docs/deck.md:1232-1252`) |
| `pex_device` | `pex_device "<mos model>" gate_contact_cap C gate_diffusion_cap C` | device-region capacitance per µm of gate width (`docs/deck.md:1254-1268`) |
| `fill` | `fill <layer> for <density layer> size L space L keep L avoid [L, ..]` | density-fill recipe for `gpurify fill` (`docs/deck.md:1270-1306`) |
| `widen` | `widen <layer> net "<glob>" by L` | power-strap widening for `gpurify enhance` (`docs/deck.md:1308-1313`) |
| `report` | `report p2p "<net>" at [(x, y), ..]`, `report coupling "<a>" "<b>"`, `report cluster L` | text-report measurements, not checks (`docs/deck.md:1323-1348`; `parse.rs:2222`) |

### 1.4 Layer operations

Derived layers combine earlier layers with `and`, `or`, `not`, left to right, parentheses to group, and chained `.op()` calls; a layer must be declared before use; touching or overlapping shapes are one shape before any operation (`docs/deck.md:117-136`). Every operation is one `DerivedOp` variant stored with its operands (`GPurify/crates/ingest/src/deck/mod.rs:609-762`):

| Deck spelling | `DerivedOp` | Notes |
|---|---|---|
| `A and B`, `A or B`, `A not B` | `And`, `Or`, `Not` | fold left over all operands (`mod.rs:612-618`) |
| `.sized(len)`, `.sized(dx, dy)` | `Sized(Dbu, Dbu)` | square corners; both negative shrinks (`mod.rs:619-623`) |
| `.interacting(L)`, `.not_interacting(L)` | `Interacting(bool)` | |
| `.inside(L)`, `.outside(L)` / `.not_overlapping(L)`, `.overlapping(L)` | `Inside`, `Outside`, `Overlapping` | |
| `.covering(L)`, `.not_covering(L)` | `Covering(bool)` | |
| `.holes()`, `.extents()` | `Holes`, `Extents` | |
| `.with_area(bounds)`, `.with_width(bounds)` | `WithArea(i128, i128)`, `WithWidth(i128, i128)` | inclusive bounds; width stored as twice its square (`mod.rs:647-655`) |
| `.with_bbox_max(b)`, `.with_bbox_min(b)` | `WithBbox{longer, lo, hi}` | |
| `.with_rect(w x h)`, `.without_rect(w x h)` | `WithRect{keep, w, h}` | |
| `.rectangles()`, `.squares()`, `.non_rectangles()`, `.non_squares()` | `Rectangles{square, keep}` | |
| `.edges()` | `Edges` | result is an edge layer |
| `.inside_part(L)`, `.outside_part(L)` | `Part(bool)` | edge layers only |
| `.with_length(bounds)` | `WithLength(i128, i128)` | edges |
| `.horizontal()`, `.vertical()` | `Oriented(bool)` | edges |
| `.extended(begin:, end:, out:, in:)` | `Extended{begin, end, outside, inside}` | edges to boxes (`mod.rs:731-740`) |
| `.with_text(T, "pat")`, `.without_text(T, "pat")` | `WithText{keep, text}` | pattern index into `LayerTable::text` |
| `T.texts_outside(L)` | `TextsOutside` | |
| `.in_cells("pat", ..)`, `.not_in_cells(..)` | `InCells{keep, first, count}` | at most 64 per deck (`mod.rs:766`) |

Edge layers combine only with edge layers (`docs/deck.md:170-186`); which checks accept an edge layer is `EDGE_KINDS` (`kinds.rs:1975-1982`). A layer expression written inside a rule becomes a hidden layer `<rule id>@k`; a nested expression's inner steps become `<layer>#k` (`mod.rs:425-429`). An unknown operation word is a located error (`parse.rs:984`).

### 1.5 DRC rule kinds

One row of `KINDS` per deck spelling and comparison; the engine kind is what `RuleSpec::kind` holds (`kinds.rs:462-1514`). "limit" is the comparison value and its dimension; named parameters are `name (engine name): dimension`, `opt` means `none` is legal. Semantics per kind are in `docs/deck.md:385-486` and `:605-619`.

| Deck spelling | Engine kind | Layers | Limit | Named parameters | kinds.rs |
|---|---|---|---|---|---|
| `width(L) >=` | `min_width` | 1 | length | | 464 |
| `width(L) <=` | `max_width` | 1 | length | | 465 |
| `size(L) == w x h` | `cut_size` | 1 | length pair `width`,`height` | | 468-475 |
| `edge_length(L) >=` | `min_edge_length` | 1 | length | | 476 |
| `min_step(L; max_edges:) >=` | `min_step` | 1 | length | `max_edges`: count | 480-487 |
| `min_step(L; corner:, length_sum:) >=` | `min_step` | 1 | length | `corner`: word outside/inside/step; opt `length_sum`: length | 488-498 |
| `min_step(L; adjacent:, except_rectangle:) >=` | `min_step_adjacent` | 1 | length | `adjacent`: length; `except_rectangle`: bool | 503-513 |
| `min_step(L; adjacent:, adjacent_eol:, except_rectangle:) >=` | `min_step_adjacent` | 1 | length | + `adjacent_eol`: length | 514-525 |
| `min_step(L; between_eol:) >=` | `min_step_adjacent` | 1 | length | `between_eol`: length | 526-533 |
| `width(L; allowed: [..])` | `allowed_width` | 1 | none | `allowed` (`width`): length list | 536-543 |
| `width(L; allowed:, open_last:)` | `allowed_width` | 1 | none | + `open_last`: bool | 559-569 |
| `origin(L)` | `origin` | 1 | none | | 545 |
| `protrusion_width(L; wide:) >=` | `protrusion_width` | 1 | length | `wide`: length | 548-555 |
| `notch(L) >=` | `notch` | 1 | length | | 570 |
| `space(L) >=` | `min_spacing` | 1 | length | | 571 |
| `space(A, B) >=` | `min_spacing_diff` | 2 | length | | 572 |
| `space(A, B, projection) >=` | `projection_spacing` | 2 | length | | 575-578 |
| `eol_space(L; eol_width:) >=` | `eol_spacing` | 1 | length | `eol_width`: length | 579-586 |
| `eol_space(L; eol_width:, within:, parallel:, end_to_end:) >=` | `eol_within_spacing` | 1 | length | `within` (`eol_within`): length; opt `parallel`: group (`space`→`par_space`, `within`→`par_within`, `two_edges`: bool); opt `end_to_end`: length | 590-602, 412-416 |
| `eol_space(L; .. side_length:, opposite_width:, parallel_more:, end_prl:, wrong_dir:, extension:, enclose_cut:) >=` | `eol_within_spacing` | 1 | length | + opt groups `side_length` (`length`,`maximum`,`two_sides`), `parallel_more` (`subtract_eol_width`, opt `min_length`), `end_prl` (`space`,`prl`), `wrong_dir` (`space`,`horizontal`), `enclose_cut` (`cut`: layer, `within`, `space`, `all_cuts`); opt `opposite_width`, `extension` (`eol_extension`): length | 604-623, 418-455 |
| `prl_space(L; prl:) >=` | `prl_spacing` | 1 | length | `prl` (`prl_threshold`): length | 624-631 |
| `corner_space(L) >=` | `corner_to_corner` | 1 | length | | 632-639 |
| `corner_space(L; corner_to_corner:, except_eol:) >=` | `convex_corner_spacing` | 1 | length | `corner_to_corner`: bool; opt `except_eol`: length | 643-653 |
| `wide_space(L; width:) >=` | `wide_dependent_spacing` | 1 | length | `width` (`width_threshold`): length | 654-661 |
| `wide_space(L; width:, attached:) >=` | `attached_wide_spacing` | 1 | length | + `attached`: length | 1329-1339 |
| `area(L) >=` | `min_area` | 1 | area | | 662 |
| `hole_area(L) >=` | `min_enclosed_area` | 1 | area | | 663 |
| `cheesing(L) <=` | `cheesing` | 1 | area (`max_unslotted`) | | 664-671 |
| `density(L[, boundary]; window:, step:) >= \| <=` | `density` | 1 + optional die | fraction; fixed flag `maximum` | `window`, `step`: length | 674-683, 259-263 |
| `global_density(L[, boundary]) >= \| <=` | `global_density` | 1 + optional die | fraction; `maximum` | | 684-707 |
| `region_density(L, R) >= \| <=` | `region_density` | 2 | fraction; `maximum` | | 710-731 |
| `count(L) >= \| <=` | `count` | 1 | count; `maximum` | | 734-741 |
| `size(L; allowed: [w x h, ..])` | `cut_class` | 1 | none | `allowed` (`width`/`height`): length-pair list | 745-752 |
| `total_area(L) <=` | `total_area` | 1 | area | | 754 |
| `aspect_ratio(L) <=` | `aspect_ratio` | 1 | scalar | | 756-763 |
| `critical_area(L; radii:, weights:) <=` | `mean_critical_area` | 1 | area | `radii`: length list; `weights`: fraction list | 767-777 |
| `critical_area(L; radius:) <=` | `critical_area` | 1 | area | `radius`: length | 779-786 |
| `density_balance(A, B[, die]; window:, step:) >=` | `density_balance` | 2 + optional | fraction | `window`, `step` | 789-799 |
| `via_redundancy(cut, lower, upper; pitch:, space:) >=` | `via_redundancy` | 3 | fraction | `pitch`, `space`: length | 802-812 |
| `dfm_score(L[, outer]; check:, required:, recommended:, rate:) >=` | `dfm_score` | 1 + optional | fraction | `check`: word space/width/enclosure; `required`, `recommended`: length; `rate`: fraction | 817-832 |
| `pattern(L; window: w x h, shapes: [..], ignore: [..]\|none)` | `pattern_match` | 1 | none | `window` (`window_w`/`window_h`); `shapes` (`shape_x`/`shape_y`) pair list; opt `ignore` (`ignore_x`/`ignore_y`) | 835-846 |
| `cut_area_change(pre, post, lower, upper; cuts:, change:)` | `cut_area_change` | 4 | none | `cuts`: count list; `change`: area list | 849-859 |
| `cover_density(L, C[, die]; window:, step:, cover:, density:, inclusive:)` | `cover_density` | 2 + optional | none | `window`, `step`: length; `cover`, `density`: fraction lists; `inclusive`: bool | 863-879 |
| `enclosure(inner, outer) >=` | `min_enclosure` | 2, stored outer first | length | optional wire band `width:` (`wire_width`), `below:` (`wire_below`) | 884-901, 311-316 |
| `enclosure(inner, outer; width:, except_extra_cut:, other:) >=` and the `extra_cut:` twin, `length:`, `length_below:` rows | `min_enclosure` | 2 | length | `except_extra_cut`/`extra_cut` (`extra_cut_within`): length, `other` (`extra_cut_other`): layer, fixed `extra_cut_only`; `length` (`length_from`), `length_below`: length | 902-909, 317-367 |
| `enclosure(inner, outer, noncoincident) >=` | `noncoincident_enclosure` | 2 | length | | 912-922 |
| `full_width(inner, outer)` | `full_width` | 2 | none | | 925 |
| `enclosure(inner, outer) <=` | `max_enclosure` | 2 | length | | 927 |
| `enclosure(inner, outer, opposite[; bands]) >=` | `asymmetric_enclosure` | 2 | length (`min_one_side`) | same band/qualifier rows as `min_enclosure` | 928-939, 386-406 |
| `enclosure(inner, outer, adjacent) >=` | `adjacent_enclosure` | 2 | length (`min_one_side`) | | 940-950 |
| `extension(A, B) >=` | `min_extension` | 2 | length | | 951 |
| `bend(A, B)`, `covers_corner(A, B)` | `gate_corner` | 2 | none | fixed `of_reference` false/true | 955-969 |
| `overlap(A, B) >=` | `overlap` | 2 | length | | 970 |
| `tap_distance(well, tap) <=` | `max_distance_to_tap` | 2 | length | | 971-978 |
| `off_grid(; pitch:)` | `off_grid` | 0 | none | `pitch`: length | 979-986 |
| `angle([L]; allowed: [deg, ..])` | `angle` | 0 or 1 | none | `allowed` (`angle`): angle list | 987-1003 |
| `redundant_via(L; within:) >=` | `redundant_via` | 1 | count (`min_count`) | `within`: length | 1004-1011 |
| `min_cuts(cut, wide, other; width:[, within:][, length:, distance:]) >=` | `minimum_cut` | 3 | count (`min_count`) | `width`, opt `within`, `length`, `distance`: length | 1015-1060 |
| `via_map(C, B, A; below_min:, below_max:, above_min:, above_max:, cut: w x h)` | `metal_width_via` | 3 | none | four lengths; `cut` (`cut_width`/`cut_height`) | 1065-1078 |
| `stack_space(A, B, M[; same_net:]) >=` | `stack_spacing` | 3 | length | opt `same_net`: bool | 1084-1099 |
| `max_space(C[, A]) <=` | `max_cut_spacing` | 1 or 2 | length | | 1103-1104 |
| `via_stack(cut1, cut2, ..) <=`, `via_stack(m1, cut1, .., single) <=` | `via_stack` | 1+ | count (`max_height`) | fixed `single` | 1107-1135 |
| `via_array_space(L; array:) >=` | `via_array_spacing` | 1 | length | `array` (`array_threshold`): count | 1136-1143 |
| `space(L, center) >=` | `cut_spacing` | 1 | length | fixed `center` | 1146-1150 |
| `adjacent_space(L[, center]; cuts:, within:) >=` | `cut_spacing` | 1 | length | `cuts` (`adjacent`): count; `within`: length | 1151-1173, 407-410 |
| `cut_class_space(L; short:, long:, space:)` | `cut_class_spacing` | 1 | none | `short` (`class_short`), `long` (`class_long`): length lists; `space`: length rows | 1176-1187 |
| `patterning(L; colors:) >=` | `multi_patterning` | 1 | length (`color_spacing`) | `colors`: count | 1188-1195 |
| `patterning(L, A1, ..; colors:) >=` | `anchored_patterning` | 2+ | length | `colors`: count | 1221-1231 |
| `interior_angle([L]) >=` | `min_interior_angle` | 0 or 1 | angle | | 1198-1213 |
| `valid([L])` | `polygon_validity` | 0 or 1 | none | | 1217-1218 |
| `sadp(L; different:, same:, track_width:, track_space:)` | `sadp` | 1 | none | two lengths, two opt lengths | 1235-1247 |
| `sadp_line_end(L; adjacent:, tip:, overlap:, corner:, offset:)` | `sadp_line_end` | 1 | none | five lengths | 1251-1264 |
| `forbidden(L)` | `forbidden` | 1 (polygon or edge) | none | | 1266 |
| `contains(outer, inner) >= \| <= \| ==` | `must_contain` | 2 | count (`min_count`/`max_count`/`count`) | | 1268-1293 |
| `inside(inner, outer)` | `must_be_inside` | 2 | none | | 1295 |
| `space(L[, L2]; nets: same\|different) >=` | `net_spacing` | 1 or 2 | length | `nets` (`same_net`): same→true | 1298-1313 |
| `space_table(L; prl:, width:, space:)` | `spacing_table` | 1 | none | `prl`, `width`: length lists; `space`: length rows | 1315-1326 |
| `tip_space_table(L; length:, space:)` | `tip_spacing_table` | 1 | none | `length`: length list; `space`: rows | 1343-1353 |
| `two_width_table(L; width:, prl:, space:)` | `two_width_spacing_table` | 1 | none | `width`: lengths; `prl`: optional-length list; `space`: rows | 1356-1367 |
| `span_length(L; x:, y:[, x_except:, y_except:])` | `span_length` | 1 | none | `x` (`x_spans`), `y` (`y_spans`): length lists; opt `x_except`, `y_except` | 1370-1394 |
| `pitch(L, vertical\|horizontal) ==` | `exact_pitch` | 1 | length | fixed `vertical` | 1397-1406 |
| `eol_keep_out(L; eol_width:, backward:, side:, forward:[, except_within:, corner_only:])` | `eol_keep_out` | 1 | none | four lengths; opt group `except_within` (`low`→`within_low`, `high`→`within_high`); `corner_only`: bool | 1409-1437, 434-437 |
| `eol_extension_space(L; eol_width: [..], extension: [..], end_to_end: [..], side_length:) >=` | `eol_extension_spacing` | 1 | length | `eol_width`, `extension`: length lists; `end_to_end`: optional-length list; opt group `side_length` (`length`, `two_sides`) | 1442-1454, 424-427 |
| `keep_out_zone(L; class: s x l, to: s x l\|none, end_side:, end_forward:, side_side:, side_forward:)` | `keep_out_zone` | 1 | none | `class` (`short`/`long`), opt `to` (`to_short`/`to_long`): length pairs; four lengths | 1457-1471 |
| `forbidden_space(L; min:, max:, prl:)` | `forbidden_spacing` | 1 | none | three lengths | 1474-1485 |
| `length(E) >=` | `edge_min_length` | 1 edge | length | | 1487 |
| `space(E[, F]) >=` | `edge_spacing` | 1 or 2 edge | length | | 1488-1489 |
| `enclosure(E, L) >=` | `edge_enclosure` | edge + polygon | length | | 1490 |
| `width(E) >=` | `edge_width` | 1 edge | length | | 1494 |
| `facing_space(E[; prl:]) >=` | `edge_facing_space` | 1 edge | length | opt row with `prl`: length | 1497-1514 |

The README counts 83 design-rule kinds (`GPurify/README.md:139`); the typed enum behind them is `drc::Rule` (`GPurify/crates/check/src/drc/ruleset.rs:71`), one variant per engine kind, with the kind name from `Rule::kind` (`ruleset.rs:6571`).

### 1.6 ERC rule kinds

Every ERC parameter is required unless marked opt, written `none` when absent (`docs/deck.md:621-624`). `models` is a quoted list that `device` statements must declare (`docs/deck.md:625-628`). Rows from `kinds.rs:1516-1949`; semantics `docs/deck.md:647-723`. "intent" means the rule reports skipped without a design-intent file (`docs/usage.md:315-324`).

| Deck spelling | Layers | Parameters (engine name): dimension | kinds.rs |
|---|---|---|---|
| `antenna(gate, collectors..; max_ratio:, sidewall:, diode:, diode_area:, diode_min:)` | 2+ | `max_ratio`: scalar; opt `sidewall` (`sidewall_thickness`): length; opt `diode` (`diode_layer`): layer; opt `diode_area`, `diode_min`: area | 1516-1527 |
| `antenna_electrical(gate, collectors..; max_ratio:, sidewall:, cut_factor:, diode:, diode_credit:, diode_bonus:, area_minus_diode:, gate_plus_diode:, diode_reduce:, diode_max_ratio:, exempt:)` | 2+ | scalars `max_ratio`, `cut_factor`, `diode_credit`, `diode_bonus`, `area_minus_diode`, `gate_plus_diode`; opt `sidewall`: length; opt `diode`, `exempt` (`exempt_layer`): layer; opt `diode_reduce`, `diode_max_ratio`: `[(area, n)]` tables | 1528-1553 |
| `conditional_tie(region, tap, well, other; within:, spacing:, max_distance:)` | 4 | `within`: length; `spacing`, `max_distance`: length lists | 1554-1563 |
| `density_cmp(L; window: w x h, step: w x h, min:, max:, max_delta:, partial_windows:, cmp:)` | 1 | `window` (`window_x`/`window_y`), `step` (`step_x`/`step_y`): length pairs; opt `min` (`min_density`), `max` (`max_density`), `max_delta` (`max_neighbour_delta`): fraction; `partial_windows` (`include_partial_windows`): bool; opt group `cmp` (`target`→`cmp_target_density` fraction, `sensitivity`→`cmp_thickness_sensitivity` length, `max_delta`→`cmp_max_abs_thickness_delta` length) | 1564-1577, 456-460 |
| `electromigration(layers..; max_density:, max_current_per_cut:, blech_limit:, reference_temperature:, activation_energy:, current_exponent:)` intent | 1+ | `max_density`: current/width; `max_current_per_cut`, `blech_limit`: current; `reference_temperature`: temperature; `activation_energy` (`activation_energy_ev`): energy; `current_exponent`: scalar | 1578-1594 |
| `em_current_density(layers..; max_density:, max_current_per_cut:, narrow:)` intent | 1+ | as above two; opt group `narrow` (`width`→`narrow_width` length, `current`→`narrow_current` current) | 1595, 277-287 |
| `em_rms(layers..; frequency:, width:, density:, max_current_per_cut:)`, `em_peak(..)` intent | 1+ | `frequency`: frequency list; `width`: length list; `density`: current/width rows; `max_current_per_cut`: current | 1596-1597, 291-296 |
| `esd_latchup(pad, guard_ring; min_guard_ring_width:, max_tap_distance:, clamps:)` intent | 2 | two lengths; `clamps` (`clamp`): models | 1598-1607 |
| `esd_cross_domain(; diodes:, local:, resistors:, min_series:, max_ground_resistance:, coupling_resistance:)` intent | 0 | `diodes` (`diode`): models; opt `local`, `resistors` (`resistor`): models; opt three resistances | 1608-1628 |
| `esd_exposed(pad; clamps:, resistors:, rated:, min_series:)` intent | 1 | `clamps`: models; opt `resistors`, `rated`: models; opt `min_series`: resistance | 1629-1639 |
| `esd_ground_coupling(; diodes:)` intent | 0 | `diodes`: models | 1640-1645 |
| `esd_parasitic(pad, diffusion; min_spacing:)` | 2 | `min_spacing`: length | 1646-1651 |
| `esd_path(pad[, source]; clamps:, current:, clamp_resistance:, max_resistance:, clamp_it2:, max_density:, max_current_per_cut:, max_voltage:, min_width:, clamp_voltage:)` intent | 1 or 2 | opt `clamps`: models; `current`: current; `clamp_resistance`: resistance; opt `max_resistance`: resistance, `clamp_it2`, `max_density`: current/width, `max_current_per_cut`: current, `max_voltage`, `clamp_voltage`: voltage, `min_width`: length | 1652-1655, 298-309 |
| `esd_topological(pad; clamps:)` intent | 1 | `clamps`: models | 1656-1661 |
| `floating_gate(; labels_are_ports:)` | 0 | bool | 1662-1667 |
| `floating_well(well, tap)` | 2 | | 1668 |
| `fanout(; max:)` | 0 | `max` (`max_gates`): count | 1669 |
| `net_type(; net:, path:, break:, exclude:)` intent | 0 | four opt name lists | 1670-1680 |
| `gate_path(; ground:, max_devices:)` intent | 0 | `ground`: bool; `max_devices`: count | 1681-1689 |
| `supply_path(; both:, labels_are_ports:)` intent | 0 | two bools | 1690-1698 |
| `combinational_loop()`, `floating_supply()`, `tie_high_low()`, `ir_drop()`, `dynamic_ir_drop()`, `domain_crossing()`, `undriven_gate()` (intent) | 0 | none | 1699-1700, 1767-1768, 1853, 1906, 1919 |
| `power_switch(; models:)` intent | 0 | `models` (`model`): models | 1701-1706 |
| `matching(group; tolerance:)` | 1 | `tolerance`: length | 1707-1712 |
| `esd_pin_pair(pad; clamps:, current:, clamp_resistance:, max_resistance:)` intent | 1 | models, current, two resistances | 1713-1723 |
| `esd_clamp_resistance(pad; clamps:, max_resistance:)` | 1 | models, resistance | 1724-1732 |
| `forward_diode(marker; models:)` intent | 1 | models | 1733-1738 |
| `guard_ring(target, ring; max_space:, min_width:, pad:, near:, within:, clear:)` | 2 | opt `max_space`, `min_width`, `within`: length; opt `pad`, `near`, `clear`: layer | 1739-1751 |
| `guard_ring_bias(ring; power:)` intent | 1 | `power`: bool | 1752-1757 |
| `hv_domain(; max_delta:, isolation:)` intent | 0 | `max_delta` (`max_domain_delta`): voltage; opt `isolation`: layer | 1758-1766 |
| `em_thermal(layers..; electromigration's six + thermal_conductivity:, spreading:, sigma:, limit_failure:, max_failure:)` intent | 1+ | + `thermal_conductivity`: W/mK; `spreading` (`heat_spreading`), `sigma`: scalar; `limit_failure`, `max_failure`: fraction | 1769-1794 |
| `latchup_victim(aggressor, region, tap; within:, max_distance:, pad:, bias:, forward:)` | 3 | two lengths; opt `pad`, `bias`: layer; opt `forward`: word above/below | 1795-1806 |
| `missing_tie(region, tap; max_distance:)` | 2 | length | 1807-1812 |
| `multiple_drivers(; max:)` intent | 0 | `max` (`max_drivers`): count | 1813-1818 |
| `p2p_resistance(; max:)` | 0 | `max` (`max_resistance`): resistance | 1819-1824 |
| `reliability(; required_lifetime:, reference_lifetime:, reference_stress:, stress_exponent:, reference_temperature:, activation_energy:, max_abs_voltage:, duty_cycle:)` intent | 0 | two times (`*_hours`), voltage, scalar, temperature, energy, voltage, fraction | 1825-1843 |
| `soft_connection(layers..)` | 1+ | | 1844 |
| `supply_short(tap_a, tap_b[, within])` | 2 or 3 | | 1845-1846 |
| `tie_area(region, contact, channel; min_ratio:)` | 3 | fraction | 1847-1852 |
| `unconnected_pin(layers..)` | 1+ | | 1854 |
| `gate_oxide(; models:, max:)`, `drain_source(; models:, max:)` intent | 0 | `models` (`model`); `max` (`max_voltage`): voltage | 1855-1872 |
| `hot_carrier(; models:, max_length:)` intent | 0 | models; length | 1873-1881 |
| `well_sharing(; models:, resistors:)` | 0 | models; bool | 1882-1890 |
| `well_bias(; pmos:, nmos:)` intent | 0 | two opt model lists | 1891-1899 |
| `missing_level_shifter(; shifters:)` intent | 0 | opt `shifters` (`shifter`): models | 1900-1905 |
| `hv_tag(L, tag; voltage:)` intent | 2 | voltage | 1910-1915 |
| `mixed_tag(L, tag)` | 2 | | 1916 |
| `missing_isolation(; isolation:)` intent | 0 | opt models | 1920-1925 |
| `dielectric_field(L[, L2]; max_field:)` intent | 1 or 2 | `max_field`: field | 1928-1939 |
| `voltage_space(L[, L2]; voltage: [..], space: [..])` → `voltage_spacing` intent | 1 or 2 | `voltage`: voltage list; `space`: length list | 1942-1949, 271-274 |

Fifty-seven kinds (`README.md:139`; `GPurify/crates/check/src/erc/ruleset.rs:1`), one SoA table each (section 2.3).

### 1.7 LVS statements

`device` (section 1.3) is the recogniser: kind, marker layer, model name, terminal layers in positional role order (mos: gate, source, drain, bulk; bjt: base, emitter, collector[, bulk]; two-pin devices: pin 0, pin 1), optional value law (`sheet` Ω/□ on a resistor, `area_cap` aF/µm² and `fringe_cap` aF/µm on a capacitor, `inductance` nH on an inductor, `on_gate_min`/`on_gate_max` mV on a mos) (`docs/deck.md:757-825`; `mod.rs:913-947`, `:955-976`; `parse.rs:1255-1345`). What LVS measures from drawn geometry per kind (`w`, `l`, `nf`, `area`, `perim`, `as`, `ad`, `ps`, `pd`, `ae`, `pe`, `r`, `c`, `ind`) is `docs/deck.md:786-807`.

The 21 `lvs` controls (`docs/deck.md:829-1069`; an unknown control word is refused at `parse.rs:1521`):

| Statement | Table field |
|---|---|
| `lvs tolerance default relative r` (once) | `LvsControls::default_tolerance: Option<f64>` (`mod.rs:1014`) |
| `lvs tolerance "model" param relative r absolute a` | `tolerance: Vec<LvsTolerance>` (`mod.rs:1017`, `:1235-1248`; absolute unit per param `absolute_unit`, `:1255-1265`) |
| `lvs ignore_parameter "model" param` | `ignore_parameter: Vec<(StrId, StrId)>` (`mod.rs:1022`) |
| `lvs asymmetric "model"` | `asymmetric: Vec<StrId>` (`:1025`) |
| `lvs permute "model" pin pin` | `permute: Vec<(StrId, u8, u8)>` positions per `pin_names` (`:1030`, `:1218-1227`) |
| `lvs ignore "model"` | `ignore: Vec<StrId>` (`:1035`) |
| `lvs ignore_shorted "model"` | `ignore_shorted` (`:1039`) |
| `lvs ports_by_name` | `ports_by_name: bool` (`:1043`) |
| `lvs range "model" param >= min <= max` | `range: Vec<LvsRange>` with `LvsLimit::{Length, Area, Count}` (`:1046`, `:1188-1210`) |
| `lvs topology relaxed\|strict` | `relaxed: bool` (`:1051`) |
| `lvs connect_implicit "glob"` | `connect_implicit: Vec<String>` (`:1057`) |
| `lvs connect_explicit "a" "b" ..` | `connect_explicit: Vec<Vec<StrId>>` (`:1061`) |
| `lvs max_res r`, `lvs min_caps c` | `max_res`, `min_caps: Option<f64>` ohm, aF (`:1065-1069`) |
| `lvs same_nets "layout" "reference"` | `same_nets: Vec<(StrId, StrId)>` (`:1074`) |
| `lvs count "model" >= min <= max` | `count: Vec<LvsCount>` (`:1078`, `:1175-1182`) |
| `lvs lde "model" od <layer> max <len> [well <layer> scref <len>]` | `lde: Vec<LvsLde>` (`:1082`, `:1157-1169`) |
| `lvs recognize_gates` | `recognize_gates: bool` (`:1087`) |
| `lvs blackbox "glob"` | `blackbox: Vec<String>` (`:1095`) |
| `lvs filter_unused ae af ag c u` | `filter_unused: Vec<UnusedFilter>` (`:1099`, `:1120-1132`) |
| `lvs supply "name" ..` | `supply: Vec<StrId>` (`:1104`) |
| `netlist_scale 1um` | `DeviceRecognition::netlist_scale_nm: Option<f64>` (`:946`) |

### 1.8 PEX statements

`pex <layer>` keys, each once, with dimensions: `thickness` length, `height` length, `sheet` resistance, `dielectric` scalar (optional), `area_cap` cap/area, `fringe_cap` cap/length, `tc1` 1/K and `tc2` 1/K² (optional together, required when `temperature` is stated) (`kinds.rs:1954-1969`; `docs/deck.md:1086-1099`, `:1136-1140`). Stored per layer id in `ProcessStack` as `f64` nm, nm, Ω/□ (Ω per cut on a via layer) at the operating temperature, aF/µm², aF/µm, k (`mod.rs:1279-1300`). `temperature T nominal T0` (`Deck::temperature`, `temperature_law`, `mod.rs:113-116`, `:130-133`, `:300-308`); `corner name layer keys..` (`Deck::corners: Vec<ProcessCorner>`, `mod.rs:120`, `:359-368`); `extraction` (`Extraction{kind, coupling_threshold_af, coupling_halo}`, `mod.rs:374-389`, kinds `mod.rs:395-412`); `pex_fill`, `pex_device`, `pex_tsv`, `pex_substrate` (`PexModels`, `mod.rs:215-282`).

### 1.9 Flow statements

`report p2p|coupling|cluster` → `Reports` (`mod.rs:148-160`); `fill`, `widen` → `Edits{fill: Vec<FillSpec>, widen: Vec<WidenSpec>}` (`mod.rs:167-208`); `via_redundancy` is a DRC rule that `gpurify enhance` also reads (`docs/deck.md:1315-1321`); `temperature` (1.8); `grid` (1.3); `boundary` (`Deck::boundary: Option<LayerId>`, `mod.rs:129`); `netlist_scale` (1.7); `stage` → `Stages{name: Vec<StrId>, of_rule: Vec<(StrId, u16)>}` (`mod.rs:333-354`); `waive` → `Deck::waivers: Vec<(StrId, LayerId)>` (`mod.rs:125`).

### 1.10 Custom or unknown statements

The text grammar admits no custom statement, key or value: an unknown keyword (`parse.rs:547-550`), check kind (`:2649`), parameter (`:2774`), layer operation (`:984`), unit (`:3191`), `connect` word (`:1250`), device kind (`:1263`), device value (`:1315`), `lvs` control (`:1521`), `report` word (`:2222`), extraction type or key (`:2253`, `:2281`) is each a located error; parsing continues to collect up to 50, then the deck is refused (`parse.rs:17`; `mod.rs:76-78`). Nothing is ignored or passed through: the only free text is a `#` comment (`docs/deck.md:65`). The deliberate design is "the deck language deliberately is not" scriptable (`GPurify/docs/deck-library-plan.md:15-16`, `:27`).

The extension mechanism is Rust, not text: `DeckBuilder` states a deck or overrides a compiled one (add, replace, remove rules, layers, devices, `pex` rows) (`GPurify/crates/check/src/deck/builder.rs:1-16`, `:717`, `:816`, `:1329-1382`); `CustomRule` is a `fn(&CustomCtx, &mut Findings)` run after the DRC rules with the runner owning the accounting (`GPurify/crates/check/src/deck/custom.rs:5-28`, `:76`, `:82`); `gpurify::cli::main_with` ships a binary whose deck is a Rust function by name (`GPurify/src/cli/mod.rs:53`). A deck with custom rules cannot be printed back to text (`GPurify/crates/check/src/deck/mod.rs:153-166`). Design intent (JSON, `--intent`) is the other input and is `deny_unknown_fields` throughout (`GPurify/crates/ingest/src/intent.rs:257-277`). Philis's `cell` sidecar is a separate JSON file Philis reads, never the deck (`Philis/pdks/sky130.json:2-4`; `Philis/backend/verify/src/pdk.rs:80-90`).

## 2. Rule tables as data

### 2.1 Primitive types and widths

| Type | Definition | Width |
|---|---|---|
| `Dbu` | `GPurify/crates/geom/src/dbu.rs:23` | `i64`, within ±2^40 (`:8`) |
| `DbuArea` | `dbu.rs:31` | `i128` |
| `Grid` | `dbu.rs:137-140`, `new(dbu_per_um)` `:166`, `to_dbu` exact-or-refused `:188`, `to_length` `:228` | `i64` dbu per µm |
| `Qty<D, P>` | `GPurify/crates/geom/src/qty.rs:77-80`; dimensions `:24-45`; `celsius` `:49`; prefixes `lib.rs:98-120` | `f64` tagged by dimension and SI exponent |
| `LayerId` | `GPurify/crates/geom/src/ids.rs:19` | `u16` |
| `PolyId` | `ids.rs:6` | `u32` |
| `StrId`, `StrTable` | `GPurify/crates/geom/src/intern.rs:9`, `:16-20`; `intern`/`get`/`resolve` `:25-48` | `u32`, ids in interning order |
| `Bbox` | `GPurify/crates/geom/src/bbox.rs:26-34` | four `Dbu` |

All are `pub` (`GPurify/crates/geom/src/lib.rs:88-92`, `:123`, `:144-148`). Ids are assigned in interning order, so reading inputs in a fixed order gives identical ids and byte-identical reports (`GPurify/crates/ingest/src/lib.rs:26-28`).

### 2.2 The parsed deck: `gpurify_ingest::deck::Deck`

`parse_deck(source, grid, &mut StrTable) -> Result<Deck, DeckError>` (`GPurify/crates/ingest/src/deck/parse.rs:115`). `Deck` and every table below are `pub` with `pub` fields unless stated (`GPurify/crates/ingest/src/deck/mod.rs:101-142`):

| Field | Type | Contents |
|---|---|---|
| `layers` | `LayerTable` (`mod.rs:431-447`, fields private) | `len()`, `id(&StrTable, name) -> Option<LayerId>` (`:464`), `of_stream(l, d) -> Option<LayerId>` binary search (`:472`), `stream_of(LayerId) -> (u16, u16)` (`:483`), `is_ignored` (`:489`), `is_derived` (`:497`), `is_edges` (`:506`), `name(LayerId) -> StrId` (`:517`), `op(LayerId) -> Option<DerivedOp>` (`:522`), `operands(LayerId) -> &[LayerId]` (`:531`), `text(u32)` (`:547`). Ids dense from 0: drawn layers first ascending by name bytes, derived layers after in declaration order, hidden layers interleaved (`:422-429`) |
| `rules` | `RuleTable` (`mod.rs:851-859`) | `spec: Vec<RuleSpec>`, `layer_ref: Vec<LayerId>`, `param: Vec<(StrId, ParamValue)>`; `layers_of(&RuleSpec) -> &[LayerId]` in engine order (enclosure outer first) (`:865`), `params_of` (`:872`), `param(&RuleSpec, StrId) -> Option<ParamValue>` first row of a list, `None` for absent or `none` (`:878`) |
| `RuleSpec` | `mod.rs:806-822` | `id: StrId`, `kind: StrId` (engine kind, e.g. `min_width`), `layer_start/layer_len: u32`, `param_start/param_len: u32` |
| `ParamValue` | `mod.rs:826-847` | `Length(Dbu)`, `Ratio(f64)` (mV, µA, A/m, ohm, K, eV, h, MHz, fraction, scalar as the name says), `Count(u32)`, `Flag(bool)` (`nets: same`, `maximum`, `warning`), `Area(DbuArea)`, `Layer(LayerId)`, `Model(StrId)` |
| `connectivity` | `Connectivity` (`mod.rs:891-911`) | `conductors: Vec<LayerId>`, `via_cut: Vec<LayerId>` paired with `via_connects: Vec<(LayerId, LayerId)>`, `intra_layer_touch: bool`, `global: Vec<LayerId>`, `label_layer: Vec<LayerId>` paired with `label_names: Vec<LayerId>` |
| `devices` | `DeviceRecognition` (`mod.rs:924-947`) | SoA: `kind: Vec<DeviceKind>`, `marker: Vec<LayerId>`, `terminal_start: Vec<u32>` (CSR, one longer), `terminal: Vec<LayerId>`, `model: Vec<StrId>`, `value: Vec<DeviceValue>`, `netlist_scale_nm: Option<f64>`; `DeviceValue{sheet_ohm_sq, area_cap_af_um2, fringe_cap_af_um, inductance_nh, on_gate_min_mv, on_gate_max_mv: Option<f64>}` (`:955-976`); `DeviceKind::{Mos, Bjt, Resistor, Capacitor, Diode, Inductor, Cell}` (`:981-1002`) |
| `stack` | `ProcessStack` (`mod.rs:1279-1300`) | per `LayerId`: `thickness_nm`, `height_nm`, `sheet_res_ohm_sq`, `area_cap_af_um2`, `fringe_cap_af_um`, `dielectric_k: Vec<f64>`; empty when no `pex`, else `layers.len()` long, `0.0` means not stated |
| `temperature` | `Option<Qty<Temperature, BASE>>` (`:115`) | K |
| `extraction` | `Extraction` (`:374-389`) | `kind: ExtractionKind`, `coupling_threshold_af: f64`, `coupling_halo: Option<f64>` |
| `corners` | `Vec<ProcessCorner>` (`:359-368`) | `name: StrId`, `stack: ProcessStack`, `stated: Vec<(LayerId, CornerValues)>` |
| `stages` | `Stages` (`:333-354`) | `name: Vec<StrId>`, `of_rule: Vec<(StrId, u16)>`, `of(StrId) -> Option<u16>` |
| `waivers` | `Vec<(StrId, LayerId)>` (`:125`) | |
| `boundary` | `Option<LayerId>` (`:129`) | |
| `temperature_law` | `Option<TemperatureLaw{operating_k, nominal_k, rows: Vec<(LayerId, f64, (f64, f64))>}>` (`:300-308`) | |
| `lvs` | `LvsControls` (`:1009-1105`, section 1.7) | |
| `pex_models` | `PexModels{fill: Vec<(String, FillTreatment)>, device: Vec<DeviceParasitics>, tsv: Vec<TsvModel>, substrate: Option<SubstrateModel>}` (`:215-231`; rows `:236-271`) | by layer and model name as written |
| `edits` | `Edits{fill: Vec<FillSpec>, widen: Vec<WidenSpec>}` (`:167-208`) | by layer name as written |
| `reports` | `Reports{p2p, coupling, cluster}` (`:148-160`) | |

`LvsSrc` (the pre-lowering `lvs` statements) is `pub` with `pub(crate)` fields (`mod.rs:1501-1537`); its setters `default_tolerance`, `blackbox`, `supply`, `filter_unused`, `from_controls` are `pub` (`:1560`, `:1578`, `:1601`, `:1621`, `:1642`).

Every length in `Deck` is already `Dbu` on the layout grid it was parsed against; a deck is meaningful only with a layout on that grid (`mod.rs:93-99`). Philis parses at 1000 dbu/µm so one dbu is one nm (`Philis/backend/verify/src/pdk.rs:13-18`).

### 2.3 The compiled deck: `gpurify_check::deck::CompiledDeck`

`compile(source, grid, &mut StrTable) -> Result<CompiledDeck, CompileError>` parses then types (`GPurify/crates/check/src/deck/mod.rs:202-209`); `CompiledDeck::from_deck(Deck, Grid, &StrTable)` types a parsed deck and refuses a rule kind in neither `drc::ruleset::KINDS` nor `erc::KINDS` (`:223-233`). `CompiledDeck` (`:61-109`): `grid: Grid`, the `Deck` tables above except `rules` (dropped at `:244`), plus `drc: drc::RuleSet`, `erc: erc::RuleSet`, `custom: Vec<CustomRule>`, `provenance: Vec<(StrId, Origin)>` with `Origin::{Base, Added, Replaced}` (`:118-127`). `rule_count()` (`:286`), `rules() -> impl Iterator<Item = (StrId, &'static str)>` (`:292`), `print(&CompiledDeck, &StrTable) -> Result<String, PrintError>` (`:153`). `CompileError::{Deck, Drc, Erc}` (`:170-181`); `DrcError` variants `UnknownKind`, `MissingParam`, `WrongParamType`, `WrongLayerCount`, `NonPositiveLimit`, `UnrepresentableAngle`, `DuplicateRule`, `BadTable` (`GPurify/crates/check/src/drc/mod.rs:35-111`); `ErcError` likewise plus `NotAFraction` (`GPurify/crates/check/src/erc/mod.rs:223-283`).

`drc::RuleSet` (`GPurify/crates/check/src/drc/ruleset.rs:3773-3800`): `rules: Vec<(StrId, Rule)>` in deck order, side tables `tables: Vec<SpacingTable>`, `classes: Vec<Vec<(Dbu, Dbu)>>`, `widths: Vec<Vec<Dbu>>`, `class_tables`, `span_tables`, `cover_bands`, `cut_area_bands`, `patterns`, `net_sources`. `Rule` is a `Copy` enum with one variant per engine kind, each carrying its `LayerId`s and `Dbu`/`DbuArea`/`f64`/`u16`/`u8` fields (`ruleset.rs:71`, variant list in the crate's rule reference `GPurify/crates/check/src/lib.rs:38-122`); `Rule::kind()` (`:6571`), `Rule::layers() -> [Option<LayerId>; 3]` (`:6664`), `Rule::validate` (`:6122`), `Rule::required_spacing(layer, tables, &SpacingQuery) -> Option<Dbu>` is the function the spacing checks themselves call (`:6525-6568`). `SpacingQuery{width_a, width_b, run_length: Dbu, end_of_line: Option<Dbu>}` (`:3635-3651`); `WireBand{from, below, extra_cut, length}` (`:3466-3500`).

`erc::RuleSet` is one SoA table per kind, each with `head: RuleHead{rule: Vec<StrId>, severity: Vec<Severity>}` (`GPurify/crates/check/src/erc/ruleset.rs:24-29`, `:49`). Tables an analog flow reads: `EmCurrentDensityTable{layer_start, layer, max_density: Vec<Qty<CurrentDensity, BASE>>, max_current_per_cut: Vec<Qty<Current, MICRO>>, narrow}` (`GPurify/crates/check/src/erc/rules/electrical.rs:145-161`); `ElectromigrationTable` adds `blech_limit`, `reference_temperature: Vec<Qty<Temperature, BASE>>`, `activation_energy_ev`, `current_exponent: Vec<f64>` (`:209-231`); `EmAcTable` (`:844-865`); `EmThermalTable` (`:1145-1183`); `MissingTieTable{region, tap: Vec<LayerId>, max_distance: Vec<Dbu>}` (`erc/rules/supply.rs:122-131`); `ConditionalTieTable` (`erc/rules/latchup.rs:769-789`); `GuardRingTable{target, ring, max_space, min_width, pad, near, clear}` (`:336-354`); `GuardRingBiasTable` (`:60-67`); `TieAreaTable` (`:172-182`); `LatchupVictimTable` (`:929-949`); `ForwardDiodeTable` (`:659-668`); `AntennaTable{gate, collector_start, collector, collector_measure: Vec<AntennaMeasure>, max_ratio, diode, diode_area, diode_min, stack}` (`erc/rules/antenna.rs:296-322`); `AntennaElectricalTable` (`:423-468`); `DensityCmpTable{layer, window, step: Vec<(Dbu, Dbu)>, min_density, max_density, max_neighbour_delta, cmp, include_partial_windows}` (`:510-529`); `ModelLimitTable{model_start, model, max_voltage: Vec<Qty<Voltage, MILLI>>}` for `gate_oxide` and `drain_source` (`erc/rules/domain.rs:44-53`); `WellBiasTable` (`:80-91`); `HotCarrierTable` (`:1242-1251`); `VoltageSpacingTable{layer, other, step_start, voltage, space}` (`:201-215`); `DielectricFieldTable` (`:253-266`); `MatchingTable{group: Vec<LayerId>, tolerance: Vec<Dbu>}` (`erc/rules/matching.rs:69-75`); `ReliabilityTable` (`erc/rules/reliability.rs:63-82`). `RuleSet::from_deck` (`erc/ruleset.rs:634`), `validate` (`:1419`), `rules()` (`:1429`), `run` (`:1926`).

### 2.4 Querying by (layer, kind) without re-parsing

Two routes, both reading tables already in memory; neither re-parses.

Route A, string-keyed, on the parsed `Deck`: intern-free lookups through `strings.get("min_width")` then a linear scan of `rules.spec` filtering `kind` and `layers_of` (`mod.rs:865-883`). No index by layer or kind exists; `param` is a linear scan of the rule's row run. This is what Philis 1.0 does (section 4.7). A renamed kind or engine parameter becomes a silent `None` (`GPurify/docs/deck-library-plan.md:48-54`).

Route B, typed, on `CompiledDeck` (`GPurify/crates/check/src/deck/query.rs`): `drc_on(layer) -> impl Iterator<Item = (StrId, &Rule)>` (`:170`); `min_width` (`:195`), `min_area` (`:206`), `min_enclosed_area` (`:217`), `required_spacing(layer, &SpacingQuery) -> Option<Dbu>` taking the largest of `min_spacing`, wide, attached-wide, prl, eol and spacing-table rules that apply, through the checker's own threshold function (`:253-259`, `:226-229`); `enclosure(inner, outer) -> Enclosure{all_sides, opposite, noncoincident, at_most: Option<Dbu>, full_width: bool}` (`:262`, `:23-41`); `allowed_widths` (`:97`), `allowed_widths_from` (`:110`), `cut_classes` (`:125`), `max_aspect_ratio` (`:136`), `max_total_area` (`:147`), `min_step` (`:158`), `via_array_spacing(cut, cuts)` (`:313`), `cut_spacing(cut, center)` (`:330`), `cut_class_spacing` (`:347`), `minimum_cuts(cut, wide, wire_width)` (`:361`), `max_via_stack` (`:380`), `projection_spacing` (`:405`), `contains` (`:421`), `density(layer) -> impl Iterator<Item = DensityLimit{rule, window: Option<(Dbu, Dbu)>, limit: f64, sense}>` (`:436`, `:45-55`), `antenna(collector) -> impl Iterator<Item = AntennaLimit{rule, gate, max_ratio, measure, diode}>` (`:469`, `:59-71`), `em_limit(layer) -> impl Iterator<Item = EmLimit{rule, max_density, max_current_per_cut}>` from `em_current_density` rows only (`:514`, `:75-82`), `fet_limits(model: StrId) -> FetLimits{gate_oxide, drain_source: Option<Qty<Voltage, MILLI>>}` (`:528`, `:87-92`). Every length is in dbu of `CompiledDeck::grid` (`:9-11`). The oracle "query equals the limit the checker reports" is ledgered as D3 done (`GPurify/docs/testing-plan.md:524`).

Not in the query API, but reachable through `drc_on` and a `match` on `Rule`: `ExactPitch`, `OffGrid`, `EdgeWidth`, `MinExtension`, `MinSpacingDiff`, `MaxDistanceToTap`, `Overlap`; and through the ERC tables directly: `electromigration` Black parameters, `missing_tie`, `guard_ring`, `matching`, `density_cmp`.

Both routes need the `StrTable` the deck was interned into; `CompiledDeck` drops the `RuleTable`, so a consumer wanting both keeps the `Deck` from `parse_deck` and passes a clone to `from_deck`.

### 2.5 The name-level form: `DeckSource`

`parse_source(source, grid) -> Result<DeckSource, DeckError>` stops before lowering (`parse.rs:128`); `lower(&DeckSource, &mut StrTable) -> Result<Deck, DeckError>` (`source.rs:234`); `print(&DeckSource, Grid) -> Result<String, PrintError>` is the inverse (`GPurify/crates/ingest/src/deck/print.rs:70`). `DeckSource` fields are all `pub` and by name: `layers: Vec<(String, (u16, u16))>`, `ignored`, `derived: Vec<DerivedSource{name, op, layers}>`, `texts`, `rules: Vec<RuleSource{id, kind: &'static str, layers: Vec<String>, params: Vec<(&'static str, ParamSource)>}>`, `connectivity: ConnectivitySource`, `devices: Vec<DeviceSource{kind, marker, model, terminals, value}>`, `pex: Vec<(String, StackSource)>`, `temperature: Option<(f64, f64)>`, `lvs: LvsSrc`, `extraction`, `corners: Vec<CornerSource>`, `stages`, `waivers`, `boundary`, `netlist_scale_nm`, `pex_models`, `edits`, `reports` (`source.rs:22-69`, rows `:73-197`). `CompiledDeck::source(&StrTable) -> DeckSource` decompiles typed rules back to rows (`GPurify/crates/check/src/deck/decompile.rs:61`). This is the form `DeckBuilder` fills (`builder.rs:3-5`).

## 3. In-process API

### 3.1 Load a deck

1. `let mut strings = StrTable::default();` (`GPurify/crates/geom/src/intern.rs:16`).
2. `let grid = Grid::new(1000)?;` for 1 nm (`dbu.rs:166`), or `library.grid()?` from a GDS `UNITS` record (`GPurify/crates/ingest/src/layout.rs:909`).
3. Either `let deck: Deck = parse_deck(text, grid, &mut strings)?` (`parse.rs:115`) or `let deck: CompiledDeck = gpurify_check::deck::compile(text, grid, &mut strings)?` (`GPurify/crates/check/src/deck/mod.rs:202`). The shipped texts are constants `gpurify::pdks::{SKY130, GF180MCU, IHP_SG13G2, GENERIC_FINFET}` and `ALL`, with `sky130(grid, &mut strings) -> Result<CompiledDeck, CompileError>` and siblings (`GPurify/src/pdks.rs:31-51`, `:59-91`).
4. For a Rust-stated or overridden deck: `DeckBuilder::new(grid, &mut strings)` or `DeckBuilder::from_deck(&CompiledDeck, &mut strings)`, then `layer`, `derived`, `rule(id, RuleRow)`, `add`, `replace`, `remove`, `device`, `pex`, `custom`, `finish() -> Result<CompiledDeck, BuildError>` (`builder.rs:783`, `:816`, `:962`, `:979`, `:1280`, `:1329`, `:1351`, `:1382`, `:1078`, `:1156`, `:1294`, `:1550`). Typed constructors per kind are generated from one macro table in `GPurify/crates/check/src/deck/rules.rs:1-16`; `RuleRow` (`:60-68`) carries `.wire(WireBand)`, `.warning()` and the other qualifiers (`:104-359`).

### 3.2 Build a `GeometryStore` from shapes in memory

1. `GeometryStoreBuilder::with_capacity(polys, verts)` or `default()`; `push(LayerId, &xs, &ys) -> u32` per open ring, counter-clockwise outer, clockwise hole (`GPurify/crates/geom/src/store.rs:237`, `:256`).
2. `finish(layer_count) -> (GeometryStore, Vec<u32>)` with `layer_count = deck.layers.len()` so empty and derived layers have ranges (`store.rs:287`; `mod.rs:456`).
3. `Provenance::default()`, `place_label(Point, LayerId, StrId)` per net name (`GPurify/crates/ingest/src/provenance.rs:116`).
4. `gpurify_ingest::layout::derive_layers(&mut store, &deck, &provenance, &strings)` computes every derived layer in id order; `deck` is anything `AsRef<LayerTable>` (`Deck` or `CompiledDeck`) (`layout.rs:209-216`; `AsRef` impls `mod.rs:312-323`, `check/deck/mod.rs:304-308`).
5. `provenance.resolve_labels(&store, &deck.connectivity)?` binds labels to rows, failing closed on a label on no shape (`provenance.rs:134-137`, `LabelError` `:37-50`).

From a GDS instead: `read_gds_bytes(&Path)` (`layout.rs:166`), `gds::Library::parse(&bytes, &mut strings)` (`:938`), `library.flatten(&deck, &strings, UnknownLayers::Reject) -> (GeometryStore, Provenance)` (`:1733-1738`), then step 5. `read_die(gds: &[u8], deck: &str, deck_file, &mut strings) -> Die{grid, deck: CompiledDeck, store, provenance}` does all of it from bytes (`GPurify/src/engine/pipeline.rs:285-301`, `:247-255`).

Philis's `build_store` is exactly steps 1 to 5, with a pre-merge of routing layers through `validate_layer_into`, `merge_into` and `decompose_into` so PEX does not double count overlaps (`Philis/backend/verify/src/geom.rs:41-94`).

### 3.3 Extract nets, devices, ports

`gpurify::engine::pipeline::extract_layout(&store, &CompiledDeck, &provenance) -> Result<Extracted, ExtractError>` runs `refuse_conducting_channels`, `extract_nets_into`, `recognise_into`, `bind_ports_into` (`pipeline.rs:482-486`; steps `GPurify/crates/check/src/topology/mod.rs:3-13`). `Extracted{nets: NetTable, devices: DeviceTable, ports: PortTable}` (`pipeline.rs:69-77`). Lower level: `extract_nets_into(&store, &connectivity, &mut NetTable)` (`topology/net.rs:294`), `NetTable::net_of(PolyId)`, `polys_of(NetId)`, `same_net`, `path` (`:139-179`); `DeviceTable::terminals_of(DeviceId) -> (&[NetId], &[TerminalRole])`, `params_of -> &[(DeviceParam, DeviceMeasure)]`, `devices_on(NetId)` (`topology/device.rs:186-210`); `PortTable::name_of(NetId)`, `net_of(StrId)`, `shorted_net()` (`topology/port.rs:51-90`).

### 3.4 Run the checks

All at once: build a `Loaded{strings, grid, deck: CompiledDeck, store, provenance, reference: Option<Netlist>, intent: Option<DesignIntent>, design: Option<lefdef::Design>}` by hand (all fields `pub`, `pipeline.rs:44-64`; call `intern_report_ids(&mut strings)` first, `:323`), then `run_checks(&Loaded, &Extracted, &RunOptions{checks: Checks{drc, erc, lvs, pex}, lvs: CompareOptions, quasistatic_nets: Vec<String>}) -> (Outputs, Summary)`; it cannot fail, a check that cannot run is a `StageStatus` (`GPurify/src/engine/run.rs:220-224`, `:24-61`). From files: `run(&Inputs{layout, lef, deck, reference, intent}, &RunOptions) -> Result<(Loaded, Extracted, Outputs, Summary), EngineError>` (`run.rs:1082`; `pipeline.rs:20-38`).

Per domain:

- DRC: `deck.drc.run_with_nets(&store, Some(&nets), &mut Violations, &mut Vec<RuleRun>)`; `run` without nets refuses `net_spacing` rows (`drc/ruleset.rs:5249`, `:5266-5271`). Custom rules: `run_custom` (`check/deck/mod.rs:41`).
- ERC: `erc::check(&deck.erc, Inputs{design: Design{store, nets, devices}, ports, intent: Option<&DesignIntent>, process: Process{grid, stack: &ProcessStack, connectivity: &Connectivity}, die: Bbox, temperature: Option<Qty<Temperature, BASE>>, antenna_pins, strings}, &mut out, &mut runs)` (`GPurify/crates/check/src/erc/mod.rs:70`, `:35-58`; `Process` `erc/power.rs:75-85`; `Design` `check/src/lib.rs:235-242`). The die is the `boundary` layer's extent else the drawn layers' union (`run.rs:182-192`).
- LVS: reference from text, `netlist::spice::read(&str, &mut strings, &deck.devices)` or `spectre::read` (`GPurify/crates/ingest/src/netlist.rs:1489`, `:1520`), or a `Netlist` built field by field (all `pub`, `:176-250`) as Philis does (`Philis/backend/verify/src/reference.rs:72-78`); `netlist.top()` then `flatten_top` or `flatten_top_with(top, &deck.lvs.blackbox, &mut strings)` (`netlist.rs:571`, `:313`, `:340`). Then `graph::from_layout(&nets, &devices, &ports, &strings, Some(grid))` (`GPurify/crates/check/src/lvs/graph.rs:223-229`), `add_lde_params`, `check_layout(&layout, design, &ports, (&strings, &deck.lvs), &mut out, &mut runs)` (`lvs/checks.rs:137-144`), `check_param_ranges(design, &deck.lvs.range, ..)` (`:184`), `check_device_counts(design, &deck.lvs.count, ..)` (`:259`), `add_black_boxes`, `join_must_connect`, `graph::from_reference(&netlist, top)` (`graph.rs:389`), `compare_with(&layout, &reference, CompareOptions, &deck.lvs) -> (Verdict, Vec<(Side, Merge)>)` (`lvs/compare.rs:289-294`; `CompareOptions{max_rounds, param_tolerance, params}` `:96-122`), `record_discrepancies(&found, &strings, &mut out)` (`checks.rs:380`). The engine's sequence is `run.rs` `run_lvs`. `Verdict::{Match, Mismatch(Vec<Discrepancy>), Inconclusive}` (`GPurify/crates/check/src/lvs/verdict.rs:21-38`); `Discrepancy` variants `:72-222`; report ids `RULE_IDS` (`checks.rs:40`).
- PEX: `analytical::extract_into(&store, &nets, &connectivity, (&stack, halo), grid, &mut ParasiticNetwork)` with `halo = deck.extraction.coupling_halo` (`GPurify/crates/extract/src/analytical.rs:162-169`); `quasistatic::extract_into(&store, &nets, &connectivity, &selected, &stack, grid, &mut CapMatrix, &mut out) -> Result<Accuracy, SolveError>` and `extract_inductance_into`, then `overlay(&mut network, &solved)` (`GPurify/crates/extract/src/quasistatic.rs:46-55`, `:107`, `:187`); `device_region::add_into` (`device_region.rs:33`), `tsv::add_into` (`tsv.rs:58`), `substrate::add_into` (`substrate.rs:551`), `fill::treat` (`fill.rs:51`), `reduce::reduce(&mut network, deck.extraction)` (`reduce.rs:54`). Output `ParasiticNetwork{node_net: Vec<NetId>, node_layer, from, to: Vec<Option<NodeId>>, value: Vec<Parasitic>}` with `Parasitic::{Resistance, GroundCap, CouplingCap, Inductance, MutualInductance, SubstrateResistance}` in Ω, fF, pH (`GPurify/crates/extract/src/network.rs:99-114`, `:14-40`); `capacitance_per_net()` (`:228`). Corners are extracted once per `deck.corners` stack (`run.rs` `run_pex`; `Outputs::corners`).

### 3.5 Results and coverage

`Outputs{violations: Violations, runs: Vec<RuleRun>, lvs: Option<Verdict>, lvs_merges, lvs_unpaired_instances, parasitics: Option<ParasiticNetwork>, waived: Violations, corners: Vec<(String, ParasiticNetwork)>}` (`run.rs:93-124`). `Violations` is SoA: `rule: Vec<StrId>`, `layer`, `severity`, `at: Vec<Point>`, `measured`, `limit: Vec<Measurement>`, `shape_a`, `shape_b`, `fix` (`GPurify/crates/check/src/report/violation.rs:73-95`), `get(i) -> Violation` (`:185`), `sort_canonical` (`:208`). `Measurement::{Length(Dbu), Area(DbuArea), Ratio(f64), Count(u32), Voltage(mV), Current(µA), Resistance(Ω)}` (`report/measure.rs:8-23`). Coverage is `RuleRun{rule, outcome: Outcome::{Ran, Skipped(SkipReason), Refused}, examined: u64, violations: u32}` per rule (`violation.rs:281-326`), and `Summary{drc, erc, lvs, pex: StageStatus, violations, errors, warnings, rules_clean, rules_skipped}` with `passed()` true only when every selected stage ran and nothing was skipped or refused (`run.rs:129-157`). Writers: `export::json::write_report(&Report{header, violations, waived, runs, strings, grid}, &mut String)` (`GPurify/src/export/json.rs:36-60`), `parasitic::write_spef(&network, &ports, &strings, &Header, &mut String)`, `write_spef_corners`, `write_dspf`, `netlist::write_spice(Extraction, Option<&ParasiticNetwork>, Detail, ..)` (`GPurify/src/export/parasitic.rs:364-370`, `:393`, `:707-713`; `export/netlist.rs:81-88`; `Header` `export/mod.rs:92-105`). Incremental re-check of an edited store: `engine::incremental::check(&CompiledDeck, GeometryStore) -> DrcState` and `recheck` (`GPurify/src/engine/incremental.rs:48`, `:82`, `:201`).

### 3.6 What needs files on disk

Nothing in the check path. Disk is touched only by the convenience readers: `read_gds_bytes(&Path)` (`layout.rs:166`), `read_intent(&Path)` (`intent.rs:670`; `parse_intent(&str)` at `:458` is the memory form), `read_system(&Path)` (`system.rs:307`), `engine::pipeline::load`/`load_with`/`load_die` and `engine::run::run` which take `Inputs` paths (`pipeline.rs:103`, `:116`, `:264`; `run.rs:1082`), and `load_bytes` which takes GDS bytes but still reads the deck, reference and intent from `Inputs` paths (`pipeline.rs:127-133`). Decks, intent, netlists and GDS all have `&str`/`&[u8]` entry points.

### 3.7 Drift between the pin and HEAD

At the pin (`be14a73f`) `Loaded.deck` was a parsed `Deck`, `Loaded` had no `design` field, `run_checks` returned `Result<(Outputs, Summary), EngineError>`, and `RunOptions` had a `quasistatic_inductance: bool` (pinned `src/engine/pipeline.rs:44-52`, `src/engine/run.rs:35-42`, `:156-160`); `crates/check/src/deck/` and `src/pdks.rs` did not exist. Philis's `Checker::new` and `Checker::run` are written against that shape (`Philis/backend/verify/src/checker.rs:80-88`, `:210-217`) and do not compile against HEAD. `parse_deck`, `Deck`, `RuleTable` and every ingest type Philis's `pdk.rs` reads are unchanged in shape at HEAD.

## 4. What Philis 1.0 adds on top

### 4.1 The sidecar and its registry

A Philis PDK is `{"deck": "<file>", "cell": {..}}` (`Philis/pdks/sky130.json:2-4`). `Pdk::deck_text` resolves the deck name against the vendored copies compiled in (`Philis/backend/verify/src/decks.rs:9-14`), then an absolute path, then GPurify's `pdks/` found at build time via `GPURIFY_PDKS` (`Philis/backend/verify/src/pdk.rs:105-127`; `Philis/backend/verify/build.rs:9-22`). Every `cell.*` key must be in the registry `sidecar::KEYS` with its `Kind` (`Nm`, `Count`, `Bool`, `Real`, `Tier` = `[MIN, MOD, EXC]`, `Text`, `List`, `Table`, `Layers`), `required`, `sourced` (a non-null value needs a non-empty `<key>_source`; `UNVERIFIED` sources load and are listed by `Pdk::unverified`) and `reader` (`Philis/backend/verify/src/sidecar.rs:19-54`, `:63-153`, `:164-206`; `pdk.rs:631-637`). An unknown key fails the load (`sidecar.rs:174-176`).

### 4.2 Every key in `pdks/sky130.json`

Units: `Nm` keys are integer nm; `Tier` arrays index `MatchClass::{Minimal, Moderate, Exceptional}` (`Philis/kernel/core/src/process.rs:107-112`); readers from `sidecar.rs:63-153`.

| Key (line) | Kind | Meaning | Reader |
|---|---|---|---|
| `bjt_ka_pct_um` (5) | Real, sourced | bipolar I_S mismatch area constant, %·µm (Hastings textbook value) | library annotation |
| `vbe_tc_uv_per_k` (7) | Real, sourced | |dV_BE/dT|, µV/K | library annotation |
| `m1_enc` (9) | Nm | met1 enclosure floor a generator raises the deck's to | `cells/builder.rs dim` |
| `min_finger_width` (10) | Nm | narrowest MOS finger a generator draws | builder dim; cellgen |
| `via_enclosure` (11) | Nm | via pad enclosure floor | builder dim |
| `cap_density_ff_um2` (12) | Real, sourced | MIM capacitance density, fF/µm² (= `camimc`) | benchmarks preprocessor |
| `gate_cap_af_um2` (14) | Count, sourced UNVERIFIED | gate oxide capacitance, aF/µm² | library annotation |
| `dummy_max_l_nm` (16) | Nm, sourced | dummy gate length cap, microloading reach | `cells/mosfet.rs` |
| `wpe_clearance_nm` (17) | Tier, sourced | well edge to matched gates, per class | mosfet, library, matching class |
| `lod_moat_ext_nm` (19) | Tier, sourced | STI/active extension past matched gates, per class | mosfet, library, matching class |
| `dummy_reach_nm` (21) | Tier, sourced | outer dummy poly reach, per class | matching class |
| `gate_ext_extra_nm` (23) | Tier, sourced | extra poly end-cap, per class | matching class |
| `res_dummy_span_nm` (25) | Tier, sourced | resistor dummy span, per class | matching class |
| `res_width_floor_permille` (27) | Tier, sourced | resistor width floor, ‰ of minimum | matching class |
| `res_length_floor_x` (29) | Tier, sourced | resistor length floor, × minimum | matching class |
| `layers` (32-60) | Layers, required | role → deck layer (`diff, poly, rpoly→poly_rs, li, licon, mcon, met1, nwell, nsdm, psdm, tap, diom→areaid_diode`), `routing_metals` (li, met1..met5), `routing_vias` (mcon, via, via2, via3, via4) | `pdk.rs parse_roles` (`:1550-1594`) |
| `sd_width` (61) | Nm, required | source/drain diffusion width between gates | mosfet |
| `dummy_gates_per_end` (62) | Count, required | dummies each end of a matched row | mosfet |
| `max_finger_width` (63) | Nm, required | finger fold limit, 0 = none | cellgen |
| `plate_spacing` (64) | Nm, required | capacitor plate gap | capacitor, cap_array |
| `res_head` (65) | Nm, required | resistor head length | resistor |
| `well_enclosure` (66) | Nm | well past device | tests only |
| `device_gap` (67) | Nm, required | gap between devices | capacitor; emit |
| `well_spacing` (68) | Nm | | unread |
| `dti` (69) | Nm | deep trench isolation (null) | unread |
| `res_corner_squares` (70) | Real | squares per resistor corner | unread |
| `res_serpentine_aspect` (71) | Real | | unread |
| `res_min_segment` (72), `res_seg_gap` (73) | Nm, required | serpentine segment length and gap | resistor |
| `bjt_max_emitter_stripe` (74), `bjt_stripe_gap` (76) | Nm | | unread |
| `bjt_min_emitter_side` (75) | Nm, required | | bjt |
| `diode_gap` (77), `diode_l` (121), `diode_w` (122) | Nm, required | diode construction | diode; elaborate |
| `min_guard_ring_width` (78) | Nm, required | guard ring width | bjt, post_cell |
| `guard_licon_pitch` (79) | Nm, required | contact pitch along a guard ring | mosfet, post_cell |
| `p_epi_thickness` (80), `p_well_depth` (112), `n_well_depth` (114), `retrograde_pwell` (116) | Nm/Bool, sourced UNVERIFIED | substrate geometry | unread (`retrograde_pwell`: post_cell drawable) |
| `substrate_kind` (82) | Text, sourced | `"bulk"` / `"epi_on_pplus"` / null, decides whether isolation distance saturates | library annotation (`process.rs:114-135`) |
| `epi_thickness_nm` (84) | Nm, sourced | epi thickness (null) | library annotation |
| `placement_space` (85-110) | Table | two-layer spacings on derived layers (`diff,diff` 270 difftap.3, `poly,tap` 300 poly.6, `licon,poly` 55 licon.11, `rpm,poly` 480 poly.9, ..), each with the deck rule and line it copies | `library/src/lib.rs place_rules` |
| `sheet_tolerance` (118), `linewidth_control_nm` (119) | Table | empty | unread |
| `cap_unit_side` (120) | Nm, required | unit capacitor side | capacitor, cap_array |
| `mom_finger_space` (123), `mom_finger_width` (124) | Nm, required | MOM finger geometry | capacitor |
| `res_min_width` (125) | Nm, required | | resistor |
| `res_value_tol_ppm` (126) | Count, required | resistor value tolerance | resistor |
| `tie_max_dist_nm` (127) | Nm, sourced | diffusion to tap reach (LU.2/LU.2.1/LU.3 15 µm) | mosfet `taps_in_reach` |
| `licon_poly_enc` (129), `polycon_to_pdiff_spacing` (130), `licon_to_gate_spacing` (131) | Nm | contact geometry floors | mosfet |
| `lod_kvth0_n_mv_um` (132), `lod_kvth0_p_mv_um` (134) | Real, sourced | BSIM4 LOD `KVTH0`, mV·µm, from the installed models | library annotation |
| `avt_n_mv_um` (136), `avt_p_mv_um` (138) | Real, sourced | Pelgrom A_VT, mV·µm, from Monte Carlo of the installed models | library annotation |
| `abeta_p_pct_um` (140) | Real, sourced | A_β, %·µm (pfet only) | library annotation |
| `svt_a_uv2_per_um2` (142), `svt_b_uv2` (144), `svt_uv_per_um` (146) | Real, sourced PROXY | distance-mismatch gradient S_VT fit, µV²/µm², µV², µV/µm (Schaper and Linnenbank 130 nm, not sky130) | library annotation |
| `em_current_density_source` (148) | Text | provenance of the deck's EM rules | documentation |
| `em_derating` (149) | Table, sourced | `{t_ref_k: 363.15, ea_ev: null, n: null}`: EM rating temperature; Ea and n fall back to 0.9 eV and 1.1 flagged assumed | `pdk.rs em_limit` (`:646-688`, `:724-729`) |
| `antenna_source` (151), `density_source` (156) | Text | provenance | documentation |
| `vt_tc_uv_per_k` (152), `vt_tc_uv_per_k_p` (154) | Real, sourced | |dV_T/dT|, µV/K, nfet and pfet, from ngspice sweeps | library annotation |
| `inapplicable_rules` (157-160) | List | deck rules (`licon.10`, `varac.4`) that must not set a generator dimension | `pdk.rs widest_on` (`:1027`) |
| `bjts` (162-170) | Table | recipes: model, aliases, `bjt_emitter_w`/`bjt_emitter_l` nm per fixed-geometry BJT | `pdk.rs recipe` |
| `resistors` (171-225) | Table | `default` and recipes: model, aliases, `layers` role→layer (`rpoly`, `rpm`, `npc`, `res_implant`), `res_contact_w/h`, `res_head`, `res_sheet_mohm`, `res_head_mohm_um`, `res_dl_nm`, `res_dw_nm`, `res_head_dw_nm`, `res_narrow_permille`, `res_knee_nm` (the model card's R(W, L) law), `k_a_pct_um` (A_R), `tc_ppm_per_k` | `pdk.rs recipe` (`:1437-1452`), `Overlay` |
| `capacitors` (226-259) | Table | `default` and recipes: `layers` (`bottom`, `plate`, `top_contact`, `top`, `strap`, `bottom_contact`), `c_area_af_um2`, `c_perim_af_um`, `c_dw_nm` | recipe, Overlay |
| `npn_isolation` (260) | Count | vertical NPN needs deep n-well ring | bjt |
| `waivers` (262-264) | Table | rule id → reason; the rule is removed from the run and reported as waived (`multiple_drivers`) | `checker.rs:62-70` |

### 4.3 Every key in `pdks/asap7.json`

Same registry; differences from sky130 (`Philis/pdks/asap7.json`): `layers` maps `diff` and `tap` both to `ACTIVE`, `poly→GATE`, `fin→FIN`, `sdt→SDT`, `lisd→LISD`, `lig→LIG`, `gcut→GCUT`, `li→M1`, `licon→V0`, `mcon→V1`, `met1→M2`, `nwell→WELL`, `nsdm→NSELECT`, `psdm→PSELECT`, `routing_metals` M1..M9, `routing_vias` V1..V8 (`:26-63`); `min_finger_width` 27 (`:9`), `via_enclosure` 5 (`:10`), `dummy_max_l_nm` null (`:11`), `sd_width` 34 (`:64`), `max_finger_width` 2700 (`:66`), `well_enclosure` 27 (`:69`), `device_gap` 108 (`:70`), `min_guard_ring_width` 54 (`:81`), `guard_licon_pitch` 54 (`:82`), `substrate_kind` null (`:85`), `tie_max_dist_nm` 30000 from the deck's `ACTIVE.LUP.1` rows (`:102-103`), `finfet_note` (`:104`). No mismatch, LOD, EM, cap-density, VT-TC, recipe, `placement_space` or `waivers` keys. The tier arrays and textbook BJT constants are the same text as sky130 (`:5-25`).

### 4.4 Deck edits (`# PHILIS:` lines in the vendored decks)

sky130 (`Philis/pdks/decks/sky130.deck`): an `npn` marker layer (`:84-85`); `ar.met3.1` sidewall 845 nm (`:476-477`); three `lu_*` derived layers and `LU.2`, `LU.2.1`, `LU.3` as `tap_distance(.., ..) <= 15um` (`:529-537`); `device bjt pnp` and `device bjt npn` recognisers (`:607-613`); `pex rbody_po`, `rbody_high_po`, `rbody_xhigh_po` rows carrying the resistor sheet resistances (`:656-664`). asap7 (`Philis/pdks/decks/asap7.deck`): front-end connectivity (`SD` layer, `connect via SDT [SD, LISD]`, V0 vias) and eight three-terminal `device mos` recognisers for nmos/pmos × rvt/lvt/slvt/sram, with MOL resistance left unknown (`:313-349`, `:446-448`). None of these is in GPurify HEAD: `grep LU.2|npn|device bjt` in `GPurify/pdks/sky130.deck` finds nothing, and `generic_finfet.deck` still recognises no transistor (`GPurify/docs/limitations.md:232-237`). The vendored files differ from HEAD by 323 (sky130), 295 (gf180mcu), 210 (ihp_sg13g2) and 220 (asap7 vs generic_finfet) lines, almost all upstream progress since the pin.

### 4.5 How `Pdk` and `Process` merge deck and sidecar

`Pdk::load(text, sidecar)` (`Philis/backend/verify/src/pdk.rs:135-214`):

1. `parse_deck(text, nm_grid(), &mut strings)` into `Pdk::deck: Deck` (the parsed form, with `RuleTable`) (`:136-138`).
2. `parse_roles(sidecar)`: `cell.layers` string entries → `roles: Vec<(role, layer name)>`, `routing_metals`, `routing_vias`, and every integer or boolean `cell.*` key → `scalars: Vec<(key, i32)>` (`:1550-1594`).
3. `layers: Vec<(name, LayerId)>` is the deck's layer table row for row, so a Philis `LayerId` is the deck `LayerId` (`:145-150`, validated `:238-251`).
4. `rules: Vec<(String, i32)>`, searched last-match-wins (`:1176-1188`), is built widest source first: every deck rule's `limit` or `min_one_side` length under its rule id (`scalar_rules`, `:1263-1277`); then `<layer>_min_width` and `<layer>_min_spacing` per deck layer from `min_width` (`:956-971`, which also reads `cut_size`) and `min_spacing` (`:949`); then `<role>_min_width`, `<role>_min_spacing`, `<role>_min_area` (side of the square) per role (`:196-209`); then the sidecar scalars (`:210`). `Process::rule(name, default)` reads that list in reverse, then `min_gate_l` from the deck's channel rules, then the default (`:1179-1188`).
5. `grid` is the `off_grid` rule's `pitch`, else 1 (`:1597-1610`).
6. `validate()` refuses: a role naming no deck layer, a broken identity map, a missing `REQUIRED_ROLES` entry (`diff, poly, li, licon, mcon, met1, nwell, nsdm, psdm, tap`, `:1280-1282`), registry violations, a bad `em_derating` shape, an empty routing stack, a cut count not one less than the metals, a cut the deck's `via_connects` does not join between its metals, a routing metal without `min_width`/`min_spacing`, a cut without `min_width` (`:218-346`).

`Process::layer(role)`: `cell.layers` map first, then positional `met{n}`/`via{n}` into the routing stack, then a deck layer of that literal name (`:1165-1175`). `Overlay{pdk, recipe}` puts a recipe's `layers` and `rules` before the `Pdk`'s, and resolves a controlled-but-unset resistor or capacitor role to no layer (`:1457-1526`, roles `:1299-1304`). `em_limit`: an `electromigration` rule's Black parameters, else `cell.em_derating` with the Lienig fallback flagged assumed (`:646-688`). `Checker::new` re-parses `pdk.source` into its own `StrTable`, removes `density`/`density_cmp` rules when asked, removes `cell.waivers` rules, defers `global_density` and any window wider than the block as chip level, and hand-builds `Loaded` (`checker.rs:49-108`, `:335-361`). `set_intent` converts Philis `Intent{supplies, currents, max_drop_mv}` to GPurify's intent JSON and `parse_intent` (`checker.rs:131-172`).

### 4.6 What backend/verify re-derives that GPurify already provides

| Philis (`pdk.rs`) | GPurify equivalent | Note |
|---|---|---|
| `min_width` (`:956-971`), `rule_limit_nm` (`:1134`), `widest_on("min_width")` (`:1025`) | `CompiledDeck::min_width` (`query.rs:195`), `cut_classes` (`:125`) | Philis adds the "carved-from" derivation walk `drawn_from`/`reaches` (`:978-997`) that GPurify has no query for |
| `min_spacing`, `route_spacing`, `wide_spacing`, `via_array_spacing`, `sized_spacing`, `Process::space` (`:529-604`, `:1099-1131`, `:1209-1220`) | `required_spacing(layer, &SpacingQuery)` (`query.rs:253`), `via_array_spacing` (`:313`), `cut_spacing` (`:330`) | Philis's spacing-table reading takes each row's widest cell (`:562-581`); GPurify's threshold function is the checker's own |
| `min_area` (`:516-527`), `pad_for` (`:499`) | `min_area` (`query.rs:206`) | |
| `max_enclosure_of`, `cut_enclosure_pair`, `Process::enclosure`/`endcap` (`:479-495`, `:1311-1325`, `:1232-1239`) | `enclosure(inner, outer) -> Enclosure` (`query.rs:262`) | |
| `density_rules` (`:801-821`), `defer_density_wider_than` (`checker.rs:335`) | `density(layer)` (`query.rs:436`) plus `erc.density_cmp` table (`antenna.rs:510-529`) | |
| `antenna_rule`, `antenna_max_ratio`, `antenna_diode_credit` (`:745-794`) | `antenna(collector)` (`query.rs:469`) | GPurify's row carries `measure` and `diode`; the cumulative flag Philis derives is implicit in the collector list |
| `em_limit` (`:646-688`) | `em_limit(layer)` (`query.rs:514`) and `erc.electromigration` (`electrical.rs:209-231`) | GPurify's getter reads `em_current_density` only; Black parameters are in the typed table |
| `fet_voltage_limits` (`:694-718`) | `fet_limits(model)` (`query.rs:528`) | |
| `p2p_max_ohm` (`:734`) | `erc.p2p_resistance.max_resistance` (`electrical.rs:61-64`) | |
| `deck_grid` (`:1597-1610`) | `drc_on` + `Rule::OffGrid` | no getter |
| `min_channel` (`:1058-1084`) | `drc_on` + `Rule::EdgeWidth`, `MinWidth` on the gate layer | no getter; GPurify's `lvs range` statement is the proper carrier (unused by every deck) |
| `pex_f32`, `pex_row`, `pex_col`, `cut_ohm`, `device_sheet_ohm` (`:843-907`, `:1198-1208`) | `ProcessStack` columns (`mod.rs:1279-1300`) | same data; Philis adds the one-conductor inference |
| `wire_af_per_um`, `lateral_af_per_um`, `overlap_af_um2` (`:912-945`) | the analytical extractor's closed forms (`GPurify/crates/extract/src/lib.rs:51-57`; `analytical::extract_into`, `node_rects` `analytical.rs:162`, `:220`) | duplicated physics with a `ponytail:` ceiling (`:925-927`) |
| `scalar_rules` (`:1263-1277`) | `drc_on(layer)`, `CompiledDeck::rules()` | |
| `label_gds`, `layer_gds`, `gv_layer_by_name` (`:356-438`) | `LayerTable::stream_of`, `id`, `Connectivity::label_*` | thin wrappers, fine |
| `model_markers`, `mos_overlay` (`:1354-1431`) | `LayerTable::op`/`operands` | a derivation walk GPurify does not offer |
| `Checker::new` re-parsing `pdk.source` (`checker.rs:49-52`) | `CompiledDeck` is `Clone` (`check/deck/mod.rs:60`) | second parse per session |
| `domain_of` (`checker.rs:420-434`) | `drc::ruleset::KINDS`, `erc::KINDS` | same lists, fine |
| the vendored decks and `GPURIFY_DIR` lookup (`decks.rs`, `build.rs`, `pdk.rs:105-127`) | `gpurify::pdks::*` constants (`src/pdks.rs:31-51`) | the plan's P0 deliverable for Philis (`docs/deck-library-plan.md:289`) |

## 5. Gaps for an analog layout recipe

Shipped-deck inventory used below (`grep` counts over `GPurify/pdks/*.deck`): sky130 has 5 `em_current_density`, 42 antenna rows (11 `antenna_electrical`), 13 `density`, 4 `density_cmp`, 3 `gate_oxide`, 2 `drain_source`, 1 `well_bias`, 1 `floating_well`, 1 `ir_drop`, 25 `pex` rows, 30 devices, `connect global psub`, no `tap_distance`/`missing_tie`; gf180mcu has 4 `tap_distance`, 2 `missing_tie`, 4 `conditional_tie`, 5 `guard_ring*`, 2 `latchup_victim`, 4 `em_current_density`; ihp_sg13g2 has 2 `missing_tie`, 7 `em_current_density`, 5 `space_table`; generic_finfet has 2 `missing_tie`, `pitch(FIN, vertical) == 27nm`, `pitch(GATE, horizontal) == 54nm`, 17 `pex` rows with `area_cap 0` and no dielectric, no devices. No shipped deck has `temperature`, `corner`, `lvs lde`, `lvs range`, `pex_device`, `pex_substrate`, `electromigration`, `matching`, or a `device` value (`sheet`, `area_cap`, `inductance`).

| Number | Deck today | Could GPurify carry it | Sidecar still needed |
|---|---|---|---|
| Matching coefficients A_VT, A_β, A_R, S_VT gradient, BJT k_A | No statement; `matching(group; tolerance)` checks a layout, carries no coefficient (`kinds.rs:1707-1712`) | Yes, with a new statement and new `Dim`s (mV·µm, %·µm, µV/µm are not in `UNITS`, `kinds.rs:106-151`), e.g. a `device` value or an `lvs`-like `mismatch "model" avt .. abeta ..` row | Yes until then (`avt_*`, `abeta_*`, `svt_*`, `bjt_ka_pct_um`, resistor `k_a_pct_um`) |
| Layout-dependent-effect reach: LOD `KVTH0`, SA/SB, WPE `SCREF`, moat extension, dummy reach | `lvs lde "model" od L max len well L scref len` carries the layers, the saturation distance and `SCREF` (`mod.rs:1157-1169`); no deck uses it; no `KVTH0` or clearance distances | `lvs lde` already carries `max` and `scref`; `KVTH0` and per-class clearances need new keys | Yes for `lod_kvth0_*`, `wpe_clearance_nm`, `lod_moat_ext_nm`, `dummy_reach_nm`, `gate_ext_extra_nm` (Hastings policy, not process data) |
| EM current limits per layer and via | `em_current_density(layers..; max_density, max_current_per_cut, narrow)` in 3 decks; `electromigration` with `blech_limit`, `reference_temperature`, `activation_energy`, `current_exponent` exists but no deck states it; `em_rms`/`em_peak` tables; query `em_limit` (`query.rs:514`) | Yes: write the shipped rows as `electromigration` with Black parameters, or add them to the `em_current_density` row | Only `em_derating` (T_ref, Ea, n) while decks lack them; `metal_family` (al/cu) has no deck home |
| Sheet resistance, via resistance, temperature coefficients | `pex ... sheet` per conductor and per cut (`mod.rs:1284-1292`), `tc1`/`tc2` with `temperature`; resistor body sheet via `device resistor ... sheet` or a `pex` row on the body marker; no shipped deck states `temperature`, so no TC | Yes, in the language today; the shipped decks must add `temperature`/`tc1`/`tc2` and `device ... sheet` | Resistor head resistance, `dl`/`dw` biases, knee width (`res_head_mohm_um`, `res_dl_nm`, `res_dw_nm`, `res_narrow_permille`, `res_knee_nm`) have no deck home |
| Capacitance per µm and per µm² | `pex area_cap`/`fringe_cap` to substrate, `dielectric` for lateral and interlayer coupling (`extract/lib.rs:51-57`); `device capacitor ... area_cap fringe_cap` for a MIM; `pex_device` gate fringe; IHP fringe is 0 and generic_finfet has none (`limitations.md:228-237`) | Yes in the language; sky130's `capm`/`cap2m` recognisers lack `area_cap`/`fringe_cap` values and could carry `camimc`/`cpmimc` | `c_dw_nm` plate bias, `gate_cap_af_um2` (C_ox), `cap_density_ff_um2` until the recogniser carries it |
| Well and tap rules | `tap_distance` (DRC), `missing_tie`, `conditional_tie`, `tie_area`, `floating_well`, `well_bias`, `latchup_victim`, `supply_short`; gf180mcu carries them, sky130 upstream does not (Philis added LU.2/2.1/3), generic_finfet carries `missing_tie` 30 µm | Yes: upstream the sky130 `LU.*` rows (sourced from `sky130A.tech:4163-4170` per the Philis edit) | `tie_max_dist_nm` becomes redundant once the rule is in the deck; well depths, epi thickness, `substrate_kind` are `unread`/annotation only and have no deck home (`pex_substrate` carries a resistivity profile, not depths) |
| Guard-ring construction | `guard_ring(target, ring; max_space, min_width, pad, near, within, clear)` and `guard_ring_bias` check a ring; nothing states how to build one | `min_width` of the ring layer and contact pitch could be read from the deck's `min_width`/`cut_spacing` of the ring and cut layers; a recipe statement would be new | `min_guard_ring_width`, `guard_licon_pitch`, `npn_isolation` |
| Minimum device sizes | `lvs range "model" w >= .. <= ..` is the carrier (`mod.rs:1188-1198`; `docs/deck.md:930-938`); no shipped deck states one; sizes are implied by `min_width`/`edge_width` rules Philis reverse-reads (`pdk.rs:1058-1084`) | Yes, by writing `lvs range` rows per model from each PDK's device documentation | `min_finger_width`, `sd_width`, `max_finger_width` until then; `max_finger_width` is a design bound |
| Fin and gate pitch, FinFET grids | `pitch(FIN, vertical) == 27nm`, `pitch(GATE, horizontal) == 54nm` in generic_finfet (`pdks/generic_finfet.deck:448`, `:454`); track grids are "needs: tracks" (`:420-439`) | `Rule::ExactPitch` is reachable via `drc_on`; a `pitch` getter would be one function | None for pitch; `finfet_note` is prose |
| Routing stack order | Not stated; implied by `connect conductors` order, `via_connects` chain and `ProcessStack::height_nm` | Yes: derive metals bottom-up from `height_nm` and the cut that joins each adjacent pair; a `stack` getter would be new | `routing_metals`/`routing_vias` until derived |
| Layer roles (`diff`, `poly`, `tap`, `nsdm` ..) | Not stated; device recognisers name gate/source/drain layers per model, taps are whatever `connect via` lands on | Partly: a mos recogniser's terminal 0 is the gate layer, terminals 1-2 the diffusion; wells, implants and taps need a naming convention or a new statement | Yes (`cell.layers` roles) |
| Two-layer placement spacings on derived layers | All present as `min_spacing_diff` rules on derived layers (each `placement_space` entry cites its deck rule and line, `sky130.json:85-111`) | Yes today: `drc_on(derived)` + `Rule::MinSpacingDiff` + `LayerTable::operands` to map back to drawn layers | None once Philis reads them through the typed rules |
| Temperature coefficients of V_T, V_BE | None | A `device` value (`vt_tc`) would be new | Yes |
| Antenna diodes | `antenna ... diode: layer` and `antenna_electrical ... diode_credit/diode_bonus` in sky130 (11 rows) | Carried | None; Philis reads `antenna_diode_credit` from the rule |
| Density windows | `density`, `density_cmp`, `global_density` carried; chip-level deferral is a Philis policy | Carried | None |
| Process corners, substrate network, TSV, fill treatment | Language exists (`corner`, `pex_substrate`, `pex_tsv`, `pex_fill`); no shipped deck states them | Decks need the numbers | None once stated |

## 6. Portability

Adding a fifth PDK to GPurify is one hand-written text deck in `pdks/`, authored from the foundry's rule manual with every number's source cited in its header and its omissions listed at its end (`GPurify/pdks/sky130.deck:1-45`; `generic_finfet.deck:1-32`, `:377-439`; `README.md:142-155`). Nothing is generated from a foundry file; the KLayout decks are used only as oracles. The repository then expects, held by tests: a `gpurify::pdks` constant and compile function for the new stem, `ALL` kept sorted (`GPurify/src/pdks.rs:44-51`; `tests/pdk_decks.rs:119-163`), a directory-discovered parse-and-build check (`tests/pdk_decks.rs:25-60`), a Rust twin of the deck built with `DeckBuilder` statement by statement (`GPurify/crates/check/tests/deck/twins/sky130.rs:1-6`), a rule-by-rule cross-check ledger against the foundry's KLayout deck pinned in `tests/foundry/pdks.tsv` (`docs/xcheck/sky130.tsv:1-11`; `docs/testing-plan.md:52`), and a limitations entry (`docs/limitations.md:150-237`). Deck lengths must sit on the deck's own `grid` (5 nm for the three foundry decks, 1 nm for generic_finfet) and compile on any layout grid that grid divides (`src/pdks.rs:9-12`). The vendoring per foundry that exists today is Philis's, not GPurify's: four copies under `Philis/pdks/decks/` compiled in by `include_str!` (`decks.rs:9-14`), each line 1 naming the GPurify commit it was copied from and checked against `Cargo.lock` (`pdk.rs:1684-1690`; `build.rs:13-17`), with `# PHILIS:` edits above each change (`decks.rs:4-6`).

`generic_finfet.deck` is ASAP7 r1p7 under a generic name: its header cites the ASAP7 DRM, layer map, the MEJ paper and OpenROAD's `setRC.tcl` (`GPurify/pdks/generic_finfet.deck:2-21`), `gpurify::pdks::GENERIC_FINFET` documents it as "ASAP7 ... standing in for a generic FinFET process" (`src/pdks.rs:40-42`), and its DRM rule ids are kept (`:440-513`). It recognises no transistors and extracts no capacitance because the DRM publishes no MOL resistance and no capacitance (`docs/limitations.md:232-237`; `generic_finfet.deck:410-418`). Philis's `asap7.deck` is that file renamed, with the front end connected and eight MOS recognisers added so LVS extracts transistors, MOL resistance still unknown (`Philis/pdks/decks/asap7.deck:313-349`, `:446-448`; `asap7.json:104`). Since the pin, upstream added 61 ASAP7 rules (`docs/testing-plan.md:547`), which Philis's copy lacks (67 `rule` lines against 100 upstream).

For Philis 2.0 to make GPurify its only PDK input: read the deck through `gpurify::pdks::*` or `compile` (no vendored copies), hold a `CompiledDeck` and query it through section 2.4, express the `# PHILIS:` edits as `DeckBuilder::from_deck` overrides or upstream them (the plan's P3, done upstream as D4, `docs/testing-plan.md:525`), and keep a sidecar only for the rows of section 5 marked "sidecar still needed". The Rust-API shape Philis compiled against has moved (section 3.7), so the first step is re-pinning and rewriting `checker.rs` against `CompiledDeck` and the current `run_checks`.
