{
  nixpkgsSrc,
  mrx,
  ...
}: system: let
  instantiatedNixpkgs = import nixpkgsSrc {
    inherit system;
  };
in
  mrx.mkProject {
    nixpkgs = instantiatedNixpkgs;
    pathAttrImports = {
      _ = import ./mrx.generated.nix;
    };
  }
