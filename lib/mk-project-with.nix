nixpkgs: {pathAttrImports, ...} @ outer: let
  inherit (nixpkgs.lib.attrsets) mapAttrs;
  utils = import ./utils {inherit nixpkgs;};

  inherit (utils.attrs) walk;

  decorate = import ./decorate {inherit nixpkgs utils;};

  mkProject = moduleInputs: let
    inputs = decorate (moduleInputs // pathAttrs);

    importAttrs = walk {whenNot = _continue: _acc: _name: value: import value inputs;} null;

    pathAttrs = mapAttrs (_: attrs: importAttrs attrs) moduleInputs.pathAttrImports;

    project = importAttrs pathAttrImports._;
  in
    project;

  project =
    mkProject outer;
in
  project
