#shellcheck shell=bash

export CONFIG_JS="${CONFIG_JS}"

git fetch origin main >/dev/null 2>&1

commitlint --config "${CONFIG_JS}" --from=origin/main
