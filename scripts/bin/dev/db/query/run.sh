# shellcheck shell=bash

export DATABASE_URL="${DATABASE_URL}"

dev_db="${DATABASE_URL#sqlite://}"

dir="$(dirname "${dev_db}")"
mkdir -p "${dir}"

function sqlite() {
	sqlite3 "${dev_db}" "${@}"
}

if test -t 0; then
	sqlite "${@}"
else
	sqlite "${@}" </dev/stdin
fi
