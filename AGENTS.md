# AGENTS.md

Philis 2.0: analog layout from a SPICE netlist, by recipe. Constraints are
recognised by the annotator, realized by recipes on a Substrate2 fork, routed
by construction first, signed off, and printable as Rust. `README.md` is the
overview; `docs/` is the contract.

## Before writing anything

- `docs/DECISIONS.md` is the design tree. A change that contradicts an entry is
  a new entry that names the one it reverses, never an edit. An open question
  blocks the code that depends on it; the doc marks which.
- `docs/ARCHITECTURE.md` says which crate a feature goes in and what it must
  not depend on. State the change radius before writing a feature.
- `GLOSSARY.md` fixes the words. "Fallback" does not exist on the recipe path:
  a recipe degrades along its ladder or the engine refuses.

## Skills to load first

Load with the Skill tool before writing or reviewing code (other agents: read
`~/.claude/skills/<name>/SKILL.md`).

| skill | use it for here |
|---|---|
| `codebase-rules` | the build order and the rulings where skills disagree |
| `data-oriented-design` | `intent`, `recipe_calls`, `claims`, `tiles`, `routes`: the six questions in writing, SoA, narrowest widths, handles |
| `codebase-clean-code` | crates as owners (`docs/ARCHITECTURE.md`), one spine per mode, request tables across owners; every new seam names its rung and a drill it passes |
| `rust` | types, memory, `zero-cost.md`: a rung-5 abstraction enters a hot loop (claim scoring, routing negotiation) only with an `asm_probe` verdict; explicit SIMD is `fearless_simd` |
| `compiler-construction` | the netlist frontend and the annotator: rows plus `extra_data`, interning, coded diagnostics, goldens |
| `test-plan`, `tdd` | `docs/TESTING.md`: every test names its oracle; red before green; a feature lands with its negative control |
| `assertions` | contracts, capacities (65 535 devices and nets), bounded loops |
| `gui-architecture` | `frontend/visualizer`: a pure function of the tables, headless frames |
| `simd-loops`, `branchless` | hot loops in claim scoring and routing; `branchless` first |
| `doc-comments` | every `pub` item |
| `domain-modeling` | a new term goes in `GLOSSARY.md` the moment it is settled |

## How this codebase is written

Follow the `codebase-rules` skill: it carries the build order and the rulings
where the engineering skills disagree. Read it before starting a new module.

Project specifics that override it:

- Rust stable, edition 2024. `crates/kernel/substrate3` is a vendored fork of
  substrate2; its upstream rev is in its README and moves only on purpose.
- Proof PDKs are sky130 and asap7: a recipe is not done until its conformance
  test passes on both.
- Every recipe output is scored by `kernel/analog` rules and signed off by
  GPurify; a claim is never trusted by construction.
- Data layout beats code brevity; `ponytail` decides how little code to write,
  never what shape the data takes.
