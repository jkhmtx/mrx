{_, ...}:
_.run.many {
  name = _.name "check";

  each = [
    _.format
    _.lint
  ];

  extraRuntimeEnv = {
    CI = true;
  };
}
