{
  build-and-test = ./scripts/bin/build-and-test/main.nix;
  check = ./scripts/bin/check/main.nix;
  dev.db.dump = ./scripts/bin/dev/db/dump/main.nix;
  dev.db.query = ./scripts/bin/dev/db/query/main.nix;
  dev.db.reset = ./scripts/bin/dev/db/reset/main.nix;
  dev.run = ./scripts/bin/dev/run/main.nix;
  fix = ./scripts/bin/fix/main.nix;
  format = ./scripts/bin/format/main.nix;
  format-nix = ./scripts/bin/format-nix/main.nix;
  format-rust = ./scripts/bin/format-rust/main.nix;
  format-shell = ./scripts/bin/format-shell/main.nix;
  format-yaml = ./scripts/bin/format-yaml/main.nix;
  lib.migrations.apply = ./scripts/lib/migrations/apply/main.nix;
  lint = ./scripts/bin/lint/main.nix;
  lint-commit = ./scripts/bin/lint-commit/main.nix;
  lint-github-actions = ./scripts/bin/lint-github-actions/main.nix;
  lint-rust = ./scripts/bin/lint-rust/main.nix;
  lint-shell = ./scripts/bin/lint-shell/main.nix;
  local-ci = ./scripts/bin/local-ci/main.nix;
  pkg.mrx = ./pkg/mrx/main.nix;
  pkg.mrx-upstream = ./pkg/mrx-upstream/main.nix;
  pkg.rust = ./pkg/rust/main.nix;
  prepare = ./scripts/bin/prepare/main.nix;
  print-rust-src-path = ./scripts/bin/print-rust-src-path/main.nix;
  shell = ./shell/main.nix;
  test-e2e = ./scripts/bin/test-e2e/main.nix;
}
