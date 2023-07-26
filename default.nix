{
  src ? ./.,
  rustPlatform,
}: let
  cargo-toml = builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"));
in
  rustPlatform.buildRustPackage {
    inherit src;
    pname = cargo-toml.package.name;
    inherit (cargo-toml.package) version;
    cargoLock.lockFile = src + "/Cargo.lock";
  }
