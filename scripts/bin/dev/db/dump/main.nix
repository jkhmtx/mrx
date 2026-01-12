{
  nixpkgs,
  _,
  ...
}:
nixpkgs.writeShellApplication {
  name = import _/name;

  runtimeInputs = [
    _.dev.db.query
  ];

  runtimeEnv = {
    QUERY = _.dev.db.query.name;
  };

  text = builtins.readFile ./run.sh;
}
