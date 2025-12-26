# shellcheck shell=bash

export CONFIG_TOML="${CONFIG_TOML}"
export FIND_GENERATED_NIX_RAW_ATTRSET="${FIND_GENERATED_NIX_RAW_ATTRSET}"
export GET_CONFIG_VALUE="${GET_CONFIG_VALUE}"

root="$(CONFIG_TOML="${CONFIG_TOML}" "${GET_CONFIG_VALUE}" path:config)"

declare -A scanned
declare -A attrs_by_name
# declare -A attrs_by_path

raw_attrs="$(CONFIG_TOML="${CONFIG_TOML}" "${FIND_GENERATED_NIX_RAW_ATTRSET}")"

while read -r attrname path; do
	attrs_by_name["${attrname}"]="${path}"
done <<<"${raw_attrs}"

# while read -r attrname path; do
# 	attrs_by_path["${path}"]="${attrname}"
# done <<<"${raw_attrs}"

patterns_lst="$(mktemp)"

for attrname in "${!attrs_by_name[@]}"; do
	# shellcheck disable=2028
	echo "_\.${attrname}\b([^-]|$)"
done >"${patterns_lst}"

entrypoint="$(CONFIG_TOML="${CONFIG_TOML}" \
	"${GET_CONFIG_VALUE}" \
	entrypoint)"

files=("$(realpath "${entrypoint}")")

{
	# Find all paths referenced by entrypoint, and its dependents.
	while test "${#files[@]}" -gt 0; do
		for file in "${files[@]}"; do
			file="${file##"${root}/"}"
			if ! test -n "${scanned["${file}"]:-}" && test "${file}" != "${file%.nix}"; then
				# TODO: Add interpolated paths detection so that the if block internal is reachable
				if false; then
					{
						echo "WARNING"
						echo "Paths that contain interpolation are not automatically added to watched files."
						echo "To silence this warning, add the file to '.envrc.watch.ignores.lst'."
						echo "${file}"
					} >&2
				fi

				# Check mtime of file
				# If file is unchanged, just cat its saved list of parents (dependencies)

				mapfile -t path_matches < <(grep \
					--extended-regexp \
					--only-matching \
					'(\.|(\.\.|\.)/[a-zA-Z0-9\./_\+\-]+)[ ;]' \
					"${file}")

				pushd "$(dirname "${file}")" >/dev/null 2>&1 || exit 1

				for match in "${path_matches[@]}"; do
					match="${match/ /}"
					match="${match/;/}"
					match="$(realpath "${match}")"
					match="${match##"${root}/"}"

					echo "${file} ${match}"

					files+=("${match}")
				done
				popd >/dev/null 2>&1 || exit 1

				drv_matches_lst="$(mktemp)"
				grep \
					--extended-regexp \
					--only-matching \
					--file "${patterns_lst}" \
					"${file}" |
					sed 's/_\.\(.*\w\).*/\1/' >"${drv_matches_lst}" || true

				mapfile -t drv_matches <"${drv_matches_lst}"
				rm "${drv_matches_lst}"

				for attrname in "${drv_matches[@]}"; do
					echo "${file}" "${attrs_by_name["${attrname}"]}"
				done
			fi

			path_matches=()

			# scanned["${file}"]='done'
			files=("${files[@]:1}")
		done
	done
} | sort | uniq
