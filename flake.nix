{
  description = "context";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      rust-overlay,
    }:
    utils.lib.eachSystem [ "x86_64-linux" ] (
      system:
      let
        # overlays
        overlays = [ (import rust-overlay) ];

        # packages for the given system
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # override here
        rust = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "cargo"
            "rustc"
          ];
        };
      in
      {
        # Package build definition (for `nix build`)
        packages.default = pkgs.callPackage ./default.nix {
          inherit rust;
        };

        # Development environment (for `nix develop`)
        devShells.default = pkgs.callPackage ./shell.nix {
          inherit rust;
        };
      }
    );
}
