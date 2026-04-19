{
  nixFilter,
  nixpkgsSrc,
  overlays,
  pathAttrImports,
  upstreamMrx,
  ...
}: let
  mkProjectWith = import ../mk-project-with.nix;

  mapSystems = systems: let
    mkProject' = system: let
      nixpkgs = import nixpkgsSrc {
        inherit system;
        inherit overlays;
      };

      project = (mkProjectWith nixpkgs) {
        inherit nixFilter nixpkgs pathAttrImports;
        upstreamMrx = upstreamMrx.packages.${system}.default;
      };
    in
      project;

    mapSystemAttrs = f:
      builtins.listToAttrs ((builtins.map (
          system: {
            name = system;
            value = f system;
          }
        ))
        systems);

    packages = mapSystemAttrs (
      system: let
        project = mkProject' system;
      in {
        _ = project;
        default = project.pkg.mrx;
      }
    );

    apps = mapSystemAttrs (
      system: let
        mrx = {
          type = "app";
          program = "${packages."${system}".default}/bin/mrx";
        };
      in {
        inherit mrx;
        default = mrx;
      }
    );
  in {
    inherit apps packages;
  };
in
  mapSystems
