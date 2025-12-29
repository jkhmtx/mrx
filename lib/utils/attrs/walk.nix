{nixpkgs, ...}: let
  inherit (nixpkgs.lib.attrsets) mapAttrs;
  inherit (nixpkgs.lib) isAttrs;
  inherit (nixpkgs.lib.trivial) defaultTo throwIf;
in
  # predicate
  # (any:acc) -> (string:name) -> (any:value) -> bool
  #
  # when/whenNot
  # (f:continue) -> (any:acc) -> (string:name) -> (any:value) -> any
  {
    predicate ? null,
    when ? null,
    whenNot ? null,
  }: let
    pred = defaultTo (_: _: isAttrs) predicate;

    defaultBranch = continue: acc: name: value: continue acc value;
    branches = throwIf (when == null && whenNot == null) "walk: Without a 'when' or 'whenNot', walk produces a function that never terminates. Provide at least one." {
      when = defaultTo defaultBranch when;
      whenNot = defaultTo defaultBranch whenNot;
    };

    continue = walkInner;

    walkInner = acc:
      mapAttrs (
        name: value:
          if pred acc name value
          then branches.when continue acc name value
          else branches.whenNot continue acc name value
      );
  in
    walkInner
