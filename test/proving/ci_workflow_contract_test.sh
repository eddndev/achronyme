#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
contract="$repo_root/test/proving/ci_workflow_contract.sh"
test_bin=$(mktemp -d "${TMPDIR:-/tmp}/achronyme-proving-contract.XXXXXX")
trap 'rm -rf "$test_bin"' EXIT

ln -s "$(command -v dirname)" "$test_bin/dirname"
ln -s "$(command -v grep)" "$test_bin/grep"

PATH="$test_bin" "$(command -v bash)" "$contract"

printf 'proving CI workflow contract has no undeclared ripgrep dependency\n'
