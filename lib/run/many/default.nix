{nixpkgs, ...}: {
  name,
  each,
  extraRuntimeEnv ? {},
}: let
  inherit (builtins) elemAt filter map readFile typeOf;
  inherit (nixpkgs.lib) getExe;
  inherit (nixpkgs.lib.attrsets) isDerivation;
  inherit (nixpkgs.lib.lists) any imap0;
  inherit (nixpkgs.lib.strings) join;
  inherit (nixpkgs.lib.trivial) throwIf;

  left = x: [x null];
  right = x: [null x];

  getLeft = x: (elemAt x 0);
  getRight = x: (elemAt x 1);

  isLeft = x: (getLeft x) != null;

  exe = i: drv:
    if isDerivation drv
    then right (getExe drv)
    else left "index ${toString i}: ${typeOf drv}";

  eithers = imap0 exe each;

  assertGetExes = list: let
    invalidTypesStr = join "\n  " (map getLeft (filter isLeft list));
    assertExes = throwIf (any isLeft list) "_.run.many: Members of `each` must be of type 'derivation'. Got:\n  ${invalidTypesStr}";
  in
    assertExes (map getRight list);
in
  nixpkgs.writeShellApplication {
    inherit name;
    runtimeEnv =
      {
        LIST = assertGetExes eithers;
      }
      // extraRuntimeEnv;

    text = readFile ./run.sh;
  }
