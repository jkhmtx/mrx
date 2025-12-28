{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "format-yaml";

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.prettier
  ];

  text = builtins.readFile ./run.sh;
}
