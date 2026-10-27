//! Claims are scored, never trusted (Q2): every claim `pair`'s and `quad`'s
//! recipes declare is met by the drawn units on sky130, as are the centroid
//! recipes' claims on `pair` at two and four fingers, and a seeded defect
//! (one unit of one member mirrored onto the other axis) violates the
//! orientation claim. Oracle: `kernel/analog`'s rules on the tiles' `Unit`
//! rows, computed independently of the recipe.

use engine::claims::{Claims, Status};

const PAIR: &str = include_str!("../../../../benchmarks/fixtures/pair.spice");
const QUAD: &str = include_str!("../../../../benchmarks/fixtures/quad.spice");

/// The spine up to `arrange`, then the claims on its tiles; `defect` edits
/// the tiles first. `kind`: one call of that recipe over every device, in
/// place of `plan_recipes`'s.
fn scored(spice: &str, kind: Option<engine::RecipeKind>, defect: impl FnOnce(&mut [analog::base::Macro])) -> Claims {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let netlist = engine::parse(spice, &pdk).expect("parses");
    let problem = annotator::annotate(&netlist.flat, &engine::annotation(&pdk));
    let calls = match kind {
        None => engine::plan_recipes(&problem, &netlist.flat, &pdk).expect("plans"),
        Some(kind) => {
            let members: Vec<_> = (0..netlist.flat.devices.len()).map(|i| analog::base::DeviceId(i as u16)).collect();
            let mut calls = engine::RecipeCalls::default();
            calls.push(kind, &members, Some(0));
            calls
        }
    };
    let mut tiles = engine::build_tiles(&calls, &problem, &netlist.flat, &pdk).expect("draws");
    let layout = engine::arrange(&mut tiles, &pdk).expect("arranges");
    tiles.truncate(calls.len());
    defect(&mut tiles);
    engine::claims::score_claims(&calls, &tiles, &layout, &problem, &pdk)
}

/// `pair` at `nf` fingers per member, each finger the original 2 µm.
fn pair_nf(nf: u8) -> String {
    PAIR.replace("W=2u L=0.5u", &format!("W={}u L=0.5u nf={nf}", 2 * nf))
}

fn met(spice: &str, kind: Option<engine::RecipeKind>) {
    let claims = scored(spice, kind, |_| {});
    print!("{claims}");
    assert!(!claims.is_empty(), "the set's call declares claims");
    assert!(claims.status.iter().all(|&s| matches!(s, Status::Met | Status::Reported)), "{claims}");
}

/// Member 1's first unit transposed (mirrored about the diagonal): its
/// channel now runs along y, its Φ on the other axis.
fn mirror_one_unit(tiles: &mut [analog::base::Macro]) {
    let u = tiles[0].units.iter_mut().find(|u| u.owner == 1).expect("member 1 has units");
    (u.x, u.y, u.phi) = (u.y, u.x, (u.phi.1, u.phi.0));
}

fn violated(spice: &str, kind: Option<engine::RecipeKind>) {
    let claims = scored(spice, kind, mirror_one_unit);
    print!("{claims}");
    let orientation = (0..claims.len()).filter(|&i| claims.rule[i] == analog::RuleKind::OrientationSet).collect::<Vec<_>>();
    assert!(!orientation.is_empty(), "every MOS recipe claims an orientation");
    assert!(orientation.iter().all(|&i| claims.status[i] == Status::Violated), "{claims}");
}

#[test]
fn pair_claims_are_met_on_sky130() {
    met(PAIR, None);
}

#[test]
fn quad_claims_are_met_on_sky130() {
    met(QUAD, None);
}

#[test]
fn centroid_claims_are_met_on_sky130() {
    met(&pair_nf(2), Some(engine::RecipeKind::MosCc1d));
    met(&pair_nf(4), Some(engine::RecipeKind::MosCc1d));
    met(&pair_nf(4), Some(engine::RecipeKind::MosCc2d));
}

#[test]
fn a_mirrored_unit_violates_pairs_orientation_claim() {
    violated(PAIR, None);
}

#[test]
fn a_mirrored_unit_violates_quads_orientation_claim() {
    violated(QUAD, None);
}

/// On a centroid row the same defect also moves member 1's centroid.
#[test]
fn a_mirrored_unit_violates_the_centroid_claims() {
    violated(&pair_nf(2), Some(engine::RecipeKind::MosCc1d));
    let claims = scored(&pair_nf(2), Some(engine::RecipeKind::MosCc1d), mirror_one_unit);
    for p in [engine::claims::Promise::Coincidence, engine::claims::Promise::MomentsCancelled(1)] {
        let i = claims.promised.iter().position(|&q| q == p).expect("Cc1d claims it");
        assert_eq!(claims.status[i], Status::Violated, "{claims}");
    }
    violated(&pair_nf(4), Some(engine::RecipeKind::MosCc2d));
}

/// The full spine carries the same table into `Outcome` and the report.
#[test]
fn the_report_lists_the_claims() {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let out = engine::run(PAIR, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{r}"));
    assert_eq!(out.claims, scored(PAIR, None, |_| {}));
    assert!(out.report.contains("claim call 0"), "{}", out.report);
}
