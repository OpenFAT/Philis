//! M1's gate (ROADMAP): `mirror_ratio` and `quad` through the recipe spine
//! are DRC 0, ERC 0 and LVS MATCH on sky130 and asap7, each set drawn by the
//! first rung of its ladder the deck can draw (`engine::ladder`).
//!
//! MOS-Ratioed: `mirror_ratio` is a bias mirror, Minimal by its role default
//! (Q30), whose ladder is Adjacent alone (`RECIPES.md` §1). Raised to
//! Moderate by the user (a sidecar `Match`), it climbs Ratioed first: one
//! common-centroid row from one unit, its claims met by the scorer, signed
//! off on both proof PDKs; so does the 1:2:4 drawn as one finger per member
//! (unequal fingers, one unit). Negative control: one member off the unit
//! (no unit the deck draws divides every member) is never Ratioed.

const MIRROR_RATIO: &str = include_str!("../../../../benchmarks/fixtures/mirror_ratio.spice");
const QUAD: &str = include_str!("../../../../benchmarks/fixtures/quad.spice");

/// The circuit at asap7's names and sizes: `nmos_rvt`, 20 nm gates, widths
/// as fin multiples (270 nm = 10 fins per µm-class finger).
fn on_asap7(spice: &str) -> String {
    spice
        .replace("nfet_01v8", "nmos_rvt")
        .replace("L=1u", "L=20n")
        .replace("L=0.5u", "L=20n")
        .replace("W=2u", "W=270n")
        .replace("W=4u", "W=540n")
        .replace("W=8u", "W=1080n")
}

/// Every hard finding but PEX (signoff-only, Q29), and LVS over every device.
fn findings(s: &verify::Signoff) -> Vec<String> {
    let mut out: Vec<String> = s.report.hard_violations.iter().map(|v| v.rule.clone()).filter(|r| !r.starts_with("engine/pex")).collect();
    out.extend(s.coverage.skipped_rules.iter().filter(|(r, _)| r == "lvs").map(|(r, why)| format!("{r} skipped: {why}")));
    out.extend(s.coverage.unverified.iter().map(|(k, m, n)| format!("unverified {n} {k:?} {m:?}")));
    out
}

fn signs_off(deck: &str, spice: &str) -> engine::Outcome {
    let pdk = verify::Pdk::builtin(deck).expect("a built-in deck");
    let out = engine::run(spice, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{deck}: {r}"));
    println!("{deck}:\n{}", out.report);
    assert!(findings(&out.signoff).is_empty(), "{deck}: {:?}\n{}", findings(&out.signoff), out.report);
    out
}

#[test]
fn mirror_ratio_signs_off_on_sky130() {
    signs_off("sky130", MIRROR_RATIO);
}

#[test]
fn mirror_ratio_signs_off_on_asap7() {
    signs_off("asap7", &on_asap7(MIRROR_RATIO));
}

#[test]
fn quad_signs_off_on_sky130() {
    signs_off("sky130", QUAD);
}

#[test]
fn quad_signs_off_on_asap7() {
    signs_off("asap7", &on_asap7(QUAD));
}

/// Two unrelated devices are two calls, so two tiles: on a fin deck the glue
/// row keeps the exact gate and fin pitches across them (it used to refuse).
#[test]
fn two_tiles_glue_on_asap7() {
    let spice = ".subckt two a b ga gb VSS\nXM1 a ga VSS VSS nmos_rvt W=270n L=20n\nXM2 b gb VSS VSS nmos_rvt W=540n L=20n nf=2\n.ends two\n";
    let out = signs_off("asap7", spice);
    assert_eq!(out.calls.len(), 2, "{}", out.report);
}

/// The plan for `spice` with every matched set raised to Moderate Current by
/// the user, as a sidecar `Match` does.
fn moderate(deck: &str, spice: &str) -> (verify::Pdk, engine::Netlist, annotator::Problem, annotator::AnnotationConfig, engine::RecipeCalls) {
    let pdk = verify::Pdk::builtin(deck).expect("a built-in deck");
    let netlist = engine::parse(spice, &pdk).unwrap_or_else(|r| panic!("{deck}: {r}"));
    let mut cfg = engine::annotation(&pdk);
    let all: Vec<_> = (0..netlist.flat.devices.len()).map(|i| analog::base::DeviceId(i as u16)).collect();
    cfg.classes.push((all, analog::intent::MatchClass::Moderate, Some(analog::intent::MatchKind::Current)));
    let problem = annotator::annotate(&netlist.flat, &cfg);
    let calls = engine::plan_recipes(&problem, &netlist.flat, &pdk).unwrap_or_else(|r| panic!("{deck}: {r}"));
    (pdk, netlist, problem, cfg, calls)
}

/// `spice` at Moderate is one MosRatioed call over all three devices, at
/// ladder step 0, every claim met, and it signs off.
fn ratioed_signs_off(deck: &str, spice: &str) {
    let (pdk, netlist, problem, cfg, calls) = moderate(deck, spice);
    assert_eq!(calls.kind, [engine::RecipeKind::MosRatioed], "{deck}: {calls:?}");
    assert_eq!((calls.members_of(0).len(), calls.ladder_step[0]), (3, 0), "{deck}: {calls:?}");
    let out = engine::run_calls(&netlist, &problem, &cfg, calls, &pdk, verify::SignoffEngine::Gpurify).unwrap_or_else(|r| panic!("{deck}: {r}"));
    println!("{deck}:\n{}", out.report);
    assert!(findings(&out.signoff).is_empty(), "{deck}: {:?}\n{}", findings(&out.signoff), out.report);
    assert!(!out.claims.is_empty(), "{deck}: Ratioed declares claims");
    assert!(out.claims.status.iter().all(|&s| s == engine::claims::Status::Met), "{deck}:\n{}", out.claims);
}

/// `mirror_ratio` with each member one finger of its whole W.
fn one_finger_each(spice: &str) -> String {
    spice.replace(" nf=2", "").replace(" nf=4", "").replace(" nf=8", "")
}

/// Oracle: `RECIPES.md` §1, a Minimal Current set's ladder is Adjacent alone.
#[test]
fn mirror_ratio_at_its_role_default_stays_adjacent() {
    let pdk = verify::Pdk::builtin("sky130").expect("a built-in deck");
    let netlist = engine::parse(MIRROR_RATIO, &pdk).expect("parses");
    let problem = annotator::annotate(&netlist.flat, &engine::annotation(&pdk));
    let calls = engine::plan_recipes(&problem, &netlist.flat, &pdk).expect("plans");
    assert_eq!(calls.kind, [engine::RecipeKind::MosAdjacent], "{calls:?}");
}

/// Oracle: GPurify signoff on the real deck and kernel/analog's claim rules
/// over the drawn units, neither of which reads the recipe.
#[test]
fn mirror_ratio_moderate_is_ratioed_on_sky130() {
    ratioed_signs_off("sky130", MIRROR_RATIO);
}

/// Oracle: GPurify signoff on the real deck and kernel/analog's claim rules
/// over the drawn units, neither of which reads the recipe.
#[test]
fn mirror_ratio_moderate_is_ratioed_on_asap7() {
    ratioed_signs_off("asap7", &on_asap7(MIRROR_RATIO));
}

/// Oracle: GPurify signoff on the real deck and kernel/analog's claim rules
/// over the drawn units, neither of which reads the recipe.
#[test]
fn unequal_fingers_draw_from_one_unit_on_sky130() {
    ratioed_signs_off("sky130", &one_finger_each(MIRROR_RATIO));
}

/// Oracle: GPurify signoff on the real deck and kernel/analog's claim rules
/// over the drawn units, neither of which reads the recipe.
#[test]
fn unequal_fingers_draw_from_one_unit_on_asap7() {
    ratioed_signs_off("asap7", &on_asap7(&one_finger_each(MIRROR_RATIO)));
}

/// One member off the unit: no call is Ratioed. sky130: M2's 1025 nm fingers
/// leave a 25 nm gcd, under the 420 nm finger minimum. asap7: M2 at 7 fins
/// against 5 leaves one fin, with M2 an odd count of it (no half fin).
/// Oracle: the gcd worked by hand above against each deck's finger minimum.
#[test]
fn a_member_off_the_unit_is_not_ratioed() {
    for (deck, spice) in [
        ("sky130", MIRROR_RATIO.replace("W=4u L=1u nf=4", "W=4.1u L=1u nf=4")),
        ("asap7", on_asap7(MIRROR_RATIO).replace("W=540n L=20n nf=4", "W=567n L=20n nf=3")),
    ] {
        let (_, _, problem, _, calls) = moderate(deck, &spice);
        assert!(problem.intent.sets.iter().any(|s| s.members.len() == 3), "{deck}: the annotator still sees the 3-member set");
        assert!(!calls.kind.contains(&engine::RecipeKind::MosRatioed), "{deck}: {calls:?}");
    }
}
