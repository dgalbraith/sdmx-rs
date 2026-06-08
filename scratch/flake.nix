# Nix overlay scratch fixture. Used for local package testing and Nix expression dry-runs.
#
# NOTE: This scratchpad intentionally uses `stable.latest.default` (floating latest stable)
# to spike general overlay resolution rather than pinning to the project's exact toolchain
# (via `fromRustupToolchainFile ./rust-toolchain.toml`). If you are trying to verify the exact
# nightly rustfmt derivation or pin to stable 1.91.0, refer to the production `flake.nix` instead.
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, rust-overlay }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ (import rust-overlay) ];
    };
  in {
    # Scratch overlay development target
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = [
        pkgs.rust-bin.stable.latest.default
      ];
    };
  };
}
