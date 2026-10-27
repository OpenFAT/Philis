//! Hands the crate `GPURIFY_REV`, the GPurify commit `flake.lock` pins (the
//! `gpurify` input, Q61): the vendored decks' line 1 must name it
//! (src/pdk.rs `vendored_decks_name_their_origin`).

// ---- private ----

fn main() {
    println!("cargo:rerun-if-changed=../../../flake.lock");
    let lock = std::fs::read_to_string("../../../flake.lock").expect("flake.lock beside the workspace");
    // ponytail: a string scan of the lock's `gpurify` node, not a JSON parse.
    let node = lock.split("\"gpurify\": {").nth(1).expect("flake.lock has a gpurify input");
    let rev = node.split("\"rev\": \"").nth(1).and_then(|r| r.split('"').next()).expect("the gpurify input is locked to a rev");
    println!("cargo:rustc-env=GPURIFY_REV={rev}");
}
