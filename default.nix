{ lib
, rustPlatform
}:
rustPlatform.buildRustPackage {
  pname = "termsnap";
  version = (lib.trivial.importTOML ./Cargo.toml).package.version;
  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
