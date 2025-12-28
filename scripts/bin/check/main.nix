{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "check";

  runtimeInputs = [
    _.format
    _.lint
  ];

  text = builtins.readFile ./run.sh;
}
