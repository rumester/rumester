{
  lib,
  rustPlatform,
  makeWrapper,
  pkg-config,
  openssl,
  wine64Packages,
}: let
  toml = (lib.importTOML ./Cargo.toml).workspace.package;

  wine = wine64Packages.staging;
in
  rustPlatform.buildRustPackage rec {
    pname = toml.name;
    inherit (toml) version;

    src = lib.sources.cleanSource ./.;

    nativeBuildInputs = [
      pkg-config
      makeWrapper
    ];

    buildInputs = [
      openssl
    ];

    cargoBuildFlags = [ "--workspace" "--bin cli" ];

    cargoLock = {
      outputHashes = {
        "winers-0.0.1" = "sha256-emIDaSr+mnhN/PVvsCvZ2+KK61YUnGoQeuguBWz6jvw=";
      };
      lockFile = ./Cargo.lock;
    };

    meta.mainProgram = pname;

    postInstall = ''
      wrapProgram $out/bin/cli \
        --prefix PATH : ${lib.makeBinPath [wine]}
    '';
  }
