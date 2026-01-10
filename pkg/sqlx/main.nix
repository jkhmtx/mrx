{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = "sqlx";
  runtimeInputs = [_.pkg.rust];
  text = builtins.readFile ./run.sh;
}
