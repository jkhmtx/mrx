{_, ...}:
_.mrx.run.many {
  name = import _/name;

  each = [
    _.fix
  ];

  extraRuntimeEnv = {
    CI = true;
  };
}
