# shellcheck shell=bash

rustc_path="$(which rustc)"

echo "${rustc_path/\/bin\/rustc/}/lib/rustlib/src/rust/library"
