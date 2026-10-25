//! The forbidden dependency edges of ARCHITECTURE.md's module DAG, over
//! `cargo metadata`: the visualizer never links the GPurify engine (it may take
//! `gpurify-geom`), and `analog` never links Substrate (`substrate3`, or any
//! crate of the fork under `crates/kernel/substrate3/upstream`). Transitive,
//! normal edges only; dev and build edges may cross.

use std::collections::HashMap;
use std::process::Command;

/// A package: its name and its manifest path.
type Node = (String, String);

/// The first path from `from` to a node `banned` accepts, walking `edges`
/// (indices into `nodes`) depth-first; `None` when every reachable node is
/// allowed. Returns names, `from` first.
fn forbidden_path(nodes: &[Node], edges: &[(usize, usize)], from: usize, banned: impl Fn(&Node) -> bool) -> Option<Vec<String>> {
    let mut parent: Vec<Option<usize>> = vec![None; nodes.len()];
    let mut seen = vec![false; nodes.len()];
    let mut stack = vec![from];
    seen[from] = true;
    while let Some(n) = stack.pop() {
        if n != from && banned(&nodes[n]) {
            let mut path = vec![nodes[n].0.clone()];
            let mut at = n;
            while let Some(p) = parent[at] {
                path.push(nodes[p].0.clone());
                at = p;
            }
            path.reverse();
            return Some(path);
        }
        for &(a, b) in edges.iter().filter(|e| e.0 == n) {
            if !seen[b] {
                seen[b] = true;
                parent[b] = Some(a);
                stack.push(b);
            }
        }
    }
    None
}

/// The workspace's resolved graph: every package and its normal edges.
fn workspace_graph() -> (Vec<Node>, Vec<(usize, usize)>) {
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--manifest-path", concat!(env!("CARGO_MANIFEST_DIR"), "/../../../Cargo.toml")])
        .output()
        .expect("cargo metadata runs");
    assert!(out.status.success(), "cargo metadata: {}", String::from_utf8_lossy(&out.stderr));
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).expect("cargo metadata is JSON");
    let mut index = HashMap::new();
    let nodes: Vec<Node> = meta["packages"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, p)| {
            index.insert(p["id"].as_str().unwrap().to_owned(), i);
            (p["name"].as_str().unwrap().to_owned(), p["manifest_path"].as_str().unwrap().to_owned())
        })
        .collect();
    let mut edges = Vec::new();
    for n in meta["resolve"]["nodes"].as_array().unwrap() {
        let a = index[n["id"].as_str().unwrap()];
        for d in n["deps"].as_array().unwrap() {
            // A null kind is a normal edge.
            if d["dep_kinds"].as_array().unwrap().iter().any(|k| k["kind"].is_null()) {
                edges.push((a, index[d["pkg"].as_str().unwrap()]));
            }
        }
    }
    (nodes, edges)
}

fn is_gpurify_engine(n: &Node) -> bool {
    n.0 == "gpurify"
}

fn is_substrate(n: &Node) -> bool {
    n.0 == "substrate" || n.0 == "substrate3" || n.1.contains("/crates/kernel/substrate3/upstream/")
}

fn check(from: &str, banned: fn(&Node) -> bool) {
    let (nodes, edges) = workspace_graph();
    let at = nodes.iter().position(|n| n.0 == from && n.1.contains("/crates/")).unwrap_or_else(|| panic!("{from} is a workspace member"));
    if let Some(path) = forbidden_path(&nodes, &edges, at, banned) {
        panic!("forbidden edge (ARCHITECTURE.md, Module DAG): {}", path.join(" -> "));
    }
}

#[test]
fn the_visualizer_never_links_the_gpurify_engine() {
    check("visualizer", is_gpurify_engine);
}

#[test]
fn analog_never_links_substrate() {
    check("analog", is_substrate);
}

/// The negative control: the checker finds a transitive banned node, ignores
/// the allowed geometry crate, and names the path.
#[test]
fn the_checker_finds_a_transitive_forbidden_edge() {
    let node = |name: &str, path: &str| (name.to_owned(), path.to_owned());
    let nodes = [
        node("visualizer", "/r/crates/frontend/visualizer/Cargo.toml"),
        node("gpurify-geom", "/git/gpurify/geom/Cargo.toml"),
        node("helper", "/reg/helper/Cargo.toml"),
        node("gpurify", "/git/gpurify/Cargo.toml"),
        node("analog", "/r/crates/kernel/analog/Cargo.toml"),
        node("atoll", "/r/crates/kernel/substrate3/upstream/libs/atoll/Cargo.toml"),
    ];
    let clean = [(0, 1), (0, 2)];
    assert_eq!(forbidden_path(&nodes, &clean, 0, is_gpurify_engine), None);
    let dirty = [(0, 1), (0, 2), (2, 3)];
    assert_eq!(forbidden_path(&nodes, &dirty, 0, is_gpurify_engine).unwrap(), ["visualizer", "helper", "gpurify"]);
    assert_eq!(forbidden_path(&nodes, &[(4, 2)], 4, is_substrate), None);
    assert_eq!(forbidden_path(&nodes, &[(4, 2), (2, 5)], 4, is_substrate).unwrap(), ["analog", "helper", "atoll"]);
}
