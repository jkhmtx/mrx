# shellcheck shell=bash

set -euo pipefail

nix build '#_.pkg.mrx-upstream' --out-link .mrx/upstream

function mrx() {
  ./.mrx/upstream/bin/mrx-upstream "${@}"
}

mrx generate

dev_shell_paths_lst="$(mktemp)"
mrx build \
  >"${dev_shell_paths_lst}"

mapfile -t path_add_paths <"${dev_shell_paths_lst}"
rm "${dev_shell_paths_lst}"

PATH_add "${path_add_paths[@]}"

watch_files_lst="$(mktemp)"
mrx show watch-files \
  >"${watch_files_lst}"

mapfile -t watch_files <"${watch_files_lst}"
rm "${watch_files_lst}"

watch_file "${watch_files[@]}"

mrx refresh

mrx hook >&2

rustc_path="$(realpath "$(nix path-info '#shell')"/bin/rustc)"
export RUST_SRC_PATH="${rustc_path/\/bin\/rustc/}/lib/rustlib/src/rust/library"
