{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "lib.mtime-database";

  text = builtins.readFile ./run.sh;
}
