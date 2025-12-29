{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    nixpkgs.actionlint
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
