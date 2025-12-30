{
  nixpkgs,
  utils,
  ...
}: let
  inherit (utils.mrx) toDerivationsList;

  mkAll = root: name:
    nixpkgs.runCommand "_.mrx.build.all" {
      buildInputs = toDerivationsList [] root;
    } "touch $out";
in
  mkAll
