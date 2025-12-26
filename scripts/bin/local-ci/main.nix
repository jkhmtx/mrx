{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "local-ci";

  runtimeInputs = [
    _.build-and-test
    _.check
  ];

  text = builtins.readFile ./run.sh;
}
