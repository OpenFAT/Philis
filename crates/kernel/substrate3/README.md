# substrate3

Fork of [Substrate2](https://github.com/ucb-substrate/substrate2) (Q35), plus
the features Philis adds on top of it.

- `upstream/` is Substrate2 at `01d9600f946437d3bc18806a6517e35001b68251`
  (`substrate` 0.10.3), vendored on 2026-10-25. It is its own Cargo workspace;
  the root workspace excludes it and depends on its crates by path. Removed
  from the vendored copy: `docs/` except `docs/snippets` (a dependency of
  `codegen`), and five
  test fixtures over 2 MB (`libs/psfparser/examples/*.tran.tran`,
  `libs/gds/resources/has_properties.{json,gds}`), whose upstream tests
  therefore do not run here. Every later change to `upstream/` is its own
  commit that says so; the rev above moves only on a deliberate resync.
- `src/` is Philis's: `pdk` (typed PDKs for every deck, `Mos`, `MosTile`),
  `signoff` (in-process GPurify signoff of a Substrate block), `cells` (the
  device blocks, Philis 1.0's generators), and the set recipes as they land.

Substrate2 is BSD-3-Clause, Copyright (c) 2023, Substrate Labs; its
`upstream/LICENSE` and `upstream/COPYRIGHT/` are kept. Philis is MIT.
