#!/usr/bin/env bash
# Bootstrap-only complex apps. These exercise patterns where the gen-2
# self-hosted compiler still has bugs (tracked in BUGS.md). They are run
# only against the bootstrap compiler and validated by the `// EXPECT:` block.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BOOT="$LIVAC_ROOT/target/release/livac"
OUT_DIR="${TMPDIR:-/tmp}/liva_selfhost_apps_out"
mkdir -p "$OUT_DIR"

PASS=0; FAIL=0
for src in "$SCRIPT_DIR"/*.liva; do
    name=$(basename "$src" .liva)
    BD="$OUT_DIR/${name}"
    rm -rf "$BD"; mkdir -p "$BD"
    cp "$src" "$BD/main.liva"
    (cd "$BD" && "$BOOT" build main.liva --release > build.log 2>&1) || true
    BIN=$(find "$BD" -name liva_project -executable 2>/dev/null | grep '/release/' | head -1)
    if [ -z "$BIN" ]; then
        echo "[FAIL] $name — compile error"
        (cd "$BD/target/liva_build" 2>/dev/null && cargo build --release 2>&1 | grep -E "^error" | head -3)
        FAIL=$((FAIL+1)); continue
    fi
    expected=$(grep -E '^// EXPECT: ' "$src" | sed 's|^// EXPECT: ||')
    actual=$("$BIN" 2>&1)
    if [ "$actual" = "$expected" ]; then
        echo "[OK ] $name"
        PASS=$((PASS+1))
    else
        echo "[FAIL] $name — output differs"
        diff <(echo "$expected") <(echo "$actual") | head -20
        FAIL=$((FAIL+1))
    fi
done
echo "===================="
echo "  Bootstrap-only: $PASS pass / $FAIL fail"
[ $FAIL -eq 0 ]
