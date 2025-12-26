{
  nixpkgs,
  infallible,
  ...
}:
infallible.with-tee {
  name = "lib.find-generated-nix-raw-attrset";
  drv = nixpkgs.writeShellApplication {
    name = "lib.find-generated-nix-raw-attrset.inner";

    runtimeInputs = [
      nixpkgs.git
      infallible.get-config-value
    ];

    runtimeEnv = {
      GET_CONFIG_VALUE = infallible.get-config-value.name;
    };

    text = builtins.readFile ./run.sh;
  };
}
