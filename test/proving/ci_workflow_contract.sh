#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
workflow="$repo_root/.github/workflows/proving-conformance.yml"

if [[ ! -f "$workflow" ]]; then
    printf 'missing reusable proving conformance workflow\n' >&2
    exit 1
fi

rg -Fq 'workflow_call:' "$workflow"
rg -Fq 'ubuntu-24.04' "$workflow"
rg -Fq 'snarkjs@0.7.6' "$workflow"
rg -Fq 'test/proving/documentation_contract.sh' "$workflow"
rg -Fq 'test/proving/boss_fight_harness_test.sh' "$workflow"
rg -Fq 'test/proving/trusted_setup_workflow.sh' "$workflow"
rg -Fq 'uses: ./.github/workflows/proving-conformance.yml' \
    "$repo_root/.github/workflows/ci.yml"
rg -Fq 'uses: ./.github/workflows/proving-conformance.yml' \
    "$repo_root/.github/workflows/release.yml"
rg -Fq 'needs: [gate, proving]' "$repo_root/.github/workflows/release.yml"

if rg -Fq 'npm install --global' "$workflow"; then
    printf 'proving workflow must not mutate the hosted global npm prefix\n' >&2
    exit 1
fi

printf 'proving CI workflow contract passed\n'
