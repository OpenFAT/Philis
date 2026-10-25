{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  # ponytail: dev shell only; the packaged `philis` binary is added with the
  # engine (frontend/engine), from Philis 1.0's flake.
  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ (import rust-overlay) ]; };
        rust = pkgs.rust-bin.stable.latest.default.override { extensions = [ "rust-src" "rust-analyzer" "clippy" ]; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.protobuf # protoc, for the fork's `cache` build script (crates/kernel/substrate3/upstream)
            pkgs.pkg-config
            pkgs.cmake
            # Nightly foundry cross-check (TESTING.md, Q42) and post-layout simulation.
            pkgs.klayout
            pkgs.magic-vlsi
            pkgs.netgen-vlsi
            pkgs.ngspice
            pkgs.python312
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # frontend/visualizer (wgpu + winit).
            pkgs.vulkan-loader pkgs.wayland pkgs.libxkbcommon pkgs.libx11 pkgs.libxcursor pkgs.libxrandr pkgs.libxi
          ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          shellHook = pkgs.lib.optionalString pkgs.stdenv.isLinux ''
            export LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          '';
        };
      });
}
