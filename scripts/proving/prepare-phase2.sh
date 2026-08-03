#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$script_dir/common.sh"

usage() {
    printf 'usage: %s --r1cs FILE --wtns FILE --phase1 FILE --work-dir DIR [--phase1-blake2b512 HASH]\n' "$0" >&2
    exit 2
}

r1cs=
wtns=
phase1=
work_dir=
phase1_blake2b512=$ACHRONYME_PHASE1_BLAKE2B512
while [[ $# -gt 0 ]]; do
    case "$1" in
        --r1cs) r1cs=${2:-}; shift 2 ;;
        --wtns) wtns=${2:-}; shift 2 ;;
        --phase1) phase1=${2:-}; shift 2 ;;
        --work-dir) work_dir=${2:-}; shift 2 ;;
        --phase1-blake2b512) phase1_blake2b512=${2:-}; shift 2 ;;
        *) usage ;;
    esac
done
[[ -n "$r1cs" && -n "$wtns" && -n "$phase1" && -n "$work_dir" ]] || usage
[[ "$phase1_blake2b512" =~ ^[0-9a-f]{128}$ ]] || die "invalid phase-1 BLAKE2b-512"

require_command b2sum
require_command /usr/bin/time
require_snarkjs
require_regular_file "$r1cs" "R1CS artifact"
require_regular_file "$wtns" "witness artifact"
require_regular_file "$phase1" "phase-1 artifact"
verify_phase1_hash "$phase1" "$phase1_blake2b512"

mkdir -p "$work_dir"
require_directory "$work_dir" "phase-2 work directory"
initial_zkey="$work_dir/circuit_0000.zkey"
metrics="$work_dir/prepare.metrics.jsonl"
ensure_absent "$initial_zkey"
create_metrics_file "$metrics"

run_measured "$metrics" phase1_transcript snarkjs powersoftau verify "$phase1"
run_measured "$metrics" r1cs_info snarkjs r1cs info "$r1cs"
run_measured "$metrics" witness_check snarkjs wtns check "$r1cs" "$wtns"
run_measured "$metrics" phase2_initial_setup \
    snarkjs groth16 setup "$r1cs" "$phase1" "$initial_zkey"

printf 'initial zero-contribution key: %s\n' "$initial_zkey"
printf 'warning: this key is not trusted and must not be used for proving\n' >&2
printf 'next: transfer it to an independent contributor and run contribute-phase2.sh interactively\n'
