{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "format";

  runtimeInputs = [
    _.format-nix
    _.format-rust
    _.format-shell
    _.format-yaml
  ];

  text = builtins.readFile ./run.sh;
}
