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
      common = pkgs.callPackage ./common/default.nix {};
      cli = pkgs.callPackage ./cli/default.nix {};
      default = pkgs.symlinkJoin {
        name = "rumester";
        paths = [
          common
          cli
        ];
      };
    });

    devShells = forAllSystems (pkgs: {
      common = pkgs.callPackage ./common/shell.nix {};
      cli = pkgs.callPackage ./cli/shell.nix {};
    });

    formatter = forAllSystems (pkgs: pkgs.alejandra);
  };
}
