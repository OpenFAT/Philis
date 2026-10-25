//! The built-in PDKs' Philis data, one module per PDK (formerly
//! `pdks/<pdk>.json`).

mod asap7;
mod gf180mcu;
mod ihp_sg13g2;
mod sky130;

/// Every built-in PDK, by the name [`crate::Pdk::builtin`] takes.
pub const NAMES: [&str; 4] = ["sky130", "gf180mcu", "ihp_sg13g2", "asap7"];

/// PDK `name`'s `cell` table, or `None` for a name not built in.
#[must_use]
pub fn cell(name: &str) -> Option<serde_json::Value> {
    Some(match name {
        "sky130" => sky130::cell(),
        "gf180mcu" => gf180mcu::cell(),
        "ihp_sg13g2" => ihp_sg13g2::cell(),
        "asap7" => asap7::cell(),
        _ => return None,
    })
}
