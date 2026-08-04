#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
gate="$repo_root/test/llvm_production_gate.sh"

grep -Fq 'dump_smoke_logs()' "$gate"
grep -Fq 'trap cleanup EXIT' "$gate"
grep -Fq 'LLVM production gate smoke logs:' "$gate"
grep -Fq 'AKRON_ALLOW_READ="$smoke_root"' "$gate"
grep -Fq 'AKRON_ALLOW_WRITE="$smoke_root"' "$gate"

printf 'LLVM production gate diagnostics and capability grants verified\n'
