{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [nixpkgs.cowsay];
  text = "cowsay $@";
}
