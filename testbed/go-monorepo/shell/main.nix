{nixpkgs, ...}: let
in
  nixpkgs.symlinkJoin {
    name = "shell";
    paths = [
      nixpkgs.coreutils
    ];
  }
