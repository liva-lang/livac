#!/usr/bin/env bash
# Bench gate: runs every *.bench.liva in this dir via `livac bench`.
# Asserts that each one compiles in release mode and emits a BENCH line.
# Absolute timings are informational only — does NOT enforce thresholds
# (CI runners are too noisy for that).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
GEN2="$LIVAC_ROOT/target/livac-gen2-release"

if [ ! -x "$GEN2" ]; then
    echo "[FAIL] $GEN2 not found — run rebuild_selfhost.sh first"
    exit 1
fi

PASS=0; FAIL=0
for bench in "$SCRIPT_DIR"/*.bench.liva; do
    [ -e "$bench" ] || continue
    name=$(basename "$bench" .bench.liva)
    OUT_LOG=$(mktemp)
    if "$GEN2" bench "$bench" > "$OUT_LOG" 2>&1 && grep -qE "^ BENCH .* — [0-9]+ ms" "$OUT_LOG"; then
        ms=$(grep -oE "— [0-9]+ ms" "$OUT_LOG" | head -1 | grep -oE "[0-9]+")
        echo "[OK ] $name (${ms} ms)"
        PASS=$((PASS+1))
    else
        echo "[FAIL] $name — output:"
        sed 's/^/  /' "$OUT_LOG"
        FAIL=$((FAIL+1))
    fi
    rm -f "$OUT_LOG"
done

echo "===================="
echo "  Bench: $PASS pass / $FAIL fail"
[ $FAIL -eq 0 ]
