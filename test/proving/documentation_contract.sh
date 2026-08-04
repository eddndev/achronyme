#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
trusted="$repo_root/docs/proving/trusted-setup.md"
boss="$repo_root/docs/proving/boss-fight-release-gate.md"

normalized_matches() {
    local flags=$1
    local pattern=$2
    local document=$3

    tr '\n' ' ' <"$document" | grep "$flags" -- "$pattern"
}

[[ -f "$trusted" ]]
[[ -f "$boss" ]]
normalized_matches -Eiq 'zero-contribution key[[:space:]]+is not trusted' "$trusted"
normalized_matches -Eiq 'independently controlled[[:space:]]+contributor' "$trusted"
grep -Fq -- '--insecure-dev-setup' "$trusted"
grep -Fq -- '--trusted-key-dir' "$trusted"
grep -Fq 'BN254 Groth16' "$trusted"
normalized_matches -Eiq 'unsupported[[:space:]]+for production' "$trusted"
grep -Fq 'n2-standard-8' "$boss"
grep -Fq -- '--max-run-duration=12h' "$boss"
grep -Fq -- '--instance-termination-action=STOP' "$boss"
normalized_matches -Eq 'does not[[:space:]]+prove that 32 GB is sufficient' "$boss"
grep -Fq 'release-evidence.json' "$boss"
grep -Fq 'run-boss-fight.sh' "$boss"
grep -Fq -- '--export-evidence' "$boss"
grep -Fq 'clean checkout' "$boss"
normalized_matches -Eq 'Achronyme binary[[:space:]]+SHA-256' "$boss"
grep -Fq 'Proving setup policy' "$repo_root/docs/migration/0.1.0.md"
grep -Fq 'record-export-evidence.sh' "$trusted"
grep -Fq -- '--export-evidence' "$trusted"
grep -Fq -- '--contributed-zkey' "$trusted"
grep -Fq 'commit-beacon.mjs' "$trusted"
grep -Fq 'fetch-beacon.mjs' "$trusted"
grep -Fq -- '--beacon-evidence' "$trusted"
normalized_matches -Eiq 'publish.*commitment.*before.*target' "$trusted"
normalized_matches -Eiq \
    'public beacon[[:space:]]+does not replace[[:space:]]+an[[:space:]]+independent' \
    "$trusted"
grep -Fq -- '--contributed-zkey' "$boss"
grep -Fq -- '--beacon-evidence' "$boss"
grep -Fq 'trusted-setup.md' "$repo_root/README.md"

if grep -Fq 'Caching de `.zkey`' "$repo_root/STRATEGY.md"; then
    printf 'STRATEGY.md still mislabels local Arkworks cache files as zkeys\n' >&2
    exit 1
fi
if grep -Fq 'snarkjs groth16 setup circuit.r1cs pot12_final.ptau circuit.zkey' \
    "$repo_root/README.md"; then
    printf 'README.md still presents a zero-contribution zkey as the production path\n' >&2
    exit 1
fi

printf 'proving documentation contract passed\n'
