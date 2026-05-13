#!/usr/bin/env bash
#
# Rebuild self-host generations and verify idempotence.
# Location: compiler/tests/rebuild_selfhost.sh
#
# Usage: ./rebuild_selfhost.sh
#
# Steps:
#   1. Bootstrap → gen-1
#   2. Gen-1 → gen-2 (new binary)
#   3. Gen-2 → gen-3 (new binary)
#   4. Verify idempotence: gen-2 source == gen-3 source
#
# Outputs:
#   target/livac-gen1-release   → gen-1 binary (compiled by bootstrap)
#   target/livac-gen2-release   → gen-2 binary (compiled by gen-1)
#   target/livac-gen3-release   → gen-3 binary (compiled by gen-2)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
# Cycle 44: allow overriding the bootstrap binary to work around fragility
# bugs in the FROZEN rust bootstrap (BS-FRAG-1/2/3 in BUGS.md). When
# LIVAC_BOOTSTRAP is set (or target/livac-bootstrap exists), use that
# instead of target/release/livac.
if [[ -n "${LIVAC_BOOTSTRAP:-}" ]]; then
    BOOTSTRAP="$LIVAC_BOOTSTRAP"
elif [[ -x "$REPO_ROOT/target/livac-bootstrap" ]]; then
    BOOTSTRAP="$REPO_ROOT/target/livac-bootstrap"
else
    BOOTSTRAP="$REPO_ROOT/target/release/livac"
fi
SELF_SRC="$REPO_ROOT/compiler/src/main.liva"

GEN1_DIR="$HOME/tmp/gen1_build"
GEN2_DIR="$HOME/tmp/gen2_build"
GEN3_DIR="$HOME/tmp/gen3_build"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

die() { echo -e "${RED}ERROR${NC}: $*" >&2; exit 1; }

[[ -x "$BOOTSTRAP" ]] || die "bootstrap not found at $BOOTSTRAP — run cargo build --release"

cd "$REPO_ROOT"

echo -e "${YELLOW}[1/4] Bootstrap → gen-1${NC}"
rm -rf "$GEN1_DIR"
"$BOOTSTRAP" build --release --output "$GEN1_DIR" "$SELF_SRC" >/dev/null 2>&1 \
    || die "bootstrap failed to compile self-host source"
GEN1_BIN=$(find "$GEN1_DIR/target/release" -maxdepth 1 -type f -executable ! -name "*.d" ! -name "*.rlib" | head -1)
[[ -n "$GEN1_BIN" ]] || die "gen-1 binary not produced"
cp "$GEN1_BIN" "$REPO_ROOT/target/livac-gen1-release"
echo "  ✓ gen-1: $GEN1_BIN"

echo -e "${YELLOW}[2/4] Gen-1 → gen-2${NC}"
rm -rf "$GEN2_DIR"
"$REPO_ROOT/target/livac-gen1-release" build --release --output "$GEN2_DIR" "$SELF_SRC" >/dev/null 2>&1 \
    || die "gen-1 failed to compile self-host source"
GEN2_BIN=$(find "$GEN2_DIR/target/release" -maxdepth 1 -type f -executable ! -name "*.d" ! -name "*.rlib" | head -1)
[[ -n "$GEN2_BIN" ]] || die "gen-2 binary not produced"
cp "$GEN2_BIN" "$REPO_ROOT/target/livac-gen2-release"
echo "  ✓ gen-2: $GEN2_BIN"

echo -e "${YELLOW}[3/4] Gen-2 → gen-3${NC}"
rm -rf "$GEN3_DIR"
"$REPO_ROOT/target/livac-gen2-release" build --release --output "$GEN3_DIR" "$SELF_SRC" >/dev/null 2>&1 \
    || die "gen-2 failed to compile self-host source"
GEN3_BIN=$(find "$GEN3_DIR/target/release" -maxdepth 1 -type f -executable ! -name "*.d" ! -name "*.rlib" | head -1)
[[ -n "$GEN3_BIN" ]] || die "gen-3 binary not produced"
cp "$GEN3_BIN" "$REPO_ROOT/target/livac-gen3-release"
echo "  ✓ gen-3: $GEN3_BIN"

echo -e "${YELLOW}[4/4] Idempotence check${NC}"
SRC_DIFF=$(diff -r "$GEN2_DIR/src" "$GEN3_DIR/src" 2>&1 | head -10 || true)
if [[ -z "$SRC_DIFF" ]]; then
    echo -e "  ${GREEN}✓ source idempotent${NC} (gen-2 src == gen-3 src)"
else
    echo -e "  ${RED}✗ source NOT idempotent${NC}:"
    echo "$SRC_DIFF" | sed 's/^/    /'
    exit 1
fi

if cmp -s "$REPO_ROOT/target/livac-gen2-release" "$REPO_ROOT/target/livac-gen3-release"; then
    echo -e "  ${GREEN}✓ binary idempotent${NC} (gen-2 bin == gen-3 bin)"
else
    echo -e "  ${YELLOW}⚠ binary differs${NC} (timestamps/embedded paths — source idempotence is the actual contract)"
fi

echo ""
echo -e "${GREEN}=== Self-host rebuild OK ===${NC}"
