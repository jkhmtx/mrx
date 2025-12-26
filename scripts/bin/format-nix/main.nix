{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "format-nix";

  runtimeInputs = [
    nixpkgs.alejandra
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
