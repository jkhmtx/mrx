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
    _.lib.find-stale-dependency-graph-nodes
    _.lib.build-and-symlink-derivations
    _.lib.get-config-value
  ];
  runtimeEnv = {
    FIND_STALE_DEPENDENCY_GRAPH_NODES = _.lib.find-stale-dependency-graph-nodes.name;
    BUILD_AND_SYMLINK = _.lib.build-and-symlink-derivations.name;
    GET_CONFIG_VALUE = _.lib.get-config-value.name;
  };
  text = builtins.readFile ./run.sh;
}
