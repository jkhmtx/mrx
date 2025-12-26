{
  nixpkgs,
  _,
  ...
}:
_.util.with-tee {
  name = "lib.find-dependency-graph-edges";
  drv = nixpkgs.writeShellApplication {
    name = "lib.find-dependency-graph-edges.inner";

    runtimeInputs = [
      nixpkgs.coreutils
      nixpkgs.git
      nixpkgs.gnugrep
      nixpkgs.gnused
      _.lib.find-generated-nix-raw-attrset
      _.lib.get-config-value
    ];

    runtimeEnv = {
      FIND_GENERATED_NIX_RAW_ATTRSET = _.lib.find-generated-nix-raw-attrset.name;
      GET_CONFIG_VALUE = _.lib.get-config-value.name;
    };

    text = builtins.readFile ./run.sh;
  };
}
