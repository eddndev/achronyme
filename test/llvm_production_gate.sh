#!/usr/bin/env bash
set -euo pipefail

temporary_root=""
smoke_root=""

dump_smoke_logs() {
    local log

    printf 'LLVM production gate smoke logs:\n' >&2
    for log in "$smoke_root"/*.stdout "$smoke_root"/*.stderr; do
        [[ -f "$log" ]] || continue
        printf '[%s]\n' "${log##*/}" >&2
        cat "$log" >&2
    done
}

cleanup() {
    local exit_status=$?

    if [[ "$exit_status" -ne 0 && -n "$smoke_root" && -d "$smoke_root" ]]; then
        dump_smoke_logs
    fi
    if [[ -n "$temporary_root" && -d "$temporary_root" ]]; then
        rm -rf "$temporary_root"
    fi
    trap - EXIT
    exit "$exit_status"
}

trap cleanup EXIT

if [[ "$(uname -s)" != "Linux" ]]; then
    echo "LLVM production gate requires Linux" >&2
    exit 1
fi

case "$(uname -m)" in
    x86_64)
        target="x86_64-unknown-linux-gnu"
        bundle="achronyme-linux-x86_64"
        ;;
    aarch64)
        target="aarch64-unknown-linux-gnu"
        bundle="achronyme-linux-aarch64"
        ;;
    *)
        echo "Unsupported LLVM production gate architecture: $(uname -m)" >&2
        exit 1
        ;;
esac

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

for command_name in cargo clang file readelf rustc sha256sum tar; do
    command -v "$command_name" >/dev/null
done

clang_version="$(clang --version | sed -n '1p')"
clang_major="$(
    printf '%s\n' "$clang_version" |
        sed -n 's/.*clang version \([0-9][0-9]*\).*/\1/p'
)"
if [[ "$clang_major" != "21" ]]; then
    echo "LLVM production gate requires Clang 21, found: $clang_version" >&2
    exit 1
fi

echo "Gate host: $(uname -s) $(uname -m)"
echo "Gate Rust: $(rustc --version)"
echo "Gate Clang: $clang_version"

bash test/llvm_production_gate_contract.sh
bash test/release_bundle_contract.sh
cargo build -p akron-aot-runtime
bash test/release_panic_contract.sh
bash test/llvm_link_contract.sh

cargo test -p cli --no-default-features --lib
cargo test -p cli --no-default-features \
    --test default_feature_contract \
    --test engine_feature_gate_test
cargo test -p akron-llvm --features llvm,aot
cargo test -p cli \
    --test llvm_engine_test \
    --test engine_oracle \
    --test aot_cli_test

cargo build --release -p cli -p akron-aot-runtime

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/akron-llvm-production-gate.XXXXXX")"
package_root="$temporary_root/package"
install_root="$temporary_root/install"
smoke_root="$temporary_root/smoke"
mkdir -p "$package_root" "$install_root" "$smoke_root"

scripts/package-release.sh \
    --target "$target" \
    --name "$bundle" \
    --binary "target/release/ach" \
    --runtime "target/release/libakron_aot_runtime.a" \
    --output "$package_root"

archive_name="$bundle.tar.gz"
(cd "$package_root" && sha256sum --check "$archive_name.sha256")
tar -xzf "$package_root/$archive_name" -C "$install_root"
installed_root="$install_root/$bundle"
ach="$installed_root/bin/ach"

if readelf -d "$ach" | grep -Eq 'NEEDED.*libLLVM'; then
    echo "Installed ach must load LLVM lazily" >&2
    exit 1
fi

printf 'return 6 * 7\n' > "$smoke_root/scalar.ach"
printf '%s\n' \
    'let path = read_line()' \
    'write_file(path, "akron-installed-io")' \
    'let content = read_file(path)' \
    'assert(content == "akron-installed-io")' \
    'print(content)' > "$smoke_root/io.ach"

(
    cd "$smoke_root"
    AKRON_ENGINE_TRACE=1 AKRON_JIT_CACHE=0 \
        "$ach" --no-config run scalar.ach --engine jit \
        > jit.stdout 2> jit.stderr
)
grep -q 'LLVM JIT native:' "$smoke_root/jit.stderr"
grep -q 'Exit Status: 42' "$smoke_root/jit.stdout"

(
    cd "$smoke_root"
    AKRON_LLVM_DYLIB="$smoke_root/missing-libLLVM.so" \
        "$ach" --no-config run scalar.ach \
        > fallback.stdout 2> fallback.stderr
)
grep -q 'LLVM JIT fallback:' "$smoke_root/fallback.stderr"
grep -q 'Exit Status: 42' "$smoke_root/fallback.stdout"

(
    cd "$smoke_root"
    env -u AKRON_AOT_RUNTIME_ARCHIVE \
        "$ach" --no-config aot scalar.ach --output scalar-native \
        > aot-build.stdout 2> aot-build.stderr
)
native_count="$(
    sed -n 's/.*(\([0-9][0-9]*\)\/\([0-9][0-9]*\) bytecode.*/\1/p' \
        "$smoke_root/aot-build.stdout"
)"
instruction_count="$(
    sed -n 's/.*(\([0-9][0-9]*\)\/\([0-9][0-9]*\) bytecode.*/\2/p' \
        "$smoke_root/aot-build.stdout"
)"
if [[ -z "$native_count" || -z "$instruction_count" || "$native_count" -eq 0 ]]; then
    echo "AOT scalar smoke did not report native lowering" >&2
    cat "$smoke_root/aot-build.stdout" >&2
    exit 1
fi
AKRON_ENGINE_TRACE=1 "$smoke_root/scalar-native" \
    > "$smoke_root/native.stdout" 2> "$smoke_root/native.stderr"
grep -q 'Exit Status: 42' "$smoke_root/native.stdout"
grep -q 'LLVM AOT native: program completed without interpreter bailout' \
    "$smoke_root/native.stderr"

if readelf -d "$smoke_root/scalar-native" | grep -Eq 'NEEDED.*libLLVM'; then
    echo "AOT executable must not depend on libLLVM" >&2
    exit 1
fi

(
    cd "$smoke_root"
    env -u AKRON_AOT_RUNTIME_ARCHIVE \
        "$ach" --no-config aot io.ach --output io-native \
        > io-build.stdout 2> io-build.stderr
    printf '%s\n' "$smoke_root/io-output.txt" | \
        AKRON_ALLOW_READ="$smoke_root" \
        AKRON_ALLOW_WRITE="$smoke_root" \
        AKRON_ENGINE_TRACE=1 ./io-native > io.stdout 2> io.stderr
)
grep -q 'akron-installed-io' "$smoke_root/io.stdout"
grep -q 'LLVM AOT native: program completed without interpreter bailout' \
    "$smoke_root/io.stderr"
test "$(< "$smoke_root/io-output.txt")" = "akron-installed-io"

file "$ach"
file "$smoke_root/scalar-native"
echo "LLVM production gate passed on $target"
