{ lib
, rustPlatform
}:
rustPlatform.buildRustPackage {
  pname = "termsnap";
  version = (lib.trivial.importTOML ./Cargo.toml).workspace.package.version;
  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  cargoTestFlags = [ "--workspace" ];
}
