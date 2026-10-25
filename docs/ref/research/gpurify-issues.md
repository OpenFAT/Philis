# GPurify upstream issues behind Philis's deck overrides

**Fixed upstream in GPurify `11b5578` (branch `philis-fixes`), with regression tests in `tests/supply_and_enclosure_regressions.rs`; Philis pins it and its two workarounds are gone.**

Two behaviours of GPurify that contradict GPurify's own text, each worked
around in `crates/kernel/verify/src/overrides.rs`. Read on 2026-10-25.

Sources. GPurify at the pinned rev `687c8bc8bf9468095a4104a05075cf44e762532b`
(`~/.cargo/git/checkouts/gpurify-92358cf428c48214/687c8bc`). Both behaviours
are unchanged at the owner's checkout HEAD `40ff66c`. KLayout DRC reference,
`layer.enclosing` and `layer.enclosed`
(<https://www.klayout.de/doc-qt5/about/drc_ref_layer.html>). sky130's own
KLayout deck, `sky130A/libs.tech/klayout/drc/sky130A_mr.drc` (volare
`fa87f8f`).

Neither issue is filed. Filing is the GPurify owner's call.

Tests: `crates/kernel/verify/tests/gpurify_upstream.rs`. Each asserts the
current result and names the fixed one in a comment. With the patches below
applied to a scratch copy of 687c8bc (wired in by cargo `[patch]`), the two
bug tests fail as intended and the other two still pass:

| Test | 687c8bc | Patched |
|---|---|---|
| `sky130_lvtn10_flags_lvtn_off_every_nwell` | pass | fails: no finding |
| `enclosure_measures_unhosted_inner_as_zero` | pass | pass (engine unchanged) |
| `soft_connection_keeps_global_substrate` | pass | fails: no finding |
| `soft_connection_flags_straps_joined_only_by_substrate` | pass | pass (control) |

When GPurify ships a fix: flip the assert, delete the override, and update
the edited `pdks/gpurify/sky130.deck` that `overrides_equal_the_edited_decks`
compares against.

## 1. sky130 lvtn.10 flags every LVT nfet

### Findings

- **The shipped deck contradicts itself.** `pdks/gpurify/sky130.deck:297-301` says
  lvtn.10 selects its wells as the foundry PVL does (`[S11]
  Include/lvtn.drc.pvl` lines 40-44) and that "an lvtn off every such well is
  not measured". The rule it writes is `enclosure(lvtn, nwell_novar,
  noncoincident) >= 380nm`, which measures every lvtn.
- **The engine measures an unhosted inner as 0, by design.** The rule docs
  say so for `Rule::NoncoincidentEnclosure` (`crates/check/src/drc/ruleset.rs`
  ~664, "one no host covers measures 0") and for `Rule::MinEnclosure` (~605,
  "An inner polygon no host covers measures 0"). `crates/check/src/drc/rules/overlay.rs`
  states it in its module doc: "an unhosted inner shape measures zero, never
  skipped". `overlay::enclosure` runs both kinds (and `asymmetric_enclosure`)
  through one loop, so `min_enclosure` behaves the same. Only `full_width`
  skips an unhosted inner, leaving it to `inside(..)`.
- **So an LVT nfet fails lvtn.10.** Its lvtn sits on p-substrate, off every
  n-well, and reads 0 against 380 nm.
- **KLayout does not report an unhosted inner.** Both kinds cite KLayout
  `enclosing` (catalog DRC-040, DRC-054). Its reference says: "Locations,
  where other_layer extends outside layer will not be reported as errors.
  Such regions can be detected by not_inside or a boolean "not" operation."
  sky130's own KLayout deck follows that split. It checks `ct.4` as
  `mconnotace.not(li)` and `m1.4` as a separate `enclosing`.
- **The engine choice is defensible; the deck line is the bug.** Folding
  "must be inside" into `min_enclosure` is documented and other rows may rely
  on it. The smallest fix that makes the deck say what its comment says is to
  select the lvtn on the deck line, as `npc.5` and `capm.4` already select
  their inner layers.

Philis's workaround replaces the row with
`noncoincident_enclosure(lvtn.interacting(nwell_novar), nwell_novar, 380nm)`.

### Draft issue

> **sky130 lvtn.10 flags every lvtn off an n-well, against its own comment**
>
> `pdks/gpurify/sky130.deck` lines 297-299 say lvtn.10 selects the wells as the
> foundry PVL does, so "an lvtn off every such well is not measured". Line 301
> writes `enclosure(lvtn, nwell_novar, noncoincident) >= 380nm`, and
> `noncoincident_enclosure` measures an inner shape no host covers as 0
> (documented on `Rule::NoncoincidentEnclosure`). Every LVT nfet's lvtn sits
> off the n-well, so every LVT nfet fails lvtn.10 with 0 < 380 nm.
>
> Repro: the shipped sky130 deck, one `lvtn` rectangle 2 um x 2 um, nothing
> else. DRC reports `lvtn.10` measured 0.
>
> Expected: no lvtn.10 finding. Proposed fix: select the lvtn on the deck line
> (and its Rust twin), as npc.5 and capm.4 already do. The engine is
> unchanged.
>
> Related: the enclosure kinds cite KLayout `enclosing`, which does not report
> an inner outside the outer ("Locations, where other_layer extends outside
> layer will not be reported as errors"). GPurify's choice to measure it as 0
> is documented, so this is a note for the catalog's TX:KL column, not a
> request to change it.

### Proposed patch (against 687c8bc, not applied)

```diff
diff --git a/crates/check/tests/deck/twins/sky130.rs b/crates/check/tests/deck/twins/sky130.rs
index 60d3d50..1ef952f 100644
--- a/crates/check/tests/deck/twins/sky130.rs
+++ b/crates/check/tests/deck/twins/sky130.rs
@@ -682,7 +682,11 @@ pub fn build(strings: &mut StrTable) -> Result<CompiledDeck, BuildError> {
     );
     d.rule(
         "lvtn.10",
-        noncoincident_enclosure(l["lvtn"], l["nwell_novar"], nm(380)),
+        noncoincident_enclosure(
+            l["lvtn"].interacting(l["nwell_novar"]),
+            l["nwell_novar"],
+            nm(380),
+        ),
     );
     d.rule(
         "lvtn.12",
diff --git a/pdks/gpurify/sky130.deck b/pdks/gpurify/sky130.deck
index ac25c6f..174fe3e 100644
--- a/pdks/gpurify/sky130.deck
+++ b/pdks/gpurify/sky130.deck
@@ -298,7 +298,7 @@ rule lvtn.9 space(lvtn, hvtp) >= 380nm
 # (exclude coincident edges)", as [S11] Include/lvtn.drc.pvl lines 40-44
 # selects the wells; an lvtn off every such well is not measured.
 layer nwell_novar = nwell.outside(var_channel)
-rule lvtn.10 enclosure(lvtn, nwell_novar, noncoincident) >= 380nm
+rule lvtn.10 enclosure(lvtn.interacting(nwell_novar), nwell_novar, noncoincident) >= 380nm
 rule lvtn.12 space(lvtn, nwell_ce) >= 380nm
 rule lvtn.13 area(lvtn) >= 0.265um2
 rule lvtn.14 hole_area(lvtn) >= 0.265um2
```

## 2. soft_connection keeps the substrate in the hard connectivity

### Findings

- **The docs say a global layer is dropped.** `SoftConnectionTable`
  (`crates/check/src/erc/rules/supply.rs` ~59) says the row re-extracts the
  nets "with the soft layers taken out: no soft conductor or global layer".
- **The code drops only a global layer the row lists.**
  `check_soft_connection` (~563) builds the hard connectivity with
  `global: all.global.iter().copied().filter(|&l| !soft(l))`, so `psub` stays.
  The doc sentence could be read as "no soft conductor or soft global layer",
  but that reading makes the rule flag every substrate-tied supply, below.
- **So every substrate-tied supply reads as split.** A supply strap reaches
  the substrate through a soft p-tap (`connect via psub_tie [ptap, psub]`).
  With p-tap out and psub kept, the strap is one piece and the substrate,
  global, is another. The net has two hard pieces and is flagged, though
  nothing but the strap holds it together.
- **The shipped sky130 deck hits this.** Its row is
  `soft_connection(ntap, ptap, nwell, pwell_iso)` and its connectivity has
  `connect global psub`.
- **The fix keeps real soft connections.** Two straps of one supply joined
  only through their taps and the substrate are still two pieces once psub
  is out, and are still flagged. The control test checks this.

Philis's workaround lists psub as soft:
`soft_connection(ntap, ptap, nwell, pwell_iso, psub)`.

### Draft issue

> **soft_connection keeps global layers, so every substrate-tied supply is flagged**
>
> `SoftConnectionTable`'s docs say the hard connectivity has "no soft
> conductor or global layer". `check_soft_connection` drops a global layer
> only when the row lists it as soft. A supply strap tied to the substrate
> through a soft tap then reads as two pieces, the strap and the global
> substrate, and is flagged although it is one strap.
>
> Repro deck: `psub` global, `ptap` and `met1` conductors, `via tie [ptap,
> psub]`, `via licon [ptap, met1]`, `rule soft soft_connection(ptap)`. Draw
> one met1 strap on one ptap with a licon and a tie, over a psub rectangle.
> ERC reports `soft` with count 2.
>
> Expected: no finding. The substrate is a resistive body like the taps.
> Two straps joined only through taps and substrate should still be flagged,
> and are with the patch. Proposed fix: treat every global layer as soft in
> the hard connectivity, and say so in the docs. The shipped sky130 row then
> needs no `psub`.

### Proposed patch (against 687c8bc, not applied)

```diff
diff --git a/crates/check/src/erc/rules/supply.rs b/crates/check/src/erc/rules/supply.rs
index 08b27f4..fdf5798 100644
--- a/crates/check/src/erc/rules/supply.rs
+++ b/crates/check/src/erc/rules/supply.rs
@@ -57,12 +57,14 @@ pub struct SupplyShortTable {
 ///
 /// The listed layers are soft: wells, substrate ties and the like, which
 /// conduct too poorly to carry a signal or a supply. A row cuts the nets
-/// again from the deck's connectivity with the soft layers taken out: no soft
-/// conductor or global layer, and no via with a soft cut or a soft layer on
-/// either side. Every net with a polygon on a soft layer whose other
-/// polygons fall into more than one of those pieces is flagged: its pieces
-/// are connected only through the soft layers. Shapes join only as the deck
-/// declares, so a strap drawn over a piece without a via joins nothing.
+/// again from the deck's connectivity with the soft layers and every global
+/// layer taken out: no soft conductor, no global layer (listed or not: the
+/// substrate is resistive too), and no via with a soft cut or a soft or
+/// global layer on either side. Every net with a polygon on a soft layer
+/// whose other polygons fall into more than one of those pieces is flagged:
+/// its pieces are connected only through the soft layers. Shapes join only
+/// as the deck declares, so a strap drawn over a piece without a via joins
+/// nothing.
 /// Needs no design intent.
 ///
 /// Catalog: LVS-003, ERC-G5 in the [check catalog]. Defined by
@@ -559,9 +561,10 @@ pub fn check_soft_connection(
         }
         let soft_nets = nets_marked(&scratch.net_marks, 1);
 
-        // What is left without the resistive bodies, joined only as declared.
-        let soft = |layer: LayerId| is_soft[layer.idx()];
+        // What is left without the resistive bodies, joined only as declared:
+        // a global layer (the substrate) is one of them, listed or not.
         let all = &table.connectivity;
+        let soft = |layer: LayerId| is_soft[layer.idx()] || all.global.contains(&layer);
         let (via_cut, via_connects) = all
             .via_cut
             .iter()
```

## GPurify's own suite on the patched copy

Both patches together, on a scratch copy of 687c8bc,
`cargo test --workspace --no-fail-fast`: 1542 passed, 0 failed, 40 ignored.
The patch's twin edit is needed: without it,
`builder::every_shipped_deck_equals_its_rust_twin` fails, since the twin must
state what the deck text does.
