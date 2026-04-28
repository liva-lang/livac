#!/usr/bin/env bash
# Benchmark suite: Liva-generated Rust vs hand-written Rust
# Usage: ./benchmarks/run_benchmarks.sh
set -e

cd "$(dirname "$0")/.."

LIVAC="${LIVAC:-./target/release/livac}"
BENCH_DIR="benchmarks"
RESULTS_FILE="$BENCH_DIR/RESULTS.md"

echo "=== Liva vs Rust Benchmark Suite ==="
echo ""

# Ensure livac is built
if [[ ! -f "$LIVAC" ]]; then
    echo "Building livac..."
    cargo build --release
fi

echo "# Benchmark Results — $(date '+%Y-%m-%d %H:%M')" > "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"
echo "## Environment" >> "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
uname -a >> "$RESULTS_FILE"
rustc --version >> "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

for bench in strings collections classes; do
    echo "--- Benchmark: $bench ---"
    echo ""

    LIVA_FILE="$BENCH_DIR/liva/bench_${bench}.liva"
    RUST_FILE="$BENCH_DIR/rust/bench_${bench}.rs"

    # Compile Liva version
    echo "  Compiling Liva version..."
    $LIVAC build "$LIVA_FILE" --release 2>/dev/null || true
    LIVA_BIN="target/liva_build/target/release/bench_${bench}"
    if [[ ! -f "$LIVA_BIN" ]]; then
        # Try alternative binary name
        LIVA_BIN="target/liva_build/target/release/liva_project"
    fi

    # Compile Rust version
    echo "  Compiling Rust version..."
    rustc -O -o "$BENCH_DIR/rust/bench_${bench}" "$RUST_FILE" 2>/dev/null
    RUST_BIN="$BENCH_DIR/rust/bench_${bench}"

    echo "" >> "$RESULTS_FILE"
    echo "## Benchmark: $bench" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"

    if [[ -f "$LIVA_BIN" ]] && [[ -f "$RUST_BIN" ]]; then
        echo "  Running Liva version..."
        LIVA_OUT=$("$LIVA_BIN" 2>&1)
        echo "  Running Rust version..."
        RUST_OUT=$("$RUST_BIN" 2>&1)

        echo "  Liva:"
        echo "$LIVA_OUT" | sed 's/^/    /'
        echo "  Rust:"
        echo "$RUST_OUT" | sed 's/^/    /'

        echo "### Liva (generated Rust)" >> "$RESULTS_FILE"
        echo '```' >> "$RESULTS_FILE"
        echo "$LIVA_OUT" >> "$RESULTS_FILE"
        echo '```' >> "$RESULTS_FILE"
        echo "" >> "$RESULTS_FILE"
        echo "### Rust (hand-written)" >> "$RESULTS_FILE"
        echo '```' >> "$RESULTS_FILE"
        echo "$RUST_OUT" >> "$RESULTS_FILE"
        echo '```' >> "$RESULTS_FILE"
    else
        echo "  SKIP — binary not found"
        echo "*Skipped — compilation failed*" >> "$RESULTS_FILE"
    fi

    echo ""
done

echo "" >> "$RESULTS_FILE"
echo "---" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"
echo "Target: <10% throughput difference, <2x allocations." >> "$RESULTS_FILE"

echo "=== Results saved to $RESULTS_FILE ==="
echo ""
cat "$RESULTS_FILE"
