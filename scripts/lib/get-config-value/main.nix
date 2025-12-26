{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "lib.get-config-value";

  runtimeInputs = [nixpkgs.tomlq];

  text = builtins.readFile ./run.sh;
}
