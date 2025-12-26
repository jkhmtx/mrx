{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = "lib.generate-ignore-patterns-file";
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
