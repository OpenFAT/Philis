//! FLOOR-RowStack (RECIPES §3): rows by stage, bottom to top; in each row the
//! self-symmetric tiles centred on the one shared axis, then pairs outward,
//! the left half mirrored.

use analog::base::{Macro, Orient, Shape};

use super::spacing::{clearance, mirrored, oriented, shifted, Spacing};
use super::{up, Floorplan, Refusal, Side, Stack};

/// Place `tiles` by `stack`. Every gap, between tiles in a row and between
/// rows, is the one the facing shapes need under `spacing`, rounded up onto
/// `lattice`; so are the axis, each row's base and every right or centred
/// tile's lower-left. A left half is its partner's mirror image about the
/// axis, so its drawn grid mirrors onto the lattice. Tiles in a row share
/// its base. ponytail: several `Axis` tiles in one row are centred as one
/// block, each off the axis.
///
/// # Errors
/// [`Refusal::PairMismatch`]: a row's `Left` and `Right` counts, or a pair's
/// outlines, differ.
pub fn row_stack(tiles: &[Macro], stack: &Stack, spacing: &Spacing, lattice: i32) -> Result<Floorplan, Refusal> {
    let n = tiles.len();
    assert!(n <= usize::from(u16::MAX), "tiles are u16-indexed");
    assert!(stack.row.len() == n && stack.side.len() == n, "a stack row per tile");
    assert!(lattice > 0);
    let mut f = Floorplan {
        x: vec![0; n],
        y: vec![0; n],
        w: tiles.iter().map(|m| m.bbox.w).collect(),
        h: tiles.iter().map(|m| m.bbox.h).collect(),
        orient: stack.side.iter().map(|&s| if s == Side::Left { Orient::Mx180 } else { Orient::R0 }).collect(),
        row: stack.row.clone(),
        axis: vec![0],
    };
    let local = |i: usize| oriented(&tiles[i].shapes, tiles[i].bbox, f.orient[i]);
    let mut rows = stack.row.clone();
    rows.sort_unstable();
    rows.dedup();
    // Every placed row's shapes, axis at x = 0.
    let mut below: Vec<Shape> = Vec::new();
    for &r in &rows {
        let of = |side: Side| (0..n).filter(move |&i| stack.row[i] == r && stack.side[i] == side);
        // Selfs left to right from 0, then the block centred on the axis.
        let mut row: Vec<Shape> = Vec::new();
        let centre: Vec<usize> = of(Side::Axis).collect();
        for &i in &centre {
            let shapes = local(i);
            f.x[i] = if row.is_empty() { 0 } else { up(clearance(&row, &shapes, spacing, false, None), lattice) };
            row.extend(shifted(&shapes, f.x[i], 0));
        }
        if let Some(&last) = centre.last() {
            let mid = ((f.x[last] + f.w[last]) / 2).div_euclid(lattice) * lattice;
            centre.iter().for_each(|&i| f.x[i] -= mid);
            row = shifted(&row, -mid, 0).collect();
        }
        // Pairs outward: right half at `axis + d`, left its mirror image;
        // `d` clears what the row holds and the half's own mirror.
        let (lefts, rights): (Vec<usize>, Vec<usize>) = (of(Side::Left).collect(), of(Side::Right).collect());
        if lefts.len() != rights.len()
            || lefts.iter().zip(&rights).any(|(&l, &rt)| tiles[l].bbox.w != tiles[rt].bbox.w || tiles[l].bbox.h != tiles[rt].bbox.h)
        {
            return Err(Refusal::PairMismatch { row: r });
        }
        let mut obstacles: Vec<Shape> = row.iter().copied().chain(mirrored(&row)).collect();
        for (&l, &rt) in lefts.iter().zip(&rights) {
            let shapes = local(rt);
            let own: Vec<Shape> = mirrored(&shapes).collect();
            let apart = (clearance(&own, &shapes, spacing, false, None) + 1) / 2;
            let d = up(clearance(&obstacles, &shapes, spacing, false, None).max(apart), lattice);
            f.x[rt] = d;
            f.x[l] = -d - f.w[l];
            let placed: Vec<Shape> = shifted(&shapes, d, 0).collect();
            row.extend(placed.iter().copied().chain(mirrored(&placed)));
            obstacles.extend(placed.iter().copied().chain(mirrored(&placed)));
        }
        let base = if below.is_empty() { 0 } else { up(clearance(&below, &row, spacing, true, None), lattice) };
        for i in (0..n).filter(|&i| stack.row[i] == r) {
            f.y[i] = base;
        }
        below.extend(shifted(&row, 0, base));
    }
    // Shift by a lattice step so every x is ≥ 0, the leftmost within one step.
    let shift = -(f.x.iter().copied().min().unwrap_or(0).div_euclid(lattice)) * lattice;
    f.x.iter_mut().for_each(|x| *x += shift);
    f.axis[0] = shift;
    Ok(f)
}
