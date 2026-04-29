#!/usr/bin/env bash
# Regression tests for B116-B123 (bugs found by complex apps testing, fixed 2026-04-29).
# Each .liva file in this directory is compiled with both bootstrap and gen-2 and run.
# Expected outputs are encoded as `// EXPECT: <stdout>` comments in the source.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BOOT="$LIVAC_ROOT/target/release/livac"
GEN2="$LIVAC_ROOT/target/livac-gen2-release"
OUT_DIR="${TMPDIR:-/tmp}/liva_regression_out"
mkdir -p "$OUT_DIR"

PASS=0; FAIL=0
for src in "$SCRIPT_DIR"/*.liva; do
    name=$(basename "$src" .liva)
    BD="$OUT_DIR/${name}_b"; GD="$OUT_DIR/${name}_g"
    rm -rf "$BD" "$GD"; mkdir -p "$BD" "$GD"
    cp "$src" "$BD/main.liva"; cp "$src" "$GD/main.liva"
    (cd "$BD" && "$BOOT" build main.liva --release > build.log 2>&1) || true
    (cd "$GD" && "$GEN2" build main.liva > build.log 2>&1) || true
    BB=$(find "$BD" -name liva_project -executable 2>/dev/null | grep release | head -1)
    GG=$(find "$GD" -name main -executable 2>/dev/null | grep '/release/' | head -1)
    [ -z "$GG" ] && (cd "$GD/target/liva_build" 2>/dev/null && cargo build --release > /dev/null 2>&1) && \
        GG=$(find "$GD" -name main -executable 2>/dev/null | grep '/release/' | head -1)
    if [ -z "$BB" ] || [ -z "$GG" ]; then
        echo "[FAIL] $name — bootstrap=$([ -n "$BB" ] && echo OK || echo FAIL) / gen2=$([ -n "$GG" ] && echo OK || echo FAIL)"
        FAIL=$((FAIL+1)); continue
    fi
    BO=$("$BB" 2>&1); GO=$("$GG" 2>&1)
    if [ "$BO" = "$GO" ]; then
        echo "[OK ] $name"; PASS=$((PASS+1))
    else
        echo "[FAIL] $name — stdout differs"; diff <(echo "$BO") <(echo "$GO") | head -10
        FAIL=$((FAIL+1))
    fi
done
echo "===================="; echo "  Regression: $PASS pass / $FAIL fail"
[ $FAIL -eq 0 ]
