{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [
    _.migrations.apply
  ];
  text = builtins.readFile ./run.sh;
}
