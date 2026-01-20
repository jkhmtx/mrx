# shellcheck shell=bash

export LIST="${LIST}"
export MODE="${MODE}"

failed=()

for item in "${LIST[@]}"; do
	unset exit_code
	"${item}" || exit_code="${?}"

	case "${MODE}" in
	keep-going*)
		if test "${exit_code:-0}" -gt 0; then
			failed+=("${item}")
		fi
		;;
	no-keep-going)
		if test "${exit_code:-0}" -gt 0; then
			exit "${exit_code}"
		fi
		;;
	*)
		echo "Unknown mode: ${MODE}" >&2
		exit 2
		;;
	esac
done

if test "${#failed[@]}" -gt 0; then
	case "${MODE}" in
	keep-going)
		{
			printf '%s: FAIL\n' "${failed[@]}"
		} >&2

		exit 1
		;;
	keep-going-no-summary)
		exit 1
		;;
	no-keep-going)
		echo "Unreachable! 'no-keep-going' mode should have exited nonzero before this point." >&2
		exit 2
		;;
	*)
		echo "Unknown mode: ${MODE}" >&2
		exit 2
		;;
	esac
fi
