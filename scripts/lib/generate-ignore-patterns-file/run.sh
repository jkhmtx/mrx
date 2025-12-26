# shellcheck shell=bash

export CONFIG_TOML="${CONFIG_TOML}"
export GET_CONFIG_VALUE="${GET_CONFIG_VALUE}"

ignore_patterns_file="$(mktemp)"

CONFIG_TOML="${CONFIG_TOML}" \
	"${GET_CONFIG_VALUE}" \
	generated-out-path \
	>"${ignore_patterns_file}"

CONFIG_TOML="${CONFIG_TOML}" \
	"${GET_CONFIG_VALUE}" \
	ignore-patterns-file \
	>"${ignore_patterns_file}"

echo "${ignore_patterns_file}"
