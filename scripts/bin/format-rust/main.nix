{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "format-rust";

  runtimeInputs = [
    nixpkgs.findutils
    nixpkgs.git
    _.pkg.rust
  ];

  text = builtins.readFile ./run.sh;
}
