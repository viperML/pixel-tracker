{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    nixpkgs,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {pkgs, ...}: {
        devShells.default = with pkgs; mkShell.override {
          stdenv = stdenvAdapters.useMoldLinker pkgs.stdenv;
        } {
          packages = [
            cargo
            rustc
            rust-analyzer-unwrapped
            rustfmt
          ];
          RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
        };
      };
    };
}
