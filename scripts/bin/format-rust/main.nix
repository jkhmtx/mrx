{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "format-rust";

  runtimeInputs = [
    nixpkgs.findutils
    nixpkgs.git
    _.pkg.rust
  ];

  text = builtins.readFile ./run.sh;
}
