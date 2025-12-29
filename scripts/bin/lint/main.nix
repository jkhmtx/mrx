{_, ...}:
_.mrx.run.many {
  name = import _/name;

  each = [
    _.lint-commit
    _.lint-github-actions
    _.lint-rust
    _.lint-shell
  ];
}
