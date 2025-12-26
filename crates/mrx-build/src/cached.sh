# shellcheck shell=bash

export CACHE_DIR="${CACHE_DIR}"
export DERIVATION="${DERIVATION}"
export THIS_MRX_BIN="${THIS_MRX_BIN}"

cached_bin="${CACHE_DIR}"/"${DERIVATION}"

# If 'mrx' is not in the PATH, substitute it with the one
# that writes this script file
mrx_bin=
if type -p mrx >/dev/null 2>&1; then
	mrx_bin=mrx
else
	mrx_bin="${THIS_MRX_BIN}"
fi

if ! test -f "${cached_bin}"; then
	"${mrx_bin}" plumbing cache "_.${DERIVATION}"
fi

if test -f "${cached_bin}" && bash -n "${cached_bin}"; then
	${cached_bin} "${@}"
else
	echo "ERR: mrx failed to build the derivation '_.${DERIVATION}'"
	exit 1
fi
