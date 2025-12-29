{_, ...}:
_.run.many {
  name = _.name "local-ci";

  each = [
    _.build-and-test
    _.check
  ];
}
