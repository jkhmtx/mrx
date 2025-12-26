# shellcheck shell=bash

export CONFIG_TOML="${CONFIG_TOML}"

function halt() {
	for arg in "${@}"; do
		echo "ERR: ${arg}" >&2
	done

	exit 1
}

function get_value() {
	tq --raw --file "${CONFIG_TOML}" "${1}" 2>/dev/null || true
}

function get_array() {
	tq \
		--output json \
		--raw \
		--file "${CONFIG_TOML}" \
		"${1}" \
		2>/dev/null |
		jq \
			--raw-output \
			'.[]' \
			2>/dev/null ||
		true
}

config_path="$(realpath "${CONFIG_TOML}")"
dir="$(dirname "${config_path}")"

PREFIX=.mrx

case "${1}" in
derivations)
	derivations_lst="$(mktemp)"
	get_array derivations >"${derivations_lst}"
	if ! test -s "${derivations_lst}"; then
		echo ':*main.nix'
	fi

	cat "${derivations_lst}"

	rm "${derivations_lst}"
	;;
ignores)
	cat "${dir}"/mrx.ignore.lst 2>/dev/null || true
	;;
entrypoint)
	entrypoint="$(get_value entrypoint)"

	if test -n "${entrypoint}"; then
		echo "${entrypoint}"
	elif test -f "${dir}"/flake.nix; then
		echo flake.nix
	elif test -f "${dir}"/default.nix; then
		echo default.nix
	else
		halt "No entrypoint found for config '${config_path}'"
	fi
	;;
eagerly-rebuild) ;;
generated-out-path)
	generated_out_path="$(get_value generated-out-path)"

	echo "${generated_out_path:-${dir}/mrx.generated.nix}"
	;;
ignore-patterns-file)
	ignore_patterns_file="$(get_value generated-out-path)"

	if test -n "${ignore_patterns_file}"; then
		echo "${ignore_patterns_file}"
	fi
	;;
installables)
	installables_lst="$(mktemp)"
	get_array installables >"${installables_lst}"
	if test -s "${installables_lst}"; then
		cat "${installables_lst}"
	fi

	rm "${installables_lst}"
	;;
path:config)
	echo "${dir}"
	;;
path:*)
	part="${1#path:}"

	mkdir -p "${dir}/${PREFIX}/${part}"
	echo "${dir}/${PREFIX}/${part}"
	;;
*)
	halt "Config value '${1}' is invalid"
	;;
esac
