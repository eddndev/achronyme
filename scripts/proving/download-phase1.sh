#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$script_dir/common.sh"

usage() {
    printf 'usage: %s DESTINATION\n' "$0" >&2
    exit 2
}

[[ $# -eq 1 ]] || usage
destination=$1
partial="${destination}.partial"

require_command curl
require_command b2sum
require_snarkjs
mkdir -p "$(dirname "$destination")"

if [[ -e "$destination" || -L "$destination" ]]; then
    require_regular_file "$destination" "phase-1 artifact"
    verify_phase1_hash "$destination" "$ACHRONYME_PHASE1_BLAKE2B512"
else
    [[ ! -L "$partial" ]] || die "partial download cannot be a symlink: $partial"
    curl --fail --location --retry 5 --continue-at - \
        --output "$partial" "$ACHRONYME_PHASE1_URL"
    require_regular_file "$partial" "partial phase-1 artifact"
    verify_phase1_hash "$partial" "$ACHRONYME_PHASE1_BLAKE2B512"
    mv "$partial" "$destination"
fi

snarkjs powersoftau verify "$destination"
printf 'verified phase 1: %s\n' "$destination"
printf 'sha256: %s\n' "$(sha256_file "$destination")"
printf 'blake2b512: %s\n' "$ACHRONYME_PHASE1_BLAKE2B512"
