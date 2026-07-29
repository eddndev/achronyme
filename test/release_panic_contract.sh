#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

release_cfg="$(cargo rustc -p akron-aot-runtime --release --lib -- --print cfg 2>&1)"
if ! grep -qx 'panic="abort"' <<<"$release_cfg"; then
    echo "release profile must compile akron-aot-runtime with panic=abort" >&2
    exit 1
fi

probe="$repository_root/target/release/ffi-panic-probe"
rustc --edition=2021 -C panic=abort test/fixtures/ffi_panic_probe.rs -o "$probe"

status=0
bash -c 'ulimit -c 0; "$1" >/dev/null 2>&1; exit $?' _ "$probe" 2>/dev/null || status=$?
if [[ "$status" -eq 0 ]]; then
    echo "an unexpected panic crossed the C ABI boundary" >&2
    exit 1
fi

echo "release panic contract verified: panic=abort and ABI panic is fatal"
