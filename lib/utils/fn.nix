{nixpkgs, ...}: let
  not = f: x: !(f x);
in {
  inherit not;
}
