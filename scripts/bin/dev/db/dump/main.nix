{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    _.dev.db.query
  ];

  text = builtins.readFile ./run.sh;
}
