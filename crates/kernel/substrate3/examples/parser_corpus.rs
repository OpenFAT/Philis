use substrate3::scir::ChildId;
use substrate3::{scir, spice};
use std::collections::HashSet;

fn leaves(lib: &scir::Library<spice::Spice>, id: scir::CellId, depth: u32) -> usize {
    if depth > 64 { return 0; }
    lib.cell(id).instances().map(|(_, i)| match i.child() {
        ChildId::Cell(c) => leaves(lib, c, depth + 1),
        ChildId::Primitive(_) => 1,
    }).sum()
}

fn main() {
    for path in std::io::stdin().lines().map(Result::unwrap) {
        let r = std::panic::catch_unwind(|| {
            let text = std::fs::read_to_string(&path).map_err(|e| format!("read: {e}"))?;
            let parsed = spice::parser::Parser::parse(spice::parser::Dialect::Spice, text.as_str()).map_err(|e| format!("parse: {e}"))?;
            let lib = parsed.to_scir().map_err(|e| format!("conv: {e}"))?;
            let ids: Vec<_> = lib.cells().map(|(id, _)| id).collect();
            let children: HashSet<_> = lib.cells().flat_map(|(_, c)| c.instances().filter_map(|(_, i)| match i.child() { ChildId::Cell(c) => Some(c), _ => None }).collect::<Vec<_>>()).collect();
            let roots: Vec<_> = ids.iter().filter(|i| !children.contains(i)).copied().collect();
            let top = roots.last().copied().or(ids.last().copied()).ok_or("no cell")?;
            Ok::<_, String>(leaves(&lib, top, 0))
        });
        match r {
            Ok(Ok(n)) => println!("OK\t{n}\t{path}"),
            Ok(Err(e)) => println!("ERR\t{}\t{path}", e.lines().next().unwrap_or("").replace('\t', " ")),
            Err(_) => println!("PANIC\t-\t{path}"),
        }
    }
}
