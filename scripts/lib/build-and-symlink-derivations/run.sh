# shellcheck shell=bash

export CACHE_DIR="${CACHE_DIR}"
export ROOT="${ROOT}"

cd "${ROOT}" || exit 1

mkdir -p "${CACHE_DIR}"

mapfile -t derivations < <(cat - | xargs printf '#%s\n')

{
	echo Building derivations:
	printf '%s\n' "${derivations[@]}"
} >&2

mapfile -t out_paths < <(
	nix build \
		--print-build-logs \
		--print-out-paths "${derivations[@]}"
)

for path in "${out_paths[@]}"; do
	derivation="${path#*-}"

	ln -fs "${path}/bin/${derivation}" "${CACHE_DIR}"/"${derivation}"
done
