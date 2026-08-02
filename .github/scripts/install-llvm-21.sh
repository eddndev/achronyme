#!/usr/bin/env bash
set -euo pipefail

if command -v clang-21 >/dev/null; then
    version="$(clang-21 --version | sed -n '1p')"
    if [[ "$version" == *"clang version 21."* ]]; then
        echo "$version"
        exit 0
    fi
fi

if [[ ! -r /etc/os-release ]] || ! command -v apt-get >/dev/null; then
    echo "Automatic LLVM 21 installation requires Debian or Ubuntu" >&2
    exit 1
fi

version_codename="$(
    sed -n 's/^VERSION_CODENAME=//p' /etc/os-release | tr -d '"'
)"
case "$version_codename" in
    noble|jammy) codename="$version_codename" ;;
    *)
        echo "Unsupported apt.llvm.org distribution: ${version_codename:-unknown}" >&2
        exit 1
        ;;
esac

sudo apt-get update
sudo apt-get install -y --no-install-recommends ca-certificates curl gnupg

temporary_root="$(mktemp -d "${RUNNER_TEMP:-/tmp}/akron-llvm-key.XXXXXX")"
trap 'rm -rf "$temporary_root"' EXIT
key_source="$temporary_root/llvm-snapshot.gpg.key"
keyring="$temporary_root/llvm-snapshot.gpg"
expected_fingerprint="6084F3CF814B57C1CF12EFD515CF4D18AF4F7421"

curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key -o "$key_source"
fingerprint="$(
    gpg --show-keys --with-colons --import-options show-only "$key_source" 2>/dev/null |
        awk -F: '$1 == "fpr" { print $10; exit }'
)"
if [[ "$fingerprint" != "$expected_fingerprint" ]]; then
    echo "Unexpected apt.llvm.org signing key fingerprint: $fingerprint" >&2
    exit 1
fi

gpg --dearmor < "$key_source" > "$keyring"
sudo install -Dm644 "$keyring" /usr/share/keyrings/apt.llvm.org.gpg
printf '%s\n' \
    "deb [signed-by=/usr/share/keyrings/apt.llvm.org.gpg] https://apt.llvm.org/$codename/ llvm-toolchain-$codename-21 main" |
    sudo tee /etc/apt/sources.list.d/apt.llvm.org.list >/dev/null

sudo apt-get update
sudo apt-get install -y --no-install-recommends clang-21 llvm-21
clang-21 --version | sed -n '1p'
