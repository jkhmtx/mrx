{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    nixpkgs.git
    nixpkgs.prettier
  ];

  text = builtins.readFile ./run.sh;
}
