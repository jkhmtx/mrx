{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "format-shell";

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.shfmt
  ];

  text = builtins.readFile ./run.sh;
}
