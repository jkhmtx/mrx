{
  nixpkgs,
  infallible,
  _,
  ...
}:
infallible.with-tee {
  name = import _/name;
  drv = nixpkgs.writeShellApplication {
    name = import _/name;

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
