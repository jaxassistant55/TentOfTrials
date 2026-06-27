#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
LOG_FILE=$(mktemp "${TMPDIR:-/tmp}/tent-backend-shutdown.XXXXXX.log")
PID=""

cleanup() {
    if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
        kill "$PID" 2>/dev/null || true
        wait "$PID" 2>/dev/null || true
    fi
    rm -f "$LOG_FILE"
}

trap cleanup EXIT INT TERM

cd "$ROOT_DIR"
cargo build --quiet

RUST_LOG=info ./target/debug/tent-backend \
    --node-id shutdown-signal-harness \
    --config /tmp/tent-backend-shutdown-harness-missing.toml \
    >"$LOG_FILE" 2>&1 &
PID=$!

ready=0
for _ in $(seq 1 200); do
    if ! kill -0 "$PID" 2>/dev/null; then
        echo "backend exited before entering the shutdown harness main loop" >&2
        sed -n '1,160p' "$LOG_FILE" >&2
        exit 1
    fi

    if grep -q "entering main loop" "$LOG_FILE"; then
        ready=1
        break
    fi

    sleep 0.1
done

if [ "$ready" -ne 1 ]; then
    echo "backend did not report readiness before timeout" >&2
    sed -n '1,160p' "$LOG_FILE" >&2
    exit 1
fi

kill -TERM "$PID"

exited=0
for _ in $(seq 1 100); do
    if ! kill -0 "$PID" 2>/dev/null; then
        wait "$PID" || true
        exited=1
        break
    fi
    sleep 0.1
done

if [ "$exited" -ne 1 ]; then
    echo "backend did not exit after SIGTERM" >&2
    sed -n '1,200p' "$LOG_FILE" >&2
    exit 1
fi

grep -q "received SIGTERM, initiating graceful shutdown" "$LOG_FILE" || {
    echo "missing SIGTERM shutdown-start log" >&2
    sed -n '1,200p' "$LOG_FILE" >&2
    exit 1
}

grep -q '"accepting_new_work":false' "$LOG_FILE" || {
    echo "missing accepting_new_work=false shutdown guard log" >&2
    sed -n '1,200p' "$LOG_FILE" >&2
    exit 1
}

grep -q "shutdown complete" "$LOG_FILE" || {
    echo "missing shutdown-complete log" >&2
    sed -n '1,200p' "$LOG_FILE" >&2
    exit 1
}

echo "shutdown signal harness passed"
