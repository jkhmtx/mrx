# shellcheck shell=bash

export MTIME_DIR="${MTIME_DIR}"

while read -r file; do
	record="${MTIME_DIR}"/"${file}"
	current_mtime="$(stat -c '%Y' "${file}")"

	old_mtime=
	if test -s "${record}"; then
		old_mtime="$(cat "${record}")"
	fi

	if test -n "${old_mtime}" && ((current_mtime > old_mtime)); then
		echo "${file}"
	fi

	mkdir -p "$(dirname "${record}")"

	echo "${current_mtime}" >"${record}"
done
