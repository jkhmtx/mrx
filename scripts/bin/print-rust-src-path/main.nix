{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    _.pkg.rust
  ];

  text = builtins.readFile ./run.sh;
}
