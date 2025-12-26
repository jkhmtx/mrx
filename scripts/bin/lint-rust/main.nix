{
  _,
  nixpkgs,
  ...
}:
nixpkgs.writeShellApplication {
  name = "lint-rust";

  runtimeInputs = [
    nixpkgs.findutils
    nixpkgs.git
    _.pkg.rust
  ];

  text = builtins.readFile ./run.sh;
}
