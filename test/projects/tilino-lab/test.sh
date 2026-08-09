#!/bin/sh

set -eu

PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ACH_BIN=${ACH_BIN:-ach}
CASE_ROOT=$(mktemp -d /tmp/achronyme-tilino-lab.XXXXXX)
PORT_BASE=${TILINO_PORT_BASE:-$((30000 + ($$ % 20000)))}

cleanup() {
    rm -rf -- "$CASE_ROOT"
}
trap cleanup EXIT HUP INT TERM

export XDG_CACHE_HOME="$CASE_ROOT/cache"

allocate_loopback_port() {
    preferred_port=$1
    if ! command -v python3 >/dev/null 2>&1; then
        printf 'python3 is required to allocate a collision-free loopback port\n' >&2
        exit 1
    fi

    python3 - "$preferred_port" <<'PY'
import socket
import sys

preferred = int(sys.argv[1])
ports = (preferred, 0) if 0 < preferred < 65536 else (0,)

for port in ports:
    candidate = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        candidate.bind(("127.0.0.1", port))
    except OSError:
        candidate.close()
        continue

    print(candidate.getsockname()[1])
    candidate.close()
    raise SystemExit(0)

raise SystemExit("unable to allocate a loopback port")
PY
}

expect_text() {
    file=$1
    text=$2
    if ! grep -Fq -- "$text" "$file"; then
        printf 'missing required text in %s: %s\n' "$file" "$text" >&2
        exit 1
    fi
}

expect_failure() {
    label=$1
    pattern=$2
    shift 2
    stdout_file="$CASE_ROOT/$label.stdout"
    stderr_file="$CASE_ROOT/$label.stderr"

    if "$@" >"$stdout_file" 2>"$stderr_file"; then
        printf 'expected failure but command passed: %s\n' "$label" >&2
        exit 1
    fi
    if ! grep -Eqi "$pattern" "$stdout_file" "$stderr_file"; then
        printf 'failure did not match contract: %s (%s)\n' "$label" "$pattern" >&2
        sed -n '1,120p' "$stdout_file" >&2
        sed -n '1,120p' "$stderr_file" >&2
        exit 1
    fi
}

run_demo() {
    engine=$1
    port=$2
    output_dir=$3
    log=$4
    TILINO_ADDRESS="127.0.0.1:$port" \
    TILINO_OUTPUT_DIR="$output_dir" \
    TILINO_ENGINE="$engine" \
        sh "$PROJECT_ROOT/scripts/run-demo.sh" >"$log" 2>&1
}

verify_bundle() {
    output_dir=$1
    log=$2
    "$ACH_BIN" verify \
        --proof "$output_dir/proof.json" \
        --public "$output_dir/public.json" \
        --vkey "$output_dir/verification_key.json" \
        --curve bn254 \
        --format json >"$log"
    grep -Eq '"curve"[[:space:]]*:[[:space:]]*"bn254"' "$log"
    grep -Eq '"valid"[[:space:]]*:[[:space:]]*true' "$log"
}

run_without_proving_authority() {
    port=$1
    output_dir="$CASE_ROOT/no-proving-authority"
    address="127.0.0.1:$port"
    mkdir -p "$output_dir"
    (
        cd "$PROJECT_ROOT"
        printf '%s\n' "$address" "$output_dir" 500 750 300 |
            "$ACH_BIN" \
                --allow-read "$output_dir" \
                --allow-write "$output_dir" \
                --allow-connect "$address" \
                --allow-listen "$address" \
                run --engine interpreter
    )
}

MAIN="$PROJECT_ROOT/src/main.ach"
AUCTION="$PROJECT_ROOT/src/auction.ach"
REGISTRY="$PROJECT_ROOT/src/registry.ach"
expect_text "$MAIN" 'import "./transport.ach" as transport'
expect_text "$MAIN" 'import "./auction.ach" as auction'
expect_text "$MAIN" 'import "./registry.ach" as registry'
expect_text "$MAIN" 'import "./artifacts.ach" as artifacts'
expect_text "$MAIN" 'await transport::exchange_commitments'
expect_text "$MAIN" 'auction::prove_winner'
expect_text "$MAIN" 'await artifacts::write_bundle'
expect_text "$AUCTION" 'bob_path: Field[2]'
expect_text "$AUCTION" 'bob_indices: Field[2]'
expect_text "$AUCTION" 'merkle_verify(registry_root, bidder_bob, bob_path, bob_indices)'
expect_text "$REGISTRY" 'export fn bob_path()'
expect_text "$REGISTRY" 'export fn bob_indices()'

for forbidden in 'tcp_listen' 'channel(' 'create_file' 'prove winner' 'merkle_verify'; do
    if grep -Fq -- "$forbidden" "$MAIN"; then
        printf 'main.ach owns extracted responsibility: %s\n' "$forbidden" >&2
        exit 1
    fi
done

main_lines=$(wc -l <"$MAIN")
if [ "$main_lines" -gt 90 ]; then
    printf 'main.ach is not a thin orchestrator: %s lines\n' "$main_lines" >&2
    exit 1
fi

INTERPRETER_OUTPUT="$CASE_ROOT/interpreter"
JIT_OUTPUT="$CASE_ROOT/jit"
INTERPRETER_PORT=$(allocate_loopback_port "$PORT_BASE")
run_demo interpreter "$INTERPRETER_PORT" "$INTERPRETER_OUTPUT" "$CASE_ROOT/interpreter.log"
expect_text "$CASE_ROOT/interpreter.log" 'commitments accepted: 3'
expect_text "$CASE_ROOT/interpreter.log" 'winner proof verified: true'
expect_text "$CASE_ROOT/interpreter.log" 'PRE-OPTIMIZATION ESTIMATE'
expect_text "$CASE_ROOT/interpreter.log" 'FINAL PROVING CONSTRAINTS'
expect_text "$CASE_ROOT/interpreter.log" 'PASS: private_concurrent_auction'
if grep -Eq 'unused function parameter: `(bob_path|bob_indices)`' "$CASE_ROOT/interpreter.log"; then
    printf 'prove array parameters were reported as unused\n' >&2
    exit 1
fi

for artifact in proof.json public.json verification_key.json receipt.txt; do
    test -s "$INTERPRETER_OUTPUT/$artifact"
done
grep -Eq '"protocol"[[:space:]]*:[[:space:]]*"groth16"' \
    "$INTERPRETER_OUTPUT/verification_key.json"
grep -Fq 'winner=bob' "$INTERPRETER_OUTPUT/receipt.txt"
grep -Fq 'accepted_commitments=3' "$INTERPRETER_OUTPUT/receipt.txt"
if grep -Eq '=(300|500|750|1111|2222|3333)$' "$INTERPRETER_OUTPUT/receipt.txt"; then
    printf 'receipt leaked a bid or nonce\n' >&2
    exit 1
fi
verify_bundle "$INTERPRETER_OUTPUT" "$CASE_ROOT/interpreter-verify.json"

TAMPERED_PUBLIC="$CASE_ROOT/public-tampered.json"
sed 's/"[0-9][0-9]*"/"1"/' "$INTERPRETER_OUTPUT/public.json" >"$TAMPERED_PUBLIC"
if "$ACH_BIN" verify \
    --proof "$INTERPRETER_OUTPUT/proof.json" \
    --public "$TAMPERED_PUBLIC" \
    --vkey "$INTERPRETER_OUTPUT/verification_key.json" \
    --curve bn254 \
    --format json >"$CASE_ROOT/tampered-verify.json" 2>"$CASE_ROOT/tampered-verify.stderr"; then
    printf 'tampered public input unexpectedly verified\n' >&2
    exit 1
fi
grep -Eq '"valid"[[:space:]]*:[[:space:]]*false' "$CASE_ROOT/tampered-verify.json"

JIT_PORT=$(allocate_loopback_port "$((PORT_BASE + 1))")
run_demo jit "$JIT_PORT" "$JIT_OUTPUT" "$CASE_ROOT/jit.log"
cmp -s "$INTERPRETER_OUTPUT/receipt.txt" "$JIT_OUTPUT/receipt.txt"
verify_bundle "$JIT_OUTPUT" "$CASE_ROOT/jit-verify.json"

expect_failure \
    missing-capabilities \
    'file\.read|file\.write|network\.connect|network\.listen|capability' \
    sh -c 'cd "$1" && exec "$2" --insecure-dev-setup run --engine interpreter' \
    sh "$PROJECT_ROOT" "$ACH_BIN"

expect_failure \
    missing-proving-authority \
    'proof generation|trusted setup|development setup|key source|insecure-dev-setup|trusted-key-dir' \
    run_without_proving_authority \
    "$(allocate_loopback_port "$((PORT_BASE + 2))")"

TASK_LIMIT_PORT=$(allocate_loopback_port "$((PORT_BASE + 3))")
expect_failure \
    bounded-task-limit \
    'live child task count exceeds 2' \
    env \
        TILINO_ADDRESS="127.0.0.1:$TASK_LIMIT_PORT" \
        TILINO_OUTPUT_DIR="$CASE_ROOT/task-limit" \
        TILINO_MAX_TASKS=2 \
        sh "$PROJECT_ROOT/scripts/run-demo.sh"

FALSE_WINNER_PORT=$(allocate_loopback_port "$((PORT_BASE + 4))")
expect_failure \
    false-winner \
    'constraint|unsatisfied|assertion failed|circom assert' \
    env \
        TILINO_ADDRESS="127.0.0.1:$FALSE_WINNER_PORT" \
        TILINO_OUTPUT_DIR="$CASE_ROOT/false-winner" \
        TILINO_ALICE_BID=900 \
        TILINO_BOB_BID=750 \
        TILINO_CHARLIE_BID=300 \
        sh "$PROJECT_ROOT/scripts/run-demo.sh"

(
    INSPECT_PORT=$(allocate_loopback_port "$((PORT_BASE + 5))")
    cd "$PROJECT_ROOT"
    "$ACH_BIN" \
        --insecure-dev-setup \
        --allow-read "$CASE_ROOT" \
        --allow-write "$CASE_ROOT" \
        --allow-connect "127.0.0.1:$INSPECT_PORT" \
        --allow-listen "127.0.0.1:$INSPECT_PORT" \
        inspect --manifest >"$CASE_ROOT/manifest.txt"
)
expect_text "$CASE_ROOT/manifest.txt" 'effects: task,io.console,io.file,io.network,io.clock,prove,verify,circuit'
expect_text "$CASE_ROOT/manifest.txt" 'proving-key-source: insecure-local'

DUMMY_RUNTIME="$CASE_ROOT/libakron_aot_runtime.a"
: >"$DUMMY_RUNTIME"
expect_failure \
    aot-capability-boundary \
    'standalone AOT runtime does not provide required capabilities: PROVE, VERIFY, CIRCOM' \
    sh -c 'cd "$1" && exec "$2" --insecure-dev-setup aot --runtime "$3" --output "$4"' \
    sh "$PROJECT_ROOT" "$ACH_BIN" "$DUMMY_RUNTIME" "$CASE_ROOT/tilino-auction"

printf 'tilino-lab-ok\n'
