{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    nixpkgs.findutils
    nixpkgs.git
    _.pkg.rust
  ];

  text = builtins.readFile ./run.sh;
}
