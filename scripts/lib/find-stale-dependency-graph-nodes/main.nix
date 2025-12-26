{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "lib.handle-stale-dependency-graph-nodes";
  runtimeInputs = [
    nixpkgs.coreutils
    nixpkgs.gnugrep
    _.lib.find-dependency-graph-edges
    _.lib.find-generated-nix-raw-attrset
    _.lib.generate-ignore-patterns-file
    _.lib.get-config-value
    _.lib.mtime-database
  ];
  runtimeEnv = {
    FIND_DEPENDENCY_GRAPH_EDGES = _.lib.find-dependency-graph-edges.name;
    FIND_GENERATED_NIX_RAW_ATTRSET = _.lib.find-generated-nix-raw-attrset.name;
    GENERATE_IGNORE_PATTERNS_FILE = _.lib.generate-ignore-patterns-file.name;
    GET_CONFIG_VALUE = _.lib.get-config-value.name;
    MTIME_DATABASE = _.lib.mtime-database.name;
  };
  text = builtins.readFile ./run.sh;
}
