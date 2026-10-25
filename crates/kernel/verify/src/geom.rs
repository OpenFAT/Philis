//! Drawn geometry → gdsverify's `GeometryStore` + `Provenance`. A Philis
//! `LayerId` is the deck's own layer id, so shapes feed the store directly.

use analog::base::{NamedCell, Shape};
use gdsverify::geom::derive::merge_into;
use gdsverify::geom::ops::Point;
use gdsverify::geom::rects::decompose_into;
use gdsverify::geom::view::validate_layer_into;
use gdsverify::geom::{Dbu, GeometryStore, GeometryStoreBuilder, LayerId as GvLayerId, ValidatedLayer};
use gdsverify::ingest::deck::Deck;
use gdsverify::ingest::layout::gds::Library;
use gdsverify::ingest::layout::{derive_layers, UnknownLayers};
use gdsverify::ingest::{Provenance, StrTable};

/// A named pin label. The point `(x, y)` (nm) must lie on a shape of `layer`,
/// the pin's drawing layer; the label goes on whichever layer the deck's
/// `connectivity.labels` binds to the conductor built from it ([`label_layer`]),
/// and that binding names the extracted net (and makes it a port).
#[derive(Clone, Debug)]
pub struct LabeledPin {
    pub name: String,
    /// Deck layer id (`analog::base::LayerId.0`).
    pub layer: u16,
    pub x: i32,
    pub y: i32,
}

/// Build the store the engine reads, in gdsverify's own load order: push →
/// `finish` (sort by layer) → derive marker layers → bind labels.
///
/// Shapes on a layer in `merge` are pushed as their union (KLayout merged
/// semantics): PEX sums area and fringe per stored polygon, so two drawn rects
/// that overlap would count the overlap twice and their shared edge as fringe.
/// A merged polygon with holes goes in as its disjoint rect decomposition
/// (a holed ring is not one store polygon); its slab edges inside the ring
/// then count as fringe. Other layers go in one polygon per shape.
///
/// `cells` names the shapes drawn inside a cell, for the deck's `in_cells`
/// selections. GPurify resolves those only while flattening a hierarchical
/// GDS (its in-memory `derive_layers` tags no shape), so a non-empty `cells`
/// loads through one written in memory: each name a structure placed once at
/// the origin under a top cell. A merged layer's union spans cells and stays
/// in the top cell.
///
/// # Errors
/// A merge refused by gdsverify's boolean, a derived-layer failure, or a pin
/// label on no shape of its conductor (fail closed: a net silently losing its
/// name would be an untraceable LVS miss). With `cells`: a range past
/// `shapes` or overlapping another name's, a layer or label layer no GDS pair
/// names alone, or GPurify refusing the written stream.
pub fn build_store(
    shapes: &[Shape],
    pins: &[LabeledPin],
    deck: &Deck,
    strings: &mut StrTable,
    merge: &[u16],
    cells: &[NamedCell],
) -> Result<(GeometryStore, Provenance), String> {
    let nm = |v: i32| Dbu::new_unchecked(i64::from(v));
    // Each shape's structure: 0 the top cell, `k + 1` the `k`th distinct name.
    let mut names: Vec<&str> = Vec::new();
    let mut cell_of = vec![0u16; if cells.is_empty() { 0 } else { shapes.len() }];
    for c in cells {
        let k = names.iter().position(|n| *n == c.name).unwrap_or_else(|| {
            names.push(&c.name);
            names.len() - 1
        });
        let tag = u16::try_from(k + 1).map_err(|_| "more than 65535 named cells".to_string())?;
        let range = c.shapes.start as usize..c.shapes.end as usize;
        let rows = cell_of.get_mut(range).ok_or_else(|| format!("cell {}: shapes {:?} past {}", c.name, c.shapes, shapes.len()))?;
        if rows.iter().any(|&t| t != 0 && t != tag) {
            return Err(format!("cell {}: shapes {:?} already in another cell", c.name, c.shapes));
        }
        rows.fill(tag);
    }
    let mut sink = if cells.is_empty() {
        Sink::Store(GeometryStoreBuilder::with_capacity(shapes.len(), shapes.len() * 4))
    } else {
        Sink::Gds(vec![Vec::new(); names.len() + 1])
    };
    let mut merged: Vec<(u16, GeometryStoreBuilder)> = Vec::new();
    for (i, s) in shapes.iter().enumerate() {
        let (x0, y0, x1, y1) = (nm(s.rect.x), nm(s.rect.y), nm(s.rect.x + s.rect.w), nm(s.rect.y + s.rect.h));
        let (xs, ys) = ([x0, x1, x1, x0], [y0, y0, y1, y1]);
        if merge.contains(&s.layer.0) {
            let i = merged.iter().position(|(l, _)| *l == s.layer.0).unwrap_or_else(|| {
                merged.push((s.layer.0, GeometryStoreBuilder::default()));
                merged.len() - 1
            });
            merged[i].1.push(GvLayerId(0), &xs, &ys);
        } else {
            sink.push(deck, s.layer.0, cell_of.get(i).copied().unwrap_or(0), &xs, &ys)?;
        }
    }
    let (mut raw, mut union, mut rects, mut start) = (ValidatedLayer::default(), ValidatedLayer::default(), Vec::new(), Vec::new());
    for (l, b) in merged {
        let (tmp, _) = b.finish(1);
        validate_layer_into(&tmp, GvLayerId(0), &mut raw).map_err(|e| format!("merge layer {l}: {e}"))?;
        merge_into(&raw, &mut union).map_err(|e| format!("merge layer {l}: {e}"))?;
        decompose_into(&union, &mut rects, &mut start);
        for i in 0..union.len() {
            let poly = union.get(i as u32);
            let (xs, ys) = poly.outer().coords();
            // ponytail: a GDS XY record holds 8191 points; a longer outline
            // goes in as its rects, as a holed one does (slab edges then read
            // as fringe). Only on the `cells` path.
            if poly.holes().next().is_none() && (matches!(sink, Sink::Store(_)) || xs.len() < 8000) {
                sink.push(deck, l, 0, xs, ys)?;
            } else {
                for r in &rects[start[i] as usize..start[i + 1] as usize] {
                    sink.push(deck, l, 0, &[r.xlo, r.xhi, r.xhi, r.xlo], &[r.ylo, r.ylo, r.yhi, r.yhi])?;
                }
            }
        }
    }
    let label_of =
        |p: &LabeledPin| label_layer(deck, p.layer).ok_or_else(|| format!("pin {}: no label layer names a conductor of layer {}", p.name, p.layer));
    let (store, mut provenance) = match sink {
        Sink::Store(builder) => {
            let mut provenance = Provenance::default();
            for p in pins {
                let name = strings.intern(&p.name);
                provenance.place_label(Point { x: nm(p.x), y: nm(p.y) }, GvLayerId(label_of(p)?), name);
            }
            let (mut store, _) = builder.finish(deck.layers.len());
            derive_layers(&mut store, deck, &provenance, strings);
            (store, provenance)
        }
        Sink::Gds(mut structures) => {
            for p in pins {
                gds::text(&mut structures[0], stream(deck, label_of(p)?)?, p.x, p.y, &p.name);
            }
            let bytes = gds::library(&names, &structures);
            let lib = Library::parse(&bytes, strings).map_err(|e| format!("cell hierarchy: {e}"))?;
            lib.flatten(deck, strings, UnknownLayers::Reject).map_err(|e| format!("cell hierarchy: {e}"))?
        }
    };
    provenance.resolve_labels(&store, &deck.connectivity).map_err(|e| format!("pin label: {e}"))?;
    Ok((store, provenance))
}

/// Where [`build_store`] puts polygons: the store directly, or one GDS
/// element list per structure (0 the top cell).
enum Sink {
    Store(GeometryStoreBuilder),
    Gds(Vec<Vec<u8>>),
}

impl Sink {
    fn push(&mut self, deck: &Deck, layer: u16, cell: u16, xs: &[Dbu], ys: &[Dbu]) -> Result<(), String> {
        match self {
            Self::Store(b) => {
                b.push(GvLayerId(layer), xs, ys);
            }
            Self::Gds(structures) => gds::boundary(&mut structures[usize::from(cell)], stream(deck, layer)?, xs, ys),
        }
        Ok(())
    }
}

/// The GDS pair that reads back as deck layer `layer`, and only it.
fn stream(deck: &Deck, layer: u16) -> Result<(u16, u16), String> {
    let pair = deck.layers.stream_of(GvLayerId(layer));
    match deck.layers.of_stream(pair.0, pair.1) {
        Some(id) if id == GvLayerId(layer) => Ok(pair),
        _ => Err(format!("cell hierarchy: layer {layer} has no GDS pair of its own")),
    }
}

/// The few GDSII records [`build_store`] writes (Calma stream format).
mod gds {
    use super::Dbu;

    const HEADER: u16 = 0x0002;
    const BGNLIB: u16 = 0x0102;
    const LIBNAME: u16 = 0x0206;
    const UNITS: u16 = 0x0305;
    const ENDLIB: u16 = 0x0400;
    const BGNSTR: u16 = 0x0502;
    const STRNAME: u16 = 0x0606;
    const ENDSTR: u16 = 0x0700;
    const BOUNDARY: u16 = 0x0800;
    const SREF: u16 = 0x0A00;
    const TEXT: u16 = 0x0C00;
    const LAYER: u16 = 0x0D02;
    const DATATYPE: u16 = 0x0E02;
    const XY: u16 = 0x1003;
    const ENDEL: u16 = 0x1100;
    const SNAME: u16 = 0x1206;
    const TEXTTYPE: u16 = 0x1602;
    const STRING: u16 = 0x1906;

    /// The top structure's name; `#` keeps it off every deck cell pattern's literal.
    const TOP: &str = "#philis_top";

    fn record(out: &mut Vec<u8>, tag: u16, payload: &[u8]) {
        let len = u16::try_from(payload.len() + 4).expect("callers keep a record under 64 KiB");
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(&tag.to_be_bytes());
        out.extend_from_slice(payload);
    }

    fn ascii(out: &mut Vec<u8>, tag: u16, text: &str) {
        let mut b = text.as_bytes().to_vec();
        if b.len() % 2 == 1 {
            b.push(0);
        }
        record(out, tag, &b);
    }

    fn xy(points: impl Iterator<Item = (i64, i64)>) -> Vec<u8> {
        // Coordinates come from i32 rects and their unions: they fit.
        points.flat_map(|(x, y)| [x as i32, y as i32]).flat_map(i32::to_be_bytes).collect()
    }

    /// An eight-byte excess-64 base-16 real; `v > 0`.
    fn real(v: f64) -> [u8; 8] {
        let (mut m, mut e) = (v, 64i32);
        while m >= 1.0 {
            m /= 16.0;
            e += 1;
        }
        while m < 1.0 / 16.0 {
            m *= 16.0;
            e -= 1;
        }
        let mut out = ((m * 72_057_594_037_927_936.0).round() as u64).to_be_bytes();
        out[0] = e as u8;
        out
    }

    /// One closed `BOUNDARY`; `xs.len() < 8191`.
    pub(super) fn boundary(out: &mut Vec<u8>, (layer, datatype): (u16, u16), xs: &[Dbu], ys: &[Dbu]) {
        record(out, BOUNDARY, &[]);
        record(out, LAYER, &layer.to_be_bytes());
        record(out, DATATYPE, &datatype.to_be_bytes());
        let ring = xs.iter().zip(ys).chain(xs.iter().zip(ys).take(1)).map(|(x, y)| (x.raw(), y.raw()));
        record(out, XY, &xy(ring));
        record(out, ENDEL, &[]);
    }

    /// One `TEXT` label at `(x, y)`.
    pub(super) fn text(out: &mut Vec<u8>, (layer, texttype): (u16, u16), x: i32, y: i32, name: &str) {
        record(out, TEXT, &[]);
        record(out, LAYER, &layer.to_be_bytes());
        record(out, TEXTTYPE, &texttype.to_be_bytes());
        record(out, XY, &xy(std::iter::once((i64::from(x), i64::from(y)))));
        ascii(out, STRING, name);
        record(out, ENDEL, &[]);
    }

    /// A library on a 1 nm database unit: structure `k + 1` of `structures`
    /// named `names[k]`, each placed once at the origin by the top cell,
    /// whose own elements are `structures[0]`.
    pub(super) fn library(names: &[&str], structures: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::with_capacity(structures.iter().map(Vec::len).sum::<usize>() + 256);
        record(&mut out, HEADER, &600u16.to_be_bytes());
        record(&mut out, BGNLIB, &[0; 24]);
        ascii(&mut out, LIBNAME, "philis");
        let mut units = real(1e-3).to_vec();
        units.extend_from_slice(&real(1e-9));
        record(&mut out, UNITS, &units);
        let structure = |out: &mut Vec<u8>, name: &str, body: &[u8], refs: &[&str]| {
            record(out, BGNSTR, &[0; 24]);
            ascii(out, STRNAME, name);
            out.extend_from_slice(body);
            for r in refs {
                record(out, SREF, &[]);
                ascii(out, SNAME, r);
                record(out, XY, &xy(std::iter::once((0, 0))));
                record(out, ENDEL, &[]);
            }
            record(out, ENDSTR, &[]);
        };
        for (name, body) in names.iter().zip(&structures[1..]) {
            structure(&mut out, name, body, &[]);
        }
        structure(&mut out, TOP, &structures[0], names);
        record(&mut out, ENDLIB, &[]);
        out
    }
}

/// The label layer that names the conductor drawn on `drawn`: a label row
/// whose conductor is `drawn` or is derived from it (sky130 `li` names
/// `li_c = li not li_rs`; ihp labels only through `metal1_label`). `drawn`
/// itself when it is such a label layer. `None` when no label reaches it.
#[must_use]
pub fn label_layer(deck: &Deck, drawn: u16) -> Option<u16> {
    let c = &deck.connectivity;
    let d = GvLayerId(drawn);
    let rows: Vec<usize> =
        (0..c.label_layer.len()).filter(|&r| c.label_names[r] == d || deck.layers.operands(c.label_names[r]).contains(&d)).collect();
    rows.iter().find(|&&r| c.label_layer[r] == d).or_else(|| rows.first()).map(|&r| c.label_layer[r].0)
}
