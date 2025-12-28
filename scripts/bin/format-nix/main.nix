{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "format-nix";

  runtimeInputs = [
    nixpkgs.alejandra
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
