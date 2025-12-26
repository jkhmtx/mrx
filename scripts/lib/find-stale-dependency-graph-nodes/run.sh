# shellcheck shell=bash

export CONFIG_TOML="${CONFIG_TOML}"

export FIND_DEPENDENCY_GRAPH_EDGES="${FIND_DEPENDENCY_GRAPH_EDGES}"
export FIND_GENERATED_NIX_RAW_ATTRSET="${FIND_GENERATED_NIX_RAW_ATTRSET}"
export GET_CONFIG_VALUE="${GET_CONFIG_VALUE}"
export GENERATE_IGNORE_PATTERNS_FILE="${GENERATE_IGNORE_PATTERNS_FILE}"
export MTIME_DATABASE="${MTIME_DATABASE}"

graph_dir="$(CONFIG_TOML="${CONFIG_TOML}" "${GET_CONFIG_VALUE}" path:graph)"
mtime_dir="$(CONFIG_TOML="${CONFIG_TOML}" "${GET_CONFIG_VALUE}" path:mtime)"

function push() {
	local path="${graph_dir}/${1}"
	mkdir -p "$(dirname "${path}")"
	touch "${path}"
	if test -v 2; then
		echo "${2}" >>"${path}"
	fi
}

function list_ancestors_recursive() {
	local path="${graph_dir}/${1}"
	if ! test -s "${path}"; then
		return
	fi

	while read -r parent; do
		echo "${parent}"
		list_ancestors_recursive "${parent}"
	done <"${path}"
}

rm -rf "${graph_dir}" >/dev/null 2>&1 || true

ignore_patterns_file="$(CONFIG_TOML="${CONFIG_TOML}" "${GENERATE_IGNORE_PATTERNS_FILE}")"

CONFIG_TOML="${CONFIG_TOML}" "${FIND_DEPENDENCY_GRAPH_EDGES}" |
	while read -r parent child; do
		if echo "${parent}" | grep --quiet --file "${ignore_patterns_file}"; then
			continue
		fi

		if test -n "${child}"; then
			push "${child}" "${parent}"
		else
			push "${parent}"
		fi
	done

mapfile -t files < <(find "${graph_dir}" -type f)

declare -A nodes

raw_attrs="$(CONFIG_TOML="${CONFIG_TOML}" "${FIND_GENERATED_NIX_RAW_ATTRSET}")"

while read -r attrname path; do
	nodes["${path}"]="${attrname}"
done <<<"${raw_attrs}"

{
	for file in "${files[@]}"; do
		file="${file##"${graph_dir}/"}"
		if test -n "$(echo "${file}" | MTIME_DIR="${mtime_dir}" "${MTIME_DATABASE}")"; then
			mapfile -t ancestors < <(list_ancestors_recursive "${file}")

			for ancestor in "${file}" "${ancestors[@]}"; do
				if test -v nodes["${ancestor}"]; then
					echo "${nodes["${ancestor}"]}"
					unset "nodes[${ancestor}]"
				fi
			done
		fi
	done
}
