{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    # GPurify 2.0, the sign-off engine (Q61): built below, `gpurify` on PATH.
    # Its build.zig.zon pins Gompute; its flake.lock pins zig-overlay. Both
    # move with it, on purpose.
    gpurify = { url = "github:OmarSiwy/GPurify/64965421c86730dfc2d06ce1df2a7d4c87330162"; flake = false; };
    gompute = { url = "github:OmarSiwy/Gompute/15ab58137b094db1a82a74f67b0b0cba1c57d7af"; flake = false; };
    zig-overlay = { url = "github:mitchellh/zig-overlay/e7e40b6e759546e3da1182ddc338c165862b5a54"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  # ponytail: dev shell only; the packaged `philis` binary is added with the
  # engine (frontend/engine), from Philis 1.0's flake.
  outputs = { nixpkgs, rust-overlay, flake-utils, gpurify, gompute, zig-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ (import rust-overlay) ]; };
        rust = pkgs.rust-bin.stable.latest.default.override { extensions = [ "rust-src" "rust-analyzer" "clippy" ]; };
        # CPU only (-Dgpu=false: no CUDA/HIP toolchain), baseline ISA so the
        # store path is the same binary on every machine. Gompute is handed
        # over as the package zig would fetch (`--system`: no network here).
        gpurify-bin = pkgs.stdenv.mkDerivation {
          pname = "gpurify";
          version = builtins.substring 0 7 gpurify.rev;
          src = gpurify;
          nativeBuildInputs = [ zig-overlay.packages.${system}.master ];
          dontConfigure = true;
          dontInstall = true;
          buildPhase = ''
            export ZIG_GLOBAL_CACHE_DIR=$TMPDIR/zig-cache
            mkdir -p $TMPDIR/deps
            cp -r ${gompute} $TMPDIR/deps/gompute-1.5.0-YJVc1qjKCAC0sRRUEgn07hJbNHviJzrSf2fHE7XnWN7g
            zig build -Doptimize=ReleaseFast -Dgpu=false -Dcpu=baseline --system $TMPDIR/deps --prefix $out
          '';
        };
      in {
        packages.gpurify = gpurify-bin;
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
            gpurify-bin
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # frontend/visualizer (wgpu + winit).
            pkgs.vulkan-loader pkgs.wayland pkgs.libxkbcommon pkgs.libx11 pkgs.libxcursor pkgs.libxrandr pkgs.libxi
          ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          # GPurify's source at the pinned rev: the upstream decks the vendored
          # ones are diffed against (crates/kernel/verify/src/decks.rs).
          GPURIFY_SRC = "${gpurify}";
          shellHook = pkgs.lib.optionalString pkgs.stdenv.isLinux ''
            export LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          '';
        };
      });
}
