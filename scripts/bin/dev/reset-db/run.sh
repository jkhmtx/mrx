# shellcheck shell=bash

export DATABASE_URL="${DATABASE_URL}"

dev_db="${DATABASE_URL#sqlite://}"

dir="$(dirname "${dev_db}")"
mkdir -p "${dir}"

rm "${dev_db}" >/dev/null 2>&1 || true
_.migrations.apply "${dev_db}"
