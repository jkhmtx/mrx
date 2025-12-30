inputs: let
  inherit (inputs.nixpkgs.lib.trivial) mergeAttrs;

  mkBuild = import ./build inputs;
  mkMrx = _: {
    mrx.build = mkBuild _;
  };
in
  _:
    mergeAttrs _ (mkMrx _)
