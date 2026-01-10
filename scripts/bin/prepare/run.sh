# shellcheck shell=bash

export DATABASE_URL="${DATABASE_URL}"

dev_db="${DATABASE_URL#sqlite://}"

dir="$(dirname "${dev_db}")"
mkdir -p "${dir}"

schema_db="${dir}/schema.db"

rm "${schema_db}" >/dev/null 2>&1 || true
_.migrations.apply "${schema_db}"

DATABASE_URL="sqlite://${schema_db}" \
	sqlx prepare --workspace

_.migrations.apply "${dev_db}"
