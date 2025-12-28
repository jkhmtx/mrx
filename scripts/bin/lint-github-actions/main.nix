{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "lint-github-actions";

  runtimeInputs = [
    nixpkgs.actionlint
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
