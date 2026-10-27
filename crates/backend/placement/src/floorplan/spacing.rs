//! The gap between neighbouring tiles from the shapes that face across it,
//! per layer pair, instead of the widest spacing any drawn layer has.
//!
//! Contents:
//! - `pub struct Spacing`
//! - `Spacing::of(process: &dyn Process, roles: &[&str]) -> Self`
//! - `pub(super) fn clearance(placed: &[Shape], moving: &[Shape], spacing: &Spacing, vertical: bool, merged: Option<LayerId>) -> i32`
//! - `pub(super) fn oriented(shapes: &[Shape], bbox: Rect, o: Orient) -> Vec<Shape>`
//! - `pub(super) fn shifted(shapes: &[Shape], dx: i32, dy: i32) -> impl Iterator<Item = Shape> + '_`
//! - `pub(super) fn mirrored(shapes: &[Shape]) -> impl Iterator<Item = Shape> + '_`

use analog::base::{LayerId, Orient, Process, Rect, Shape};

/// Spacing rows: shapes on `a` and `b` (either order; `a == b` for a
/// layer's own spacing) stay `nm` apart. A layer pair with no row may touch.
#[derive(Clone, Default, Debug)]
pub struct Spacing {
    pub a: Vec<LayerId>,
    pub b: Vec<LayerId>,
    pub nm: Vec<i32>,
}

impl Spacing {
    /// Every rule `process` states among `roles`: each role's own `space`
    /// and every pair's `space_between` (sky130 diff to nwell, poly to diff).
    /// A caller names the roles its tiles draw; `Process` cannot list them.
    #[must_use]
    pub fn of(process: &dyn Process, roles: &[&str]) -> Self {
        let mut s = Spacing::default();
        for (i, &ra) in roles.iter().enumerate() {
            let Some(la) = process.layer(ra) else { continue };
            for &rb in &roles[i..] {
                let Some(lb) = process.layer(rb) else { continue };
                let nm = if ra == rb { process.space(ra) } else { process.space_between(ra, rb) };
                if let Some(nm) = nm.filter(|&nm| nm > 0) {
                    s.a.push(la);
                    s.b.push(lb);
                    s.nm.push(nm);
                }
            }
        }
        s
    }

    fn get(&self, a: LayerId, b: LayerId) -> Option<i32> {
        (0..self.nm.len()).filter(|&i| (self.a[i], self.b[i]) == (a, b) || (self.a[i], self.b[i]) == (b, a)).map(|i| self.nm[i]).max()
    }
}

/// The least shift of `moving` along x (along y when `vertical`) that puts
/// it past every `placed` shape and each facing pair, shapes whose gap
/// across is under their layer pair's spacing, at least that spacing apart.
/// `merged`: a layer whose shapes may meet across tiles (a bridged well);
/// its own spacing is skipped. ponytail: Manhattan facing (the full spacing
/// whenever the across gap is short of it) and O(placed × moving) over the
/// shapes within the widest spacing of the facing edges.
pub(super) fn clearance(placed: &[Shape], moving: &[Shape], spacing: &Spacing, vertical: bool, merged: Option<LayerId>) -> i32 {
    let span = |r: Rect| if vertical { (r.y, r.y + r.h, r.x, r.x + r.w) } else { (r.x, r.x + r.w, r.y, r.y + r.h) };
    let hi = placed.iter().map(|s| span(s.rect).1).max().unwrap_or(i32::MIN / 2);
    let lo = moving.iter().map(|s| span(s.rect).0).min().unwrap_or(0);
    let widest = spacing.nm.iter().copied().max().unwrap_or(0);
    let mut shift = hi - lo;
    for pa in placed.iter().filter(|s| span(s.rect).1 > hi - widest) {
        let (_, a_hi, a0, a1) = span(pa.rect);
        for pb in moving.iter().filter(|s| span(s.rect).0 < lo + widest) {
            if pa.layer == pb.layer && Some(pa.layer) == merged {
                continue;
            }
            let Some(s) = spacing.get(pa.layer, pb.layer) else { continue };
            let (b_lo, _, b0, b1) = span(pb.rect);
            if (b0 - a1).max(a0 - b1) < s {
                shift = shift.max(a_hi + s - b_lo);
            }
        }
    }
    shift
}

/// `shapes` turned by `o` about the box `bbox`, then moved so the turned
/// box's lower-left is the origin: a tile's shapes as `place_rect` stamps
/// them, before the shift.
pub(super) fn oriented(shapes: &[Shape], bbox: Rect, o: Orient) -> Vec<Shape> {
    let at = o.apply_rect(bbox);
    shapes
        .iter()
        .map(|s| {
            let r = o.apply_rect(s.rect);
            Shape { layer: s.layer, rect: Rect { x: r.x - at.x, y: r.y - at.y, ..r } }
        })
        .collect()
}

/// `shapes` moved by `(dx, dy)`.
pub(super) fn shifted(shapes: &[Shape], dx: i32, dy: i32) -> impl Iterator<Item = Shape> + '_ {
    shapes.iter().map(move |s| Shape { layer: s.layer, rect: Rect { x: s.rect.x + dx, y: s.rect.y + dy, ..s.rect } })
}

/// `shapes` mirrored about `x = 0`.
pub(super) fn mirrored(shapes: &[Shape]) -> impl Iterator<Item = Shape> + '_ {
    shapes.iter().map(|s| Shape { layer: s.layer, rect: Rect { x: -(s.rect.x + s.rect.w), ..s.rect } })
}
