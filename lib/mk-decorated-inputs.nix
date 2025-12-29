{nixpkgs, ...} @ inputs: let
  name = s: "_.${s}";

  run = let
    parallel = import ./run/many inputs;
    many = import ./run/many inputs;
  in {
    inherit parallel many;
  };
in
  inputs:
    inputs
    // {
      _ =
        inputs._
        // {
          inherit run name;
        };
    }
