//! The built-in PDKs' decks and their dumps (Q61), compiled in: a moved or
//! packaged `philis` loads and signs off without this repository on disk,
//! and a build script reads a `Pdk` with no `gpurify` binary. Each deck is
//! GPurify's shipped deck at the rev `flake.lock` pins (line 1 names it),
//! Philis's edits made in place under a `# PHILIS:` line; each dump is
//! `gpurify deck <deck> --format json` of it (`pdks/gpurify/dump.sh`).
//!
//! Contents:
//! - `pub(crate) const DECKS: &[(&str, &str, &str)]`
//! - `pub(crate) fn vendored(pdk: &str) -> Option<(&'static str, &'static str)>`

/// `(PDK name, deck text, deck dump)`.
pub(crate) const DECKS: &[(&str, &str, &str)] = &[
    ("sky130", include_str!("../../../../pdks/gpurify/sky130.deck"), include_str!("../../../../pdks/gpurify/sky130.deck.json")),
    ("gf180mcu", include_str!("../../../../pdks/gpurify/gf180mcu.deck"), include_str!("../../../../pdks/gpurify/gf180mcu.deck.json")),
    ("ihp_sg13g2", include_str!("../../../../pdks/gpurify/ihp_sg13g2.deck"), include_str!("../../../../pdks/gpurify/ihp_sg13g2.deck.json")),
    ("asap7", include_str!("../../../../pdks/gpurify/asap7.deck"), include_str!("../../../../pdks/gpurify/asap7.deck.json")),
];

/// `(deck text, deck dump)` of built-in PDK `pdk`, `None` for a name not
/// built in.
pub(crate) fn vendored(pdk: &str) -> Option<(&'static str, &'static str)> {
    DECKS.iter().find(|d| d.0 == pdk).map(|d| (d.1, d.2))
}

#[cfg(test)]
mod tests {
    /// The FinFET deck is ASAP7 and is named so; the old name is gone.
    #[test]
    fn asap7_is_built_in_under_its_own_name() {
        assert!(crate::Pdk::builtin("asap7").is_ok());
        assert!(crate::Pdk::builtin("generic_finfet").is_err());
        assert!(super::vendored("asap7").is_some_and(|(t, _)| t.contains("pdks/asap7.deck")));
        assert!(super::vendored("generic_finfet").is_none());
    }

    /// Every statement a vendored deck changes, adds or drops against
    /// GPurify's shipped deck sits under a `# PHILIS:` comment: an edit is
    /// never silent. Comment lines may change freely.
    ///
    /// Oracle: GPurify's own decks at the pinned rev (`$GPURIFY_SRC/pdks`,
    /// the flake input the dev shell exports).
    #[test]
    fn every_deck_edit_is_marked_philis() {
        let src = std::env::var("GPURIFY_SRC").expect("GPURIFY_SRC is unset: run under `nix develop`");
        for (pdk, text, _) in super::DECKS {
            let upstream = std::fs::read_to_string(format!("{src}/pdks/{pdk}.deck")).unwrap_or_else(|e| panic!("{pdk}: {e}"));
            let unmarked = unmarked_edits(&upstream, text);
            assert!(unmarked.is_empty(), "{pdk}.deck: edits with no # PHILIS: comment above them: {unmarked:#?}");
        }
    }

    /// Each checked-in dump is what the pinned `gpurify` dumps from its deck
    /// now: a deck edit or a re-pin without `pdks/gpurify/dump.sh` fails.
    ///
    /// Oracle: `gpurify deck <deck> --format json`, the binary the flake pins.
    #[test]
    fn dumps_are_fresh() {
        let gpurify = crate::foundry::tool("gpurify").unwrap_or_else(|r| panic!("{r}"));
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../pdks/gpurify");
        for (pdk, _, dump) in super::DECKS {
            let o =
                std::process::Command::new(&gpurify).args(["deck", &format!("{pdk}.deck"), "--format", "json"]).current_dir(dir).output().unwrap();
            assert!(o.status.success(), "{pdk}: {}", String::from_utf8_lossy(&o.stderr));
            assert!(o.stdout == dump.as_bytes(), "{pdk}.deck.json is stale: run `nix develop -c pdks/gpurify/dump.sh`");
        }
    }

    /// Oracle: hand-made three-line decks, one marked and one unmarked edit.
    #[test]
    fn an_unmarked_edit_is_caught() {
        let up = "layer a = gds(1, 0)\nrule a.1 width(a) >= 1nm\nrule a.2 space(a) >= 1nm\n";
        let marked = "layer a = gds(1, 0)\n# PHILIS: stricter\nrule a.1 width(a) >= 2nm\nrule a.2 space(a) >= 1nm\n";
        assert!(unmarked_edits(up, marked).is_empty());
        let silent = "layer a = gds(1, 0)\nrule a.1 width(a) >= 1nm\n\nrule a.2 space(a) >= 2nm\n";
        assert_eq!(unmarked_edits(up, silent).len(), 2, "the changed a.2 and the line it replaced");
        assert_eq!(unmarked_edits(up, "layer a = gds(1, 0)\nrule a.2 space(a) >= 1nm\n").len(), 1, "a silent deletion");
    }

    /// The statement lines (not comments, not blank) `vendored` adds or
    /// drops against `upstream` that no `# PHILIS:` comment covers. An
    /// added line is covered when a `# PHILIS:` line sits above it with no
    /// blank line between; a dropped one when such a line sits in the
    /// vendored deck where it was. ponytail: an O(n·m) LCS over ~1000
    /// statements; a marked edit also covers an unmarked one directly below
    /// it, the ceiling of "no blank line between".
    fn unmarked_edits(upstream: &str, vendored: &str) -> Vec<String> {
        let code = |t: &str| -> Vec<(usize, String)> {
            t.lines()
                .enumerate()
                .filter(|(_, l)| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
                .map(|(i, l)| (i, l.trim().to_string()))
                .collect()
        };
        let (a, b) = (code(upstream), code(vendored));
        let lines: Vec<&str> = vendored.lines().collect();
        let covered = |at: usize| lines[..=at].iter().rev().take_while(|l| !l.trim().is_empty()).any(|l| l.trim_start().starts_with("# PHILIS:"));
        let mut lcs = vec![vec![0u16; b.len() + 1]; a.len() + 1];
        for i in (0..a.len()).rev() {
            for j in (0..b.len()).rev() {
                lcs[i][j] = if a[i].1 == b[j].1 { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
            }
        }
        // `last`: the vendored line of the last statement both decks share.
        let (mut i, mut j, mut last, mut out) = (0, 0, 0, Vec::new());
        while i < a.len() || j < b.len() {
            if i < a.len() && j < b.len() && a[i].1 == b[j].1 {
                last = b[j].0;
                (i, j) = (i + 1, j + 1);
            } else if j < b.len() && (i == a.len() || lcs[i][j + 1] >= lcs[i + 1][j]) {
                if !covered(b[j].0) {
                    out.push(format!("+ {}", b[j].1));
                }
                j += 1;
            } else {
                // Dropped: covered by a `# PHILIS:` since the last shared statement.
                let to = b.get(j).map_or(lines.len(), |x| x.0 + 1);
                if !lines[last..to].iter().any(|l| l.trim_start().starts_with("# PHILIS:")) {
                    out.push(format!("- {}", a[i].1));
                }
                i += 1;
            }
        }
        out
    }
}
