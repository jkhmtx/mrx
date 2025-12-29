{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.findutils
    nixpkgs.shellcheck
  ];

  text = builtins.readFile ./run.sh;
}
