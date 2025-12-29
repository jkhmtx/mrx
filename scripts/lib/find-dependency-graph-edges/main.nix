{
  nixpkgs,
  _,
  ...
}:
_.util.with-tee {
  name = import _/name;
  drv = nixpkgs.writeShellApplication {
    name = "${import _/name}.inner";

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
