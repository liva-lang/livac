#!/usr/bin/env bash
# Error test runner — validates that livac check emits expected error codes
# Usage: ./run_error_tests.sh [path_to_livac]
set -euo pipefail

LIVAC="${1:-./target/release/livac}"
DIR="$(cd "$(dirname "$0")" && pwd)"
ERRORS_DIR="$DIR/errors"

PASS=0
FAIL=0
TOTAL=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "========================================"
echo " Liva Error Tests"
echo "========================================"
echo ""

for file in "$ERRORS_DIR"/*.liva; do
    [ -f "$file" ] || continue
    TOTAL=$((TOTAL + 1))
    basename=$(basename "$file")

    # Extract expected error code from comment: // EXPECT: Exxxx
    expected=$(grep -oP '// EXPECT: \K(E\d{4})' "$file" 2>/dev/null || true)
    if [ -z "$expected" ]; then
        echo -e "  ${YELLOW}SKIP${NC} $basename — no // EXPECT: Exxxx comment"
        continue
    fi

    # Run livac check and capture output
    output=$($LIVAC check "$file" 2>&1 || true)

    # Check if expected error code appears in output
    if echo "$output" | grep -q "$expected"; then
        echo -e "  ${GREEN}PASS${NC} $basename → $expected"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $basename — expected $expected"
        actual=$(echo "$output" | grep -oE 'E[0-9]{4}' | head -1 || echo "none")
        echo -e "       got: $actual"
        FAIL=$((FAIL + 1))
    fi
done

echo ""
echo "========================================"
echo " Results: $PASS/$TOTAL PASS, $FAIL FAIL"
echo "========================================"

[ "$FAIL" -eq 0 ] && exit 0 || exit 1
