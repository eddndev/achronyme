#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$script_dir/common.sh"

usage() {
    printf 'usage: %s --input FILE --output FILE --name CONTRIBUTOR_ID [--metrics FILE]\n' "$0" >&2
    exit 2
}

input=
output=
name=
metrics=
while [[ $# -gt 0 ]]; do
    case "$1" in
        --input) input=${2:-}; shift 2 ;;
        --output) output=${2:-}; shift 2 ;;
        --name) name=${2:-}; shift 2 ;;
        --metrics) metrics=${2:-}; shift 2 ;;
        *) usage ;;
    esac
done
[[ -n "$input" && -n "$output" && -n "$name" ]] || usage
[[ "$name" =~ ^[A-Za-z0-9._@\ -]{1,128}$ ]] || die "invalid contributor id"
[[ -n "$metrics" ]] || metrics="$(dirname "$output")/contribute.metrics.jsonl"

require_command /usr/bin/time
require_snarkjs
require_regular_file "$input" "input zkey"
ensure_absent "$output"
create_metrics_file "$metrics"

printf 'snarkjs will request private entropy interactively. Do not paste it into a command, log, or chat.\n' >&2
run_measured "$metrics" phase2_contribution \
    snarkjs zkey contribute "$input" "$output" --name="$name" -v
require_regular_file "$output" "contributed zkey"
printf 'contributed key: %s\n' "$output"
printf 'sha256: %s\n' "$(sha256_file "$output")"
