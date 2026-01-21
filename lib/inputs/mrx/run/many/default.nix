{
  nixpkgs,
  utils,
  ...
}: {
  name,
  each,
  extraRuntimeEnv ? {},
  keepGoing ? false,
}: let
  inherit (builtins) elem isBool readFile typeOf;
  inherit (nixpkgs.lib.attrsets) isDerivation;
  inherit (nixpkgs.lib.lists) any imap0;
  inherit (nixpkgs.lib.strings) getName join;
  inherit (nixpkgs.lib.trivial) boolToString throwIf;
  inherit (utils.fn) not;
  inherit (utils.fp) filterLeft left right getRight isLeft;

  mapLeftErrInfo = i: drv:
    if isDerivation drv
    then right drv
    else left "index ${toString i}: ${typeOf drv}";

  eithers = imap0 mapLeftErrInfo each;

  assertAllDerivations = list: let
    invalidTypesStr = join "\n  " (filterLeft list);
    validate = throwIf (any isLeft list) "_.mrx.run.many: Members of `each` must be of type 'derivation'. Got:\n  ${invalidTypesStr}";
  in
    validate list;

  toStringOrBoolString = v:
    if isBool v
    then boolToString v
    else toString v;

  assertKeepGoing = strKeepGoing: let
    allowed = ["true" "false" "no-stderr-summary"];

    validate = throwIf (not (elem strKeepGoing) allowed) "'keepGoing' may be skipped, or provided any of: [${join ", " allowed}]. Got: '${strKeepGoing}'";
  in
    validate strKeepGoing;

  mode = let
    modes = {
      false = "no-keep-going";
      true = "keep-going";
      "no-stderr-summary" = "keep-going-no-summary";
    };

    strKeepGoing = toStringOrBoolString keepGoing;
  in
    modes.${assertKeepGoing strKeepGoing};

  derivations = map getRight (assertAllDerivations eithers);
in
  nixpkgs.writeShellApplication {
    inherit name;
    runtimeInputs = derivations ++ [nixpkgs.coreutils];
    runtimeEnv =
      {
        LIST = map getName derivations;
        MODE = mode;
      }
      // extraRuntimeEnv;

    text = readFile ./run.sh;
  }
