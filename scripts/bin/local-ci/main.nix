{_, ...}:
_.run.many {
  name = import _/name;

  each = [
    _.build-and-test
    _.check
  ];
}
