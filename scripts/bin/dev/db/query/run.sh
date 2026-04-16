# shellcheck shell=bash

export MRX_DATABASE_PATH="${MRX_DATABASE_PATH}"

dir="$(dirname "${MRX_DATABASE_PATH}")"
mkdir -p "${dir}"

function sqlite() {
	sqlite3 "${MRX_DATABASE_PATH}" "${@}"
}

if test -t 0; then
	sqlite "${@}"
else
	sqlite "${@}" </dev/stdin
fi
