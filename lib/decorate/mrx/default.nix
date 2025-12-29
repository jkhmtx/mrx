inputs: let
  inherit (inputs.nixpkgs.lib.trivial) mergeAttrs;

  mrx.run = import ./run inputs;
in
  _: mergeAttrs _ {inherit mrx;}
