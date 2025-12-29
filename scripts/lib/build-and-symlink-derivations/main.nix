{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = _.lib.name "build-and-symlink-derivations.inner";

  runtimeInputs = [
    nixpkgs.coreutils
    nixpkgs.findutils
    nixpkgs.git
  ];

  text = builtins.readFile ./run.sh;
}
