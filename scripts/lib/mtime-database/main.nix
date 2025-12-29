{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.lib.name "mtime-database";

  text = builtins.readFile ./run.sh;
}
