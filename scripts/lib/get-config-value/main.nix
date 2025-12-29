{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [nixpkgs.tomlq];

  text = builtins.readFile ./run.sh;
}
