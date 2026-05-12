{ pkgs, makeRustPlatform, rust, ... }:

let
  cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
  rustPlatform = makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;
  src = pkgs.lib.cleanSource ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
