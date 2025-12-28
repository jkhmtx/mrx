{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "lib.build-and-symlink-derivations.inner";

  runtimeInputs = [
    nixpkgs.coreutils
    nixpkgs.findutils
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
