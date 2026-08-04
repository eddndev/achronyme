#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)
source "$script_dir/common.sh"

usage() {
    printf '%s\n' \
        "usage: $0 --ach-bin FILE --work-dir DIR [--timeout-seconds N]" \
        "  [--max-virtual-memory-gib N] [--min-free-disk-gib N] [--dry-run]" >&2
    exit 2
}

ach_bin=
work_dir=
timeout_seconds=21600
max_virtual_memory_gib=30
min_free_disk_gib=20
dry_run=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --ach-bin) ach_bin=${2:-}; shift 2 ;;
        --work-dir) work_dir=${2:-}; shift 2 ;;
        --timeout-seconds) timeout_seconds=${2:-}; shift 2 ;;
        --max-virtual-memory-gib) max_virtual_memory_gib=${2:-}; shift 2 ;;
        --min-free-disk-gib) min_free_disk_gib=${2:-}; shift 2 ;;
        --dry-run) dry_run=true; shift ;;
        *) usage ;;
    esac
done
[[ -n "$ach_bin" && -n "$work_dir" ]] || usage
[[ "$timeout_seconds" =~ ^[1-9][0-9]*$ ]] || die "timeout must be a positive integer"
[[ "$max_virtual_memory_gib" =~ ^[1-9][0-9]*$ ]] || die "memory limit must be a positive integer"
[[ "$min_free_disk_gib" =~ ^[1-9][0-9]*$ ]] || die "disk limit must be a positive integer"

source_file="$repo_root/test/circomlib/ecdsa_verify_test.circom"
input_file="$repo_root/test/proving/ecdsa_verify.inputs.toml"
library_dir="$repo_root/test/circomlib"
require_regular_file "$ach_bin" "Achronyme binary"
require_regular_file "$source_file" "ECDSA boss-fight circuit"
require_regular_file "$input_file" "ECDSA boss-fight input fixture"
require_directory "$library_dir" "circomlib directory"
require_command git
require_command sha256sum

git_commit=$(git -C "$repo_root" rev-parse --verify HEAD)
[[ "$git_commit" =~ ^[0-9a-f]{40,64}$ ]] || die "cannot resolve the tested Git commit"
achronyme_binary_sha256=$(sha256_file "$ach_bin")

max_virtual_memory_bytes=$((max_virtual_memory_gib * 1024 * 1024 * 1024))
declare -a export_command=(
    timeout --signal=TERM --kill-after=60 "$timeout_seconds"
    prlimit --as="$max_virtual_memory_bytes"
    "$ach_bin" --no-config circom "$source_file"
    --input-file "$input_file"
    --lib "$library_dir"
    --r1cs "$work_dir/export/circuit.r1cs"
    --wtns "$work_dir/export/witness.wtns"
    --low-memory
)

if [[ "$dry_run" == true ]]; then
    printf 'boss-fight source: %s\n' "$source_file"
    printf 'boss-fight inputs: %s\n' "$input_file"
    printf 'phase-1 power: %s\n' "$ACHRONYME_PHASE1_POWER"
    printf 'timeout_seconds: %s\n' "$timeout_seconds"
    printf 'max_virtual_memory_gib: %s\n' "$max_virtual_memory_gib"
    printf 'min_free_disk_gib: %s\n' "$min_free_disk_gib"
    printf 'git_commit: %s\n' "$git_commit"
    printf 'achronyme_binary_sha256: %s\n' "$achronyme_binary_sha256"
    printf 'command:'
    printf ' %q' "${export_command[@]}"
    printf '\n'
    exit 0
fi

require_command prlimit
require_command timeout
require_command /usr/bin/time
require_snarkjs
require_clean_tracked_checkout "$repo_root"
ensure_absent "$work_dir"
mkdir -p "$(dirname "$work_dir")"
available_kib=$(df -Pk "$(dirname "$work_dir")" | awk 'NR == 2 {print $4}')
required_kib=$((min_free_disk_gib * 1024 * 1024))
[[ "$available_kib" =~ ^[0-9]+$ && "$available_kib" -ge "$required_kib" ]] || \
    die "boss-fight host has less than ${min_free_disk_gib} GiB free disk"
mkdir -p "$work_dir/export"

metrics="$work_dir/export.metrics.jsonl"
log="$work_dir/export.log"
create_metrics_file "$metrics"
run_measured_logged "$metrics" bossfight_export "$log" "${export_command[@]}"

r1cs="$work_dir/export/circuit.r1cs"
wtns="$work_dir/export/witness.wtns"
require_regular_file "$r1cs" "boss-fight R1CS"
require_regular_file "$wtns" "boss-fight witness"
run_measured "$metrics" bossfight_witness_check snarkjs wtns check "$r1cs" "$wtns"

evidence="$work_dir/bossfight-export.json"
"$script_dir/record-export-evidence.sh" \
    --ach-bin "$ach_bin" --source "$source_file" --input-file "$input_file" \
    --r1cs "$r1cs" --wtns "$wtns" --output "$evidence" --metrics "$metrics" \
    --phase1-power "$ACHRONYME_PHASE1_POWER" \
    --timeout-seconds "$timeout_seconds" \
    --max-virtual-memory-gib "$max_virtual_memory_gib" \
    --min-free-disk-gib "$min_free_disk_gib" \
    --expected-git-commit "$git_commit" \
    --expected-binary-sha256 "$achronyme_binary_sha256"

printf 'boss-fight export evidence: %s\n' "$evidence"
printf 'next: verify/download phase 1, then run prepare-phase2.sh with these R1CS and witness artifacts\n'
