nixpkgs: {pathAttrImports, ...} @ outer: let
  inherit (nixpkgs.lib) isAttrs;
  inherit (nixpkgs.lib.attrsets) mapAttrs;

  mkDecoratedInputs = import ./mk-decorated-inputs.nix {inherit nixpkgs;};

  mkProject = moduleInputs: let
    inputs = mkDecoratedInputs (moduleInputs // pathAttrs);

    importAttrs = mapAttrs (_: attrsOrTerminal:
      if isAttrs attrsOrTerminal
      then importAttrs attrsOrTerminal
      else (import attrsOrTerminal inputs));

    pathAttrs = mapAttrs (_: attrs: importAttrs attrs) moduleInputs.pathAttrImports;

    project = importAttrs pathAttrImports._;
  in
    project;

  project =
    mkProject outer;
in
  project
