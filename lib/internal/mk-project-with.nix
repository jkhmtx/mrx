nixpkgs: {pathAttrImports, ...} @ outer: let
  inherit (nixpkgs.lib) isAttrs;
  inherit (nixpkgs.lib.attrsets) mapAttrs;
  name = s: "_.${s}";

  mkProject = moduleInputs: let
    inputs = let
      allInputs =
        moduleInputs
        // pathAttrs;
    in
      allInputs // {_ = allInputs._ // {inherit name;};};

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
