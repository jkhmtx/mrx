{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.shfmt
  ];

  text = builtins.readFile ./run.sh;
}
