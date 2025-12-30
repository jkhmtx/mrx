inputs: let
  mkAll = import ./all inputs;
in
  _: {
    all = mkAll _;
  }
