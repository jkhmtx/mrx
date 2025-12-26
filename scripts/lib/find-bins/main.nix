{
  nixpkgs,
  _,
  ...
}:
_.util.with-tee {
  name = "lib.find-bins";
  drv = nixpkgs.writeShellApplication {
    name = "lib.find-bins.inner";

    runtimeInputs = [
      _.lib.find-generated-nix-raw-attrset
    ];
    runtimeEnv = {
      FIND_GENERATED_NIX_RAW_ATTRSET = _.lib.find-generated-nix-raw-attrset.name;
    };

    text = builtins.readFile ./run.sh;
  };
}
