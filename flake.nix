{
  description = "mrx";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    nix-filter.url = "github:numtide/nix-filter";

    rustOverlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    mrx = {
      url = "github:jkhmtx/mrx";
      inputs = {
        mrx.follows = "mrx";
        nixpkgs.follows = "nixpkgs";
        rustOverlay.follows = "rustOverlay";
        nix-filter.follows = "nix-filter";
      };
    };
  };

  outputs = {
    nixpkgs,
    rustOverlay,
    mrx,
    nix-filter,
    ...
  }: let
    mapSystems = import ./lib/internal/map-systems.nix {
      inherit overlays pathAttrImports nixFilter;
      nixpkgsSrc = nixpkgs;
      upstreamMrx = mrx;
    };

    mkProject = import ./lib/mk-project.nix {
      inherit pathAttrImports nixFilter overlays;
    };

    nixFilter = nix-filter.lib;

    overlays = [rustOverlay.overlays.default];

    pathAttrImports = {
      _ = import ./mrx.generated.nix;
    };

    systems = mapSystems ["aarch64-darwin" "x86_64-linux"];
  in {
    inherit mkProject;

    apps.aarch64-darwin = systems.apps.aarch64-darwin;
    apps.x86_64-linux = systems.apps.x86_64-linux;

    packages.aarch64-darwin = systems.packages.aarch64-darwin;
    packages.x86_64-linux = systems.packages.x86_64-linux;
  };
}
