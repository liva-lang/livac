#!/usr/bin/env bash
# Official benchmark: self-host gen-2 generated Rust vs hand-written Rust
# Runs each binary 5 times and reports the median.
#
# Usage:
#   LIVAC=./target/livac-gen2-release ./benchmarks/run_official.sh
set -e

cd "$(dirname "$0")/.."

LIVAC="${LIVAC:-./target/livac-gen2-release}"
BENCH_DIR="benchmarks"
OUT="$BENCH_DIR/RESULTS.md"
RUNS=5

if [[ ! -x "$LIVAC" ]]; then
    echo "Liva compiler not found at $LIVAC"
    exit 1
fi

LIVAC_VERSION_LABEL="${LIVAC_LABEL:-self-host gen-2 (release)}"

{
    echo "# Benchmark Results — $(date '+%Y-%m-%d %H:%M')"
    echo
    echo "Liva compiler: \`$LIVAC\` ($LIVAC_VERSION_LABEL)"
    echo "Each binary executed $RUNS times; the **median** is reported."
    echo
    echo "## Environment"
    echo '```'
    uname -a
    rustc --version
    echo '```'
    echo
} > "$OUT"

median_ms() {
    # given lines of the form "Label: NNNms ..." filtered to one label,
    # extract the integer ms values and print the median.
    awk '{
        for (i = 1; i <= NF; i++) {
            if (match($i, /[0-9]+ms/)) {
                tok = substr($i, RSTART, RLENGTH - 2)
                v[NR] = tok+0
                break
            }
        }
    } END {
        n = 0; for (k in v) { a[++n] = v[k] }
        for (i = 1; i <= n; i++) for (j = i+1; j <= n; j++) if (a[i] > a[j]) { t=a[i]; a[i]=a[j]; a[j]=t }
        if (n == 0) print "N/A"
        else if (n % 2 == 1) print a[(n+1)/2]
        else printf "%d\n", (a[n/2] + a[n/2+1]) / 2
    }'
}

run_n() {
    local bin="$1"; local n="$2"
    local out=""
    for ((i=0; i<n; i++)); do
        out="$out"$'\n'"$("$bin")"
    done
    echo "$out"
}

# Collect all metric labels (text before the colon) from one run's output.
labels_in() {
    awk -F: 'NF >= 2 && /[0-9]+ms/ { print $1 }' <<<"$1"
}

for bench in strings collections classes; do
    echo "=== $bench ==="

    LIVA_FILE="$BENCH_DIR/liva/bench_${bench}.liva"
    RUST_FILE="$BENCH_DIR/rust/bench_${bench}.rs"

    # 1. Generate Rust from Liva (ignore self-host's misleading exit code; verify via cargo)
    rm -rf target/liva_build
    "$LIVAC" build "$LIVA_FILE" --release >/dev/null 2>&1 || true
    ( cd target/liva_build && cargo build --release >/dev/null 2>&1 )

    # Find the produced binary (first executable >500K in target/release).
    LIVA_BIN=""
    for f in target/liva_build/target/release/*; do
        if [[ -f "$f" && -x "$f" && $(stat -c%s "$f") -gt 500000 ]]; then
            LIVA_BIN="$f"; break
        fi
    done
    if [[ -z "$LIVA_BIN" ]]; then
        echo "  Liva binary not found — skipping" >&2
        echo "## $bench — skipped (Liva build failed)" >> "$OUT"
        continue
    fi

    # 2. Compile hand-written Rust with `rustc -O`.
    RUST_BIN="$BENCH_DIR/rust/bench_${bench}"
    rustc -O -o "$RUST_BIN" "$RUST_FILE" 2>/dev/null

    # 3. Warm up (first run is JIT/page-fault heavy).
    "$LIVA_BIN" >/dev/null
    "$RUST_BIN" >/dev/null

    # 4. Run RUNS times, capturing all output.
    LIVA_ALL=$(run_n "$LIVA_BIN" "$RUNS")
    RUST_ALL=$(run_n "$RUST_BIN" "$RUNS")

    # 5. Per-label medians.
    {
        echo
        echo "## Benchmark: $bench"
        echo
        echo "| Metric | Liva (median) | Rust (median) | Liva/Rust |"
        echo "|---|---:|---:|---:|"

        # Build label list from first non-empty Rust run
        first_rust_run=$("$RUST_BIN")
        while IFS= read -r label; do
            [[ -z "$label" ]] && continue
            # collect all values across RUNS for this label, both sides
            liva_vals=$(grep -F "$label:" <<<"$LIVA_ALL" | median_ms)
            rust_vals=$(grep -F "$label:" <<<"$RUST_ALL" | median_ms)
            ratio="N/A"
            if [[ "$liva_vals" =~ ^[0-9]+$ && "$rust_vals" =~ ^[0-9]+$ && "$rust_vals" -gt 0 ]]; then
                ratio=$(awk -v l="$liva_vals" -v r="$rust_vals" 'BEGIN { printf "%.2fx", l/r }')
            elif [[ "$rust_vals" == "0" ]]; then
                ratio="(rust ≈ 0ms)"
            fi
            printf "| %s | %sms | %sms | %s |\n" "$label" "$liva_vals" "$rust_vals" "$ratio"
        done <<<"$(labels_in "$first_rust_run")"

        echo
        echo "<details><summary>raw output (5 runs each)</summary>"
        echo
        echo "**Liva**"
        echo '```'
        echo "$LIVA_ALL"
        echo '```'
        echo
        echo "**Rust**"
        echo '```'
        echo "$RUST_ALL"
        echo '```'
        echo
        echo "</details>"
    } >> "$OUT"
done

echo
echo "Saved to $OUT"
