{
  nixpkgs,
  utils,
  ...
}: let
  inherit (builtins) elemAt filter isList map;
  inherit (utils.fn) not;
  inherit (nixpkgs.lib.trivial) pipe;

  left = x: ["either" x null];
  right = x: ["either" null x];

  getLeft = x: (elemAt x 1);
  getRight = x: (elemAt x 2);

  isEither = x: isList x && (elemAt x 0) == "either";

  isLeft = x: (getLeft x) != null;
  isRight = not isLeft;

  filterLeft = xs: (map getLeft (filter isLeft xs));
  filterRight = xs: (map getRight (filter isRight xs));

  map' = f: either:
    if isRight either
    then right (f (getRight either))
    else either;

  pipe' = either: fs:
    if isRight either
    then pipe either ([getRight] ++ fs ++ [right])
    else either;

  from = x:
    if isEither x && isRight x
    then getRight x
    else if isEither x
    then getLeft x
    else x;
in {
  inherit left right getLeft getRight isLeft filterLeft filterRight from;

  inherit map' pipe';
}
