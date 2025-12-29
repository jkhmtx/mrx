inputs: let
  utils = rec {
    final = inputs // {inherit utils;};

    attrs = import ./attrs final;
    fn = import ./fn.nix final;
    fp = import ./fp.nix final;
  };
in
  utils
