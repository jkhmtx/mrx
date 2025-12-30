inputs: let
  mkWithout = import ./without.nix inputs;
in
  _: {
    without = mkWithout _;
  }
