{nixpkgs, ...} @ inputs: {
  name,
  drv,
  teeFilePrefix ? null,
}:
nixpkgs.writeShellApplication {
  inherit name;

  runtimeInputs = [
    nixpkgs.coreutils
    inputs.infallible.get-config-value
    drv
  ];

  runtimeEnv =
    {
      DRV_NAME = drv.name;
      GET_CONFIG_VALUE = inputs.infallible.get-config-value.name;
    }
    // (
      if teeFilePrefix == null
      then {}
      else {
        TEE_FILE_PREFIX = teeFilePrefix;
      }
    );

  text = builtins.readFile ./run.sh;
}
