{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
}: let
  toml = (lib.importTOML ./Cargo.toml).workspace.package;
in
  rustPlatform.buildRustPackage rec {
    pname = toml.name;
    inherit (toml) version;

    src = lib.sources.cleanSource ./.;

    nativeBuildInputs = [
      pkg-config
    ];

    buildInputs = [
      openssl
    ];

    cargoBuildFlags = [ "--workspace" "--bin cli" ];

    cargoLock = {
      outputHashes = {
        "winers-0.0.1" = "sha256-6VjLSNc08DK3AGvLRFLhP83q3aTwkniwxp7iYyW+VGs=";
      };
      lockFile = ./Cargo.lock;
    };

    meta.mainProgram = pname;
  }
