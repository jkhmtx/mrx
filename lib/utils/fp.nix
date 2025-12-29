{utils, ...}: let
  inherit (builtins) elemAt filter map;
  inherit (utils.fn) not;

  left = x: [x null];
  right = x: [null x];

  getLeft = x: (elemAt x 0);
  getRight = x: (elemAt x 1);

  isLeft = x: (getLeft x) != null;
  isRight = not isLeft;

  filterLeft = xs: (map getLeft (filter isLeft xs));
  filterRight = xs: (map getRight (filter isRight xs));

  fapply = fs: x: (map (f: f x) fs);
in {
  inherit left right getLeft getRight isLeft filterLeft filterRight fapply;
}
