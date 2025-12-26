{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "lint-github-actions";

  runtimeInputs = [
    nixpkgs.actionlint
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
