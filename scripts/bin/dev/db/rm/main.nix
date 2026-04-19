{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = import _/name;
  text = builtins.readFile ./run.sh;
}
