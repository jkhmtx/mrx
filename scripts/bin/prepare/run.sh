# shellcheck shell=bash

export DATABASE_PATH="${DATABASE_PATH}"

dir="$(dirname "${DATABASE_PATH}")"
mkdir -p "${dir}"

schema_db="${dir}/schema.db"

rm "${schema_db}" >/dev/null 2>&1 || true >/dev/null 2>&1
_.lib.migrations.apply "${schema_db}"
_.lib.migrations.apply "${DATABASE_PATH}"
