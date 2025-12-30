{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    (_.mrx.build.without ["build-and-test" "local-ci"])
    _.test-e2e
  ];

  text = builtins.readFile ./run.sh;
}
