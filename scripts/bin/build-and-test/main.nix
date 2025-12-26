{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "build-and-test";

  runtimeInputs = [_.test-e2e];

  text = builtins.readFile ./run.sh;
}
