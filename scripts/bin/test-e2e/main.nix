{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "test-e2e";

  runtimeInputs = [
    _.pkg.mrx
  ];

  text = builtins.readFile ./run.sh;
}
