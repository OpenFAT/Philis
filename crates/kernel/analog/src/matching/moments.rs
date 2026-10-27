//! Unit moments and orientation (Hastings §13.3): the weighted first/second
//! moments a common-centroid row must equalise, the signed-current (Φ) test a
//! mirrored finger order must pass, and the cancelled-moment order a figure
//! (ABBA, ABBA/BAAB, …) achieves.
//!
//! In → out: weighted unit points → [`Sums`] (moments, Φ, current axes) and the
//! cancelled moment order of a figure. Cites: Hastings §13.3 (centroid, Φ);
//! the Nth-order central-symmetry paper (fig. 3) for orders above 2.
//!
//! Contents:
//! - `pub struct Pt`
//! - `pub enum Axis`
//! - `pub struct Sums`
//! - `pub fn sums(pts: impl IntoIterator<Item = Pt>) -> Sums`
//! - `Sums::centroid(&self) -> Option<(f64, f64)>`
//! - `Sums::second(&self, c: (f64, f64)) -> [f64; 3]`
//! - `Sums::phi(&self) -> (f64, f64)`
//! - `pub fn phi_equal(a: &Sums, b: &Sums) -> bool`
//! - `pub fn phi_equal_all(units: &[crate::base::Unit], members: usize) -> bool`
//! - `pub fn mirror_allowed(members: &[Sums]) -> bool`
//! - `pub fn mirror_allowed_units(units: &[crate::base::Unit]) -> bool`
//! - `pub fn cancelled_order(devs: &[&[Pt]], nmax: u8, tol: f64) -> (u8, [f64; 5])`

/// One active unit in a common frame: `w` is its electrical weight (never the
/// drawn outline), `phi` its signed S→D direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pt {
    /// Position x, in whatever frame every point of the set shares.
    pub x: f64,
    /// Position y, same frame as `x`.
    pub y: f64,
    /// Electrical weight (MOS gate area); ≥ 0.
    pub w: f64,
    /// Signed S→D direction, each component in `{-1, 0, 1}`.
    pub phi: (i8, i8),
}

/// Which axes a unit set's current runs on, ignoring zero-φ units (a
/// capacitor or an axis that carries no current says nothing about layout
/// symmetry there).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    /// No unit carries current (or the set is empty).
    #[default]
    None,
    /// Every current runs along x.
    H,
    /// Every current runs along y.
    V,
    /// Currents on both axes.
    Mixed,
}

/// Weighted first and second moments of a unit set, plus its mean signed
/// current and how many units carried a non-zero φ on each axis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sums {
    /// Σw.
    pub w: f64,
    /// Σw·x (raw, not divided by Σw).
    pub x: f64,
    /// Σw·y.
    pub y: f64,
    /// Σw·x², about the frame origin.
    pub xx: f64,
    /// Σw·x·y, about the frame origin.
    pub xy: f64,
    /// Σw·y², about the frame origin.
    pub yy: f64,
    /// Σφx, unweighted.
    pub phi_x: i32,
    /// Σφy, unweighted.
    pub phi_y: i32,
    /// Units summed, zero-φ ones included.
    pub n: u32,
    /// The axes the non-zero φ run on.
    pub axis: Axis,
}

impl From<crate::base::PlacedUnit> for Pt {
    fn from(u: crate::base::PlacedUnit) -> Self {
        Pt { x: f64::from(u.x), y: f64::from(u.y), w: u.weight as f64, phi: u.phi }
    }
}

impl From<crate::base::Unit> for Pt {
    fn from(u: crate::base::Unit) -> Self {
        Pt { x: f64::from(u.x), y: f64::from(u.y), w: u.weight as f64, phi: u.phi }
    }
}

/// One pass over a unit set's points: weighted sums plus the current axes.
pub fn sums(pts: impl IntoIterator<Item = Pt>) -> Sums {
    let mut s = Sums::default();
    let (mut any_x, mut any_y) = (false, false);
    for p in pts {
        s.w += p.w;
        s.x += p.w * p.x;
        s.y += p.w * p.y;
        s.xx += p.w * p.x * p.x;
        s.xy += p.w * p.x * p.y;
        s.yy += p.w * p.y * p.y;
        s.phi_x += i32::from(p.phi.0);
        s.phi_y += i32::from(p.phi.1);
        s.n += 1;
        any_x |= p.phi.0 != 0;
        any_y |= p.phi.1 != 0;
    }
    s.axis = match (any_x, any_y) {
        (false, false) => Axis::None,
        (true, false) => Axis::H,
        (false, true) => Axis::V,
        (true, true) => Axis::Mixed,
    };
    s
}

impl Sums {
    /// Weighted centroid, world units. `None` when the set carries no weight.
    #[must_use]
    pub fn centroid(&self) -> Option<(f64, f64)> {
        (self.w > 0.0).then(|| (self.x / self.w, self.y / self.w))
    }

    /// `[Σw(x−cx)², Σw(x−cx)(y−cy), Σw(y−cy)²] / Σw`: the second moment about
    /// `c`, expanded from the raw sums so a caller need not re-walk the
    /// units. `[0; 3]` when the set carries no weight.
    #[must_use]
    pub fn second(&self, c: (f64, f64)) -> [f64; 3] {
        if self.w <= 0.0 {
            return [0.0; 3];
        }
        let (cx, cy) = c;
        [
            self.xx / self.w - 2.0 * cx * self.x / self.w + cx * cx,
            self.xy / self.w - cx * self.y / self.w - cy * self.x / self.w + cx * cy,
            self.yy / self.w - 2.0 * cy * self.y / self.w + cy * cy,
        ]
    }

    /// Mean signed current per unit, `(Σφx/n, Σφy/n)`. `(0, 0)` when empty.
    #[must_use]
    pub fn phi(&self) -> (f64, f64) {
        if self.n == 0 {
            (0.0, 0.0)
        } else {
            (f64::from(self.phi_x) / f64::from(self.n), f64::from(self.phi_y) / f64::from(self.n))
        }
    }
}

/// Hastings' Φ test, compared exactly as `Σφ_a·n_b == Σφ_b·n_a` so it never
/// rounds two sets with different unit counts into a false match.
#[must_use]
pub fn phi_equal(a: &Sums, b: &Sums) -> bool {
    i64::from(a.phi_x) * i64::from(b.n) == i64::from(b.phi_x) * i64::from(a.n)
        && i64::from(a.phi_y) * i64::from(b.n) == i64::from(b.phi_y) * i64::from(a.n)
}

/// Every member of a macro's units runs the same mean current direction. A
/// macro without units (or none of a member's units set φ) has nothing to
/// compare and passes.
#[must_use]
pub fn phi_equal_all(units: &[crate::base::Unit], members: usize) -> bool {
    let live: Vec<Sums> =
        (0..members).map(|d| sums(units.iter().filter(|u| usize::from(u.owner) == d).map(|&u| Pt::from(u)))).filter(|s| s.n > 0).collect();
    live.windows(2).all(|w| phi_equal(&w[0], &w[1]))
}

/// Whether an `Mx180` mirror (which negates φx) still runs every member's
/// current the direction it started: only true when no member carries φx.
#[must_use]
pub fn mirror_allowed(members: &[Sums]) -> bool {
    members.iter().all(|s| s.phi_x == 0)
}

/// [`mirror_allowed`] over a macro's units, grouped per owner as in
/// [`phi_equal_all`]: an odd finger count leaves a net φx and refuses Mirror.
#[must_use]
pub fn mirror_allowed_units(units: &[crate::base::Unit]) -> bool {
    let mut owners: Vec<u8> = units.iter().map(|u| u.owner).collect();
    owners.sort_unstable();
    owners.dedup();
    let per: Vec<Sums> = owners.iter().map(|&o| sums(units.iter().filter(|u| u.owner == o).map(|&u| Pt::from(u)))).collect();
    mirror_allowed(&per)
}

/// How many leading moment orders (1, 2, …) cancel between every pair of
/// devices in a figure, Hastings §13.3: order 1 is a common centroid, order 2
/// also equalises second moments (ABBA/BAAB), and so on up to `nmax` (<= 4).
/// Returns `(order, r)` where `r[n]` is the worst residual at order `n`
/// (`r[0] = 0`, unused entries above `nmax` are `0.0`). O(devices² · nmax² · units).
///
/// # Panics
/// When `nmax > 4` (a debug assert; past it `r` is indexed out of range).
#[must_use]
pub fn cancelled_order(devs: &[&[Pt]], nmax: u8, tol: f64) -> (u8, [f64; 5]) {
    debug_assert!(nmax <= 4);
    let all: Vec<Pt> = devs.iter().flat_map(|d| d.iter().copied()).collect();
    let w: f64 = all.iter().map(|p| p.w).sum();
    let r = [0.0f64; 5];
    if w <= 0.0 {
        return (nmax, r);
    }
    let c = (all.iter().map(|p| p.w * p.x).sum::<f64>() / w, all.iter().map(|p| p.w * p.y).sum::<f64>() / w);
    let l = all.iter().map(|p| (p.x - c.0).hypot(p.y - c.1)).fold(0.0, f64::max);
    if l == 0.0 {
        return (nmax, r);
    }
    let devices: Vec<&[Pt]> = devs.iter().copied().filter(|d| d.iter().map(|p| p.w).sum::<f64>() > 0.0).collect();
    let mut r = r;
    let mut order = 0u8;
    let mut still_ok = true;
    for n in 1..=nmax {
        let mut worst = 0.0f64;
        for (ai, a) in devices.iter().enumerate() {
            for b in &devices[ai + 1..] {
                for p in 0..=i32::from(n) {
                    let q = i32::from(n) - p;
                    let d = (moment(a, c, l, p, q) - moment(b, c, l, p, q)).abs();
                    worst = worst.max(d);
                }
            }
        }
        r[usize::from(n)] = worst;
        if still_ok && worst <= tol {
            order = n;
        } else {
            still_ok = false;
        }
    }
    (order, r)
}

// ---- private ----

fn moment(pts: &[Pt], c: (f64, f64), l: f64, p: i32, q: i32) -> f64 {
    let w: f64 = pts.iter().map(|pt| pt.w).sum();
    if w <= 0.0 {
        return 0.0;
    }
    pts.iter().map(|pt| pt.w * ((pt.x - c.0) / l).powi(p) * ((pt.y - c.1) / l).powi(q)).sum::<f64>() / w
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: spec example, the net φx of each finger set counted by hand.
    /// Mx180 negates φx: an odd finger pair keeps a net φx and must not mirror.
    #[test]
    fn mirror_is_refused_for_an_odd_finger_pair() {
        let units = |phis: &[(i8, i8)]| -> Vec<crate::base::Unit> {
            phis.iter().map(|&phi| crate::base::Unit { owner: 0, x: 0, y: 0, weight: 1, phi, sa: 0, sb: 0 }).collect()
        };
        assert!(!mirror_allowed_units(&units(&[(1, 0), (-1, 0), (1, 0)])));
        assert!(mirror_allowed_units(&units(&[(1, 0), (-1, 0)])));
        assert!(mirror_allowed_units(&units(&[(0, 1); 3])));
    }

    fn pt(x: f64, y: f64) -> Pt {
        Pt { x, y, w: 1.0, phi: (0, 0) }
    }

    /// Oracle: theory, ABBA is a common centroid but not second-order symmetric (Hastings §13.3).
    #[test]
    fn abba_cancels_first_order_only() {
        let a: Vec<Pt> = [0.0, 3000.0].iter().map(|&x| pt(x, 0.0)).collect();
        let b: Vec<Pt> = [1000.0, 2000.0].iter().map(|&x| pt(x, 0.0)).collect();
        let (order, r) = cancelled_order(&[&a, &b], 2, 1e-9);
        assert_eq!(order, 1);
        assert!(r[2] > 0.5, "r[2] = {}", r[2]);
    }

    /// A's and B's points for one row at `y`: `abba` puts A outermost
    /// (`x` 0, 3000) and B innermost (1000, 2000); `baab` swaps them.
    fn row(abba: bool, y: f64) -> (Vec<Pt>, Vec<Pt>) {
        let outer: Vec<Pt> = [0.0, 3000.0].iter().map(|&x| pt(x, y)).collect();
        let inner: Vec<Pt> = [1000.0, 2000.0].iter().map(|&x| pt(x, y)).collect();
        if abba {
            (outer, inner)
        } else {
            (inner, outer)
        }
    }

    /// Oracle: theory, ABBA over BAAB equalises second moments (Hastings §13.3).
    #[test]
    fn abba_baab_cancels_second_order() {
        let (a0, b0) = row(true, 0.0);
        let (a1, b1) = row(false, 1000.0);
        let dev_a: Vec<Pt> = a0.into_iter().chain(a1).collect();
        let dev_b: Vec<Pt> = b0.into_iter().chain(b1).collect();
        let (order, r) = cancelled_order(&[&dev_a, &dev_b], 3, 1e-9);
        assert_eq!(order, 2);
        assert!(r[3] > 0.2, "r[3] = {}", r[3]);
    }

    /// Oracle: spec example, the Nth-order paper's fig. 3 4×4 pattern.
    #[test]
    fn nth_fig3_4x4_cancels_third_order() {
        let (a0, b0) = row(true, 0.0);
        let (a1, b1) = row(false, 1000.0);
        let (a2, b2) = row(false, 2000.0);
        let (a3, b3) = row(true, 3000.0);
        let dev_a: Vec<Pt> = a0.into_iter().chain(a1).chain(a2).chain(a3).collect();
        let dev_b: Vec<Pt> = b0.into_iter().chain(b1).chain(b2).chain(b3).collect();
        let (order, _r) = cancelled_order(&[&dev_a, &dev_b], 3, 1e-9);
        assert!(order >= 3, "order = {order}");
    }

    /// Oracle: spec example, xx moments of two crossed pairs by hand.
    #[test]
    fn radial_equal_is_not_second_order() {
        let a: Vec<Pt> = [-2000.0, 2000.0].iter().map(|&x| pt(x, 0.0)).collect();
        let b: Vec<Pt> = [-2000.0, 2000.0].iter().map(|&y| pt(0.0, y)).collect();
        let (order, _) = cancelled_order(&[&a, &b], 2, 1e-9);
        assert_eq!(order, 1);
        let sa = sums(a.iter().copied());
        let sb = sums(b.iter().copied());
        assert!((sa.second((0.0, 0.0))[0] - 4e6).abs() < 1e-6);
        assert_eq!(sb.second((0.0, 0.0))[0], 0.0);
    }

    /// Oracle: spec example, Hastings §13.3's Φ example (3:1 over 4 equals 9:3 over 12).
    #[test]
    fn hastings_phi_example() {
        let mk = |xs: &[i8]| -> Sums { sums(xs.iter().map(|&s| Pt { x: 0.0, y: 0.0, w: 1.0, phi: (s, 0) })) };
        let a = mk(&[1, 1, 1, -1]);
        let b = mk(&[[1i8; 9].as_slice(), &[-1; 3]].concat());
        assert!(phi_equal(&a, &b));
        let c = mk(&[-1, -1, -1, 1]);
        assert!(!phi_equal(&a, &c));
    }

    /// Oracle: spec example, two single points by hand (r1 > 0, r2 = 0).
    #[test]
    fn order_counts_leading_orders_only() {
        // Two single points: r1 > 0 but every second moment is equal (r2 = 0).
        let (a, b) = ([pt(0.0, 0.0)], [pt(1000.0, 0.0)]);
        let (order, r) = cancelled_order(&[&a, &b], 2, 1e-9);
        assert_eq!((order, r[2]), (0, 0.0), "r = {r:?}");
    }

    /// Oracle: theory, a 1:2 set with the single unit at the pair's centroid.
    #[test]
    fn ratio_weights_make_one_to_two_exact() {
        let a = vec![pt(0.0, 0.0)];
        let b: Vec<Pt> = [-1000.0, 1000.0].iter().map(|&x| pt(x, 0.0)).collect();
        let (order, _) = cancelled_order(&[&a, &b], 1, 1e-9);
        assert!(order >= 1);
    }

    /// Oracle: spec example, each φ combination by hand.
    #[test]
    fn axis_ignores_zero_phi_units() {
        let s = sums([Pt { x: 0.0, y: 0.0, w: 1.0, phi: (1, 0) }, Pt { x: 0.0, y: 0.0, w: 1.0, phi: (0, 0) }]);
        assert_eq!(s.axis, Axis::H);
        let s = sums([Pt { x: 0.0, y: 0.0, w: 1.0, phi: (0, 0) }]);
        assert_eq!(s.axis, Axis::None);
        let s = sums([Pt { x: 0.0, y: 0.0, w: 1.0, phi: (1, 0) }, Pt { x: 0.0, y: 0.0, w: 1.0, phi: (0, 1) }]);
        assert_eq!(s.axis, Axis::Mixed);
    }
}
