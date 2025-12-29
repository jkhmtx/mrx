{nixpkgs, ...}: let
  inherit (nixpkgs.lib.lists) any;
  inherit (nixpkgs.lib.trivial) id;

  not = f: x: !(f x);
in {
  inherit not;
}
