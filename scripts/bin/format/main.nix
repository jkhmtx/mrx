{_, ...}:
_.run.many {
  name = import _/name;

  each = [
    _.format-nix
    _.format-rust
    _.format-shell
    _.format-yaml
  ];
}
