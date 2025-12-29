{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;
  runtimeInputs = [
    nixpkgs.coreutils
    nixpkgs.gnused
    _.lib.get-config-value
  ];

  runtimeEnv = {
    GET_CONFIG_VALUE = _.lib.get-config-value.name;
  };
  text = builtins.readFile ./run.sh;
}
