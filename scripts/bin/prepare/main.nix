{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [
    _.lib.migrations.apply
    _.pkg.sqlx
  ];
  text = builtins.readFile ./run.sh;
}
