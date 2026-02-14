{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [
    _.lib.migrations.apply
  ];
  text = builtins.readFile ./run.sh;
}
