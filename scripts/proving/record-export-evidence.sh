#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)
source "$script_dir/common.sh"

usage() {
    printf '%s\n' \
        "usage: $0 --ach-bin FILE --source CIRCOM --input-file TOML" \
        "  --r1cs FILE --wtns FILE --output FILE [--metrics FILE]" \
        "  [--phase1-power N] [--timeout-seconds N]" \
        "  [--max-virtual-memory-gib N] [--min-free-disk-gib N]" \
        "  [--expected-git-commit HASH] [--expected-binary-sha256 HASH]" >&2
    exit 2
}

ach_bin=
source_file=
input_file=
r1cs=
wtns=
output=
metrics=
phase1_power=$ACHRONYME_PHASE1_POWER
timeout_seconds=
max_virtual_memory_gib=
min_free_disk_gib=
expected_git_commit=
expected_binary_sha256=
while [[ $# -gt 0 ]]; do
    case "$1" in
        --ach-bin) ach_bin=${2:-}; shift 2 ;;
        --source) source_file=${2:-}; shift 2 ;;
        --input-file) input_file=${2:-}; shift 2 ;;
        --r1cs) r1cs=${2:-}; shift 2 ;;
        --wtns) wtns=${2:-}; shift 2 ;;
        --output) output=${2:-}; shift 2 ;;
        --metrics) metrics=${2:-}; shift 2 ;;
        --phase1-power) phase1_power=${2:-}; shift 2 ;;
        --timeout-seconds) timeout_seconds=${2:-}; shift 2 ;;
        --max-virtual-memory-gib) max_virtual_memory_gib=${2:-}; shift 2 ;;
        --min-free-disk-gib) min_free_disk_gib=${2:-}; shift 2 ;;
        --expected-git-commit) expected_git_commit=${2:-}; shift 2 ;;
        --expected-binary-sha256) expected_binary_sha256=${2:-}; shift 2 ;;
        *) usage ;;
    esac
done
[[ -n "$ach_bin" && -n "$source_file" && -n "$input_file" ]] || usage
[[ -n "$r1cs" && -n "$wtns" && -n "$output" ]] || usage
[[ "$phase1_power" =~ ^[1-9][0-9]*$ && "$phase1_power" -le 30 ]] || \
    die "phase-1 power must be an integer from 1 through 30"
for value in "$timeout_seconds" "$max_virtual_memory_gib" "$min_free_disk_gib"; do
    [[ -z "$value" || "$value" =~ ^[1-9][0-9]*$ ]] || \
        die "resource bounds must be positive integers"
done
if [[ -n "$expected_git_commit" ]]; then
    [[ "$expected_git_commit" =~ ^[0-9a-f]{40,64}$ ]] || die "invalid expected Git commit"
fi
if [[ -n "$expected_binary_sha256" ]]; then
    [[ "$expected_binary_sha256" =~ ^[0-9a-f]{64}$ ]] || \
        die "invalid expected binary SHA-256"
fi

require_command git
require_command jq
require_command od
require_command sha256sum
require_snarkjs
require_regular_file "$ach_bin" "Achronyme binary"
require_regular_file "$source_file" "Circom source"
require_regular_file "$input_file" "witness input"
require_regular_file "$r1cs" "R1CS artifact"
require_regular_file "$wtns" "witness artifact"
if [[ -n "$metrics" ]]; then
    require_regular_file "$metrics" "metrics artifact"
fi
require_clean_tracked_checkout "$repo_root"
ensure_absent "$output"

git_commit=$(git -C "$repo_root" rev-parse --verify HEAD)
achronyme_binary_sha256=$(sha256_file "$ach_bin")
[[ -z "$expected_git_commit" || "$git_commit" == "$expected_git_commit" ]] || \
    die "Git commit changed during the measured export"
[[ -z "$expected_binary_sha256" || \
    "$achronyme_binary_sha256" == "$expected_binary_sha256" ]] || \
    die "Achronyme binary changed during the measured export"

snarkjs wtns check "$r1cs" "$wtns" >/dev/null
variables=$(od -An -tu4 -j60 -N4 "$r1cs" | tr -d ' ')
public_inputs=$(od -An -tu4 -j68 -N4 "$r1cs" | tr -d ' ')
constraints=$(od -An -tu4 -j84 -N4 "$r1cs" | tr -d ' ')
for value in "$variables" "$public_inputs" "$constraints"; do
    [[ "$value" =~ ^[0-9]+$ ]] || die "cannot read canonical R1CS header"
done
phase1_capacity=$((1 << phase1_power))
required_domain=$((constraints + public_inputs + 1))
[[ "$required_domain" -le "$phase1_capacity" ]] || \
    die "power-$phase1_power phase 1 is too small for $required_domain domain entries"

source_sha256=$(sha256_file "$source_file")
inputs_sha256=$(sha256_file "$input_file")
r1cs_sha256=$(sha256_file "$r1cs")
witness_sha256=$(sha256_file "$wtns")
metrics_json='[]'
if [[ -n "$metrics" ]]; then
    metrics_json=$(jq -c -s '.' "$metrics") || die "metrics artifact is not valid JSONL"
fi

output_parent=$(dirname "$output")
mkdir -p "$output_parent"
require_directory "$output_parent" "export evidence directory"
tmp_output=$(mktemp "$output.tmp.XXXXXX")
cleanup() {
    rm -f -- "$tmp_output"
}
trap cleanup EXIT
jq -n \
    --arg git_commit "$git_commit" \
    --arg achronyme_binary_sha256 "$achronyme_binary_sha256" \
    --arg achronyme_version "$("$ach_bin" --version)" \
    --arg source_sha256 "$source_sha256" \
    --arg inputs_sha256 "$inputs_sha256" \
    --arg r1cs_sha256 "$r1cs_sha256" \
    --arg witness_sha256 "$witness_sha256" \
    --arg timeout_seconds "$timeout_seconds" \
    --arg max_virtual_memory_gib "$max_virtual_memory_gib" \
    --arg min_free_disk_gib "$min_free_disk_gib" \
    --argjson variables "$variables" \
    --argjson public_inputs "$public_inputs" \
    --argjson constraints "$constraints" \
    --argjson phase1_power "$phase1_power" \
    --argjson r1cs_bytes "$(stat -c %s "$r1cs")" \
    --argjson witness_bytes "$(stat -c %s "$wtns")" \
    --argjson metrics_data "$metrics_json" \
    '{
        format: "achronyme-proving-export",
        version: 1,
        tracked_checkout_clean: true,
        git_commit: $git_commit,
        achronyme_binary_sha256: $achronyme_binary_sha256,
        achronyme_version: $achronyme_version,
        fixture: {source_sha256: $source_sha256, inputs_sha256: $inputs_sha256},
        circuit: {
            constraints: $constraints,
            public_inputs: $public_inputs,
            variables: $variables,
            r1cs: {sha256: $r1cs_sha256, bytes: $r1cs_bytes},
            witness: {sha256: $witness_sha256, bytes: $witness_bytes}
        },
        bounds: {
            timeout_seconds: (if $timeout_seconds == "" then null else ($timeout_seconds | tonumber) end),
            max_virtual_memory_gib: (if $max_virtual_memory_gib == "" then null else ($max_virtual_memory_gib | tonumber) end),
            min_free_disk_gib: (if $min_free_disk_gib == "" then null else ($min_free_disk_gib | tonumber) end),
            phase1_power: $phase1_power
        },
        metrics: $metrics_data
    }' >"$tmp_output"
chmod 600 "$tmp_output"

require_clean_tracked_checkout "$repo_root"
[[ "$(git -C "$repo_root" rev-parse --verify HEAD)" == "$git_commit" ]] || \
    die "Git commit changed while recording export evidence"
[[ "$(sha256_file "$ach_bin")" == "$achronyme_binary_sha256" ]] || \
    die "Achronyme binary changed while recording export evidence"
[[ "$(sha256_file "$source_file")" == "$source_sha256" ]] || \
    die "Circom source changed while recording export evidence"
[[ "$(sha256_file "$input_file")" == "$inputs_sha256" ]] || \
    die "witness input changed while recording export evidence"
[[ "$(sha256_file "$r1cs")" == "$r1cs_sha256" ]] || \
    die "R1CS changed while recording export evidence"
[[ "$(sha256_file "$wtns")" == "$witness_sha256" ]] || \
    die "witness changed while recording export evidence"
mv "$tmp_output" "$output"
trap - EXIT
printf 'export evidence: %s\n' "$output"
