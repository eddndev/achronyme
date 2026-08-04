#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

readonly ACHRONYME_SNARKJS_VERSION="snarkjs@0.7.6"
readonly ACHRONYME_PHASE1_POWER=21
readonly ACHRONYME_PHASE1_URL="https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_21.ptau"
readonly ACHRONYME_PHASE1_BLAKE2B512="9aef0573cef4ded9c4a75f148709056bf989f80dad96876aadeb6f1c6d062391f07a394a9e756d16f7eb233198d5b69407cca44594c763ab4a5b67ae73254678"

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

require_regular_file() {
    local path=$1
    local label=$2
    [[ ! -L "$path" && -f "$path" ]] || die "$label must be a regular file, not a symlink: $path"
}

require_directory() {
    local path=$1
    local label=$2
    [[ ! -L "$path" && -d "$path" ]] || die "$label must be a directory, not a symlink: $path"
}

require_clean_tracked_checkout() {
    local checkout=$1
    require_directory "$checkout" "Git checkout"
    require_command git
    git -C "$checkout" rev-parse --is-inside-work-tree >/dev/null 2>&1 || \
        die "not a Git checkout: $checkout"
    local status
    status=$(git -C "$checkout" status --porcelain=v1 --untracked-files=no) || \
        die "cannot inspect Git checkout: $checkout"
    [[ -z "$status" ]] || die "tracked checkout must be clean: $checkout"
}

ensure_absent() {
    local path=$1
    [[ ! -e "$path" && ! -L "$path" ]] || die "refusing to replace existing output: $path"
}

require_snarkjs() {
    require_command snarkjs
    local installed
    installed=$({ snarkjs --version 2>&1 || true; } | sed -n '1p')
    [[ "$installed" == "$ACHRONYME_SNARKJS_VERSION" ]] || \
        die "expected $ACHRONYME_SNARKJS_VERSION, found ${installed:-unknown}"
}

sha256_file() {
    sha256sum "$1" | awk '{print $1}'
}

blake2b512_file() {
    b2sum "$1" | awk '{print $1}'
}

verify_phase1_hash() {
    local phase1=$1
    local expected=$2
    local actual
    actual=$(blake2b512_file "$phase1")
    [[ "$actual" == "$expected" ]] || \
        die "phase-1 BLAKE2b-512 mismatch: expected $expected, found $actual"
}

create_metrics_file() {
    local metrics=$1
    ensure_absent "$metrics"
    mkdir -p "$(dirname "$metrics")"
    install -m 600 /dev/null "$metrics"
}

run_measured() {
    local metrics=$1
    local stage=$2
    shift 2
    [[ "$stage" =~ ^[a-z0-9_-]+$ ]] || die "invalid metric stage: $stage"
    /usr/bin/time -a -o "$metrics" \
        -f "{\"stage\":\"$stage\",\"elapsed_seconds\":%e,\"max_rss_kib\":%M,\"exit_status\":%x}" \
        "$@"
}

run_measured_logged() {
    local metrics=$1
    local stage=$2
    local log=$3
    shift 3
    ensure_absent "$log"
    if ! /usr/bin/time -a -o "$metrics" \
        -f "{\"stage\":\"$stage\",\"elapsed_seconds\":%e,\"max_rss_kib\":%M,\"exit_status\":%x}" \
        "$@" >"$log" 2>&1; then
        sed -n '1,240p' "$log" >&2
        return 1
    fi
    sed -n '1,240p' "$log" >&2
}

validate_contributor_pair() {
    local pair=$1
    [[ "$pair" == *=* ]] || die "contributor must use ID=HASH: $pair"
    local id=${pair%%=*}
    local hash=${pair#*=}
    [[ "$id" =~ ^[A-Za-z0-9._@\ -]{1,128}$ ]] || die "invalid contributor id: $id"
    [[ "$hash" =~ ^[0-9a-f]{128}$ ]] || die "invalid contribution hash for: $id"
}

extract_zkey_contributions() {
    local log=$1
    awk '
        /contribution #[0-9]+ / {
            name = $0
            sub(/^.*contribution #[0-9]+ /, "", name)
            sub(/:.*/, "", name)
            hash = ""
            remaining = 4
            next
        }
        remaining > 0 {
            for (i = 1; i <= NF; i++) {
                if (length($i) == 8 && $i ~ /^[0-9a-f]+$/) {
                    hash = hash $i
                }
            }
            remaining--
            if (remaining == 0) {
                print name "|" hash
            }
        }
    ' "$log"
}

assert_contributors_in_log() {
    local log=$1
    shift
    local extracted
    extracted=$(extract_zkey_contributions "$log")
    local pair id hash
    for pair in "$@"; do
        validate_contributor_pair "$pair"
        id=${pair%%=*}
        hash=${pair#*=}
        grep -Fqx -- "$id|$hash" <<<"$extracted" || \
            die "contributor metadata not present in verified zkey: $id"
    done
}
