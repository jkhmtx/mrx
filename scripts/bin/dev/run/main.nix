{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    _.pkg.rust
    _.prepare
  ];

  text = builtins.readFile ./run.sh;
}
