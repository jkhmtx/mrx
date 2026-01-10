{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [_.migrations.apply _.pkg.sqlx];
  text = builtins.readFile ./run.sh;
}
