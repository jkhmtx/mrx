# shellcheck shell=bash

export MRX_DATABASE_PATH="${MRX_DATABASE_PATH}"

dir="$(dirname "${MRX_DATABASE_PATH}")"
mkdir -p "${dir}"

rm "${MRX_DATABASE_PATH}" >/dev/null 2>&1 || true
