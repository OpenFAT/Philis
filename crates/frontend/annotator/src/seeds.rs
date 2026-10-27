//! Symmetry seeds for [`crate::symmetry::analyze`]: recognised blocks, user
//! seeds and identical instance pairs → the ordered seed list, user seeds first
//! (EXT-26), then the disjoint DiffPair/Load/CascodePair leaves, then the
//! couples of instance pairs whose ports pair up (EXT-27).
//!
//! Contents:
//! - `pub(crate) fn seeds(netlist: &Netlist, drawn: &[Drawn], blocks: &[Block], canon: &[u64], names: &[&str], user: &[Seed]) -> (Vec<Seed>, Vec<(DeviceId, DeviceId)>)`

use std::collections::HashSet;

use analog::base::ids::DeviceId;
use analog::base::Netlist;
use analog::intent::ConstraintId;

use crate::block::{self, Block, BlockKind};
use crate::hier;
use crate::size::Drawn;
use crate::symmetry::Seed;

/// The seed list, and every corresponding device couple of identical
/// instance pairs (ports paired or not) for the proximity graph. Leaf seeds
/// are ordered by `(canon, name)`, so the list is permutation-invariant;
/// instance couples take ids from `u32::MAX / 2` up.
pub(crate) fn seeds(
    netlist: &Netlist,
    drawn: &[Drawn],
    blocks: &[Block],
    canon: &[u64],
    names: &[&str],
    user: &[Seed],
) -> (Vec<Seed>, Vec<(DeviceId, DeviceId)>) {
    // Symmetry seeds: the disjoint DiffPair/Load/CascodePair leaves (never contradictory),
    // each couple and the list in canonical order, names breaking exact ties.
    let ck = |d: DeviceId| (canon[d.0 as usize], names[d.0 as usize]);
    let mut seeds: Vec<(DeviceId, DeviceId, usize)> = block::leaves(blocks)
        .iter()
        .enumerate()
        .filter(|(_, b)| matches!(b.kind, BlockKind::DiffPair | BlockKind::Load | BlockKind::CascodePair))
        .map(|(i, b)| if ck(b.devices[0]) <= ck(b.devices[1]) { (b.devices[0], b.devices[1], i) } else { (b.devices[1], b.devices[0], i) })
        .collect();
    seeds.sort_by_key(|&(a, b, _)| (ck(a), ck(b)));
    // User seeds first (EXT-26).
    let mut seeds: Vec<Seed> = user.iter().copied().chain(seeds.into_iter().map(|(a, b, i)| Seed::Devices(a, b, ConstraintId(i as u32)))).collect();
    // EXT-27: couples of identical instance pairs whose ports pair up, after the leaf seeds;
    // a device already seeded (leaf symmetry inside an instance) keeps its own couple.
    let inst_pairs = hier::same_template(netlist, drawn);
    let hier_couples: Vec<(DeviceId, DeviceId)> =
        inst_pairs.iter().flat_map(|&(a, b)| hier::corresponding(netlist, drawn, a, b).unwrap_or_default()).collect();
    let seeded: HashSet<DeviceId> = seeds
        .iter()
        .flat_map(|s| match *s {
            Seed::Devices(a, b, _) => vec![a, b],
            Seed::SelfDevice(a, _) => vec![a],
            Seed::Nets(..) => vec![],
        })
        .collect();
    let mut k = 0;
    for &(a, b) in &inst_pairs {
        if hier::ports_pair(netlist, a, b) {
            for (x, y) in hier::corresponding(netlist, drawn, a, b).unwrap_or_default() {
                if !seeded.contains(&x) && !seeded.contains(&y) {
                    seeds.push(Seed::Devices(x, y, ConstraintId(u32::MAX / 2 + k)));
                    k += 1;
                }
            }
        }
    }
    (seeds, hier_couples)
}
