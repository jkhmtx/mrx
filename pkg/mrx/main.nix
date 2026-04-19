{
  nixpkgs,
  nixFilter,
  _,
  ...
}: let
  rustPlatform = nixpkgs.makeRustPlatform {
    cargo = _.pkg.rust;
    rustc = _.pkg.rust;
  };
in
  rustPlatform.buildRustPackage {
    pname = "mrx";
    version = "0.0.1";

    nativeBuildInputs = [
      nixpkgs.pkg-config
    ];

    buildInputs = [
      nixpkgs.openssl
    ];

    # No tests!
    doCheck = false;

    src = nixFilter {
      root = ../../.;
      include = [
        "sql"
        "crates"
        "Cargo.lock"
        "Cargo.toml"
        (nixFilter.matchExt "rs")
        (nixFilter.matchExt "sql")
      ];
    };

    cargoLock.lockFile = ../../Cargo.lock;

    meta = {
      mainProgram = "mrx";
      description = "A Nix DevOps framework for monorepos";
      homepage = "https://github.com/jkhmtx/mrx";
      license = nixpkgs.lib.licenses.unlicense;
      maintainers = ["jakehamtexas@gmail.com"];
    };
  }
