{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    nixpkgs.alejandra
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
