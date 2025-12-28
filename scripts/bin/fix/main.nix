{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "fix";

  runtimeInputs = [_.check];

  text = builtins.readFile ./run.sh;
}
