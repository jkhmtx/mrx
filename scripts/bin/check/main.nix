{_, ...}:
_.mrx.run.many {
  name = import _/name;

  each = [
    _.format
    _.lint
  ];

  extraRuntimeEnv = {
    CI = true;
  };
}
