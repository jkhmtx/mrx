{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [
    nixpkgs.sqlite
  ];
  text = builtins.readFile ./run.sh;
}
