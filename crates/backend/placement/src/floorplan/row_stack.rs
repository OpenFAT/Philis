//! FLOOR-RowStack (RECIPES §3): rows by stage, bottom to top; in each row the
//! self-symmetric tiles centred on the one shared axis, then pairs outward,
//! the left half mirrored.

use analog::base::Orient;

use super::{up, Floorplan, Refusal, Side, Stack};

/// Place tiles of outline `size[i] = (w, h)` by `stack`, `gap` apart within
/// and between rows: the axis, every row's base and every right or centred
/// tile's lower-left on `lattice`; a left half is its partner's mirror image
/// about the axis, so its drawn grid mirrors onto the lattice.
/// Rows sit bottom-aligned at their tallest tile. The axis is the only
/// horizontal reference, so a pair's halves are exact mirrors of each other.
/// ponytail: one `gap`; the class matrix (P-17) replaces it per tile pair,
/// and several `Axis` tiles in one row are centred as one block, each off
/// the axis.
///
/// # Errors
/// [`Refusal::PairMismatch`]: a row's `Left` and `Right` counts, or a pair's
/// outlines, differ.
pub fn row_stack(size: &[(i32, i32)], stack: &Stack, gap: i32, lattice: i32) -> Result<Floorplan, Refusal> {
    let n = size.len();
    assert!(n <= usize::from(u16::MAX), "tiles are u16-indexed");
    assert!(stack.row.len() == n && stack.side.len() == n, "a stack row per tile");
    assert!(lattice > 0 && gap >= 0);
    let mut f = Floorplan {
        x: vec![0; n],
        y: vec![0; n],
        w: size.iter().map(|s| s.0).collect(),
        h: size.iter().map(|s| s.1).collect(),
        orient: stack.side.iter().map(|&s| if s == Side::Left { Orient::Mx180 } else { Orient::R0 }).collect(),
        row: stack.row.clone(),
        axis: vec![0],
    };
    let mut rows = stack.row.clone();
    rows.sort_unstable();
    rows.dedup();
    let mut base = 0;
    for &r in &rows {
        let of = |side: Side| (0..n).filter(move |&i| stack.row[i] == r && stack.side[i] == side);
        // Selfs as one block centred on the axis (x = 0).
        let centre: Vec<usize> = of(Side::Axis).collect();
        let cw: i32 = centre.iter().map(|&i| size[i].0).sum::<i32>() + gap * (centre.len() as i32 - 1).max(0);
        let mut cursor = (-cw / 2).div_euclid(lattice) * lattice;
        let reach = if centre.is_empty() { up(gap / 2, lattice) } else { up((cursor + cw).max(-cursor) + gap, lattice) };
        for &i in &centre {
            f.x[i] = cursor;
            cursor = up(cursor + size[i].0 + gap, lattice);
        }
        // Pairs outward: right half at `axis + d`, left its mirror image.
        let (lefts, rights): (Vec<usize>, Vec<usize>) = (of(Side::Left).collect(), of(Side::Right).collect());
        if lefts.len() != rights.len() || lefts.iter().zip(&rights).any(|(&l, &rt)| size[l] != size[rt]) {
            return Err(Refusal::PairMismatch { row: r });
        }
        let mut d = reach;
        for (&l, &rt) in lefts.iter().zip(&rights) {
            f.x[rt] = d;
            f.x[l] = -d - size[l].0;
            d = up(d + size[rt].0 + gap, lattice);
        }
        let mut tallest = 0;
        for i in (0..n).filter(|&i| stack.row[i] == r) {
            f.y[i] = base;
            tallest = tallest.max(size[i].1);
        }
        base = up(base + tallest + gap, lattice);
    }
    // Shift by a lattice step so every x is ≥ 0, the leftmost within one step.
    let shift = -(f.x.iter().copied().min().unwrap_or(0).div_euclid(lattice)) * lattice;
    f.x.iter_mut().for_each(|x| *x += shift);
    f.axis[0] = shift;
    Ok(f)
}
