#!/usr/bin/env bash
# Liva self-hosted compiler — full validation gate runner.
# Exits 0 only if all five gates pass.
#
# Usage:
#   bash compiler/tests/run_all.sh           # run all
#   bash compiler/tests/run_all.sh --quick   # skip rebuild_selfhost (fastest dev loop)
#
# Mirrors `make test-full`. Prints a final summary table.

set -u
cd "$(dirname "$0")/../.."  # repo root = livac/

SCRIPT_DIR="compiler/tests"
QUICK=${1:-}

declare -a NAMES
declare -a STATUSES
declare -a DURATIONS

run_gate() {
    local name="$1"
    local cmd="$2"
    local start
    start=$(date +%s)
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "▶ Gate: $name"
    echo "═══════════════════════════════════════════════════════════"
    if eval "$cmd"; then
        local end=$(date +%s)
        NAMES+=("$name")
        STATUSES+=("PASS")
        DURATIONS+=("$((end - start))s")
        return 0
    else
        local end=$(date +%s)
        NAMES+=("$name")
        STATUSES+=("FAIL")
        DURATIONS+=("$((end - start))s")
        return 1
    fi
}

OVERALL=0

if [[ "$QUICK" != "--quick" ]]; then
    run_gate "rebuild_selfhost (gen-2 ≡ gen-3)" "bash $SCRIPT_DIR/rebuild_selfhost.sh" || OVERALL=1
fi
run_gate "bootstrap_apps (gen-2)"   "bash $SCRIPT_DIR/bootstrap_apps/run_gen2.sh"   || OVERALL=1
run_gate "multifile_apps (gen-2)"   "bash $SCRIPT_DIR/multifile_apps/run.sh"        || OVERALL=1
run_gate "regression"               "bash $SCRIPT_DIR/regression/run.sh"            || OVERALL=1
run_gate "complex_apps"             "bash $SCRIPT_DIR/complex_apps/run.sh"          || OVERALL=1
run_gate "e2e_selfhost"             "bash $SCRIPT_DIR/e2e_selfhost.sh"              || OVERALL=1
run_gate "cargo test --release"     "cargo test --release --quiet 2>&1 | tail -1"   || OVERALL=1

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Summary"
echo "═══════════════════════════════════════════════════════════"
for i in "${!NAMES[@]}"; do
    printf "  [%-4s] %-40s %s\n" "${STATUSES[i]}" "${NAMES[i]}" "${DURATIONS[i]}"
done

if [[ $OVERALL -eq 0 ]]; then
    echo ""
    echo "✅ All gates passed."
else
    echo ""
    echo "❌ One or more gates failed."
fi
exit $OVERALL
