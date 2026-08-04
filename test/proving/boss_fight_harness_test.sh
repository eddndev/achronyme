#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ach_bin=${ACH_BIN:-$repo_root/target/debug/ach}
harness="$repo_root/scripts/proving/run-boss-fight.sh"
common="$repo_root/scripts/proving/common.sh"

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT
checkout="$tmp_dir/checkout"
git init -q "$checkout"
git -C "$checkout" config user.email test@example.invalid
git -C "$checkout" config user.name "Achronyme test"
printf 'tracked\n' >"$checkout/tracked.txt"
git -C "$checkout" add tracked.txt
git -C "$checkout" commit -qm initial

bash -c 'source "$1"; require_clean_tracked_checkout "$2"' _ "$common" "$checkout"
printf 'private input\n' >"$checkout/untracked-input.toml"
bash -c 'source "$1"; require_clean_tracked_checkout "$2"' _ "$common" "$checkout"
printf 'dirty\n' >>"$checkout/tracked.txt"
if bash -c 'source "$1"; require_clean_tracked_checkout "$2"' _ \
    "$common" "$checkout" >"$tmp_dir/dirty.out" 2>"$tmp_dir/dirty.err"; then
    printf 'dirty checkout unexpectedly accepted\n' >&2
    exit 1
fi
grep -Fq 'tracked checkout must be clean' "$tmp_dir/dirty.err"

bash -n "$harness"
plan=$("$harness" --ach-bin "$ach_bin" --work-dir /tmp/achronyme-bossfight-test \
    --timeout-seconds 120 --max-virtual-memory-gib 30 --min-free-disk-gib 1 --dry-run)

grep -Fq 'ecdsa_verify_test.circom' <<<"$plan"
grep -Fq 'ecdsa_verify.inputs.toml' <<<"$plan"
grep -Fq 'phase-1 power: 21' <<<"$plan"
grep -Fq -- '--low-memory' <<<"$plan"
grep -Fq 'max_virtual_memory_gib: 30' <<<"$plan"
grep -Eq 'git_commit: [0-9a-f]{40}' <<<"$plan"
grep -Eq 'achronyme_binary_sha256: [0-9a-f]{64}' <<<"$plan"
grep -Fq 'achronyme_binary_sha256' "$harness"

printf 'boss-fight harness test passed\n'
