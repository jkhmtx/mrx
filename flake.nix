{
  description = "mrx";

  inputs = {
    nixpkgsSrc.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    rustOverlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgsSrc";
    };

    mrx = {
      url = "github:jkhmtx/mrx";
      inputs = {
        mrx.follows = "mrx";
        nixpkgsSrc.follows = "nixpkgsSrc";
        rustOverlay.follows = "rustOverlay";
      };
    };
  };

  outputs = {
    nixpkgsSrc,
    rustOverlay,
    mrx,
    ...
  }: let
    pathAttrImports = {
      _ = import ./mrx.generated.nix;
      infallible = import ./infallible.nix;
    };

    mapSystems = import ./lib/internal/map-systems.nix {
      inherit nixpkgsSrc pathAttrImports rustOverlay;
      upstreamMrx = mrx;
    };
    mkProject = import ./lib/mk-project.nix pathAttrImports;

    systems = mapSystems ["aarch64-darwin" "x86_64-linux"];
  in {
    packages.aarch64-darwin = systems.packages.aarch64-darwin;
    packages.x86_64-linux = systems.packages.x86_64-linux;
    apps.aarch64-darwin = systems.apps.aarch64-darwin;
    apps.x86_64-linux = systems.apps.x86_64-linux;
    inherit mkProject;
  };
}
