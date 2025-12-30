{
  nixpkgs,
  utils,
  ...
}: let
  inherit (utils.mrx) toDerivationsList;

  mkWithout = root: exclusions: let
  in
    nixpkgs.runCommand "_.mrx.build.without" {
      buildInputs = toDerivationsList exclusions root;
    } "touch $out";
in
  mkWithout
