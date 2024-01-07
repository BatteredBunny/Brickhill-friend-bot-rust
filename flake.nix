{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nightly-rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = [
              nightly-rust
              openssl
              pkg-config
            ];
          };
          packages.default = pkgs.callPackage ./build.nix {nightly-rust = nightly-rust;};
        }
    );
}
