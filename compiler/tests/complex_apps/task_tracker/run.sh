#!/usr/bin/env bash
# Build, run, and test the multi-module Task Tracker exemplar with
# both bootstrap and gen-2 compilers; assert stdout parity.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
BOOT="$LIVAC_ROOT/target/release/livac"
GEN2="$LIVAC_ROOT/target/livac-gen2-release"
OUT_DIR="${TMPDIR:-/tmp}/liva_task_tracker_out"
PASS=0; FAIL=0

if [[ ! -x "$BOOT" ]]; then
    echo "[FAIL] bootstrap compiler not found at $BOOT"
    echo "       run \`cargo build --release\` from $LIVAC_ROOT first"
    exit 1
fi

mkdir -p "$OUT_DIR"

echo "============================================================"
echo "  Task Tracker — bootstrap build & run"
echo "============================================================"
BOOT_DIR="$OUT_DIR/boot"
rm -rf "$BOOT_DIR"
"$BOOT" build "$SCRIPT_DIR/main.liva" --output "$BOOT_DIR" > "$OUT_DIR/boot_build.log" 2>&1
BOOT_BIN="$BOOT_DIR/target/debug/liva_project"
if [[ ! -x "$BOOT_BIN" ]]; then
    echo "[FAIL] bootstrap build failed (see $OUT_DIR/boot_build.log)"
    grep -E "^error" "$BOOT_DIR/target/liva_build" 2>/dev/null | head -5 || true
    exit 1
fi
"$BOOT_BIN" > "$OUT_DIR/boot.stdout" 2>"$OUT_DIR/boot.stderr"
echo "[OK ] bootstrap built and ran"
PASS=$((PASS+1))

if [[ -x "$GEN2" ]]; then
    echo
    echo "============================================================"
    echo "  Task Tracker — gen-2 parity"
    echo "============================================================"
    GEN2_DIR="$OUT_DIR/gen2"
    rm -rf "$GEN2_DIR"
    "$GEN2" build "$SCRIPT_DIR/main.liva" --output "$GEN2_DIR" > "$OUT_DIR/gen2_build.log" 2>&1
    GEN2_BIN="$GEN2_DIR/target/debug/main"
    if [[ ! -x "$GEN2_BIN" ]]; then
        echo "[FAIL] gen-2 build failed (see $OUT_DIR/gen2_build.log)"
        FAIL=$((FAIL+1))
    else
        "$GEN2_BIN" > "$OUT_DIR/gen2.stdout" 2>"$OUT_DIR/gen2.stderr"
        if diff -q "$OUT_DIR/boot.stdout" "$OUT_DIR/gen2.stdout" > /dev/null; then
            echo "[OK ] stdout identical (bootstrap ≡ gen-2)"
            PASS=$((PASS+1))
        else
            echo "[FAIL] stdout differs:"
            diff "$OUT_DIR/boot.stdout" "$OUT_DIR/gen2.stdout" | head -20
            FAIL=$((FAIL+1))
        fi
    fi
else
    echo "[SKIP] gen-2 compiler not found at $GEN2"
fi

echo
echo "============================================================"
echo "  Task Tracker — livac test (Liva test framework)"
echo "============================================================"
(cd "$SCRIPT_DIR" && "$BOOT" test) > "$OUT_DIR/test.log" 2>&1
TEST_RC=$?
TAIL=$(tail -10 "$OUT_DIR/test.log")
echo "$TAIL"
if [[ $TEST_RC -eq 0 ]] && grep -qE "^Tests:.*0 failed" "$OUT_DIR/test.log"; then
    echo "[OK ] livac test passed"
    PASS=$((PASS+1))
else
    echo "[FAIL] livac test failed (full log: $OUT_DIR/test.log)"
    FAIL=$((FAIL+1))
fi

echo
echo "============================================================"
echo "  SUMMARY: $PASS pass / $FAIL fail"
echo "============================================================"
exit $FAIL
