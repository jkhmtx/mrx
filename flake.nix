{
  description = "mrx";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

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
      };
    };
  };

  outputs = {
    nixpkgs,
    rustOverlay,
    mrx,
    ...
  }: let
    mapSystems = import ./lib/internal/map-systems.nix {
      inherit pathAttrImports;
      nixpkgsSrc = nixpkgs;
      rustOverlay = overlay;
      upstreamMrx = mrx;
    };
    mkProject = import ./lib/mk-project.nix pathAttrImports;

    overlay = rustOverlay.overlays.default;

    pathAttrImports = {
      _ = import ./mrx.generated.nix;
    };

    systems = mapSystems ["aarch64-darwin" "x86_64-linux"];
  in
    {
      apps.aarch64-darwin = systems.apps.aarch64-darwin;
      apps.x86_64-linux = systems.apps.x86_64-linux;

      packages.aarch64-darwin = systems.packages.aarch64-darwin;
      packages.x86_64-linux = systems.packages.x86_64-linux;
    }
    // {
      inherit mkProject;
      rustOverlay = overlay;
    };
}
