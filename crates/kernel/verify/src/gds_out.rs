//! Drawn shapes and pin labels → a GDSII library, for GPurify and the
//! foundry tools: every shape a `BOUNDARY` on its deck layer's stream pair,
//! every pin a `TEXT` on the deck's label pair for its conductor
//! ([`Pdk::label_gds`]). Shapes a [`NamedCell`] claims go in a structure of
//! that name, placed once at the origin by `top` (the deck's `in_cells`
//! selections read the name); the rest, and every pin, in `top`. 1 dbu = 1 nm.
//!
//! Contents:
//! - `pub fn gds(shapes: &[Shape], pins: &[LabeledPin], cells: &[NamedCell], pdk: &Pdk, top: &str) -> Result<Vec<u8>, String>`

use analog::base::{NamedCell, Shape};

use crate::geom::LabeledPin;
use crate::pdk::Pdk;

/// `shapes` and `pins` as a GDSII stream whose top structure is `top`.
///
/// # Errors
/// A shape on a layer the deck computes rather than draws (it has no stream
/// pair), a pin whose conductor no deck label layer names (either would drop
/// geometry or a net name), or a cell whose shape range runs past `shapes`
/// or overlaps another cell's.
pub fn gds(shapes: &[Shape], pins: &[LabeledPin], cells: &[NamedCell], pdk: &Pdk, top: &str) -> Result<Vec<u8>, String> {
    let table = pdk.layer_gds();
    // Each shape's structure: 0 the top, `k + 1` the `k`th distinct cell name.
    let mut names: Vec<&str> = Vec::new();
    let mut cell_of = vec![0usize; shapes.len()];
    for c in cells {
        let k = names.iter().position(|n| *n == c.name).unwrap_or_else(|| {
            names.push(&c.name);
            names.len() - 1
        });
        let rows = cell_of
            .get_mut(c.shapes.start as usize..c.shapes.end as usize)
            .ok_or_else(|| format!("cell {}: shapes {:?} past {}", c.name, c.shapes, shapes.len()))?;
        if rows.iter().any(|&t| t != 0 && t != k + 1) {
            return Err(format!("cell {}: shapes {:?} already in another cell", c.name, c.shapes));
        }
        rows.fill(k + 1);
    }
    let mut bodies = vec![Vec::with_capacity(48 * shapes.len()); names.len() + 1];
    for (s, &cell) in shapes.iter().zip(&cell_of) {
        let pair = table.get(s.layer.0 as usize).copied().filter(|&p| p != (0, 0));
        let (layer, datatype) = pair.ok_or_else(|| format!("shape on deck layer {} has no GDS pair (a computed layer)", s.layer.0))?;
        let r = s.rect;
        let ring = [(r.x, r.y), (r.x + r.w, r.y), (r.x + r.w, r.y + r.h), (r.x, r.y + r.h), (r.x, r.y)];
        let out = &mut bodies[cell];
        record(out, 0x0800, &[]);
        record(out, 0x0D02, &layer.to_be_bytes());
        record(out, 0x0E02, &datatype.to_be_bytes());
        record(out, 0x1003, &xy(&ring));
        record(out, 0x1100, &[]);
    }
    for p in pins {
        let (layer, texttype) = pdk.label_gds(p.layer).ok_or_else(|| format!("pin {}: no label layer names deck layer {}", p.name, p.layer))?;
        let out = &mut bodies[0];
        record(out, 0x0C00, &[]);
        record(out, 0x0D02, &layer.to_be_bytes());
        record(out, 0x1602, &texttype.to_be_bytes());
        record(out, 0x1003, &xy(&[(p.x, p.y)]));
        string(out, 0x1906, &p.name);
        record(out, 0x1100, &[]);
    }
    for name in &names {
        let out = &mut bodies[0];
        record(out, 0x0A00, &[]);
        string(out, 0x1206, name);
        record(out, 0x1003, &xy(&[(0, 0)]));
        record(out, 0x1100, &[]);
    }
    let mut out = Vec::with_capacity(bodies.iter().map(Vec::len).sum::<usize>() + 256);
    record(&mut out, 0x0002, &600_i16.to_be_bytes());
    record(&mut out, 0x0102, &[0; 24]);
    string(&mut out, 0x0206, "PHILIS");
    let mut units = real(1e-3).to_vec();
    units.extend(real(1e-9));
    record(&mut out, 0x0305, &units);
    for (name, body) in names.iter().copied().chain([top]).zip(bodies[1..].iter().chain([&bodies[0]])) {
        record(&mut out, 0x0502, &[0; 24]);
        string(&mut out, 0x0606, name);
        out.extend_from_slice(body);
        record(&mut out, 0x0700, &[]);
    }
    record(&mut out, 0x0400, &[]);
    Ok(out)
}

// ---- private ----

fn xy(points: &[(i32, i32)]) -> Vec<u8> {
    points.iter().flat_map(|&(x, y)| x.to_be_bytes().into_iter().chain(y.to_be_bytes())).collect()
}

fn record(out: &mut Vec<u8>, tag: u16, payload: &[u8]) {
    let len = u16::try_from(payload.len() + 4).expect("a record under 64 KiB");
    out.extend(len.to_be_bytes());
    out.extend(tag.to_be_bytes());
    out.extend(payload);
}

/// An ASCII record, NUL-padded to an even length.
fn string(out: &mut Vec<u8>, tag: u16, text: &str) {
    let mut bytes = text.as_bytes()[..text.len().min(65_000)].to_vec();
    if bytes.len() % 2 == 1 {
        bytes.push(0);
    }
    record(out, tag, &bytes);
}

/// An eight-byte GDSII real: sign, base-16 exponent in excess 64, 56-bit fraction.
fn real(value: f64) -> [u8; 8] {
    debug_assert!(value > 0.0, "GDS units are positive");
    let (mut v, mut exponent) = (value, 64_i32);
    while v >= 1.0 {
        v /= 16.0;
        exponent += 1;
    }
    while v < 1.0 / 16.0 {
        v *= 16.0;
        exponent -= 1;
    }
    let fraction = ((v * 72_057_594_037_927_936.0).round() as u64).min((1 << 56) - 1);
    let mut out = fraction.to_be_bytes();
    out[0] = exponent as u8;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use analog::base::{LayerId, NamedCell, Rect};

    /// Records walked by length: one BOUNDARY per shape on the deck's
    /// met1 pair, one TEXT per pin on met1's label pair carrying its name.
    #[test]
    fn writes_one_boundary_per_shape_and_one_text_per_pin() {
        let pdk = Pdk::builtin("sky130").unwrap();
        let met1 = pdk.layers.iter().find(|(n, _)| n == "met1").unwrap().1;
        let shapes = [Shape { layer: met1, rect: Rect { x: 0, y: 0, w: 500, h: 140 } }; 2];
        let pins = [LabeledPin { name: "d".into(), layer: met1.0, x: 10, y: 10 }];
        let bytes = gds(&shapes, &pins, &[], &pdk, "top").unwrap();
        let (mut at, mut tags, mut layers, mut names) = (0, Vec::new(), Vec::new(), Vec::new());
        while at < bytes.len() {
            let len = u16::from_be_bytes([bytes[at], bytes[at + 1]]) as usize;
            let tag = u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]);
            let body = &bytes[at + 4..at + len];
            match tag {
                0x0D02 | 0x0E02 | 0x1602 => layers.push(u16::from_be_bytes([body[0], body[1]])),
                0x1906 => names.push(String::from_utf8_lossy(body).trim_end_matches('\0').to_string()),
                _ => {}
            }
            tags.push(tag);
            at += len;
        }
        assert_eq!(tags.iter().filter(|&&t| t == 0x0800).count(), 2);
        assert_eq!(tags.iter().filter(|&&t| t == 0x0C00).count(), 1);
        assert_eq!(&layers[..4], &[68, 20, 68, 20], "met1 drawing is 68/20");
        assert_eq!(&layers[4..], &[68, 5], "met1 label is 68/5");
        assert_eq!(names, ["d"]);
        assert_eq!(tags.last(), Some(&0x0400));
    }

    /// Oracle: GDSII stream format 6 (BGNSTR 0x0502, STRNAME 0x0606, SREF
    /// 0x0A00, SNAME 0x1206): a named cell's shape sits in a structure of
    /// that name, written before `top`, and `top` places it once by SREF.
    #[test]
    fn a_named_cell_is_its_own_structure_placed_by_top() {
        let pdk = Pdk::builtin("sky130").unwrap();
        let met1 = pdk.layers.iter().find(|(n, _)| n == "met1").unwrap().1;
        let shapes = [Shape { layer: met1, rect: Rect { x: 0, y: 0, w: 500, h: 140 } }; 2];
        let cells = [NamedCell { name: "pnp_cell".into(), shapes: 1..2 }];
        let bytes = gds(&shapes, &[], &cells, &pdk, "top").unwrap();
        let mut at = 0;
        let mut seen = Vec::new();
        while at < bytes.len() {
            let len = u16::from_be_bytes([bytes[at], bytes[at + 1]]) as usize;
            let tag = u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]);
            let body = String::from_utf8_lossy(&bytes[at + 4..at + len]).trim_end_matches('\0').to_string();
            match tag {
                0x0606 => seen.push(format!("str {body}")),
                0x0800 => seen.push("boundary".into()),
                0x1206 => seen.push(format!("sref {body}")),
                _ => {}
            }
            at += len;
        }
        assert_eq!(seen, ["str pnp_cell", "boundary", "str top", "boundary", "sref pnp_cell"]);
        let overlapping = [NamedCell { name: "a".into(), shapes: 0..2 }, NamedCell { name: "b".into(), shapes: 1..2 }];
        assert!(gds(&shapes, &[], &overlapping, &pdk, "top").is_err(), "a shape in two cells");
        assert!(gds(&shapes, &[], &[NamedCell { name: "a".into(), shapes: 1..3 }], &pdk, "top").is_err(), "a range past the shapes");
    }

    /// The negative control: a shape on a computed layer is refused.
    #[test]
    fn a_shape_on_a_computed_layer_is_refused() {
        let pdk = Pdk::builtin("sky130").unwrap();
        let derived = (0..pdk.deck.layers.len() as u16).find(|&i| pdk.deck.layers.is_derived(crate::pdk::GvLayerId(i))).unwrap();
        let shapes = [Shape { layer: LayerId(derived), rect: Rect { x: 0, y: 0, w: 10, h: 10 } }];
        assert!(gds(&shapes, &[], &[], &pdk, "top").is_err());
    }
}
