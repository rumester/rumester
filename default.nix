{
  lib,
  rustPlatform,
  makeWrapper,
  pkg-config,
  openssl,
  wineWow64Packages,
}: let
  toml = (lib.importTOML ./Cargo.toml).workspace.package;

  wine = wineWow64Packages.staging;
in
  rustPlatform.buildRustPackage {
    pname = toml.name;
    inherit (toml) version;

    src = lib.sources.cleanSource ./.;

    nativeBuildInputs = [
      pkg-config
      makeWrapper
    ];

    buildInputs = [
      openssl
      wine
    ];

    cargoBuildFlags = [ "--workspace" "--bin cli" ];

    cargoLock = {
      outputHashes = {
        "winers-1.0.0" = "sha256-Z28U9ovCunhWNUZp9+nvd7ofwwjbsIPrNBrLR2yLLiM=";
      };
      lockFile = ./Cargo.lock;
    };

    meta.mainProgram = "cli";

    postInstall = ''
      wrapProgram $out/bin/cli \
        --prefix PATH : ${lib.makeBinPath [wine]}
    '';
  }
