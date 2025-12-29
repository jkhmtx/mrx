pathAttrImports: let
  mkProjectWith = import ./mk-project-with.nix;
in
  inputs: let
    nixpkgs =
      if inputs ? pkgs
      then inputs.pkgs
      else if inputs ? nixpkgs
      then inputs.nixpkgs
      else throw "mkProject must be called with either 'pkgs' or 'nixpkgs'";

    mkProject = mkProjectWith nixpkgs;

    project = mkProject inputs;

    mrxProject = mkProject {
      inherit nixpkgs;
      inherit pathAttrImports;
    };
  in
    project
    // {
      mrx = mrxProject.pkg.mrx;
    }
