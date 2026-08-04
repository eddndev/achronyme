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
require_command node
require_command npm

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
source_file="$repo_root/test/circom/multiplier.circom"
input_file="$repo_root/test/proving/multiplier.inputs.toml"

"$ach_bin" --no-config circom "$source_file" --input-file "$input_file" \
    --r1cs "$r1cs" --wtns "$wtns"

evidence_checkout="$work_root/evidence-checkout"
mkdir -p "$evidence_checkout/scripts/proving"
cp "$scripts_dir/common.sh" "$scripts_dir/finalize-phase2.sh" \
    "$scripts_dir/record-export-evidence.sh" "$evidence_checkout/scripts/proving/"
mkdir "$evidence_checkout/scripts/proving/drand"
cp "$scripts_dir/drand"/*.mjs "$scripts_dir/drand"/package*.json \
    "$evidence_checkout/scripts/proving/drand/"
cp "$repo_root/.gitignore" "$evidence_checkout/.gitignore"
git init -q "$evidence_checkout"
git -C "$evidence_checkout" config user.email test@example.invalid
git -C "$evidence_checkout" config user.name "Achronyme test"
git -C "$evidence_checkout" add .gitignore scripts
git -C "$evidence_checkout" commit -qm initial
npm ci --prefix "$evidence_checkout/scripts/proving/drand" --ignore-scripts \
    >/dev/null
recorded_export="$work_root/recorded-export.json"
"$evidence_checkout/scripts/proving/record-export-evidence.sh" \
    --ach-bin "$ach_bin" --source "$source_file" --input-file "$input_file" \
    --r1cs "$r1cs" --wtns "$wtns" --output "$recorded_export"
jq -e '
    .format == "achronyme-proving-export" and
    .version == 1 and
    .tracked_checkout_clean == true and
    (.achronyme_binary_sha256 | test("^[0-9a-f]{64}$")) and
    .circuit.constraints == 1
' "$recorded_export" >/dev/null
printf '\n' >>"$evidence_checkout/scripts/proving/common.sh"
if "$evidence_checkout/scripts/proving/record-export-evidence.sh" \
    --ach-bin "$ach_bin" --source "$source_file" --input-file "$input_file" \
    --r1cs "$r1cs" --wtns "$wtns" --output "$work_root/dirty-export.json" \
    >"$work_root/dirty-export.out" 2>"$work_root/dirty-export.err"; then
    die "dirty export evidence checkout unexpectedly passed"
fi
grep -Fq 'checkout must be clean' "$work_root/dirty-export.err"
[[ ! -e "$work_root/dirty-export.json" ]] || \
    die "dirty checkout wrote export evidence"
git -C "$evidence_checkout" restore scripts/proving/common.sh

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

contributed_zkey="$ceremony_dir/circuit_0001.zkey"
printf '%s\n' 'test-only deterministic entropy' | \
    "$scripts_dir/contribute-phase2.sh" \
        --input "$ceremony_dir/circuit_0000.zkey" \
        --output "$contributed_zkey" --name "test independent contributor" \
        --metrics "$ceremony_dir/contribute.metrics.jsonl" >/dev/null
snarkjs zkey verify "$r1cs" "$phase1" "$contributed_zkey" \
    >"$work_root/test-zkey-verify.log" 2>&1
contribution=$(extract_zkey_contributions "$work_root/test-zkey-verify.log" | sed -n '1p')
[[ "$contribution" == *"|"* ]] || die "test contribution hash was not parsed"
contributor="${contribution/|/=}"
beacon_evidence="$repo_root/test/proving/fixtures/drand_quicknet_round_31006463.json"
beacon_evidence_sha256=$(sha256_file "$beacon_evidence")

export_evidence="$recorded_export"
finalizer="$evidence_checkout/scripts/proving/finalize-phase2.sh"

declare -a finalize_args=(
    --r1cs "$r1cs" --wtns "$wtns" --phase1 "$phase1"
    --contributed-zkey "$contributed_zkey"
    --beacon-evidence "$beacon_evidence" --beacon-iterations 10
    --source "$source_file" --input-file "$input_file"
    --ach-bin "$ach_bin" --phase1-source "https://example.invalid/test-only.ptau"
    --phase1-blake2b512 "$phase1_blake2b512" --contributor "$contributor"
)

tampered_export="$work_root/tampered-export.json"
jq '.circuit.witness.sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"' \
    "$export_evidence" >"$tampered_export"
if "$finalizer" "${finalize_args[@]}" \
    --export-evidence "$tampered_export" --work-dir "$work_root/tampered-finalize" \
    --store "$work_root/tampered-store" >"$work_root/tampered.out" \
    2>"$work_root/tampered.err"; then
    die "tampered export evidence unexpectedly passed finalization preflight"
fi
grep -Fq 'export evidence witness SHA-256 mismatch' "$work_root/tampered.err"
[[ ! -e "$work_root/tampered-store" ]] || \
    die "tampered export evidence created a trusted-key store"

"$finalizer" "${finalize_args[@]}" \
    --export-evidence "$export_evidence" --work-dir "$ceremony_dir" --store "$store"

evidence="$ceremony_dir/release-evidence.json"
jq -e --arg beacon_evidence_sha256 "$beacon_evidence_sha256" '
    .format == "achronyme-proving-release-evidence" and
    .version == 3 and
    (.git_commit | test("^[0-9a-f]{40,64}$")) and
    (.achronyme_binary_sha256 | test("^[0-9a-f]{64}$")) and
    (.export_evidence.sha256 | test("^[0-9a-f]{64}$")) and
    .circuit.constraints == 1 and
    .ceremony.final_beacon.source == "https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/public/31006463" and
    .ceremony.final_beacon.round == 31006463 and
    .ceremony.final_beacon.randomness == "06664dcb57258c3ad1142e1f19575f3e597d29ee8eb49e957355dbab9d6935c9" and
    .ceremony.final_beacon.evidence_sha256 == $beacon_evidence_sha256 and
    .ceremony.final_beacon.commitment_publication == "https://example.invalid/test-only/commitment-31006463" and
    .ceremony.final_beacon.commitment_sha256 == "54703c6c0df8236e97524ec5f0aaa8733566cf8131188d280a84b9b3f0e18a59" and
    .ceremony.final_beacon.iterations == 10 and
    (.ceremony.final_beacon.contribution_hash | test("^[0-9a-f]{128}$")) and
    (.metrics | length >= 12) and
    (.verification | all(. == true))
' "$evidence" >/dev/null
r1cs_sha256=$(sha256_file "$r1cs")
require_regular_file "$store/$r1cs_sha256/manifest.json" "trusted-key manifest"
require_regular_file "$store/$r1cs_sha256/transcript.json" "ceremony transcript"
require_regular_file "$store/$r1cs_sha256/proving_key.zkey" "trusted proving key"
jq -e '
    .version == 3 and
    .ceremony.final_beacon.round == 31006463 and
    .ceremony.final_beacon.commitment_publication == "https://example.invalid/test-only/commitment-31006463" and
    (.ceremony.final_beacon.evidence_sha256 | test("^[0-9a-f]{64}$"))
' "$store/$r1cs_sha256/manifest.json" >/dev/null

printf 'trusted setup workflow test passed\n'
