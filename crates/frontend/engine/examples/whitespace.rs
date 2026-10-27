//! M6 spike measurement (`docs/ref/research/compaction-verdict.md`): where the
//! recipe spine's area goes. Per fixture on sky130: tile areas, the floorplan
//! and routed bounding boxes, and per neighbouring pair the drawn gap against
//! the gap the facing shapes actually need (same-layer `min_spacing`, on the
//! cut lattice). `cargo run -p engine --example whitespace [fixture…]`.

use analog::base::{Macro, Rect};

const FIXTURES: &[&str] = &["pair", "quad", "mirror_ratio"];

fn area(r: Rect) -> i64 {
    i64::from(r.w) * i64::from(r.h)
}

fn union(a: Rect, b: Rect) -> Rect {
    let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
    let (x1, y1) = ((a.x + a.w).max(b.x + b.w), (a.y + a.h).max(b.y + b.h));
    Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
}

/// The smallest gap between `a`'s right edge and `b`'s left edge (b slid in
/// x only) at which no same-layer shape pair is closer than its spacing.
/// ponytail: O(shapes²), Manhattan (y-gap < s counts as facing); a spike.
fn needed_gap(a: &Macro, b: &Macro, pdk: &verify::Pdk) -> i32 {
    let (ar, bl) = (a.bbox.x + a.bbox.w, b.bbox.x);
    let mut need = i32::MIN;
    for sa in &a.shapes {
        let Some(s) = pdk.min_spacing(sa.layer.0) else { continue };
        for sb in b.shapes.iter().filter(|sb| sb.layer == sa.layer) {
            let ygap = (sb.rect.y - (sa.rect.y + sa.rect.h)).max(sa.rect.y - (sb.rect.y + sb.rect.h));
            if ygap >= s {
                continue;
            }
            // Gap g between bboxes puts sb.left at ar + g + (sb.x − bl).
            need = need.max(sa.rect.x + sa.rect.w + s - ar - (sb.rect.x - bl));
        }
    }
    need.max(0)
}

fn main() {
    let names: Vec<String> = std::env::args().skip(1).collect();
    let names: Vec<&str> = if names.is_empty() { FIXTURES.to_vec() } else { names.iter().map(String::as_str).collect() };
    let pdk = verify::Pdk::builtin("sky130").expect("sky130 deck");
    let lattice = substrate3::cells::builder::cut_lattice(&pdk).max(1);
    for name in names {
        let path = format!("{}/../../../benchmarks/fixtures/{name}.spice", env!("CARGO_MANIFEST_DIR"));
        let spice = std::fs::read_to_string(&path).expect("fixture");
        let out = match engine::run(&spice, &pdk, verify::SignoffEngine::Gpurify) {
            Ok(o) => o,
            Err(r) => {
                println!("{name}: refused {r}");
                continue;
            }
        };
        let tiles = &out.placed;
        let sum: i64 = tiles.iter().map(|m| area(m.bbox)).sum();
        let plan = tiles.iter().map(|m| m.bbox).reduce(union).expect("a tile");
        let routed = out.routes.wires.iter().flatten().map(|s| s.rect).fold(plan, union);
        let hard = out.signoff.report.hard_violations.iter().filter(|v| !v.rule.starts_with("engine/pex")).count();
        println!(
            "{name}: tiles {} | Σ tile {:.3} µm² | floorplan bbox {}×{} nm = {:.3} µm² | routed bbox {}×{} nm = {:.3} µm² | hard {hard}",
            tiles.len(),
            sum as f64 / 1e6,
            plan.w,
            plan.h,
            area(plan) as f64 / 1e6,
            routed.w,
            routed.h,
            area(routed) as f64 / 1e6,
        );
        for (i, m) in tiles.iter().enumerate() {
            println!("  tile {i}: {:?} {}×{} nm, {} shapes", out.calls.kind[i], m.bbox.w, m.bbox.h, m.shapes.len());
        }
        let mut order: Vec<usize> = (0..tiles.len()).collect();
        order.sort_by_key(|&i| tiles[i].bbox.x);
        let mut recoverable = 0;
        for w in order.windows(2) {
            let (a, b) = (&tiles[w[0]], &tiles[w[1]]);
            let drawn = b.bbox.x - (a.bbox.x + a.bbox.w);
            let need = (needed_gap(a, b, &pdk) + lattice - 1) / lattice * lattice;
            recoverable += drawn - need;
            println!("  gap {}→{}: drawn {drawn} nm, needed {need} nm", w[0], w[1]);
        }
        if tiles.len() > 1 {
            println!(
                "  x recoverable by 1-D compaction: {recoverable} nm of {} nm ({:.1} %)",
                plan.w,
                100.0 * f64::from(recoverable) / f64::from(plan.w)
            );
        }
    }
}
