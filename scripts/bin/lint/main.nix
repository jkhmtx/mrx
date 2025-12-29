{_, ...}:
_.run.many {
  name = _.name "lint";

  each = [
    _.lint-commit
    _.lint-github-actions
    _.lint-rust
    _.lint-shell
  ];
}
