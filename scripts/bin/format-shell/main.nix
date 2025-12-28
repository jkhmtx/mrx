{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "format-shell";

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.shfmt
  ];

  text = builtins.readFile ./run.sh;
}
