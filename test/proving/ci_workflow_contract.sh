#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
workflow="$repo_root/.github/workflows/proving-conformance.yml"

if [[ ! -f "$workflow" ]]; then
    printf 'missing reusable proving conformance workflow\n' >&2
    exit 1
fi

grep -Fq 'workflow_call:' "$workflow"
grep -Fq 'ubuntu-24.04' "$workflow"
grep -Fq 'snarkjs@0.7.6' "$workflow"
grep -Fq 'npm ci --prefix scripts/proving/drand --ignore-scripts' "$workflow"
grep -Fq 'npm test --prefix scripts/proving/drand' "$workflow"
grep -Fq 'test/proving/ci_workflow_contract_test.sh' "$workflow"
grep -Fq 'test/proving/documentation_contract.sh' "$workflow"
grep -Fq 'test/proving/boss_fight_harness_test.sh' "$workflow"
grep -Fq 'test/proving/trusted_setup_workflow.sh' "$workflow"
grep -Fq 'uses: ./.github/workflows/proving-conformance.yml' \
    "$repo_root/.github/workflows/ci.yml"
grep -Fq 'uses: ./.github/workflows/proving-conformance.yml' \
    "$repo_root/.github/workflows/release.yml"
grep -Fq 'needs: [gate, proving]' "$repo_root/.github/workflows/release.yml"

if grep -Fq 'npm install --global' "$workflow"; then
    printf 'proving workflow must not mutate the hosted global npm prefix\n' >&2
    exit 1
fi

printf 'proving CI workflow contract passed\n'
