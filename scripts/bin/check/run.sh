# shellcheck shell=bash

export CI=true

if test "${FIX:-}" = true; then
	unset CI
fi

log() {
	>&2 echo "${PREFIX:-CHECK}: " "${@}"
}

log Linting...
_.lint
log "Linting done"

log Formatting...
_.format
log "Formatting done"
