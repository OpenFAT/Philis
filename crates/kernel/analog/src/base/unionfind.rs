//! Union-find over device indices: rules' `extract` unions matched devices, and
//! the annotator reads the sets back as the group table.
//! In → out: `union` pairs over `0..n` → the non-singleton sets. Cites: Tarjan's
//! union by rank with path compression (no paper clause beyond the textbook form).
//!
//! Contents:
//! - `pub struct UnionFind`
//! - `UnionFind::new(n: usize) -> Self`
//! - `UnionFind::find(&mut self, x: u32) -> u32`
//! - `UnionFind::union(&mut self, a: u32, b: u32)`
//! - `UnionFind::groups(&mut self) -> Vec<Vec<u32>>`

/// Disjoint-set forest with path compression + union by rank.
pub struct UnionFind {
    parent: Vec<u32>,
    rank: Vec<u8>,
}

impl UnionFind {
    /// A forest of `n` singletons.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self { parent: (0..n as u32).collect(), rank: vec![0; n] }
    }

    /// Representative of `x`'s set (path-compressed).
    pub fn find(&mut self, x: u32) -> u32 {
        let mut root = x;
        while self.parent[root as usize] != root {
            root = self.parent[root as usize];
        }
        let mut cur = x;
        while self.parent[cur as usize] != root {
            let next = self.parent[cur as usize];
            self.parent[cur as usize] = root;
            cur = next;
        }
        root
    }

    /// Merge the sets of `a` and `b`.
    pub fn union(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        let (lo, hi) = if self.rank[ra as usize] < self.rank[rb as usize] { (ra, rb) } else { (rb, ra) };
        self.parent[lo as usize] = hi;
        if self.rank[lo as usize] == self.rank[hi as usize] {
            self.rank[hi as usize] += 1;
        }
    }

    /// The non-singleton sets, each as a sorted member list, ordered by their
    /// smallest member: the same on every run and thread (AGENTS.md
    /// determinism). Singletons (devices in no group) are omitted — they stay
    /// addressable as `Target::Device`.
    pub fn groups(&mut self) -> Vec<Vec<u32>> {
        let n = self.parent.len();
        let mut by_root: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
        for i in 0..n as u32 {
            let r = self.find(i);
            by_root.entry(r).or_default().push(i);
        }
        // Ascending `i` pushed each set sorted; the sets come out in hash order.
        let mut out: Vec<Vec<u32>> = by_root.into_values().filter(|s| s.len() > 1).collect();
        out.sort_unstable_by_key(|s| s[0]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: spec example, the sets by hand, ordered by their smallest
    /// member. Forty fresh forests (each `groups` call a fresh hasher seed
    /// before the fix) must all give the one order.
    #[test]
    fn groups_come_in_smallest_member_order_on_every_run() {
        // Sets {0, 9}, {1, 5, 7}, {2, 3}, {4, 8}, {10, 11}, ... {38, 39}; 6 a singleton.
        let pairs: Vec<(u32, u32)> =
            [(9, 0), (7, 5), (5, 1), (3, 2), (8, 4), (11, 10)].into_iter().chain((12..40).step_by(2).map(|a| (a + 1, a))).collect();
        let mut want: Vec<Vec<u32>> = vec![vec![0, 9], vec![1, 5, 7], vec![2, 3], vec![4, 8], vec![10, 11]];
        want.extend((12..40).step_by(2).map(|a| vec![a, a + 1]));
        for _ in 0..40 {
            let mut uf = UnionFind::new(40);
            pairs.iter().for_each(|&(a, b)| uf.union(a, b));
            assert_eq!(uf.groups(), want);
        }
    }
}
