{_, ...}:
_.mrx.run.many {
  name = import _/name;

  keepGoing = "no-stderr-summary";

  each = [
    _.format
    _.lint
  ];
}
