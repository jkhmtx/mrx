# shellcheck shell=bash

export CI=true

if test "${FIX:-}" = true; then
	unset CI
fi

log() {
	>&2 echo "${PREFIX:-CHECK}: " "${@}"
}

log Linting...
lint
log "Linting done"

log Formatting...
format
log "Formatting done"
