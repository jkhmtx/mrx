# shellcheck shell=bash

export LIST="${LIST}"

for item in "${LIST[@]}"; do
	"${item}"
done
