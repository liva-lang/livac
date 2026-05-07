#!/usr/bin/env bash
# Binary size measurement for livac compilers.
# Compares bootstrap (target/release/livac) and gen-1/2/3 self-host binaries.
# Reports both raw size (as built) and stripped size (apples-to-apples).
#
# Usage:
#   ./benchmarks/binary_size.sh
set -e

cd "$(dirname "$0")/.."

BINS=(
    "target/release/livac"
    "target/livac-gen1-release"
    "target/livac-gen2-release"
    "target/livac-gen3-release"
)

LABELS=(
    "bootstrap (Rust, full compiler+LSP+fmt+lint)"
    "gen-1 (self-host built by bootstrap)"
    "gen-2 (self-host built by gen-1)"
    "gen-3 (self-host built by gen-2)"
)

human() {
    # bytes -> "X.XX MB (N bytes)"
    local b=$1
    awk -v b="$b" 'BEGIN { printf "%.2f MB (%s bytes)", b/1024/1024, b }'
}

echo "## Binary size — $(date '+%Y-%m-%d %H:%M')"
echo
echo "| Binary | Raw | Stripped |"
echo "|--------|-----|----------|"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

for i in "${!BINS[@]}"; do
    bin="${BINS[$i]}"
    label="${LABELS[$i]}"
    if [[ ! -f "$bin" ]]; then
        echo "| $label | _missing_ | _missing_ |"
        continue
    fi
    raw=$(stat -c%s "$bin")
    cp "$bin" "$TMP/b"
    strip "$TMP/b" 2>/dev/null || true
    stripped=$(stat -c%s "$TMP/b")
    echo "| $label | $(human $raw) | $(human $stripped) |"
done
echo
echo "Idempotency check (gen-2 vs gen-3 stripped):"
if [[ -f target/livac-gen2-release && -f target/livac-gen3-release ]]; then
    cp target/livac-gen2-release "$TMP/g2"
    cp target/livac-gen3-release "$TMP/g3"
    strip "$TMP/g2" "$TMP/g3"
    if cmp -s "$TMP/g2" "$TMP/g3"; then
        echo "  ✓ identical (byte-for-byte)"
    else
        s2=$(stat -c%s "$TMP/g2")
        s3=$(stat -c%s "$TMP/g3")
        echo "  ✗ differ: gen-2=$s2  gen-3=$s3"
    fi
else
    echo "  _gen-2 or gen-3 missing_"
fi
