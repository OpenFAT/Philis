//! The hand-edited decks, compiled into the tests: a moved or packaged `philis`
//! loads its PDKs without GPurify's checkout or this repository on disk.
//!
//! The decks themselves are GPurify's, edited in Rust ([`crate::overrides`]).
//! `pdks/*.deck` are 1.0's hand-edited copies (line 1 names the
//! upstream commit, each edit under a `# PHILIS:` line), kept only as the
//! fixtures the overrides are proven equal to.

/// `(deck file name, 1.0's hand-edited deck text)`.
#[cfg(test)]
pub(crate) const DECKS: &[(&str, &str)] = &[
    ("sky130.deck", include_str!("../../../../pdks/gpurify/sky130.deck")),
    ("gf180mcu.deck", include_str!("../../../../pdks/gpurify/gf180mcu.deck")),
    ("ihp_sg13g2.deck", include_str!("../../../../pdks/gpurify/ihp_sg13g2.deck")),
    ("asap7.deck", include_str!("../../../../pdks/gpurify/asap7.deck")),
];

#[cfg(test)]
mod tests {
    /// The FinFET deck is ASAP7 and is named so; the old name is gone.
    #[test]
    fn asap7_is_built_in_under_its_own_name() {
        assert!(crate::Pdk::builtin("asap7").is_ok());
        assert!(crate::Pdk::builtin("generic_finfet").is_err());
    }
}
