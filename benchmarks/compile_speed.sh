#!/usr/bin/env bash
# Compile-speed benchmark: measure how fast gen-2 compiles real programs.
# Reports median of N runs per program. Used as a regression gate.
#
# Usage:
#   ./compile_speed.sh                  # default 3 runs, all corpora
#   ./compile_speed.sh --runs 5         # 5 runs per program
#   ./compile_speed.sh --quick          # only bootstrap_apps (fastest corpus)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GEN2="$ROOT/target/livac-gen2-release"
BOOTSTRAP="$ROOT/target/livac-gen1-release"

RUNS=3
QUICK=0
while [ $# -gt 0 ]; do
    case "$1" in
        --runs) RUNS="$2"; shift 2 ;;
        --quick) QUICK=1; shift ;;
        *) echo "unknown arg: $1"; exit 2 ;;
    esac
done

[ -x "$GEN2" ] || { echo "gen-2 not built: ./scripts/rebuild_selfhost.sh"; exit 2; }

WORK="${TMPDIR:-/tmp}/liva_compile_speed"
rm -rf "$WORK"; mkdir -p "$WORK"

# median of integers (ms)
median() {
    local sorted
    sorted=$(printf '%s\n' "$@" | sort -n)
    local n=$#
    local mid=$((n / 2))
    echo "$sorted" | sed -n "$((mid + 1))p"
}

# time a single compile in ms (--check, no codegen → measures front-end only)
time_check() {
    local src="$1"
    local t0 t1
    t0=$(date +%s%N)
    "$GEN2" check "$src" >/dev/null 2>&1 || true
    t1=$(date +%s%N)
    echo $(( (t1 - t0) / 1000000 ))
}

# time a full release build in ms
time_build() {
    local src="$1" outdir="$2"
    rm -rf "$outdir"; mkdir -p "$outdir"
    cp "$src" "$outdir/main.liva"
    local t0 t1
    t0=$(date +%s%N)
    (cd "$outdir" && "$GEN2" build --release --output "$outdir/build" main.liva >/dev/null 2>&1) || true
    t1=$(date +%s%N)
    echo $(( (t1 - t0) / 1000000 ))
}

run_corpus() {
    local label="$1" pattern="$2" mode="$3"
    echo ""
    echo "## $label ($mode)"
    echo ""
    printf '| Program | LOC | min | median | max |\n'
    printf '|---|---:|---:|---:|---:|\n'
    local total_median=0 count=0
    for src in $pattern; do
        [ -f "$src" ] || continue
        local name loc results=()
        name=$(basename "$src" .liva)
        loc=$(wc -l < "$src")
        for r in $(seq 1 "$RUNS"); do
            if [ "$mode" = "check" ]; then
                results+=("$(time_check "$src")")
            else
                results+=("$(time_build "$src" "$WORK/${label}_${name}_${r}")")
            fi
        done
        local mn mx md
        mn=$(printf '%s\n' "${results[@]}" | sort -n | head -1)
        mx=$(printf '%s\n' "${results[@]}" | sort -n | tail -1)
        md=$(median "${results[@]}")
        printf '| %s | %d | %dms | **%dms** | %dms |\n' "$name" "$loc" "$mn" "$md" "$mx"
        total_median=$((total_median + md))
        count=$((count + 1))
    done
    if [ "$count" -gt 0 ]; then
        echo ""
        echo "**Sum of medians:** ${total_median}ms across ${count} programs"
    fi
}

echo "# Compile-speed benchmark"
echo ""
echo "- Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "- Compiler: gen-2 ($(stat -c %s "$GEN2" 2>/dev/null || stat -f %z "$GEN2") bytes)"
echo "- Runs per program: $RUNS"
echo "- Mode: \`check\` (front-end) and \`build --release\` (full pipeline + rustc)"
echo ""

run_corpus "bootstrap_apps" "$ROOT/compiler/tests/bootstrap_apps/*.liva" "check"

if [ "$QUICK" = "0" ]; then
    run_corpus "bootstrap_apps_full" "$ROOT/compiler/tests/bootstrap_apps/*.liva" "build"
fi

echo ""
echo "Done. Save this output to \`benchmarks/COMPILE_SPEED.md\` to update the baseline."
