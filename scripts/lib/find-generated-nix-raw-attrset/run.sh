# shellcheck shell=bash

function find_files() {
	git ls-files --others --exclude-standard -- "${@}"
	git ls-files -- "${@}"
}

export CONFIG_TOML="${CONFIG_TOML}"
export GET_CONFIG_VALUE="${GET_CONFIG_VALUE}"

derivations_lst="$(mktemp)"

{
	CONFIG_TOML="${CONFIG_TOML}" "${GET_CONFIG_VALUE}" derivations
	CONFIG_TOML="${CONFIG_TOML}" "${GET_CONFIG_VALUE}" ignores | sed 's/^/:!:/'
} >"${derivations_lst}"

mapfile -t derivations <"${derivations_lst}"

rm "${derivations_lst}"

mapfile -t paths < <(
	find_files "${derivations[@]}" | while read -r file; do
		if test -f "${file}"; then
			echo "${file}"
		fi
	done | sort | uniq
)

for path in "${paths[@]}"; do
	# Start: ./path/to/scripts/bin/name/main.nix

	# Start: path/to/scripts/bin/name/main.nix
	path="${path//\.\//}"

	# After: path.to.scripts.bin.name.main.nix
	attr_path="${path//\//.}"

	case "${attr_path}" in
	scripts.bin* | scripts.lib* | scripts.util*)
		# When there is no sub-project (e.g. 'testbed')
		attr_path_prefix=root.
		;;
	*)
		attr_path_prefix=
		;;
	esac

	# After: path.to[.lib].name.main.nix
	attr_path="${attr_path//scripts\.bin\./}"
	attr_path="${attr_path//scripts\.lib\./lib.}"
	attr_path="${attr_path//scripts\.util\./util.}"

	# After: path.to.name
	attr_path="${attr_path//\.main\.nix/}"

	echo "${attr_path_prefix}${attr_path} ${path}"
done
