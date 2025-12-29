{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    nixpkgs.coreutils
    nixpkgs.findutils
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
