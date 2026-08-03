#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ach_bin=${ACH_BIN:-$repo_root/target/debug/ach}
harness="$repo_root/scripts/proving/run-boss-fight.sh"

bash -n "$harness"
plan=$("$harness" --ach-bin "$ach_bin" --work-dir /tmp/achronyme-bossfight-test \
    --timeout-seconds 120 --max-virtual-memory-gib 30 --min-free-disk-gib 1 --dry-run)

grep -Fq 'ecdsa_verify_test.circom' <<<"$plan"
grep -Fq 'ecdsa_verify.inputs.toml' <<<"$plan"
grep -Fq 'phase-1 power: 21' <<<"$plan"
grep -Fq -- '--low-memory' <<<"$plan"
grep -Fq 'max_virtual_memory_gib: 30' <<<"$plan"

printf 'boss-fight harness test passed\n'
