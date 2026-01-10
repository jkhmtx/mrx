# shellcheck shell=bash

export MIGRATIONS="${MIGRATIONS}"

db="${1}"

dir="$(dirname "${db}")"
mkdir -p "${dir}"

for migration in "${MIGRATIONS}"/*.sql; do
  sqlite3 "${db}" <"${migration}"
done
