# ASAP7 middle-of-line resistance: no citable source

Research for the asap7 PEX refusal. Read on 2026-10-25.

```
engine/pex: refused: layer LISD: no pex row gives this conductor a positive sheet resistance (0)
```

That refusal fails `cargo test -p substrate3 --test pdks` in two tests,
`asap7` and `flavoured_models_extract_as_themselves`. The asap7 overrides
(`crates/kernel/verify/src/overrides.rs`, `asap7()`) connect the front end,
so LVS extracts transistors. Five layers there have no `pex` row:

| Layer | Kind | Row needed |
|---|---|---|
| SD | conductor, derived: diffusion off the gate that SDT touches | sheet, Ω/sq |
| GATE_C | conductor, derived: GATE_UNCUT or LIG, touching LIG | sheet, Ω/sq |
| LISD | conductor | sheet, Ω/sq |
| SDT | via, SD to LISD | Ω per cut |
| V0 | via, LISD to M1 and GATE_C to M1 | Ω per cut |

GPurify reports only the first conductor it meets, which is LISD. The other
four have no row either.

**Verdict.** No public, primary source gives a resistance for any of the
five. No `pex` row was added, so the two tests still fail. Each layer stays
unsourced until one of the inputs listed at the end is found.

## Sources searched

| Source | What it gives | MOL resistance? |
|---|---|---|
| [MEJ] Clark et al., "ASAP7: A 7-nm finFET predictive process design kit", Microelectronics J. 53 (2016) 105-115. Local copy: `docs/mej_paper_asap7.pdf` in the ASAP7 PDK (`~/Documents/University/ECE636/MiniProj1/.schemify/pdks/asap7/<rev>/root/`). | Table 1 widths and pitches: SDT/LISD 25/24 nm, LIG 16/16 nm, VIA0-VIA3 18/18 nm. Section 4.3: "LISD is drawn at 24 nm, the same width as SDT, to lower resistance." Section 4.4: self-forming barriers "reduce via resistance by up to 3×". Tables 3 and 4 are device currents. | No. The paper contains no Ω value at all. |
| [DRM] `asap7_drm.pdf`, `asap7_drm_201207a.pdf`, same PDK. | Design rules only. | No. No resistance anywhere in either revision. |
| [RC] Clark et al., "Design flows and collateral for the ASAP7 7nm FinFET predictive process design kit" (HMC copy, <https://pages.hmc.edu/harris/research/asap7.pdf>), Table II. | M1-M3 1.2 Ω/sq, V1-V3 17.2 Ω, M4-M5 0.77 Ω/sq, V4-V5 11.8 Ω, M6-M7 0.50 Ω/sq, V6-V7 8.2 Ω, M8-M9 0.36 Ω/sq, V8 6.3 Ω. This is the table the existing M1-M9 rows already cite. | No. The table starts at M1 and V1. |
| [Diss] V. Vashishtha, "Predictive Process Design Kits for the 7 nm and 5 nm Technology Nodes", PhD dissertation, Arizona State University, 2019. Public PDF on ASU's CDN: `https://d1rbsgppyrdqq4.cloudfront.net/s3fs-public/c7/220511/Vashishtha_asu_0010E_19583.pdf`. | Section 3.3, Eq. 8: the resistivity model behind the Calibre MIPT file. Section 4.2, p. 51: "local interconnects favor tungsten over copper, and therefore suffer from higher resistivity". Section 7.3, p. 100: "while we computed the resistivity change with CD for copper, we did not do so for the tungsten MOL layers." Fig. 7.3 plots W line resistivity against CD for the ASAP5 study. | No. MOL resistivity appears only as a plot for ASAP5. No MOL layer thickness is given, so no sheet resistance follows. |
| ASAP7 BSIM-CMG cards, `models/hspice/7nm_TT_160803.pm`, every model. | `rdsw = 200`, `rshs = 0`, `rshd = 0`, `rgatemod = 0`, `rgeomod = 0`. | No. `rdsw` is the intrinsic source/drain resistance in Ω·µm inside the device model. The zero `rshs`/`rshd` and `rgatemod = 0` mean the model expects no S/D diffusion or gate resistance from the layout. None of these is a resistance for an SD, GATE_C or LISD shape. |
| QRC tech files, `qrc/qrcTechFile_typ03_unscaledV02` and `qrc/qrcTechFile_typ03_scaled4xV06` in <https://github.com/The-OpenROAD-Project/asap7sc7p5t_28>. | Readable header: conductors `GATE LISD M1 ... M9`. Techgen ran with `-encrypt`. Readable cap tables. `[Analog Mbb] LISD 0.6160` and `[Digital Mbb] LISD 0.4350` with no unit or meaning given. | Not readable. The resistance sections are encrypted, and the two unlabelled LISD numbers cannot be cited as resistance. GATE and LISD resistances exist in this file but cannot be read. |
| OpenROAD-flow-scripts asap7 platform: `setRC.tcl`, `rcx_patterns.rules`, `lef/asap7_tech_1x_*.lef`. | M1-M9 per µm and V1-V9 per cut, the same numbers as [RC]. | No. Nothing below M1 and V1. The tech LEF has V0 with width and spacing only. |
| Calibre PEX rules, `calibre/ruledirs/rcx` in the PDK and in `The-OpenROAD-Project/asap7_pdk_r1p7`. | README only: the files are distributed from asap.asu.edu after registration. | Not public, so not used. |
| [ICCAD17] Vashishtha, Vangala, Clark, "ASAP7 predictive design kit development and cell design technology co-optimization", ICCAD 2017. | | Not read. asap.asu.edu answers 403 to scripted fetches. The dissertation is the same author's later and fuller account. |

### A secondary claim, not used

J.-S. Yoon et al., "Performance, Power, and Area of Standard Cells in Sub 3 nm
Node Using Buried Power Rail", IEEE TED 69(3), 2022, p. 895
(<https://gtcad.gatech.edu/www/papers/ted22-a.pdf>), says:

> Resistance of MOL metals is 523 Ω/μm from ASAP7 [18].

Its [18] is [MEJ], and [MEJ] contains no resistance value. The sentence also
does not say which MOL metal or which width it means. Read as LISD at 24 nm
it would be 523 Ω/µm × 0.024 µm = 12.55 Ω/sq. Read as LIG at 16 nm it would
be 8.37 Ω/sq. The paper's V0 value, 63.5 Ω, cites Moroz et al. (VLSI 2020,
a 1 nm-node power-rail study), not ASAP7. Neither number is used here. The
first is unverifiable against its own citation and ambiguous in layer, and
the second belongs to a different node.

This is also no partial fix. One LISD row would only move the refusal to SD
or GATE_C, which have no source at all.

## What would unblock it

Any one of these would let the rows be written with a source line:

1. **The ASAP7 Calibre xRC/xACT rule deck** from asap.asu.edu, after the
   owner registers. [Diss] Section 3.3 says its MIPT process file carries
   the per-layer resistivity.
2. **A decrypted QRC tech file**, or the `procfile`/ICT it was built from
   (`tech_dir/procfile`, per its own header). Its conductor list already has
   GATE and LISD.
3. **The ICCAD 2017 paper** read by hand in a browser, in case it has a
   MOL resistance table. Nothing found so far says it does.

Until then, SD, GATE_C, LISD, SDT and V0 stay unsourced. asap7 PEX keeps
refusing them, which is the correct outcome under GPurify's
unknown-is-never-zero rule.
