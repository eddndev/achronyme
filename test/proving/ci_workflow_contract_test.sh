#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
contract="$repo_root/test/proving/ci_workflow_contract.sh"
documentation_contract="$repo_root/test/proving/documentation_contract.sh"
trusted_setup_workflow="$repo_root/test/proving/trusted_setup_workflow.sh"
test_bin=$(mktemp -d "${TMPDIR:-/tmp}/achronyme-proving-contract.XXXXXX")
trap 'rm -rf "$test_bin"' EXIT

ln -s "$(command -v dirname)" "$test_bin/dirname"
ln -s "$(command -v grep)" "$test_bin/grep"
ln -s "$(command -v tr)" "$test_bin/tr"

PATH="$test_bin" "$(command -v bash)" "$contract"
PATH="$test_bin" "$(command -v bash)" "$documentation_contract"

if grep -En '(^|[[:space:]])rg([[:space:]]|$)' \
    "$contract" "$documentation_contract" "$trusted_setup_workflow"; then
    printf 'proving workflow contracts have an undeclared ripgrep dependency\n' >&2
    exit 1
fi

printf 'proving workflow contracts have no undeclared ripgrep dependency\n'
