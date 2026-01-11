{
  nixpkgs,
  _,
  ...
}: let
  rustPlatform = nixpkgs.makeRustPlatform {
    cargo = _.pkg.rust;
    rustc = _.pkg.rust;
  };

  crateSrcOf = dir: crate: [
    dir
    "${dir}/${crate}"
    "${dir}/${crate}/src"
    ".+\.rs"
    "^Cargo\.lock$"
    ".*Cargo\.toml"
  ];

  package = rustPlatform.buildRustPackage {
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

    src = nixpkgs.lib.sourceByRegex ../../. (
      []
      ++ [".sqlx" ".+\.json"]
      ++ (crateSrcOf "crates" ".+")
      ++ (crateSrcOf "xtask" "src")
      ++ ["cached.sh"]
    );

    cargoLock.lockFile = ../../Cargo.lock;

    meta = {
      mainProgram = "mrx";
      description = "A Nix DevOps framework for monorepos";
      homepage = "https://github.com/jkhmtx/mrx";
      license = nixpkgs.lib.licenses.unlicense;
      maintainers = ["jakehamtexas@gmail.com"];
    };
  };
in
  nixpkgs.writeShellApplication {
    name = "mrx";
    runtimeInputs = [
      _.lib.handle-stale-dependency-graph-nodes
      package
    ];

    runtimeEnv = {
      HANDLE_STALE_DEPENDENCY_GRAPH_NODES = _.lib.handle-stale-dependency-graph-nodes.name;
      #######
      PACKAGE = package.pname;
    };
    text = builtins.readFile ./run.sh;
  }
