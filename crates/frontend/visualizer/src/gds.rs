//! GDS bytes → the flat polygon and text tables the viewer and SVG export
//! draw: BOUNDARY, BOX, PATH, SREF, AREF and TEXT records (GDSII stream
//! format, Calma rev 6), hierarchy flattened from the top cell; also
//! `analog::base` shapes → the same polygons.
//!
//! Contents:
//! - `pub struct Poly`
//! - `pub struct TextEntry`
//! - `pub(crate) fn bounds(polys: &[Poly]) -> Option<(i32, i32, i32, i32)>`
//! - `pub fn parse_gds(data: &[u8]) -> (Vec<Poly>, Vec<TextEntry>)`
//! - `Poly::key(&self) -> (u16, u16)`
//! - `pub fn polys_from_shapes(shapes: &[analog::base::Shape], gds: &[(u16, u16)]) -> Vec<Poly>`

use std::collections::HashMap;

/// One polygon in top-cell coordinates (database units), the closing vertex
/// not repeated; at least 3 points when it came from a GDS BOUNDARY.
#[derive(Clone)]
pub struct Poly {
    /// GDS layer number.
    pub layer: u16,
    /// GDS datatype: sky130 poly and licon share layer 66 and differ here.
    pub datatype: u16,
    /// Vertices `[x, y]`, in drawing order.
    pub pts: Vec<[i32; 2]>,
}

/// One GDS TEXT label at its anchor, top-cell coordinates.
#[derive(Clone)]
pub struct TextEntry {
    /// Anchor x, database units.
    pub x: i32,
    /// Anchor y, database units.
    pub y: i32,
    /// The label, NUL padding stripped, invalid UTF-8 replaced.
    pub text: String,
}

/// A placement transform: mirror about x, then rotate by `angle` degrees,
/// then translate by `(tx, ty)` (GDS STRANS order).
#[derive(Clone, Copy)]
struct Xform {
    tx: i32,
    ty: i32,
    mirror: bool,
    angle: f64,
}

struct GdsCell {
    polys: Vec<Poly>,
    srefs: Vec<SRef>,
    texts: Vec<TextEntry>,
}

struct SRef {
    name: String,
    x: i32,
    y: i32,
    mirror_x: bool,
    angle_deg: f64,
    cols: u16,
    rows: u16,
    col_vec: (i32, i32),
    row_vec: (i32, i32),
}

/// `(x0, y0, x1, y1)` over every vertex; `None` when there are none.
pub(crate) fn bounds(polys: &[Poly]) -> Option<(i32, i32, i32, i32)> {
    let mut pts = polys.iter().flat_map(|p| p.pts.iter());
    let &[x, y] = pts.next()?;
    Some(pts.fold((x, y, x, y), |(x0, y0, x1, y1), &[x, y]| (x0.min(x), y0.min(y), x1.max(x), y1.max(y))))
}

/// Every polygon and text label of the top cell (the one no other cell
/// references), hierarchy flattened. A stream with no cell gives empty
/// tables; a truncated record ends the parse; references nested deeper than
/// 64 levels are dropped without a diagnostic.
pub fn parse_gds(data: &[u8]) -> (Vec<Poly>, Vec<TextEntry>) {
    let cells = parse_gds_cells(data);
    let top = match find_top_cell(&cells) {
        Some(t) => t,
        None => return (Vec::new(), Vec::new()),
    };
    let mut out = Vec::new();
    let mut texts = Vec::new();
    flatten_cell(&cells, &top, Xform { tx: 0, ty: 0, mirror: false, angle: 0.0 }, &mut out, &mut texts, 0);
    (out, texts)
}

impl Poly {
    pub(crate) fn key(&self) -> (u16, u16) {
        (self.layer, self.datatype)
    }
}

/// `analog::base` shapes as 4-point polygons on their GDS `(layer, datatype)`:
/// `gds[LayerId.0]` (`Pdk::layer_gds`); a missing row keeps the layer id
/// with datatype 0.
#[must_use]
pub fn polys_from_shapes(shapes: &[analog::base::Shape], gds: &[(u16, u16)]) -> Vec<Poly> {
    shapes
        .iter()
        .map(|s| {
            let r = s.rect;
            let (layer, datatype) = gds.get(s.layer.0 as usize).copied().unwrap_or((s.layer.0, 0));
            Poly { layer, datatype, pts: vec![[r.x, r.y], [r.x + r.w, r.y], [r.x + r.w, r.y + r.h], [r.x, r.y + r.h]] }
        })
        .collect()
}

// ---- private ----

fn gds_real(b: &[u8]) -> f64 {
    let sign = if b[0] & 0x80 != 0 { -1.0 } else { 1.0 };
    let exp = (b[0] & 0x7F) as i32 - 64;
    let mut mant: u64 = 0;
    for &byte in &b[1..8] {
        mant = (mant << 8) | byte as u64;
    }
    sign * (mant as f64 / (1u64 << 56) as f64) * 16.0f64.powi(exp)
}

fn read_i16(data: &[u8], off: usize) -> i16 {
    i16::from_be_bytes([data[off], data[off + 1]])
}

fn read_i32(data: &[u8], off: usize) -> i32 {
    i32::from_be_bytes(data[off..off + 4].try_into().unwrap())
}

fn read_string(data: &[u8], off: usize, len: usize) -> String {
    let s = &data[off..off + len];
    let end = s.iter().position(|&b| b == 0).unwrap_or(len);
    String::from_utf8_lossy(&s[..end]).into_owned()
}

fn path_to_polys(layer: u16, datatype: u16, half_w: i32, pts: &[[i32; 2]]) -> Vec<Poly> {
    let mut out = Vec::new();
    for seg in pts.windows(2) {
        let [x0, y0] = seg[0];
        let [x1, y1] = seg[1];
        let hw = half_w;
        let rect = if y0 == y1 {
            let (lx, rx) = (x0.min(x1), x0.max(x1));
            vec![[lx, y0 - hw], [rx, y0 - hw], [rx, y0 + hw], [lx, y0 + hw]]
        } else if x0 == x1 {
            let (by, ty) = (y0.min(y1), y0.max(y1));
            vec![[x0 - hw, by], [x0 + hw, by], [x0 + hw, ty], [x0 - hw, ty]]
        } else {
            let dx = (x1 - x0) as f64;
            let dy = (y1 - y0) as f64;
            let len = (dx * dx + dy * dy).sqrt();
            let nx = (-dy / len * hw as f64) as i32;
            let ny = (dx / len * hw as f64) as i32;
            vec![[x0 + nx, y0 + ny], [x1 + nx, y1 + ny], [x1 - nx, y1 - ny], [x0 - nx, y0 - ny]]
        };
        out.push(Poly { layer, datatype, pts: rect });
    }
    out
}

fn parse_gds_cells(data: &[u8]) -> HashMap<String, GdsCell> {
    let mut cells: HashMap<String, GdsCell> = HashMap::new();
    let mut i = 0;

    let mut cur_name: Option<String> = None;
    let mut cur_polys: Vec<Poly> = Vec::new();
    let mut cur_srefs: Vec<SRef> = Vec::new();
    let mut cur_texts: Vec<TextEntry> = Vec::new();

    // Element state
    let mut in_boundary = false;
    let mut in_path = false;
    let mut in_sref = false;
    let mut in_aref = false;
    let mut in_text = false;
    let mut layer: u16 = 0;
    let mut datatype: u16 = 0;
    let mut path_width: i32 = 0;
    let mut sref_name = String::new();
    let mut sref_mirror = false;
    let mut sref_angle = 0.0f64;
    let mut aref_cols: u16 = 1;
    let mut aref_rows: u16 = 1;
    let mut text_string = String::new();
    let mut text_xy: (i32, i32) = (0, 0);

    while i + 4 <= data.len() {
        let len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
        if len < 4 || i + len > data.len() {
            break;
        }
        let rec = data[i + 2];

        match rec {
            // Structure
            0x05 => {
                // BGNSTR
                cur_polys.clear();
                cur_srefs.clear();
                cur_texts.clear();
            }
            0x06 => {
                // STRNAME
                cur_name = Some(read_string(data, i + 4, len - 4));
            }
            0x07 => {
                // ENDSTR
                if let Some(name) = cur_name.take() {
                    cells.insert(
                        name,
                        GdsCell {
                            polys: std::mem::take(&mut cur_polys),
                            srefs: std::mem::take(&mut cur_srefs),
                            texts: std::mem::take(&mut cur_texts),
                        },
                    );
                }
            }

            // Element begin
            0x08 | 0x2D => {
                in_boundary = true;
                layer = 0;
                datatype = 0;
            }
            0x09 => {
                in_path = true;
                layer = 0;
                datatype = 0;
                path_width = 0;
            }
            0x0A => {
                in_sref = true;
                sref_mirror = false;
                sref_angle = 0.0;
            }
            0x0B => {
                in_aref = true;
                sref_mirror = false;
                sref_angle = 0.0;
                aref_cols = 1;
                aref_rows = 1;
            }
            0x0C => {
                // TEXT
                in_text = true;
                text_string.clear();
                text_xy = (0, 0);
            }

            // LAYER
            0x0D if (in_boundary || in_path) && len >= 6 => {
                layer = read_i16(data, i + 4) as u16;
            }

            // DATATYPE
            0x0E if (in_boundary || in_path) && len >= 6 => {
                datatype = read_i16(data, i + 4) as u16;
            }

            // WIDTH (path)
            0x0F if in_path && len >= 8 => {
                path_width = read_i32(data, i + 4);
            }

            // STRING (text content)
            0x19 if in_text => {
                text_string = read_string(data, i + 4, len - 4);
            }

            // SNAME
            0x12 if in_sref || in_aref => {
                sref_name = read_string(data, i + 4, len - 4);
            }

            // COLROW (aref)
            0x13 if in_aref && len >= 8 => {
                aref_cols = read_i16(data, i + 4) as u16;
                aref_rows = read_i16(data, i + 6) as u16;
            }

            // STRANS
            0x1A if (in_sref || in_aref) && len >= 6 => {
                let flags = u16::from_be_bytes([data[i + 4], data[i + 5]]);
                sref_mirror = flags & 0x8000 != 0;
            }

            // ANGLE
            0x1C if (in_sref || in_aref) && len >= 12 => {
                sref_angle = gds_real(&data[i + 4..i + 12]);
            }

            // XY
            0x10 => {
                let np = (len - 4) / 8;
                let mut pts: Vec<[i32; 2]> = Vec::with_capacity(np);
                for j in 0..np {
                    let o = i + 4 + j * 8;
                    if o + 8 > data.len() {
                        break;
                    }
                    pts.push([read_i32(data, o), read_i32(data, o + 4)]);
                }

                if in_boundary {
                    if pts.len() > 1 && pts.first() == pts.last() {
                        pts.pop();
                    }
                    if pts.len() >= 3 {
                        cur_polys.push(Poly { layer, datatype, pts });
                    }
                } else if in_path {
                    if pts.len() >= 2 {
                        let hw = (path_width / 2).max(1);
                        cur_polys.extend(path_to_polys(layer, datatype, hw, &pts));
                    }
                } else if in_sref && !pts.is_empty() {
                    cur_srefs.push(SRef {
                        name: sref_name.clone(),
                        x: pts[0][0],
                        y: pts[0][1],
                        mirror_x: sref_mirror,
                        angle_deg: sref_angle,
                        cols: 1,
                        rows: 1,
                        col_vec: (0, 0),
                        row_vec: (0, 0),
                    });
                } else if in_aref && pts.len() >= 3 {
                    let (x0, y0) = (pts[0][0], pts[0][1]);
                    cur_srefs.push(SRef {
                        name: sref_name.clone(),
                        x: x0,
                        y: y0,
                        mirror_x: sref_mirror,
                        angle_deg: sref_angle,
                        cols: aref_cols,
                        rows: aref_rows,
                        col_vec: ((pts[1][0] - x0) / aref_cols.max(1) as i32, (pts[1][1] - y0) / aref_cols.max(1) as i32),
                        row_vec: ((pts[2][0] - x0) / aref_rows.max(1) as i32, (pts[2][1] - y0) / aref_rows.max(1) as i32),
                    });
                } else if in_text && !pts.is_empty() {
                    text_xy = (pts[0][0], pts[0][1]);
                }
            }

            // ENDEL
            0x11 => {
                if in_text && !text_string.is_empty() {
                    cur_texts.push(TextEntry { x: text_xy.0, y: text_xy.1, text: text_string.clone() });
                }
                in_boundary = false;
                in_path = false;
                in_sref = false;
                in_aref = false;
                in_text = false;
            }
            _ => {}
        }
        i += len;
    }
    cells
}

fn transform_point(x: i32, y: i32, mirror_x: bool, angle_deg: f64) -> (i32, i32) {
    let (px, mut py) = (x as f64, y as f64);
    if mirror_x {
        py = -py;
    }
    let rad = angle_deg.to_radians();
    let (s, c) = (rad.sin(), rad.cos());
    ((px * c - py * s).round() as i32, (px * s + py * c).round() as i32)
}

fn flatten_cell(cells: &HashMap<String, GdsCell>, name: &str, at: Xform, out: &mut Vec<Poly>, out_texts: &mut Vec<TextEntry>, depth: usize) {
    let Xform { tx, ty, mirror, angle } = at;
    if depth > 64 {
        return;
    }
    let cell = match cells.get(name) {
        Some(c) => c,
        None => return,
    };
    for poly in &cell.polys {
        let pts: Vec<[i32; 2]> = poly
            .pts
            .iter()
            .map(|&[x, y]| {
                let (rx, ry) = transform_point(x, y, mirror, angle);
                [rx + tx, ry + ty]
            })
            .collect();
        out.push(Poly { layer: poly.layer, datatype: poly.datatype, pts });
    }
    for t in &cell.texts {
        let (rx, ry) = transform_point(t.x, t.y, mirror, angle);
        out_texts.push(TextEntry { x: rx + tx, y: ry + ty, text: t.text.clone() });
    }
    for sref in &cell.srefs {
        for row in 0..sref.rows {
            for col in 0..sref.cols {
                let sx = sref.x + col as i32 * sref.col_vec.0 + row as i32 * sref.row_vec.0;
                let sy = sref.y + col as i32 * sref.col_vec.1 + row as i32 * sref.row_vec.1;
                let (px, py) = transform_point(sx, sy, mirror, angle);
                let m = mirror ^ sref.mirror_x;
                let a = if mirror { angle - sref.angle_deg } else { angle + sref.angle_deg };
                flatten_cell(cells, &sref.name, Xform { tx: tx + px, ty: ty + py, mirror: m, angle: a }, out, out_texts, depth + 1);
            }
        }
    }
}

fn find_top_cell(cells: &HashMap<String, GdsCell>) -> Option<String> {
    let mut referenced = std::collections::HashSet::new();
    for cell in cells.values() {
        for sref in &cell.srefs {
            referenced.insert(sref.name.clone());
        }
    }
    cells.keys().find(|k| !referenced.contains(*k)).cloned().or_else(|| cells.keys().next().cloned())
}
