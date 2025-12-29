{_, ...}:
_.run.many {
  name = _.name "format";

  each = [
    _.format-nix
    _.format-rust
    _.format-shell
    _.format-yaml
  ];
}
