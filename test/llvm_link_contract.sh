#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
    echo "LLVM link contract check skipped: Linux only"
    exit 0
fi

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
command -v readelf >/dev/null

interpreter_target="$repository_root/target/llvm-link-contract/interpreter"
llvm_target="$repository_root/target/llvm-link-contract/llvm"

cargo build -p cli --no-default-features --target-dir "$interpreter_target"
if readelf -d "$interpreter_target/debug/ach" | grep -Eq 'NEEDED.*libLLVM'; then
    echo "interpreter-only CLI must not link LLVM" >&2
    exit 1
fi

cargo build -p cli --features llvm --target-dir "$llvm_target"
if readelf -d "$llvm_target/debug/ach" | grep -Eq 'NEEDED.*libLLVM'; then
    echo "LLVM CLI must load LLVM lazily instead of recording an ELF dependency" >&2
    exit 1
fi

echo "LLVM link contract verified: both CLI variants start without an ELF LLVM dependency"
