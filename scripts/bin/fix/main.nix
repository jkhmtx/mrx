{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "fix";

  runtimeInputs = [_.check];

  text = builtins.readFile ./run.sh;
}
