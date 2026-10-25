# Philis 2.0

**Analog layout from a SPICE netlist, by recipe.** Philis 2.0 recognises the
circuit's analog constraints, realizes each one with a known layout technique
written as a Substrate2-style generator, routes the constrained nets by
construction, signs off the GDS, and can print the whole layout as a Rust
generator program. Free-form place-and-route remains available as a mode the
user selects.

Status: design phase. The architecture is decided and recorded; the code moves
from [Philis 1.0](https://github.com/UW-ASIC/Philis) milestone by milestone.

## Read in this order

1. [`docs/DECISIONS.md`](docs/DECISIONS.md): the design tree, every decision
   with its question, options, recommendation and answer. The source of truth.
2. [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md): crates as owners, the two
   spines, the tables, the seams, the invariants, how to add a feature, the
   migration from 1.0.
3. [`docs/TRAITS.md`](docs/TRAITS.md): the trait system the Substrate2 fork adds.
4. [`docs/RECIPES.md`](docs/RECIPES.md): the initial recipe catalog, from the
   textbooks and the 1.0 constraint model.
5. [`docs/TESTING.md`](docs/TESTING.md): oracles, layers, gates, exit criteria.
6. [`docs/ROADMAP.md`](docs/ROADMAP.md): milestones and the acceptance set.
7. [`GLOSSARY.md`](GLOSSARY.md): the words.

Reference library: [`docs/ref/`](docs/ref/) holds the textbook and paper
digests (`textbooks/`), the requirement-family notes (`Notes/`), and the layout
fundamentals audit from 1.0. The PDFs sit beside them in the working tree and
are not committed.

## Layout

```
crates/
  kernel/   analog (theory core), substrate3 (Substrate2 fork, PDKs, device blocks, set recipes)
  backend/  placement (floorplan recipes, free-form placer), routing (routing recipes, router), post-processing (compaction)
  frontend/ annotator (netlist → intent), engine (spines, call tree, printer, CLI)
docs/       the documents above
```

## License

MIT. `crates/kernel/substrate3` is a fork of
[ucb-substrate/substrate2](https://github.com/ucb-substrate/substrate2) (BSD-3);
its notice is kept.
