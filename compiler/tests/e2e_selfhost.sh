#!/usr/bin/env bash
#
# E2E Self-host Validation
# Location: compiler/tests/e2e_selfhost.sh
#
# Compiles and runs each curated .liva file with BOTH the Rust bootstrap
# compiler and the self-host gen-2 binary, then diffs stdout. Verifies
# that gen-2 generates programs with identical runtime behavior to the
# bootstrap (the contract for self-hosting).
#
# Usage:
#   ./e2e_selfhost.sh
#
# Exit code: 0 if all tests pass, 1 otherwise.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LIVAC_BOOTSTRAP="$REPO_ROOT/target/release/livac"
LIVAC_SELFHOST="$REPO_ROOT/target/livac-gen2-release"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Curated test set: deterministic .liva programs (no time, no random,
# no I/O, no network). All compile cleanly with the Rust bootstrap.
# The self-host gen-2 must produce binaries with identical stdout.
TESTS=(
    "compiler/tests/e2e_progs/basics.liva"
    "compiler/tests/e2e_progs/enums_match.liva"
    "compiler/tests/e2e_progs/errors.liva"
    "compiler/tests/e2e_progs/stdlib.liva"
    "examples/calculator/calculator.liva"
)

if [[ ! -x "$LIVAC_BOOTSTRAP" ]]; then
    echo -e "${RED}Error: bootstrap not found at $LIVAC_BOOTSTRAP${NC}" >&2
    echo "Run: cargo build --release" >&2
    exit 1
fi
if [[ ! -x "$LIVAC_SELFHOST" ]]; then
    echo -e "${RED}Error: self-host gen-2 not found at $LIVAC_SELFHOST${NC}" >&2
    echo "Run: ./compiler/tests/bootstrap_test.sh && cp ~/tmp/gen2_build/target/release/main $LIVAC_SELFHOST" >&2
    exit 1
fi

PASS=0
FAIL=0
SKIP=0
FAILED_TESTS=()

for t in "${TESTS[@]}"; do
    src="$REPO_ROOT/$t"
    if [[ ! -f "$src" ]]; then
        echo -e "${YELLOW}SKIP${NC} $t (not found)"
        SKIP=$((SKIP + 1))
        continue
    fi

    tmp_a=$(mktemp -d)
    tmp_b=$(mktemp -d)
    trap "rm -rf '$tmp_a' '$tmp_b'" EXIT

    # Compile with bootstrap
    "$LIVAC_BOOTSTRAP" build --release --output "$tmp_a" "$src" >/dev/null 2>&1 || true
    bin_a=$(find "$tmp_a/target/release" -maxdepth 1 -type f -executable ! -name "*.d" 2>/dev/null | head -1)
    if [[ -z "$bin_a" ]]; then
        echo -e "${YELLOW}SKIP${NC} $t (bootstrap compile failed)"
        SKIP=$((SKIP + 1))
        rm -rf "$tmp_a" "$tmp_b"
        continue
    fi

    # Compile with self-host gen-2
    # Note: as of v2.0 (BUG-1 fixed) gen-2 correctly reports build status.
    # We still tolerate non-zero exit codes here for forward compatibility
    # with future error-propagation work, and rely on the binary's presence.
    "$LIVAC_SELFHOST" build --release --output "$tmp_b" "$src" >/dev/null 2>&1 || true
    bin_b=$(find "$tmp_b/target/release" -maxdepth 1 -type f -executable ! -name "*.d" 2>/dev/null | head -1)
    if [[ -z "$bin_b" ]]; then
        echo -e "${RED}FAIL${NC} $t (self-host did not produce binary)"
        FAIL=$((FAIL + 1))
        FAILED_TESTS+=("$t")
        rm -rf "$tmp_a" "$tmp_b"
        continue
    fi

    # Run both with timeout — compare stdout only (stderr may differ in cosmetics)
    out_a=$(timeout 30 "$bin_a" 2>/dev/null) || true
    out_b=$(timeout 30 "$bin_b" 2>/dev/null) || true

    if [[ "$out_a" == "$out_b" ]]; then
        echo -e "${GREEN}PASS${NC} $t"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC} $t (stdout mismatch)"
        # Show first 5 differing lines
        diff <(echo "$out_a") <(echo "$out_b") | head -10 | sed 's/^/    /'
        FAIL=$((FAIL + 1))
        FAILED_TESTS+=("$t")
    fi

    rm -rf "$tmp_a" "$tmp_b"
done

echo ""
echo "===================="
echo "  E2E self-host"
echo "===================="
echo -e "  ${GREEN}Passed${NC}: $PASS"
echo -e "  ${RED}Failed${NC}: $FAIL"
echo -e "  ${YELLOW}Skipped${NC}: $SKIP"

if [[ $FAIL -gt 0 ]]; then
    echo ""
    echo -e "${RED}Failed tests:${NC}"
    for t in "${FAILED_TESTS[@]}"; do
        echo "  - $t"
    done
    exit 1
fi
exit 0
