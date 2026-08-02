#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/achronyme-release-bundle.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT

fixture_root="$temporary_root/fixtures"
output_root="$temporary_root/output"
repeat_output_root="$temporary_root/repeat-output"
install_root="$temporary_root/install"
target="x86_64-unknown-linux-gnu"
bundle="achronyme-linux-x86_64"
archive="$output_root/$bundle.tar.gz"

mkdir -p "$fixture_root" "$output_root" "$repeat_output_root" "$install_root"
printf '#!/usr/bin/env bash\nprintf "installed-ach\\n"\n' > "$fixture_root/ach"
chmod +x "$fixture_root/ach"
printf 'runtime-archive\n' > "$fixture_root/libakron_aot_runtime.a"

"$repository_root/scripts/package-release.sh" \
    --target "$target" \
    --name "$bundle" \
    --binary "$fixture_root/ach" \
    --runtime "$fixture_root/libakron_aot_runtime.a" \
    --output "$output_root"
"$repository_root/scripts/package-release.sh" \
    --target "$target" \
    --name "$bundle" \
    --binary "$fixture_root/ach" \
    --runtime "$fixture_root/libakron_aot_runtime.a" \
    --output "$repeat_output_root"

test -f "$archive"
test -f "$archive.sha256"
cmp "$archive" "$repeat_output_root/$bundle.tar.gz"
cmp "$archive.sha256" "$repeat_output_root/$bundle.tar.gz.sha256"
(cd "$output_root" && sha256sum --check "$(basename "$archive.sha256")")
tar -xzf "$archive" -C "$install_root"

test -x "$install_root/$bundle/bin/ach"
test -f "$install_root/$bundle/lib/libakron_aot_runtime.a"
test -f "$install_root/$bundle/LICENSE"
test -f "$install_root/$bundle/NOTICE"
test "$("$install_root/$bundle/bin/ach")" = "installed-ach"
cmp "$fixture_root/libakron_aot_runtime.a" \
    "$install_root/$bundle/lib/libakron_aot_runtime.a"

echo "release bundle contract verified: executable, AOT runtime, licenses, and checksum"
