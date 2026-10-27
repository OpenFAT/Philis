//! The three sequence-pair code moves (Balasa §1.3.3): a [`Tree`] node's
//! code → the same node with two kids swapped in α (M1), in β (M2) or in
//! both (M3), rewritten so a symmetry node stays symmetric-feasible.
//!
//! Contents:
//! - `pub(crate) fn m1(t: &mut Tree, node: usize, rng: &mut SplitMix64)`
//! - `pub(crate) fn m2(t: &mut Tree, node: usize, rng: &mut SplitMix64)`
//! - `pub(crate) fn m3(t: &mut Tree, node: usize, rng: &mut SplitMix64) -> bool`

use super::Tree;
use crate::gp::mechanics::SplitMix64;

/// M1: swap two kids in α; a symmetry node then rewrites β ([`Tree::make_sf`]),
/// which swaps their mates there (Balasa §1.3.3).
pub(crate) fn m1(t: &mut Tree, node: usize, rng: &mut SplitMix64) {
    let (i, j) = two(rng, t.nodes[node].alpha.len());
    t.nodes[node].alpha.swap(i, j);
    t.make_sf(node as u16);
}

/// M2: swap two kids in β; a symmetry node then rewrites α as β reversed
/// through `mate` (the same S-F relation read the other way).
pub(crate) fn m2(t: &mut Tree, node: usize, rng: &mut SplitMix64) {
    let nd = &mut t.nodes[node];
    let (i, j) = two(rng, nd.beta.len());
    nd.beta.swap(i, j);
    if let Some(s) = &nd.sym {
        nd.alpha = nd.beta.iter().rev().map(|&g| s.mate[usize::from(g)]).collect();
    }
}

/// M3: swap two kids in both sequences; in a symmetry node, a pair flip
/// `x ↔ mate[x]` (relabelling by the involution keeps eq 1.1). `false` when
/// the node has no pair.
pub(crate) fn m3(t: &mut Tree, node: usize, rng: &mut SplitMix64) -> bool {
    let nd = &mut t.nodes[node];
    let (a, b) = match &nd.sym {
        Some(s) => {
            let paired: Vec<u16> = (0..s.mate.len() as u16).filter(|&x| s.mate[usize::from(x)] != x).collect();
            if paired.is_empty() {
                return false;
            }
            let x = paired[rng.below(paired.len())];
            (x, s.mate[usize::from(x)])
        }
        None => {
            let (i, j) = two(rng, nd.kids.len());
            (i as u16, j as u16)
        }
    };
    for seq in [&mut nd.alpha, &mut nd.beta] {
        let (i, j) = (seq.iter().position(|&g| g == a).unwrap(), seq.iter().position(|&g| g == b).unwrap());
        seq.swap(i, j);
    }
    true
}

// ---- private ----

/// Two distinct uniform indices below `k` (`k ≥ 2`).
fn two(rng: &mut SplitMix64, k: usize) -> (usize, usize) {
    let i = rng.below(k);
    let j = (i + 1 + rng.below(k - 1)) % k;
    (i, j)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: Balasa's symmetric-feasible condition, checked by `Tree::is_sf` after every random move.
    #[test]
    fn sf_moves_preserve_sf() {
        let mut rng = SplitMix64::new(7);
        for _ in 0..10_000 {
            let (p, s) = (1 + rng.below(4), rng.below(3));
            let mut pairs: Vec<(u32, u32, u16)> = (0..p as u32).map(|i| (2 * i, 2 * i + 1, 0)).collect();
            pairs.extend((2 * p..2 * p + s).map(|c| (c as u32, c as u32, 0)));
            let (mut t, _) = Tree::build(2 * p + s, &pairs, &[]);
            match rng.below(3) {
                0 => m1(&mut t, 0, &mut rng),
                1 => m2(&mut t, 0, &mut rng),
                _ => {
                    m3(&mut t, 0, &mut rng);
                }
            }
            assert!(t.is_sf(0), "{:?}", t.nodes[0]);
        }
    }
}
