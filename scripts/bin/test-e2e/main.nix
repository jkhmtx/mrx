{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "test-e2e";

  runtimeInputs = [
    _.pkg.mrx
  ];

  text = builtins.readFile ./run.sh;
}
