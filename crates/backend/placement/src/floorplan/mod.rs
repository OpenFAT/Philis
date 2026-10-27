//! Floorplan recipes (Q20 to Q22): [`targets`] hands each set a width and
//! height bound and its axis top-down; [`row_stack`] places tiles in rows by
//! stage, pairs mirrored about one shared vertical axis; [`glue`] places a
//! row on the gate lattice of a fin deck with field-gate fill. Both size
//! every gap from the shapes that face across it ([`Spacing`]). Each returns a
//! [`Floorplan`] table, one row per tile, that [`Floorplan::layout`] turns
//! into the `Layout` columns `place_macros` stamps.
//!
//! Contents:
//! - `pub enum Side`
//! - `pub struct Stack`
//! - `pub struct Floorplan`
//! - `pub enum Refusal`
//! - `pub struct Targets`
//! - `Floorplan::layout(&self) -> Layout`
//! - `pub fn targets(stack: &Stack, bounds: (i32, i32), gap: i32) -> Targets`

mod glue;
mod row_stack;
mod spacing;

pub use glue::glue;
pub use row_stack::row_stack;
pub use spacing::Spacing;

use analog::base::{Layout, Orient};

/// Where a tile sits in its row relative to the shared axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    /// Self-symmetric: centred on the axis (tails, shared bias).
    Axis,
    /// Left half of a pair, drawn mirrored (`x ↦ −x`). The k-th `Left` of a
    /// row pairs with its k-th `Right`, outward from the axis in input order.
    Left,
    /// Right half of a pair, drawn as is.
    Right,
}

/// The row stack's input, one row per tile (SoA, parallel to the tiles).
#[derive(Clone, Default, Debug)]
pub struct Stack {
    /// Stage index along the signal or current flow (`Order`); row 0 is the
    /// bottom row. ≤ 255 stages.
    pub row: Vec<u8>,
    pub side: Vec<Side>,
}

/// The floorplan table (ARCHITECTURE `floorplan`): one row per tile.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Floorplan {
    /// Lower-left x of the oriented tile, nm.
    pub x: Vec<i32>,
    /// Lower-left y of the oriented tile, nm.
    pub y: Vec<i32>,
    /// Outline width after `orient`, nm.
    pub w: Vec<i32>,
    /// Outline height after `orient`, nm.
    pub h: Vec<i32>,
    pub orient: Vec<Orient>,
    /// The tile's row (stage index).
    pub row: Vec<u8>,
    /// Vertical symmetry axes' x by `AxisId`, nm; empty for a glue row.
    pub axis: Vec<i32>,
}

/// Why a floorplan recipe cannot place its tiles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// A row's `Left` and `Right` counts differ, or a pair's two outlines do.
    PairMismatch { row: u8 },
    /// A fin-deck glue tile whose gates share no height with its left
    /// neighbour's: no field gate can link the two gate columns, so an exact
    /// gate pitch (ASAP7 GATE.S.1) reads a missing gate.
    GateSpan { tile: u16 },
}

/// Top-down bounds per set (Q22), parallel to the [`Stack`].
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Targets {
    /// Width bound, nm: the set recipe folds to fit it.
    pub w: Vec<i32>,
    /// Height bound, nm: the set's row height.
    pub h: Vec<i32>,
    /// x of the vertical axis the set is symmetric about, nm. The row stack
    /// has one shared axis; nested symmetry (P-06) would give a set its own.
    pub axis: Vec<i32>,
}

impl Floorplan {
    /// The table as `Layout` columns: centre `x + w/2`, so `place_macros`
    /// (which stamps the lower-left at `x − hw`) lands each tile on `x`
    /// exactly, odd widths included.
    #[must_use]
    pub fn layout(&self) -> Layout {
        let n = self.x.len();
        let half = |v: &[i32]| v.iter().map(|w| w / 2).collect::<Vec<_>>();
        let (hw, hh) = (half(&self.w), half(&self.h));
        Layout {
            x: self.x.iter().zip(&hw).map(|(x, h)| x + h).collect(),
            y: self.y.iter().zip(&hh).map(|(y, h)| y + h).collect(),
            hw,
            hh,
            axis: self.axis.clone(),
            groups: Vec::new(),
            orient: self.orient.clone(),
            variant: vec![0; n],
            branch: Vec::new(),
            power_uw: vec![0; n],
            temp_mc: vec![0; n],
            units: Default::default(),
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::PairMismatch { row } => write!(f, "row {row}: a pair's halves differ in count or outline"),
            Refusal::GateSpan { tile } => write!(f, "glue tile {tile}: gates share no height with its left neighbour's"),
        }
    }
}

/// Each set's bounds inside a `bounds = (width, height)` block, tiles `gap`
/// apart: every row gets the full width, split equally among its tiles (a
/// pair's halves get equal shares), the height split equally among rows; the
/// axis is the block's centre line.
/// ponytail: equal shares; weight by the sets' shape functions (P-14) when
/// recipes report them.
#[must_use]
pub fn targets(stack: &Stack, bounds: (i32, i32), gap: i32) -> Targets {
    assert_eq!(stack.row.len(), stack.side.len(), "a side per tile");
    let mut per_row = [0i32; 256];
    for &r in &stack.row {
        per_row[usize::from(r)] += 1;
    }
    let rows = per_row.iter().filter(|&&n| n > 0).count() as i32;
    let h = (bounds.1 - gap * (rows - 1).max(0)) / rows.max(1);
    Targets {
        w: stack.row.iter().map(|&r| (bounds.0 - gap * (per_row[usize::from(r)] - 1)) / per_row[usize::from(r)]).collect(),
        h: vec![h; stack.row.len()],
        axis: vec![bounds.0 / 2; stack.row.len()],
    }
}

// ---- private ----

/// `v` rounded up onto the `lattice`.
fn up(v: i32, lattice: i32) -> i32 {
    (v + lattice - 1).div_euclid(lattice) * lattice
}

#[cfg(test)]
mod tests;
