{
  nixpkgs,
  _,
  ...
}: let
  rustPlatform = nixpkgs.makeRustPlatform {
    cargo = _.pkg.rust;
    rustc = _.pkg.rust;
  };

  crateSrc = crate: ["crates/${crate}" "crates/${crate}/src"];

  package = rustPlatform.buildRustPackage {
    pname = "mrx";
    version = "0.0.1";

    src = nixpkgs.lib.sourceByRegex ../../. (
      ["crates"]
      ++ (crateSrc ".+")
      ++ [".+\.rs" "^Cargo\.lock$" ".*Cargo\.toml"]
      ++ ["cached.sh"]
    );

    cargoHash = "sha256-0klTXMz3pyX8B1TIzCgbKl3JGPdRNIf/rbsDztZy83M=";

    meta = {
      mainProgram = "mrx";
      description = "A Nix DevOps framework for monorepos";
      homepage = "https://github.com/jkhmtx/bingo";
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
