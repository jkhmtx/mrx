# shellcheck shell=bash

export DATABASE_PATH="${DATABASE_PATH}"

dir="$(dirname "${DATABASE_PATH}")"
mkdir -p "${dir}"

function sqlite() {
	sqlite3 "${DATABASE_PATH}" "${@}"
}

if test -t 0; then
	sqlite "${@}"
else
	sqlite "${@}" </dev/stdin
fi
