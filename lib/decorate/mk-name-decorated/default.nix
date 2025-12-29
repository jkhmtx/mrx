{
  nixpkgs,
  utils,
  ...
}: let
  inherit (nixpkgs.lib.attrsets) isAttrs isDerivation;
  inherit (nixpkgs.lib.strings) join;
  inherit (nixpkgs.lib.trivial) id;
  inherit (nixpkgs.lib.lists) any;
  inherit (utils.attrs) walk;
  inherit (utils.fn) not;
  inherit (utils.fp) fapply;

  mkName = components: name: "_.${join "." (components ++ [name])}";
in
  _: let
    decorateWithName = walk {
      predicate = _: _: value: any id (fapply [(not isAttrs) isDerivation] value);
      when = _: _: _: id; # Base case
      whenNot = continue: acc: name: value: (continue (acc ++ [name]) (value // {name = mkName acc;}));
    };
  in
    (decorateWithName [] _) // {name = mkName [];}
