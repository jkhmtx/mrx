# shellcheck shell=bash

export __MRX_DERIVATION="${__MRX_DERIVATION}"
export __MRX_THIS_MRX_BIN="${__MRX_THIS_MRX_BIN}"

# If 'THIS_MRX_BIN' does not exist, try to substitute it with the one in PATH
mrx_bin=
if test -s "${__MRX_THIS_MRX_BIN}"; then
	mrx_bin="${__MRX_THIS_MRX_BIN}"
else
	mrx_bin=mrx
fi

if ! type "${mrx_bin}" >/dev/null 2>&1; then
	echo "ERROR: 'mrx' not found. Run 'mrx build', then try again."

	exit 1
fi

bin="$("${mrx_bin}" plumbing cache "${__MRX_DERIVATION}")"

exec "${bin}" "${@}"
