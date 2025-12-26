# shellcheck shell=bash

export CONFIG_TOML="${CONFIG_TOML}"
export FIND_GENERATED_NIX_RAW_ATTRSET="${FIND_GENERATED_NIX_RAW_ATTRSET}"

set -euo pipefail

CONFIG_TOML="${CONFIG_TOML}" \
	"${FIND_GENERATED_NIX_RAW_ATTRSET}" |
	grep '/bin/' |
	while read -r attrname _; do
		echo "${attrname}"
	done
