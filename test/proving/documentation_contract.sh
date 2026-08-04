#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
trusted="$repo_root/docs/proving/trusted-setup.md"
boss="$repo_root/docs/proving/boss-fight-release-gate.md"

[[ -f "$trusted" ]]
[[ -f "$boss" ]]
rg -Uiq 'zero-contribution key[[:space:]]+is not trusted' "$trusted"
rg -Uiq 'independently controlled[[:space:]]+contributor' "$trusted"
rg -Fq -- '--insecure-dev-setup' "$trusted"
rg -Fq -- '--trusted-key-dir' "$trusted"
rg -Fq 'BN254 Groth16' "$trusted"
rg -Uiq 'unsupported[[:space:]]+for production' "$trusted"
rg -Fq 'n2-standard-8' "$boss"
rg -Fq -- '--max-run-duration=12h' "$boss"
rg -Fq -- '--instance-termination-action=STOP' "$boss"
rg -Uq 'does not[[:space:]]+prove that 32 GB is sufficient' "$boss"
rg -Fq 'release-evidence.json' "$boss"
rg -Fq 'run-boss-fight.sh' "$boss"
rg -Fq -- '--export-evidence' "$boss"
rg -Fq 'clean checkout' "$boss"
rg -Uq 'Achronyme binary[[:space:]]+SHA-256' "$boss"
rg -Fq 'Proving setup policy' "$repo_root/docs/migration/0.1.0.md"
rg -Fq 'record-export-evidence.sh' "$trusted"
rg -Fq -- '--export-evidence' "$trusted"
rg -Fq -- '--contributed-zkey' "$trusted"
rg -Fq -- '--beacon-source' "$trusted"
rg -Uiq 'public beacon\s+does not replace\s+an\s+independent' "$trusted"
rg -Fq -- '--contributed-zkey' "$boss"
rg -Fq -- '--beacon-source' "$boss"
rg -Fq 'trusted-setup.md' "$repo_root/README.md"

if rg -Fq 'Caching de `.zkey`' "$repo_root/STRATEGY.md"; then
    printf 'STRATEGY.md still mislabels local Arkworks cache files as zkeys\n' >&2
    exit 1
fi
if rg -Fq 'snarkjs groth16 setup circuit.r1cs pot12_final.ptau circuit.zkey' \
    "$repo_root/README.md"; then
    printf 'README.md still presents a zero-contribution zkey as the production path\n' >&2
    exit 1
fi

printf 'proving documentation contract passed\n'
