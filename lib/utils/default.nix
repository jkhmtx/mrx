inputs: let
  utils = rec {
    final = inputs // {inherit utils;};

    attrs = import ./attrs final;
    fn = import ./fn.nix final;
    fp = import ./fp.nix final;
    lists = import ./lists.nix final;
    mrx = import ./mrx.nix final;
  };
in
  utils
