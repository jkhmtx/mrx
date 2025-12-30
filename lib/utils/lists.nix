{nixpkgs, ...}: let
  inherit (builtins) map;
  inherit (nixpkgs.lib.lists) flatten;

  flatMap = f: xs: flatten (map f xs);
in {
  inherit flatMap;
}
