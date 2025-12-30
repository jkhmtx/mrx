inputs: let
  run.parallel = import ./parallel inputs;
  run.many = import ./many inputs;
in
  run
