{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [nixpkgs.commitlint nixpkgs.git];

  runtimeEnv = {
    CONFIG_JS = ./config.js;
  };

  text = builtins.readFile ./run.sh;
}
