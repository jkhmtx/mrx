{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.lib.name "get-config-value";

  runtimeInputs = [nixpkgs.tomlq];

  text = builtins.readFile ./run.sh;
}
