{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [nixpkgs.sqlite];
  runtimeEnv = {
    MIGRATIONS = toString ../../../../sql/migrations;
  };
  text = builtins.readFile ./run.sh;
}
