{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    mrx = {
      url = "path:../../.";
      inputs = {
        mrx.follows = "mrx";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = {
    nixpkgs,
    mrx,
    ...
  }: let
    mrxProjectForSystem = import ./mrx.project.nix {
      inherit mrx;
      nixpkgsSrc = nixpkgs;
    };
  in {
    packages.x86_64-linux._ = mrxProjectForSystem "x86_64-linux";
    packages.aarch64-darwin._ = mrxProjectForSystem "aarch64-darwin";
  };
}
