#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
    echo "LLVM link contract check skipped: Linux only"
    exit 0
fi

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
command -v readelf >/dev/null

contract_target="$repository_root/target/llvm-link-contract/default"

cargo build -p cli --target-dir "$contract_target"
if readelf -d "$contract_target/debug/ach" | grep -Eq 'NEEDED.*libLLVM'; then
    echo "default CLI must not link LLVM" >&2
    exit 1
fi

cargo build -p cli --features llvm --target-dir "$contract_target"
if ! readelf -d "$contract_target/debug/ach" | grep -Eq 'NEEDED.*libLLVM[^]]*21'; then
    echo "LLVM CLI must record a link-time LLVM 21 dependency" >&2
    exit 1
fi

echo "LLVM link contract verified: default is independent, llvm feature needs LLVM 21"
