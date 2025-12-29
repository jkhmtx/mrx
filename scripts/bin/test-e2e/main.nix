{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    _.pkg.mrx
  ];

  text = builtins.readFile ./run.sh;
}
