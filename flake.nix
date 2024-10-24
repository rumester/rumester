{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = {nixpkgs, ...}: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
        system: function nixpkgs.legacyPackages.${system}
      );
  in {
    packages = forAllSystems (pkgs: rec {
      cli = pkgs.callPackage ./cli/default.nix {};
      default = pkgs.symlinkJoin {
        name = "rumester";
        paths = [
          cli
        ];
      };
    });

    devShells = forAllSystems (pkgs: {
      cli = pkgs.callPackage ./cli/shell.nix {};
    });

    formatter = forAllSystems (pkgs: pkgs.alejandra);
  };
}
