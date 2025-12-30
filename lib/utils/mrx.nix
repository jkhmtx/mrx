{
  nixpkgs,
  utils,
  ...
}: let
  inherit (nixpkgs.lib.attrsets) attrValues filterAttrs isAttrs isDerivation;
  inherit (utils.lists) flatMap;
  inherit (utils.fn) not;
  inherit (utils.fp) left right pipe' from;

  toDerivationsList = exclusions: let
    continue = allDerivations;

    isValidAttr = name: _: not (builtins.elem name) (["mrx"] ++ exclusions);
    withoutInvalidAttrs = filterAttrs isValidAttr;

    allDerivations = acc: node: let
      continue' = flatMap (continue acc);

      nextAttrs =
        if isAttrs node
        then right node
        else left acc;
    in
      if isDerivation node
      then acc ++ [node]
      else
        from (pipe' nextAttrs [
          withoutInvalidAttrs
          attrValues
          continue'
        ]);
  in
    allDerivations [];
in {inherit toDerivationsList;}
