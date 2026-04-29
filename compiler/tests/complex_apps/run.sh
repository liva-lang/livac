#!/usr/bin/env bash
# Test complex Liva apps with bootstrap vs gen-2 self-hosted compiler.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BOOT="$LIVAC_ROOT/target/release/livac"
GEN2="$LIVAC_ROOT/target/livac-gen2-release"
APPS_DIR="$SCRIPT_DIR"
OUT_DIR="${TMPDIR:-/tmp}/liva_complex_apps_out"
mkdir -p "$OUT_DIR"

PASS=0; FAIL=0; RESULTS=()
APPS=(app4_library app5_numerical app6_bench app7_inventory)

for app in "${APPS[@]}"; do
    SRC="$APPS_DIR/${app}.liva"
    echo "============================================================"
    echo " $app  ($(wc -l < "$SRC") lines)"
    echo "============================================================"

    BOOT_DIR="$OUT_DIR/${app}_boot"
    GEN2_DIR="$OUT_DIR/${app}_gen2"
    rm -rf "$BOOT_DIR" "$GEN2_DIR"
    mkdir -p "$BOOT_DIR" "$GEN2_DIR"
    cp "$SRC" "$BOOT_DIR/main.liva"
    cp "$SRC" "$GEN2_DIR/main.liva"

    echo "[boot] compiling --release..."
    BOOT_T0=$(date +%s%N)
    (cd "$BOOT_DIR" && "$BOOT" build main.liva --release > build.log 2>&1) || true
    BOOT_T1=$(date +%s%N)
    BOOT_BIN=$(find "$BOOT_DIR" -name liva_project -type f -executable 2>/dev/null | grep '/release/' | head -1)
    [ -z "$BOOT_BIN" ] && BOOT_BIN=$(find "$BOOT_DIR" -name main -type f -executable 2>/dev/null | grep '/release/' | head -1)

    echo "[gen2] compiling..."
    GEN2_T0=$(date +%s%N)
    (cd "$GEN2_DIR" && "$GEN2" build main.liva > build.log 2>&1) || true
    GEN2_T1=$(date +%s%N)
    GEN2_BIN=$(find "$GEN2_DIR" -name main -type f -executable 2>/dev/null | grep '/release/' | head -1)
    if [ -z "$GEN2_BIN" ] && [ -f "$GEN2_DIR/target/liva_build/src/main.rs" ]; then
        (cd "$GEN2_DIR/target/liva_build" && cargo build --release > /dev/null 2>&1) || true
        GEN2_BIN=$(find "$GEN2_DIR" -name main -type f -executable 2>/dev/null | grep '/release/' | head -1)
    fi

    if [ -z "$BOOT_BIN" ] || [ -z "$GEN2_BIN" ]; then
        echo "[FAIL] bootstrap=$([ -n "$BOOT_BIN" ] && echo OK || echo FAIL) / gen2=$([ -n "$GEN2_BIN" ] && echo OK || echo FAIL)"
        if [ -z "$BOOT_BIN" ]; then
            echo "  bootstrap rust errors:"
            (cd "$BOOT_DIR/target/liva_build" 2>/dev/null && cargo build --release 2>&1 | grep -E "^error" | head -5)
        fi
        if [ -z "$GEN2_BIN" ]; then
            echo "  gen-2 rust errors:"
            (cd "$GEN2_DIR/target/liva_build" 2>/dev/null && cargo build --release 2>&1 | grep -E "^error" | head -5)
        fi
        FAIL=$((FAIL+1))
        continue
    fi

    "$BOOT_BIN" > "$BOOT_DIR/stdout.txt" 2>"$BOOT_DIR/stderr.txt"
    "$GEN2_BIN" > "$GEN2_DIR/stdout.txt" 2>"$GEN2_DIR/stderr.txt"

    if diff -q "$BOOT_DIR/stdout.txt" "$GEN2_DIR/stdout.txt" > /dev/null; then
        echo "[OK ] stdout identical"
        PASS=$((PASS+1))
    else
        echo "[FAIL] stdout differs"
        diff "$BOOT_DIR/stdout.txt" "$GEN2_DIR/stdout.txt" | head -20
        FAIL=$((FAIL+1))
        continue
    fi

    BOOT_COMPILE_S=$(awk -v t="$(( (BOOT_T1 - BOOT_T0) / 1000000 ))" 'BEGIN{printf "%.2f", t/1000}')
    GEN2_COMPILE_S=$(awk -v t="$(( (GEN2_T1 - GEN2_T0) / 1000000 ))" 'BEGIN{printf "%.2f", t/1000}')

    BOOT_BEST=999999; GEN2_BEST=999999
    for i in 1 2 3 4 5 6 7; do
        T0=$(date +%s%N); "$BOOT_BIN" > /dev/null; T1=$(date +%s%N)
        DT=$(( (T1 - T0) / 1000000 )); [ "$DT" -lt "$BOOT_BEST" ] && BOOT_BEST=$DT
        T0=$(date +%s%N); "$GEN2_BIN" > /dev/null; T1=$(date +%s%N)
        DT=$(( (T1 - T0) / 1000000 )); [ "$DT" -lt "$GEN2_BEST" ] && GEN2_BEST=$DT
    done

    BOOT_RS=$(find "$BOOT_DIR" -path '*/src/main.rs' | head -1)
    GEN2_RS=$(find "$GEN2_DIR" -path '*/src/main.rs' | head -1)
    BOOT_LINES=$(wc -l < "$BOOT_RS"); GEN2_LINES=$(wc -l < "$GEN2_RS")
    BOOT_BYTES=$(wc -c < "$BOOT_RS"); GEN2_BYTES=$(wc -c < "$GEN2_RS")
    BOOT_SIZE=$(stat -c%s "$BOOT_BIN"); GEN2_SIZE=$(stat -c%s "$GEN2_BIN")
    RATIO=$(awk -v b="$BOOT_BEST" -v g="$GEN2_BEST" 'BEGIN{ if (b==0) print "n/a"; else printf "%.2fx", g/b }')

    echo
    printf "  %-19s | %9s | %9s | %s\n" "metric" "bootstrap" "gen-2" "gen-2/boot"
    echo   "  --------------------+-----------+-----------+----------"
    printf "  %-19s | %9s | %9s | %s\n" "rust lines" "$BOOT_LINES" "$GEN2_LINES" "$(awk -v b=$BOOT_LINES -v g=$GEN2_LINES 'BEGIN{printf "%.2fx", g/b}')"
    printf "  %-19s | %9s | %9s | %s\n" "rust bytes" "$BOOT_BYTES" "$GEN2_BYTES" "$(awk -v b=$BOOT_BYTES -v g=$GEN2_BYTES 'BEGIN{printf "%.2fx", g/b}')"
    printf "  %-19s | %9s | %9s | %s\n" "bin size (KB)" "$((BOOT_SIZE/1024))" "$((GEN2_SIZE/1024))" "$(awk -v b=$BOOT_SIZE -v g=$GEN2_SIZE 'BEGIN{printf "%.2fx", g/b}')"
    printf "  %-19s | %9s | %9s | %s\n" "compile time (s)" "$BOOT_COMPILE_S" "$GEN2_COMPILE_S" "$(awk -v b=$BOOT_COMPILE_S -v g=$GEN2_COMPILE_S 'BEGIN{printf "%.2fx", g/b}')"
    printf "  %-19s | %9s | %9s | %s\n" "runtime min (ms)" "$BOOT_BEST" "$GEN2_BEST" "$RATIO"
    echo
    RESULTS+=("$app: stdout-match | rt $BOOT_BEST→$GEN2_BEST ms ($RATIO) | rs lines $BOOT_LINES→$GEN2_LINES")
done

echo "============================================================"
echo "  SUMMARY"
echo "============================================================"
for r in "${RESULTS[@]}"; do echo "  $r"; done
echo
echo "  $PASS pass / $FAIL fail"
exit $FAIL
