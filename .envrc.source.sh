# shellcheck shell=bash

set -euo pipefail

export DATABASE_PATH="${DATABASE_PATH}"

mkdir -p .direnv

# Build and add paths discovered by 'mrx build'
if envrc-mrx build \
  --generate \
  --hook \
  >.direnv/mrx-build-paths; then
  while read -r path; do
    PATH_add "${path}"
  done <.direnv/mrx-build-paths
else
  # If envrc-mrx fails for any reason,
  # ensure we have 'cargo' in the path for debugging purposes.
  nix --print-build-logs build --out-link .direnv/failover-shell
  PATH_add .direnv/failover-shell/bin
fi

DATABASE_PATH="${DATABASE_PATH}" \
  _.prepare ||
  DATABASE_PATH="${DATABASE_PATH}" \
    nix run '#_.prepare' ||
  true

RUST_SRC_PATH="$(_.print-rust-src-path || nix run '#_.print-rust-src-path')"
export RUST_SRC_PATH

# After this point, we want there to be a shell available even if the script fails.
# In reality, this is only fallible if doing some local debugging.
set +eo pipefail

# Add watch-files for dependencies within this file
dependencies=(
  _.shell
  _.prepare
  _.print-rust-src-path
)

while read -r file; do
  watch_file "${file}"
done < <(
  envrc-mrx show \
    watch-files \
    "${dependencies[@]}"
)

watch_file flake.lock
