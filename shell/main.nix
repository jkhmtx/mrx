{
  nixpkgs,
  _,
  ...
}:
nixpkgs.symlinkJoin {
  name = "shell";
  paths = [
    _.pkg.rust
    # _.pkg.mrx-upstream
    nixpkgs.coreutils
  ];
}
