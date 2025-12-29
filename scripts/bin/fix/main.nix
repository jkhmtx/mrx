{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [_.check];

  text = builtins.readFile ./run.sh;
}
