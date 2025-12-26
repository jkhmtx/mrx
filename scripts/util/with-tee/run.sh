# shellcheck shell=bash

export CONFIG_TOML="${CONFIG_TOML}"
export DRV_NAME="${DRV_NAME}"
export GET_CONFIG_VALUE="${GET_CONFIG_VALUE}"

dir=
if test -z "${TEE_FILE_PREFIX:-}"; then
	dir="$(CONFIG_TOML="${CONFIG_TOML}" "${GET_CONFIG_VALUE}" path:tee)"
else
	dir="${TEE_FILE_PREFIX}"
fi

mkdir -p "${dir}"

"${DRV_NAME}" "${@}" | tee "${dir}/${DRV_NAME}"
