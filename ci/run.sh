#!/usr/bin/env bash
# The four gate tiers of docs/TESTING.md ("Gates"): ci/run.sh local|merge|nightly|release.
# Runs on the Philis crates only, never the vendored Substrate2 members.
# Every step runs; the exit status is non-zero if any step failed.
set -u
cd "$(dirname "$0")/.."
export PROTOC="${PROTOC:-/nix/store/armsfbcsbzax7pnkim9cqgkx8n747n31-protobuf-34.1/bin/protoc}"

crates=(analog verify annotator visualizer placement routing substrate3 engine)
pkgs=()
for c in "${crates[@]}"; do pkgs+=(-p "$c"); done

failed=()
step() {
  echo "::group::$*"
  "$@" || failed+=("$*")
  echo "::endgroup::"
}

case "${1:-}" in
  local)
    # ponytail: report only; the tree is not rustfmt-clean yet, so fmt never fails the tier.
    n=$(cargo fmt --check "${pkgs[@]}" -- -l 2>/dev/null | grep -c '\.rs$')
    echo "cargo fmt --check: $n unformatted files (report only, not gating)"
    step cargo clippy "${pkgs[@]}" --all-targets
    # substrate3's lib tests draw and DRC every device variant (~160 s): merge.
    fast=()
    for c in "${crates[@]}"; do [ "$c" != substrate3 ] && fast+=(-p "$c"); done
    step cargo test "${fast[@]}" --lib --no-fail-fast
    ;;
  merge)
    # Every lib, unit and integration test: engine's pair, conformance and deps included.
    step cargo test "${pkgs[@]}" --no-fail-fast
    ;;
  nightly)
    step cargo test "${pkgs[@]}" --no-fail-fast
    step cargo test "${pkgs[@]}" --no-fail-fast -- --ignored
    ;;
  release)
    step cargo test --release "${pkgs[@]}" --no-fail-fast
    step cargo test --release "${pkgs[@]}" --no-fail-fast -- --ignored
    ;;
  *)
    echo "usage: ci/run.sh local|merge|nightly|release" >&2
    exit 2
    ;;
esac

if ((${#failed[@]})); then
  printf 'FAILED: %s\n' "${failed[@]}"
  exit 1
fi
echo "tier ${1} passed"
