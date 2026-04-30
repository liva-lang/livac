#!/usr/bin/env bash
# Multi-file gen-2 gate.
# Compiles each multifile_apps/m*_*/main.liva with both the bootstrap
# and the gen-2 self-host compiler, runs the resulting binary, and
# diffs stdout against an "// EXPECT:" header in main.liva.
#
# Used to give `module.rs` non-zero coverage in self-host (Tier C10).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BS="$LIVAC_ROOT/target/release/livac"
G2="$LIVAC_ROOT/target/livac-gen2-release"
APPS_DIR="$LIVAC_ROOT/compiler/tests/multifile_apps"
OUT="${TMPDIR:-/tmp}/liva_multifile_apps_out"
mkdir -p "$OUT"

[[ -x "$BS" ]] || { echo "bootstrap missing: cargo build --release"; exit 2; }
[[ -x "$G2" ]] || { echo "gen-2 missing: rebuild_selfhost.sh"; exit 2; }

PASS=0; FAIL=0
for proj in "$APPS_DIR"/m*; do
    [[ -d "$proj" ]] || continue
    name=$(basename "$proj")
    main="$proj/main.liva"

    expected=$(grep -E '^// EXPECT: ' "$main" | sed 's|^// EXPECT: ||')
    expected_lines=$(grep -E '^//\s{8}' "$main" | sed 's|^//        ||')
    if [[ -n "$expected_lines" ]]; then
        expected=$(printf "%s\n%s" "$expected" "$expected_lines")
    fi

    BD="$OUT/$name"; rm -rf "$BD"; mkdir -p "$BD"
    "$BS" build --output "$BD/bs" "$main" >"$BD/bs.log" 2>&1 || true
    "$G2" build --output "$BD/g2" "$main" >"$BD/g2.log" 2>&1 || true

    bs_bin=$(find "$BD/bs/target/debug" -maxdepth 1 -type f -executable ! -name "*.d" 2>/dev/null | head -1)
    g2_bin=$(find "$BD/g2/target/debug" -maxdepth 1 -type f -executable ! -name "*.d" 2>/dev/null | head -1)

    if [[ -z "$g2_bin" ]]; then
        echo "[FAIL] $name — gen-2 compile error"
        FAIL=$((FAIL + 1))
        continue
    fi

    out_g2=$("$g2_bin" 2>&1 || true)

    if [[ "$out_g2" == "$expected" ]]; then
        echo "[OK ] $name"
        PASS=$((PASS + 1))
    else
        echo "[FAIL] $name"
        echo "  expected: $expected"
        echo "  got:      $out_g2"
        FAIL=$((FAIL + 1))
    fi
done

echo "===================="
echo "  Multifile gen-2: $PASS pass / $FAIL fail"
[[ $FAIL -eq 0 ]]
