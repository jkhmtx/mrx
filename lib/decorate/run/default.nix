inputs: let
  inherit (inputs.nixpkgs.lib.trivial) mergeAttrs;

  run = let
    parallel = import ./parallel inputs;
    many = import ./many inputs;
  in {
    inherit parallel many;
  };
in
  _: mergeAttrs _ {inherit run;}
