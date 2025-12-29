{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [_.test-e2e];

  text = builtins.readFile ./run.sh;
}
