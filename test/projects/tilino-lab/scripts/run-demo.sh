#!/bin/sh

set -eu

PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ACH_BIN=${ACH_BIN:-ach}
ADDRESS=${TILINO_ADDRESS:-127.0.0.1:39417}
OUTPUT_DIR=${TILINO_OUTPUT_DIR:-$PROJECT_ROOT/build/demo-output}
ENGINE=${TILINO_ENGINE:-interpreter}
MAX_TASKS=${TILINO_MAX_TASKS:-24}
ALICE_BID=${TILINO_ALICE_BID:-500}
BOB_BID=${TILINO_BOB_BID:-750}
CHARLIE_BID=${TILINO_CHARLIE_BID:-300}

mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR=$(CDPATH= cd -- "$OUTPUT_DIR" && pwd)

(
    cd "$PROJECT_ROOT"
    printf '%s\n' \
        "$ADDRESS" \
        "$OUTPUT_DIR" \
        "$ALICE_BID" \
        "$BOB_BID" \
        "$CHARLIE_BID" |
        "$ACH_BIN" \
            --insecure-dev-setup \
            --allow-read "$OUTPUT_DIR" \
            --allow-write "$OUTPUT_DIR" \
            --allow-connect "$ADDRESS" \
            --allow-listen "$ADDRESS" \
            --max-tasks "$MAX_TASKS" \
            run \
            --engine "$ENGINE" \
            --circuit-stats
)
