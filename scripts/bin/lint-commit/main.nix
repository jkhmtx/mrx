{nixpkgs, ...}:
nixpkgs.writeShellApplication {
  name = "lint-commit";

  runtimeInputs = [nixpkgs.commitlint nixpkgs.git];

  runtimeEnv = {
    CONFIG_JS = ./config.js;
  };

  text = builtins.readFile ./run.sh;
}
