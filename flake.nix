{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        # experimental
        "x86_64-darwin"
        "aarch64-darwin"
      ] (system: function nixpkgs.legacyPackages.${system});

    rev = self.shortRev or self.dirtyShortRev or "dirty";
  in {
    overlays.default = final: prev: {
      magnetz = final.callPackage ./package.nix {
        inherit rev;
      };
    };

    packages = forAllSystems (pkgs: rec {
      magnetz = pkgs.callPackage ./package.nix {
        inherit rev;
      };
      default = magnetz;
    });

    devShells = forAllSystems (pkgs: {
      default = pkgs.callPackage ./devshell.nix {};
    });

    nixosModules.default = import ./module.nix;
  };
}
