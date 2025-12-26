nixpkgs: {pathAttrImports, ...} @ outer: let
  inherit (nixpkgs.lib) isAttrs;
  inherit (nixpkgs.lib.attrsets) mapAttrs;

  mkProject = moduleInputs: let
    inputs =
      moduleInputs
      // pathAttrs;

    importAttrs = let
      importAttrs = mapAttrs (_: attrsOrTerminal:
        if isAttrs attrsOrTerminal
        then importAttrs attrsOrTerminal
        else (import attrsOrTerminal inputs));
    in
      importAttrs;

    pathAttrs = mapAttrs (_: attrs: importAttrs attrs) moduleInputs.pathAttrImports;

    project = importAttrs pathAttrImports._;
  in
    project;

  project =
    mkProject outer;
in
  project
