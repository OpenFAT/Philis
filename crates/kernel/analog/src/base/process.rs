//! The PDK seam generators are written against. Layers are looked up by role
//! (`"poly"`, `"metal1"`) and rules by name, so no generator hard-codes a PDK
//! number. The real impl is `verify::Pdk`.

use crate::base::LayerId;

pub trait Process {
    /// Layer for a role; `None` when this process omits that (optional) layer.
    fn layer(&self, role: &str) -> Option<LayerId>;

    /// Rule value in nm, or `default` when the process does not specify it.
    fn rule(&self, name: &str, default: i32) -> i32;

    /// Fabrication grid, nm. Every emitted coordinate snaps to it.
    fn grid(&self) -> i32;

    /// Sheet resistance of a role's layer, Ω/□ (for a cut, Ω per cut);
    /// `None` when the process does not characterise it.
    fn sheet_ohm(&self, role: &str) -> Option<f32> {
        let _ = role;
        None
    }

    /// Resistance of one cut of role `cut` landing on role `onto`, Ω per cut
    /// (a deck states contact resistance per landing: sky130 licon on poly
    /// 152, on a p-tap 585); `None` when the process does not characterise it.
    fn cut_ohm(&self, cut: &str, onto: &str) -> Option<f32> {
        let _ = (cut, onto);
        None
    }

    /// Spacing a role's layer needs between its shapes, nm: the widest of its
    /// `min_spacing`, its wide-metal spacing (a strap easily passes the width
    /// threshold) and, for a cut, its array spacing (a cell's cuts easily
    /// reach the array threshold). `None` = not given.
    fn space(&self, role: &str) -> Option<i32> {
        let _ = role;
        None
    }

    /// Spacing between two minimum-width shapes of a role's layer, nm: the
    /// plain `min_spacing`, without [`Process::space`]'s wide-metal and
    /// array steps (a contact landing is far under any width threshold).
    fn min_space(&self, role: &str) -> Option<i32> {
        self.space(role)
    }

    /// Spacing a role's line end (a narrow edge) keeps to anything, nm;
    /// `None` = not given.
    fn eol_space(&self, role: &str) -> Option<i32> {
        let _ = role;
        None
    }

    /// A role's layer's minimum width, nm; `None` = not given.
    fn width(&self, role: &str) -> Option<i32> {
        let _ = role;
        None
    }

    /// How far `outer` must enclose `inner` on every side, nm (the widest
    /// such rule); `None` = not given.
    fn enclosure(&self, outer: &str, inner: &str) -> Option<i32> {
        let _ = (outer, inner);
        None
    }

    /// Whether `inner` must span its `outer` host exactly across one axis
    /// (ASAP7 V0.M1.AUX.3: a via as wide as its wire); `false` = not given.
    fn full_width(&self, inner: &str, outer: &str) -> bool {
        let _ = (inner, outer);
        false
    }

    /// The end-cap: how far `outer` must pass `inner` on at least one side
    /// of each axis (an asymmetric enclosure), nm; `None` = not given.
    fn endcap(&self, outer: &str, inner: &str) -> Option<i32> {
        let _ = (outer, inner);
        None
    }

    /// How far `outer` must extend past `inner`'s edges (a salicide block
    /// past its poly), nm; `None` = not given.
    fn extension(&self, outer: &str, inner: &str) -> Option<i32> {
        let _ = (outer, inner);
        None
    }

    /// A role's layer's minimum polygon area, nm²; `None` = not given.
    fn area(&self, role: &str) -> Option<i64> {
        let _ = role;
        None
    }

    /// Horizontal spacing a role's layer needs between facing vertical
    /// edges, nm: the deck's spacing on that layer's vertical edges, which
    /// may exceed [`Process::space`] (ASAP7 ACTIVE.S.2A, 92 nm between
    /// device actives side by side); `None` = not given.
    fn horizontal_space(&self, role: &str) -> Option<i32> {
        let _ = role;
        None
    }

    /// Spacing the deck requires between two roles' layers, nm (its
    /// two-layer spacing rules, widest); `None` = not given.
    fn space_between(&self, a: &str, b: &str) -> Option<i32> {
        let _ = (a, b);
        None
    }

    /// The device model this view draws (a recipe's); `None` for the bare
    /// process, which draws whatever the netlist sizes.
    fn model(&self) -> Option<&str> {
        None
    }

    /// The only widths, nm, a shape may take when a cell draws `roles`'
    /// layers together: the deck's discrete-width rule on a layer built from
    /// them (sky130 rpm.1b: a precision poly body is 350, 690, 1410, 2850 or
    /// 5730 nm). `None` = any width at or above the minimum.
    fn allowed_widths(&self, roles: &[&str]) -> Option<Vec<i32>> {
        let _ = roles;
        None
    }

}

/// Hastings §13.3 matching class (PDF p.712): what a matched set's
/// environment and limits scale with. The discriminant indexes the sidecar's
/// tier arrays.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum MatchClass {
    Minimal = 0,
    #[default]
    Moderate = 1,
    Exceptional = 2,
}

/// What the active area sits on, from the sidecar's `cell.substrate_kind`
/// (`"bulk"`, `"epi_on_pplus"`, null). It decides whether substrate isolation
/// has a calibrated distance: on epi over a low-resistivity p+ substrate it
/// saturates at a few epi thicknesses (Charbon et al. 2001 ch.8, PDF p.127; Su
/// et al. 4×); on bulk it keeps improving with distance (PDF pp.127–130), so
/// there is no plateau to budget.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SubstrateKind {
    Bulk,
    EpiOnLowRes,
    /// The deck does not say: isolation reads unknown.
    #[default]
    Unknown,
}

impl SubstrateKind {
    /// The sidecar spelling; anything else (or absent) is `Unknown`.
    #[must_use]
    pub fn from_key(s: Option<&str>) -> Self {
        match s {
            Some("bulk") => Self::Bulk,
            Some("epi_on_pplus") => Self::EpiOnLowRes,
            _ => Self::Unknown,
        }
    }
}
