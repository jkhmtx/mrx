{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "lint-shell";

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.findutils
    nixpkgs.shellcheck
  ];

  text = builtins.readFile ./run.sh;
}
