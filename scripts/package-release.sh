#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 --target TARGET [--name NAME] --binary PATH --runtime PATH --output DIRECTORY" >&2
}

target=""
bundle_name=""
binary=""
runtime=""
output_root=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)
            target="${2:-}"
            shift 2
            ;;
        --binary)
            binary="${2:-}"
            shift 2
            ;;
        --name)
            bundle_name="${2:-}"
            shift 2
            ;;
        --runtime)
            runtime="${2:-}"
            shift 2
            ;;
        --output)
            output_root="${2:-}"
            shift 2
            ;;
        *)
            usage
            exit 2
            ;;
    esac
done

if [[ -z "$target" || -z "$binary" || -z "$runtime" || -z "$output_root" ]]; then
    usage
    exit 2
fi
if [[ ! "$target" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "Invalid target name: $target" >&2
    exit 2
fi
if [[ -n "$bundle_name" && ! "$bundle_name" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "Invalid bundle name: $bundle_name" >&2
    exit 2
fi
if [[ "$target" != *-unknown-linux-gnu ]]; then
    echo "Release bundles with AOT support currently require a Linux GNU target" >&2
    exit 2
fi
if [[ ! -x "$binary" ]]; then
    echo "Release binary is missing or not executable: $binary" >&2
    exit 1
fi
if [[ ! -f "$runtime" ]]; then
    echo "AOT runtime archive is missing: $runtime" >&2
    exit 1
fi

for command_name in gzip install sha256sum tar; do
    command -v "$command_name" >/dev/null
done

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bundle="${bundle_name:-achronyme-$target}"
archive_name="$bundle.tar.gz"
mkdir -p "$output_root"
staging_root="$(mktemp -d "$output_root/.${bundle}.XXXXXX")"

cleanup() {
    rm -rf -- "$staging_root"
}
trap cleanup EXIT

bundle_root="$staging_root/$bundle"
install -Dm755 "$binary" "$bundle_root/bin/ach"
install -Dm644 "$runtime" "$bundle_root/lib/libakron_aot_runtime.a"
install -Dm644 "$repository_root/LICENSE" "$bundle_root/LICENSE"
install -Dm644 "$repository_root/NOTICE" "$bundle_root/NOTICE"
install -Dm644 "$repository_root/README.md" "$bundle_root/README.md"

archive_temp="$staging_root/$archive_name"
tar \
    --sort=name \
    --mtime='UTC 1970-01-01' \
    --owner=0 \
    --group=0 \
    --numeric-owner \
    -C "$staging_root" \
    -cf - "$bundle" | gzip -n > "$archive_temp"

checksum="$(sha256sum "$archive_temp" | cut -d ' ' -f 1)"
checksum_temp="$staging_root/$archive_name.sha256"
printf '%s  %s\n' "$checksum" "$archive_name" > "$checksum_temp"

install -m 0644 "$archive_temp" "$output_root/$archive_name"
install -m 0644 "$checksum_temp" "$output_root/$archive_name.sha256"

echo "$output_root/$archive_name"
