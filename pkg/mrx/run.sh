# shellcheck shell=bash

export HANDLE_STALE_DEPENDENCY_GRAPH_NODES="${HANDLE_STALE_DEPENDENCY_GRAPH_NODES}"
export PACKAGE="${PACKAGE}"

export CONFIG_TOML=mrx.toml

case "${1}" in
refresh) "${HANDLE_STALE_DEPENDENCY_GRAPH_NODES}" ;;
*)
	"${PACKAGE}" "${@}"
	;;
esac
