#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)
source "$script_dir/common.sh"

usage() {
    printf '%s\n' \
        "usage: $0 --r1cs FILE --wtns FILE --phase1 FILE --contributed-zkey FILE" \
        "  --export-evidence FILE" \
        "  --source CIRCOM --input-file TOML [--lib DIR ...] --work-dir DIR" \
        "  --store DIR --ach-bin FILE --phase1-source URL" \
        "  --contributor ID=HASH [--contributor ID=HASH ...]" \
        "  --beacon-source URL --beacon-randomness 64_HEX --beacon-iterations N" \
        "  [--phase1-blake2b512 HASH]" >&2
    exit 2
}

r1cs=
wtns=
phase1=
contributed_zkey=
export_evidence=
source_file=
input_file=
work_dir=
store=
ach_bin=
phase1_source=
phase1_blake2b512=$ACHRONYME_PHASE1_BLAKE2B512
beacon_source=
beacon_randomness=
beacon_iterations=
declare -a lib_dirs=()
declare -a contributors=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --r1cs) r1cs=${2:-}; shift 2 ;;
        --wtns) wtns=${2:-}; shift 2 ;;
        --phase1) phase1=${2:-}; shift 2 ;;
        --contributed-zkey) contributed_zkey=${2:-}; shift 2 ;;
        --export-evidence) export_evidence=${2:-}; shift 2 ;;
        --source) source_file=${2:-}; shift 2 ;;
        --input-file) input_file=${2:-}; shift 2 ;;
        --lib) lib_dirs+=("${2:-}"); shift 2 ;;
        --work-dir) work_dir=${2:-}; shift 2 ;;
        --store) store=${2:-}; shift 2 ;;
        --ach-bin) ach_bin=${2:-}; shift 2 ;;
        --phase1-source) phase1_source=${2:-}; shift 2 ;;
        --phase1-blake2b512) phase1_blake2b512=${2:-}; shift 2 ;;
        --contributor) contributors+=("${2:-}"); shift 2 ;;
        --beacon-source) beacon_source=${2:-}; shift 2 ;;
        --beacon-randomness) beacon_randomness=${2:-}; shift 2 ;;
        --beacon-iterations) beacon_iterations=${2:-}; shift 2 ;;
        *) usage ;;
    esac
done
[[ -n "$r1cs" && -n "$wtns" && -n "$phase1" && -n "$contributed_zkey" ]] || usage
[[ -n "$export_evidence" ]] || usage
[[ -n "$source_file" && -n "$input_file" && -n "$work_dir" ]] || usage
[[ -n "$store" && -n "$ach_bin" && -n "$phase1_source" ]] || usage
[[ ${#contributors[@]} -gt 0 ]] || die "at least one --contributor is required"
[[ "$beacon_source" =~ ^https://[^[:space:]]+$ && ${#beacon_source} -le 2048 ]] || \
    die "final beacon source must be a non-empty HTTPS URL"
[[ "$beacon_randomness" =~ ^[0-9a-f]{64}$ ]] || \
    die "final beacon randomness must be lowercase 64-hex"
[[ "$beacon_iterations" =~ ^[0-9]+$ ]] || die "invalid final beacon iterations"
((beacon_iterations >= 10 && beacon_iterations <= 63)) || \
    die "final beacon iterations must be between 10 and 63"
[[ "$phase1_blake2b512" =~ ^[0-9a-f]{128}$ ]] || die "invalid phase-1 BLAKE2b-512"
for contributor in "${contributors[@]}"; do
    validate_contributor_pair "$contributor"
done

require_command b2sum
require_command git
require_command jq
require_command od
require_command sha256sum
require_command /usr/bin/time
require_snarkjs
require_regular_file "$r1cs" "R1CS artifact"
require_regular_file "$wtns" "witness artifact"
require_regular_file "$phase1" "phase-1 artifact"
require_regular_file "$contributed_zkey" "contributed zkey"
require_regular_file "$export_evidence" "proving export evidence"
require_regular_file "$source_file" "Circom source"
require_regular_file "$input_file" "witness input"
require_regular_file "$ach_bin" "Achronyme binary"
require_clean_tracked_checkout "$repo_root"
verify_phase1_hash "$phase1" "$phase1_blake2b512"
phase1_sha256_start=$(sha256_file "$phase1")
contributed_zkey_sha256_start=$(sha256_file "$contributed_zkey")

jq -e '
    type == "object" and
    .format == "achronyme-proving-export" and
    .tracked_checkout_clean == true and
    .version == 1 and
    (.git_commit | type == "string") and
    (.achronyme_binary_sha256 | type == "string") and
    (.fixture.source_sha256 | type == "string") and
    (.fixture.inputs_sha256 | type == "string") and
    (.circuit.r1cs.sha256 | type == "string") and
    (.circuit.r1cs.bytes | type == "number") and
    (.circuit.witness.sha256 | type == "string") and
    (.circuit.witness.bytes | type == "number") and
    (.circuit.constraints | type == "number")
' "$export_evidence" >/dev/null || die "invalid proving export evidence"

export_evidence_format=$(jq -r '.format' "$export_evidence")
export_git_commit=$(jq -r '.git_commit' "$export_evidence")
export_binary_sha256=$(jq -r '.achronyme_binary_sha256' "$export_evidence")
export_source_sha256=$(jq -r '.fixture.source_sha256' "$export_evidence")
export_inputs_sha256=$(jq -r '.fixture.inputs_sha256' "$export_evidence")
export_r1cs_sha256=$(jq -r '.circuit.r1cs.sha256' "$export_evidence")
export_wtns_sha256=$(jq -r '.circuit.witness.sha256' "$export_evidence")
export_r1cs_bytes=$(jq -r '.circuit.r1cs.bytes' "$export_evidence")
export_wtns_bytes=$(jq -r '.circuit.witness.bytes' "$export_evidence")
export_constraints=$(jq -r '.circuit.constraints' "$export_evidence")
for digest in "$export_binary_sha256" "$export_source_sha256" \
    "$export_inputs_sha256" "$export_r1cs_sha256" "$export_wtns_sha256"; do
    [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || die "invalid digest in proving export evidence"
done
[[ "$export_git_commit" =~ ^[0-9a-f]{40,64}$ ]] || \
    die "invalid Git commit in proving export evidence"
for value in "$export_r1cs_bytes" "$export_wtns_bytes" "$export_constraints"; do
    [[ "$value" =~ ^[0-9]+$ ]] || die "invalid count in proving export evidence"
done

[[ "$(git -C "$repo_root" rev-parse --verify HEAD)" == "$export_git_commit" ]] || \
    die "export evidence Git commit does not match the finalization checkout"
[[ "$(sha256_file "$ach_bin")" == "$export_binary_sha256" ]] || \
    die "export evidence Achronyme binary SHA-256 mismatch"
[[ "$(sha256_file "$source_file")" == "$export_source_sha256" ]] || \
    die "export evidence source SHA-256 mismatch"
[[ "$(sha256_file "$input_file")" == "$export_inputs_sha256" ]] || \
    die "export evidence input SHA-256 mismatch"
[[ "$(sha256_file "$r1cs")" == "$export_r1cs_sha256" ]] || \
    die "export evidence R1CS SHA-256 mismatch"
[[ "$(sha256_file "$wtns")" == "$export_wtns_sha256" ]] || \
    die "export evidence witness SHA-256 mismatch"
[[ "$(stat -c %s "$r1cs")" == "$export_r1cs_bytes" ]] || \
    die "export evidence R1CS size mismatch"
[[ "$(stat -c %s "$wtns")" == "$export_wtns_bytes" ]] || \
    die "export evidence witness size mismatch"
export_evidence_sha256=$(sha256_file "$export_evidence")

mkdir -p "$work_dir"
require_directory "$work_dir" "finalization work directory"
snarkjs_dir="$work_dir/snarkjs"
achronyme_dir="$work_dir/achronyme"
ensure_absent "$snarkjs_dir"
ensure_absent "$achronyme_dir"
mkdir "$snarkjs_dir" "$achronyme_dir"
metrics="$work_dir/finalize.metrics.jsonl"
create_metrics_file "$metrics"

phase1_log="$work_dir/phase1-verify.log"
contributed_zkey_log="$work_dir/contributed-zkey-verify.log"
final_zkey_log="$work_dir/final-zkey-verify.log"
run_measured_logged "$metrics" phase1_transcript "$phase1_log" \
    snarkjs powersoftau verify "$phase1"
run_measured "$metrics" witness_check snarkjs wtns check "$r1cs" "$wtns"
run_measured_logged "$metrics" contributed_zkey_verify "$contributed_zkey_log" \
    snarkjs zkey verify "$r1cs" "$phase1" "$contributed_zkey"
assert_contributors_in_log "$contributed_zkey_log" "${contributors[@]}"

beacon_name="Achronyme final beacon"
final_zkey="$work_dir/circuit_final.zkey"
partial_final_zkey="$work_dir/.circuit_final.zkey.partial"
ensure_absent "$final_zkey"
ensure_absent "$partial_final_zkey"
run_measured "$metrics" final_beacon \
    snarkjs zkey beacon "$contributed_zkey" "$partial_final_zkey" \
        "$beacon_randomness" "$beacon_iterations" -n="$beacon_name"
require_regular_file "$partial_final_zkey" "unverified final zkey"
run_measured_logged "$metrics" final_zkey_verify "$final_zkey_log" \
    snarkjs zkey verify "$r1cs" "$phase1" "$partial_final_zkey"
assert_contributors_in_log "$final_zkey_log" "${contributors[@]}"

beacon_contribution_hash=
while IFS='|' read -r contribution_id contribution_hash; do
    if [[ "$contribution_id" == "$beacon_name" ]]; then
        [[ -z "$beacon_contribution_hash" ]] || \
            die "verified zkey contains duplicate final beacon records"
        beacon_contribution_hash=$contribution_hash
    fi
done < <(extract_zkey_contributions "$final_zkey_log")
[[ "$beacon_contribution_hash" =~ ^[0-9a-f]{128}$ ]] || \
    die "final beacon contribution hash was not found in verified zkey"
mv -- "$partial_final_zkey" "$final_zkey"
zkey_sha256_start=$(sha256_file "$final_zkey")

snarkjs_vkey="$snarkjs_dir/verification_key.json"
snarkjs_proof="$snarkjs_dir/proof.json"
snarkjs_public="$snarkjs_dir/public.json"
run_measured "$metrics" export_verification_key \
    snarkjs zkey export verificationkey "$final_zkey" "$snarkjs_vkey"
run_measured "$metrics" snarkjs_prove \
    snarkjs groth16 prove "$final_zkey" "$wtns" "$snarkjs_proof" "$snarkjs_public"
run_measured "$metrics" snarkjs_verify \
    snarkjs groth16 verify "$snarkjs_vkey" "$snarkjs_public" "$snarkjs_proof"
run_measured "$metrics" achronyme_verify_snarkjs \
    "$ach_bin" verify --proof "$snarkjs_proof" --public "$snarkjs_public" \
        --vkey "$snarkjs_vkey" --curve bn254 --format json

declare -a contributor_args=()
for contributor in "${contributors[@]}"; do
    contributor_args+=(--contributor "$contributor")
done
run_measured "$metrics" package_trusted_key \
    "$ach_bin" trusted-setup package \
        --r1cs "$r1cs" --zkey "$final_zkey" \
        --contributed-zkey "$contributed_zkey" \
        --phase1 "$phase1" --store "$store" \
        --tool "$ACHRONYME_SNARKJS_VERSION" --phase1-source "$phase1_source" \
        --phase1-blake2b512 "$phase1_blake2b512" \
        --beacon-source "$beacon_source" --beacon-randomness "$beacon_randomness" \
        --beacon-iterations "$beacon_iterations" \
        --beacon-contribution-hash "$beacon_contribution_hash" \
        "${contributor_args[@]}" --format json

achronyme_r1cs="$achronyme_dir/circuit.r1cs"
achronyme_wtns="$achronyme_dir/witness.wtns"
declare -a lib_args=()
for lib_dir in "${lib_dirs[@]}"; do
    require_directory "$lib_dir" "Circom library directory"
    lib_args+=(--lib "$lib_dir")
done
run_measured "$metrics" achronyme_trusted_prove \
    "$ach_bin" --no-config --trusted-key-dir "$store" circom "$source_file" \
        --input-file "$input_file" --r1cs "$achronyme_r1cs" \
        --wtns "$achronyme_wtns" --prove "${lib_args[@]}"

original_r1cs_sha256=$(sha256_file "$r1cs")
achronyme_r1cs_sha256=$(sha256_file "$achronyme_r1cs")
[[ "$achronyme_r1cs_sha256" == "$original_r1cs_sha256" ]] || \
    die "Achronyme re-exported R1CS does not match ceremony R1CS"

achronyme_proof="$achronyme_dir/proof.json"
achronyme_public="$achronyme_dir/public.json"
achronyme_vkey="$achronyme_dir/vkey.json"
run_measured "$metrics" achronyme_verify_achronyme \
    "$ach_bin" verify --proof "$achronyme_proof" --public "$achronyme_public" \
        --vkey "$achronyme_vkey" --curve bn254 --format json
run_measured "$metrics" snarkjs_verify_achronyme \
    snarkjs groth16 verify "$snarkjs_vkey" "$achronyme_public" "$achronyme_proof"

r1cs_constraints=$(od -An -tu4 -j84 -N4 "$r1cs" | tr -d ' ')
[[ "$r1cs_constraints" =~ ^[0-9]+$ ]] || die "cannot read R1CS constraint count"
[[ "$r1cs_constraints" == "$export_constraints" ]] || \
    die "export evidence constraint count mismatch"
r1cs_sha256=$original_r1cs_sha256
zkey_sha256=$(sha256_file "$final_zkey")
contributed_zkey_sha256=$contributed_zkey_sha256_start
phase1_sha256=$(sha256_file "$phase1")
artifact_dir="$store/$r1cs_sha256"
require_directory "$artifact_dir" "packaged trusted-key artifact"

[[ "$(sha256_file "$export_evidence")" == "$export_evidence_sha256" ]] || \
    die "proving export evidence changed during finalization"
[[ "$(git -C "$repo_root" rev-parse --verify HEAD)" == "$export_git_commit" ]] || \
    die "Git commit changed during phase-2 finalization"
[[ "$(sha256_file "$ach_bin")" == "$export_binary_sha256" ]] || \
    die "Achronyme binary changed during phase-2 finalization"
[[ "$(sha256_file "$source_file")" == "$export_source_sha256" ]] || \
    die "Circom source changed during phase-2 finalization"
[[ "$(sha256_file "$input_file")" == "$export_inputs_sha256" ]] || \
    die "witness input changed during phase-2 finalization"
[[ "$(sha256_file "$r1cs")" == "$export_r1cs_sha256" ]] || \
    die "R1CS changed during phase-2 finalization"
[[ "$(sha256_file "$wtns")" == "$export_wtns_sha256" ]] || \
    die "witness changed during phase-2 finalization"
[[ "$(sha256_file "$phase1")" == "$phase1_sha256_start" ]] || \
    die "phase-1 artifact changed during phase-2 finalization"
[[ "$(sha256_file "$contributed_zkey")" == "$contributed_zkey_sha256_start" ]] || \
    die "contributed zkey changed during phase-2 finalization"
[[ "$(sha256_file "$final_zkey")" == "$zkey_sha256_start" ]] || \
    die "final zkey changed during phase-2 finalization"
require_clean_tracked_checkout "$repo_root"

contributors_json="$work_dir/contributors.json"
metrics_json="$work_dir/metrics.json"
evidence="$work_dir/release-evidence.json"
ensure_absent "$contributors_json"
ensure_absent "$metrics_json"
ensure_absent "$evidence"
printf '%s\n' "${contributors[@]}" | jq -R -s '
    split("\n") | map(select(length > 0)) |
    map(split("=") | {id: .[0], contribution_hash: .[1]})
' >"$contributors_json"
shopt -s nullglob
metric_files=("$work_dir"/*.metrics.jsonl)
shopt -u nullglob
jq -s '.' "${metric_files[@]}" >"$metrics_json"

ach_version=$("$ach_bin" --version)
jq -n \
    --arg git_commit "$export_git_commit" \
    --arg achronyme_binary_sha256 "$export_binary_sha256" \
    --arg export_evidence_format "$export_evidence_format" \
    --arg export_evidence_sha256 "$export_evidence_sha256" \
    --arg achronyme_version "$ach_version" \
    --arg snarkjs_version "$ACHRONYME_SNARKJS_VERSION" \
    --arg r1cs_sha256 "$r1cs_sha256" \
    --arg wtns_sha256 "$(sha256_file "$wtns")" \
    --arg contributed_zkey_sha256 "$contributed_zkey_sha256" \
    --arg zkey_sha256 "$zkey_sha256" \
    --arg phase1_sha256 "$phase1_sha256" \
    --arg phase1_blake2b512 "$phase1_blake2b512" \
    --arg beacon_name "$beacon_name" \
    --arg beacon_source "$beacon_source" \
    --arg beacon_randomness "$beacon_randomness" \
    --arg beacon_contribution_hash "$beacon_contribution_hash" \
    --argjson beacon_iterations "$beacon_iterations" \
    --argjson constraints "$r1cs_constraints" \
    --argjson r1cs_bytes "$(stat -c %s "$r1cs")" \
    --argjson wtns_bytes "$(stat -c %s "$wtns")" \
    --argjson contributed_zkey_bytes "$(stat -c %s "$contributed_zkey")" \
    --argjson zkey_bytes "$(stat -c %s "$final_zkey")" \
    --slurpfile contributors "$contributors_json" \
    --slurpfile metrics_data "$metrics_json" \
    '{
        format: "achronyme-proving-release-evidence",
        version: 2,
        git_commit: $git_commit,
        achronyme_binary_sha256: $achronyme_binary_sha256,
        export_evidence: {
            format: $export_evidence_format,
            version: 1,
            sha256: $export_evidence_sha256
        },
        tools: {achronyme: $achronyme_version, snarkjs: $snarkjs_version},
        circuit: {
            constraints: $constraints,
            r1cs: {sha256: $r1cs_sha256, bytes: $r1cs_bytes},
            witness: {sha256: $wtns_sha256, bytes: $wtns_bytes}
        },
        ceremony: {
            phase1: {sha256: $phase1_sha256, blake2b512: $phase1_blake2b512},
            contributed_zkey: {
                sha256: $contributed_zkey_sha256,
                bytes: $contributed_zkey_bytes
            },
            final_beacon: {
                name: $beacon_name,
                source: $beacon_source,
                randomness: $beacon_randomness,
                iterations: $beacon_iterations,
                contribution_hash: $beacon_contribution_hash
            },
            final_zkey: {sha256: $zkey_sha256, bytes: $zkey_bytes},
            contributors: $contributors[0]
        },
        verification: {
            phase1: true,
            witness: true,
            contributed_zkey: true,
            final_beacon: true,
            final_zkey: true,
            snarkjs_proof_in_achronyme: true,
            achronyme_proof_in_snarkjs: true
        },
        metrics: $metrics_data[0]
    }' >"$evidence"

jq -e '.verification | all(. == true)' "$evidence" >/dev/null
printf 'release evidence: %s\n' "$evidence"
printf 'trusted-key artifact: %s\n' "$artifact_dir"
