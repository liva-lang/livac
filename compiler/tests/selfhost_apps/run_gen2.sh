#!/usr/bin/env bash
# Run selfhost_apps against the gen-2 self-hosted compiler.
# Used to track parity between bootstrap (Rust) and gen-2 (Liva).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
GEN2="$LIVAC_ROOT/target/livac-gen2-release"
APPS_DIR="$LIVAC_ROOT/compiler/tests/selfhost_apps"
OUT_DIR="${TMPDIR:-/tmp}/liva_gen2_apps_out"
mkdir -p "$OUT_DIR"

[ -x "$GEN2" ] || { echo "gen-2 not built: run rebuild_selfhost.sh"; exit 2; }

PASS=0; FAIL=0; FAILED=()
for src in "$APPS_DIR"/*.liva; do
    name=$(basename "$src" .liva)
    BD="$OUT_DIR/${name}"
    rm -rf "$BD"; mkdir -p "$BD"
    cp "$src" "$BD/main.liva"
    (cd "$BD" && "$GEN2" build --release --output "$BD/build" main.liva > build.log 2>&1) || true
    BIN=$(find "$BD/build/target/release" -maxdepth 1 -type f -executable ! -name "*.d" ! -name "*.rlib" 2>/dev/null | head -1)
    if [ -z "$BIN" ]; then
        echo "[FAIL] $name — gen-2 compile error"
        FAIL=$((FAIL+1)); FAILED+=("$name (compile)")
        continue
    fi
    expected=$(grep -E '^// EXPECT: ' "$src" | sed 's|^// EXPECT: ||')
    actual=$("$BIN" 2>&1)
    if [ "$actual" = "$expected" ]; then
        echo "[OK ] $name"
        PASS=$((PASS+1))
    else
        echo "[FAIL] $name — output differs"
        FAIL=$((FAIL+1)); FAILED+=("$name (output)")
    fi
done
echo "===================="
echo "  Gen-2 vs selfhost_apps: $PASS pass / $FAIL fail"
if [ ${#FAILED[@]} -gt 0 ]; then
    echo "  Failed:"
    for f in "${FAILED[@]}"; do echo "    - $f"; done
fi
[ $FAIL -eq 0 ]
