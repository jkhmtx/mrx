# shellcheck shell=bash

set -euo pipefail

export DATABASE_URL="${DATABASE_URL}"

# Build and add paths discovered by 'mrx build'
while read -r file; do
  PATH_add "${file}"
done < <(
  envrc-mrx build \
    --generate \
    --hook
)

RUST_SRC_PATH="$(_.print-rust-src-path)"
export RUST_SRC_PATH

DATABASE_URL="${DATABASE_URL}" \
  _.prepare || true

# After this point, we want there to be a shell available even if the script fails. Otherwise, debugging is really hard.

# In reality, this should never fail (especially for the end user)
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
