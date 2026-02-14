# shellcheck shell=bash

export DATABASE_PATH="${DATABASE_PATH}"

dir="$(dirname "${DATABASE_PATH}")"
mkdir -p "${dir}"

rm "${DATABASE_PATH}" >/dev/null 2>&1 || true
_.lib.migrations.apply "${DATABASE_PATH}"
