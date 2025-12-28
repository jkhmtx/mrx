{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.name "lint";

  runtimeInputs = [
    _.lint-commit
    _.lint-github-actions
    _.lint-rust
    _.lint-shell
  ];

  text = builtins.readFile ./run.sh;
}
