{
  description = "Development shell for the freestanding Wasmtime sample";

  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay?ref=stable";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";

  outputs =
    { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      perSystem = { pkgs, system, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };
        devShells.default = pkgs.mkShell {
          packages = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-analyzer" ];
              targets = [
                "aarch64-unknown-none"
                "riscv64gc-unknown-none-elf"
                "wasm32-unknown-unknown"
                "wasm32-wasip1"
                "wasm32-wasip2"
              ];
            })
            pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc
            pkgs.qemu
            pkgs.wabt
          ];
          # The cross compiler's setup hook otherwise makes it the default C
          # compiler, which breaks normal host builds in this shell.
          shellHook = ''
            export CC=cc
            export CXX=c++
            export AR=ar
          '';
        };
      };
    };
}
