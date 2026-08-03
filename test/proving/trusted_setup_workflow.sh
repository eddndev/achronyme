#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ach_bin=${ACH_BIN:-$repo_root/target/debug/ach}
scripts_dir="$repo_root/scripts/proving"
source "$scripts_dir/common.sh"

for script in "$scripts_dir"/*.sh; do
    bash -n "$script"
done
if rg -n -- '--entropy|[[:space:]]-e=' \
    "$scripts_dir"/*.sh >/dev/null; then
    die "production ceremony scripts must not accept command-line entropy"
fi

require_regular_file "$ach_bin" "Achronyme test binary"
require_snarkjs
require_command jq

work_root=$(mktemp -d)
cleanup() {
    chmod -R u+w "$work_root" 2>/dev/null || true
    rm -rf -- "$work_root"
}
trap cleanup EXIT

export_dir="$work_root/export"
ceremony_dir="$work_root/ceremony"
store="$work_root/trusted-keys"
mkdir "$export_dir" "$ceremony_dir"
r1cs="$export_dir/circuit.r1cs"
wtns="$export_dir/witness.wtns"
phase1="$work_root/phase1.ptau"

"$ach_bin" --no-config circom "$repo_root/test/circom/multiplier.circom" \
    --input-file "$repo_root/test/proving/multiplier.inputs.toml" \
    --r1cs "$r1cs" --wtns "$wtns"
snarkjs powersoftau new bn128 8 "$work_root/pot_0000.ptau" >/dev/null
snarkjs powersoftau beacon \
    "$work_root/pot_0000.ptau" "$work_root/pot_final.ptau" \
    0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef \
    10 -n="test phase1 beacon" >/dev/null
snarkjs powersoftau prepare phase2 "$work_root/pot_final.ptau" "$phase1" >/dev/null
phase1_blake2b512=$(blake2b512_file "$phase1")

"$scripts_dir/prepare-phase2.sh" \
    --r1cs "$r1cs" --wtns "$wtns" --phase1 "$phase1" \
    --work-dir "$ceremony_dir" --phase1-blake2b512 "$phase1_blake2b512"

final_zkey="$ceremony_dir/circuit_final.zkey"
snarkjs zkey beacon \
    "$ceremony_dir/circuit_0000.zkey" "$final_zkey" \
    abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789 \
    10 -n="test independent beacon" >/dev/null
snarkjs zkey verify "$r1cs" "$phase1" "$final_zkey" \
    >"$work_root/test-zkey-verify.log" 2>&1
contribution=$(extract_zkey_contributions "$work_root/test-zkey-verify.log" | sed -n '1p')
[[ "$contribution" == *"|"* ]] || die "test contribution hash was not parsed"
contributor="${contribution/|/=}"

"$scripts_dir/finalize-phase2.sh" \
    --r1cs "$r1cs" --wtns "$wtns" --phase1 "$phase1" --zkey "$final_zkey" \
    --source "$repo_root/test/circom/multiplier.circom" \
    --input-file "$repo_root/test/proving/multiplier.inputs.toml" \
    --work-dir "$ceremony_dir" --store "$store" --ach-bin "$ach_bin" \
    --phase1-source "https://example.invalid/test-only.ptau" \
    --phase1-blake2b512 "$phase1_blake2b512" --contributor "$contributor"

evidence="$ceremony_dir/release-evidence.json"
jq -e '
    .format == "achronyme-proving-release-evidence" and
    .version == 1 and
    .circuit.constraints == 1 and
    (.metrics | length >= 10) and
    (.verification | all(. == true))
' "$evidence" >/dev/null
r1cs_sha256=$(sha256_file "$r1cs")
require_regular_file "$store/$r1cs_sha256/manifest.json" "trusted-key manifest"
require_regular_file "$store/$r1cs_sha256/transcript.json" "ceremony transcript"
require_regular_file "$store/$r1cs_sha256/proving_key.zkey" "trusted proving key"

printf 'trusted setup workflow test passed\n'
