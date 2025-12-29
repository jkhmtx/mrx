{nixpkgs, ...} @ inputs: let
  inherit (builtins) map;
  inherit (nixpkgs.lib.trivial) pipe;

  callPkg = map (p: import p inputs);

  decorateSigil = {_, ...}:
    pipe _ (callPkg [
      ./run
    ]);
in
  inputs: inputs // {_ = decorateSigil inputs;}
