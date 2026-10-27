//! FLOOR-GlueRow (Q20): tiles left to right in the caller's order. On a
//! planar deck each sits as close as the shapes facing its neighbours allow,
//! on the cut lattice. On a fin deck every tile's gates join one gate
//! lattice and the breaks between tiles carry field gates, so the deck's
//! exact pitches hold across tile boundaries (ASAP7 GATE.S.1, FIN.S.1).

use analog::base::{LayerId, Macro, Orient, Process, Rect, Shape};

use super::spacing::{clearance, oriented, shifted, Spacing};
use super::{up, Floorplan, Refusal};

/// Place `tiles` in one row (row 0, no axis) and return the floorplan with
/// the fill it needs, world frame (empty off a fin deck).
///
/// Planar (`process` has no `fin` layer): tile `k` starts where its shapes
/// clear every placed tile's facing shapes by their layer pair's `spacing`
/// (wells included), rounded up onto `lattice`.
///
/// Fin deck: each tile's actives start the deck's horizontal active spacing
/// (`horizontal_space("diff")`, ASAP7 ACTIVE.S.2A 92 nm) past the previous
/// tile's, rounded up so its gates land on tile 0's gate lattice (pitch
/// `width + space` of `poly`, GATE.S.1 54 nm); actives bottom-align, then
/// shift up onto tile 0's fin lattice (FIN.S.1 27 nm). The fill is one field
/// gate per empty pitch slot between neighbours, spanning the height both
/// neighbours' gates share: an exact pitch reads only the nearest gate that
/// overlaps across, so the band links the two tiles' gate columns whatever
/// their heights (tiles of different fin counts);
/// the tiles' end gates (PODE) keep it the deck's gate-to-field spacing off
/// their actives (GATE.ACTIVE.S.4), as within a cell. Neighbours that both
/// draw a well are bridged by one (the break is narrower than the well
/// spacing, ASAP7 WELL.S.2). The facing shapes' `spacing` (the well's own
/// aside) can only push a tile further, a whole gate pitch at a time;
/// `lattice` is unused: the gate pitch replaces it.
///
/// # Errors
/// [`Refusal::GateSpan`]: on a fin deck, neighbours whose gates share no
/// height once their actives align.
pub fn glue(tiles: &[Macro], process: &dyn Process, spacing: &Spacing, lattice: i32) -> Result<(Floorplan, Macro), Refusal> {
    let n = tiles.len();
    assert!(n <= usize::from(u16::MAX), "tiles are u16-indexed");
    assert!(lattice > 0);
    let mut f = Floorplan {
        x: vec![0; n],
        y: vec![0; n],
        w: tiles.iter().map(|m| m.bbox.w).collect(),
        h: tiles.iter().map(|m| m.bbox.h).collect(),
        orient: vec![Orient::R0; n],
        row: vec![0; n],
        axis: Vec::new(),
    };
    let mut fill = Macro::default();
    // Every placed tile's shapes, world frame.
    let mut placed: Vec<Shape> = Vec::new();
    let (Some(fin), Some(gate), Some(act)) = (process.layer("fin"), process.layer("poly"), process.layer("diff")) else {
        for (k, m) in tiles.iter().enumerate() {
            let shapes = oriented(&m.shapes, m.bbox, Orient::R0);
            f.x[k] = if k == 0 { 0 } else { up(clearance(&placed, &shapes, spacing, false, None), lattice) };
            placed.extend(shifted(&shapes, f.x[k], 0));
        }
        return Ok((f, fill));
    };
    let gate_p = process.width("poly").unwrap_or(0) + process.space("poly").unwrap_or(0);
    let fin_p = process.width("fin").unwrap_or(0) + process.space("fin").unwrap_or(0);
    let act_s = process.horizontal_space("diff").unwrap_or(0).max(process.space("diff").unwrap_or(0));
    assert!(gate_p > 0 && fin_p > 0, "a fin deck states its gate and fin pitches");

    // Each tile's shift `(dx, dy)` from its local frame to the world.
    let (mut dx, mut dy) = (vec![0; n], vec![0; n]);
    // The previous tile's gate span, world frame.
    let mut span = (0, 0);
    let well = process.layer("nwell");
    let mut prev: Option<(i32, i32)> = None; // previous tile's last gate x, active right edge
    for (k, m) in tiles.iter().enumerate() {
        let gates = extent(m, gate);
        let acts = extent(m, act);
        let fins = extent(m, fin);
        assert!(gates.w > 0 && acts.w > 0 && fins.h > 0, "tile {k}: no gate, active or fin on a fin deck");
        if k == 0 {
            dx[0] = -m.bbox.x;
            dy[0] = -m.bbox.y;
            span = (gates.y + dy[0], gates.y + gates.h + dy[0]);
        } else {
            let (last_gate, act_end) = prev.expect("tile 0 set it");
            // Bottom on tile 0's active bottom, then up onto its fin lattice.
            let y = extent(&tiles[0], act).y + dy[0] - acts.y;
            let fin0 = extent(&tiles[0], fin).y + dy[0];
            dy[k] = y + (fin0 - (fins.y + y)).rem_euclid(fin_p);
            // Clear the active spacing and the facing shapes, then land the
            // first gate on the lattice.
            let lifted: Vec<Shape> = shifted(&m.shapes, 0, dy[k]).collect();
            let x = (act_end + act_s - acts.x).max(clearance(&placed, &lifted, spacing, false, well));
            dx[k] = x + (last_gate - (gates.x + x)).rem_euclid(gate_p);
            let own = (gates.y + dy[k], gates.y + gates.h + dy[k]);
            let band = (span.0.max(own.0), span.1.min(own.1));
            if band.0 >= band.1 {
                return Err(Refusal::GateSpan { tile: k as u16 });
            }
            span = own;
            let w = width_of(m, gate);
            let mut g = last_gate + gate_p;
            while g < gates.x + dx[k] {
                fill.shapes.push(Shape { layer: gate, rect: Rect { x: g, y: band.0, w, h: band.1 - band.0 } });
                g += gate_p;
            }
            // ponytail: bridges the wells' common height; wells of unequal
            // height leave a notch the deck may flag (WELL.S.1).
            if let Some(well) = well {
                let (a, b) = (extent(&tiles[k - 1], well), extent(m, well));
                if a.w > 0 && b.w > 0 {
                    let (ax, ay, bx, by) = (a.x + a.w + dx[k - 1], a.y + dy[k - 1], b.x + dx[k], b.y + dy[k]);
                    let (y0, y1) = (ay.max(by), (ay + a.h).min(by + b.h));
                    fill.shapes.push(Shape { layer: well, rect: Rect { x: ax, y: y0, w: bx - ax, h: y1 - y0 } });
                }
            }
        }
        let last = m.shapes.iter().filter(|s| s.layer == gate).map(|s| s.rect.x).max().expect("a gate");
        prev = Some((last + dx[k], acts.x + acts.w + dx[k]));
        placed.extend(shifted(&m.shapes, dx[k], dy[k]));
        f.x[k] = m.bbox.x + dx[k];
        f.y[k] = m.bbox.y + dy[k];
    }
    fill.bbox = fill.shapes.iter().map(|s| s.rect).reduce(union).unwrap_or_default();
    Ok((f, fill))
}

/// The box around `m`'s shapes on `layer` (zero when it has none).
fn extent(m: &Macro, layer: LayerId) -> Rect {
    m.shapes.iter().filter(|s| s.layer == layer).map(|s| s.rect).reduce(union).unwrap_or_default()
}

/// The drawn gate width: `m`'s first gate's.
fn width_of(m: &Macro, gate: LayerId) -> i32 {
    m.shapes.iter().find(|s| s.layer == gate).map_or(0, |s| s.rect.w)
}

fn union(a: Rect, b: Rect) -> Rect {
    let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
    let (x1, y1) = ((a.x + a.w).max(b.x + b.w), (a.y + a.h).max(b.y + b.h));
    Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
}
