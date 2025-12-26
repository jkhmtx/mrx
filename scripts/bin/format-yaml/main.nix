{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "format-yaml";

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.prettier
  ];

  text = builtins.readFile ./run.sh;
}
