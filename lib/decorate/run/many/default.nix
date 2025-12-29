{
  nixpkgs,
  utils,
  ...
}: {
  name,
  each,
  extraRuntimeEnv ? {},
}: let
  inherit (builtins) readFile typeOf;
  inherit (nixpkgs.lib) getExe;
  inherit (nixpkgs.lib.attrsets) isDerivation;
  inherit (nixpkgs.lib.lists) any imap0;
  inherit (nixpkgs.lib.strings) join;
  inherit (nixpkgs.lib.trivial) throwIf;
  inherit (utils.fp) filterLeft left right filterRight isLeft;

  exe = i: drv:
    if isDerivation drv
    then right (getExe drv)
    else left "index ${toString i}: ${typeOf drv}";

  eithers = imap0 exe each;

  assertGetExes = list: let
    invalidTypesStr = join "\n  " (filterLeft list);
    assertExes = throwIf (any isLeft list) "_.run.many: Members of `each` must be of type 'derivation'. Got:\n  ${invalidTypesStr}";
  in
    assertExes (filterRight list);
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
