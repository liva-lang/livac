#!/usr/bin/env bash
#
# Liva Test Suite Runner
# Location: compiler/tests/liva/run_tests.sh
#
# Usage:
#   ./run_tests.sh              # all layers except stdlib-io
#   ./run_tests.sh --all        # all layers including stdlib-io
#   ./run_tests.sh --only syntax
#   ./run_tests.sh --only compile
#   ./run_tests.sh --only e2e
#   ./run_tests.sh --only stdlib
#   ./run_tests.sh --only stdlib-io
#   ./run_tests.sh --only errors
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
LIVAC="$REPO_ROOT/target/release/livac"
BUILD_DIR="/tmp/liva_test_builds"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0
ERRORS=()

# Parse arguments
ONLY=""
INCLUDE_IO=false
for arg in "$@"; do
    case "$arg" in
        --all) INCLUDE_IO=true ;;
        --only) shift; ONLY="${1:-}" ;;
        --only=*) ONLY="${arg#--only=}" ;;
    esac
done

# Ensure livac exists
if [[ ! -x "$LIVAC" ]]; then
    echo -e "${RED}Error: livac not found at $LIVAC${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

mkdir -p "$BUILD_DIR"

#───────────────────────────────────────────────────────
# Layer 1: Syntax — livac check must pass
#───────────────────────────────────────────────────────
run_syntax_tests() {
    local dir="$SCRIPT_DIR/syntax"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 1: Syntax Tests (livac check) ═══${NC}"
    for f in "$dir"/*.liva; do
        [[ ! -f "$f" ]] && continue
        local name
        name="$(basename "$f")"
        if $LIVAC check "$f" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            ((PASS++))
        else
            echo -e "  ${RED}✗${NC} $name"
            ((FAIL++))
            ERRORS+=("syntax/$name: livac check failed")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 2: Compile — livac build + cargo check
#───────────────────────────────────────────────────────
run_compile_tests() {
    local dir="$SCRIPT_DIR/compile"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 2: Compile Tests (livac build) ═══${NC}"
    for f in "$dir"/*.liva; do
        [[ ! -f "$f" ]] && continue
        local name base out_dir
        name="$(basename "$f")"
        base="${name%.liva}"
        out_dir="$BUILD_DIR/compile_$base"
        rm -rf "$out_dir"
        if $LIVAC build "$f" --output "$out_dir" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            ((PASS++))
        else
            echo -e "  ${RED}✗${NC} $name"
            ((FAIL++))
            ERRORS+=("compile/$name: livac build failed")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 3: E2E — build + run + compare output
#───────────────────────────────────────────────────────
run_e2e_tests() {
    local dir="$SCRIPT_DIR/e2e"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 3: E2E Tests (build + run + compare) ═══${NC}"
    for f in "$dir"/*.liva; do
        [[ ! -f "$f" ]] && continue
        local name base expected out_dir actual
        name="$(basename "$f")"
        base="${name%.liva}"
        expected="$dir/$base.expected"
        if [[ ! -f "$expected" ]]; then
            echo -e "  ${YELLOW}⊘${NC} $name (missing .expected)"
            ((SKIP++))
            continue
        fi
        out_dir="$BUILD_DIR/e2e_$base"
        rm -rf "$out_dir"
        # Build
        if ! $LIVAC build "$f" --output "$out_dir" > /dev/null 2>&1; then
            echo -e "  ${RED}✗${NC} $name (build failed)"
            ((FAIL++))
            ERRORS+=("e2e/$name: build failed")
            continue
        fi
        # Find binary
        local binary
        binary=$(find "$out_dir/target/debug" -maxdepth 1 -type f -executable ! -name "*.d" 2>/dev/null | head -1)
        if [[ -z "$binary" ]]; then
            echo -e "  ${RED}✗${NC} $name (no binary found)"
            ((FAIL++))
            ERRORS+=("e2e/$name: no binary found after build")
            continue
        fi
        # Run and compare
        actual=$("$binary" 2>&1 || true)
        local expected_content
        expected_content=$(cat "$expected")
        if [[ "$actual" == "$expected_content" ]]; then
            echo -e "  ${GREEN}✓${NC} $name"
            ((PASS++))
        else
            echo -e "  ${RED}✗${NC} $name (output mismatch)"
            ((FAIL++))
            ERRORS+=("e2e/$name: output mismatch")
            echo -e "    ${YELLOW}Expected:${NC} $(head -3 "$expected")"
            echo -e "    ${YELLOW}Actual:${NC}   $(echo "$actual" | head -3)"
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 4: Stdlib — build + run (same as E2E)
#───────────────────────────────────────────────────────
run_stdlib_tests() {
    local dir="$SCRIPT_DIR/stdlib"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 4: Stdlib Tests (build + run + compare) ═══${NC}"
    for f in "$dir"/*.liva; do
        [[ ! -f "$f" ]] && continue
        local name base expected out_dir actual
        name="$(basename "$f")"
        base="${name%.liva}"
        expected="$dir/$base.expected"
        if [[ ! -f "$expected" ]]; then
            echo -e "  ${YELLOW}⊘${NC} $name (missing .expected)"
            ((SKIP++))
            continue
        fi
        out_dir="$BUILD_DIR/stdlib_$base"
        rm -rf "$out_dir"
        if ! $LIVAC build "$f" --output "$out_dir" > /dev/null 2>&1; then
            echo -e "  ${RED}✗${NC} $name (build failed)"
            ((FAIL++))
            ERRORS+=("stdlib/$name: build failed")
            continue
        fi
        local binary
        binary=$(find "$out_dir/target/debug" -maxdepth 1 -type f -executable ! -name "*.d" 2>/dev/null | head -1)
        if [[ -z "$binary" ]]; then
            echo -e "  ${RED}✗${NC} $name (no binary found)"
            ((FAIL++))
            ERRORS+=("stdlib/$name: no binary found")
            continue
        fi
        actual=$("$binary" 2>&1 || true)
        local expected_content
        expected_content=$(cat "$expected")
        if [[ "$actual" == "$expected_content" ]]; then
            echo -e "  ${GREEN}✓${NC} $name"
            ((PASS++))
        else
            echo -e "  ${RED}✗${NC} $name (output mismatch)"
            ((FAIL++))
            ERRORS+=("stdlib/$name: output mismatch")
            echo -e "    ${YELLOW}Expected:${NC} $(head -3 "$expected")"
            echo -e "    ${YELLOW}Actual:${NC}   $(echo "$actual" | head -3)"
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 5: Stdlib-IO — opt-in (same mechanism)
#───────────────────────────────────────────────────────
run_stdlib_io_tests() {
    local dir="$SCRIPT_DIR/stdlib-io"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 5: Stdlib-IO Tests (opt-in) ═══${NC}"
    for f in "$dir"/*.liva; do
        [[ ! -f "$f" ]] && continue
        local name base expected out_dir actual
        name="$(basename "$f")"
        base="${name%.liva}"
        expected="$dir/$base.expected"
        if [[ ! -f "$expected" ]]; then
            echo -e "  ${YELLOW}⊘${NC} $name (missing .expected)"
            ((SKIP++))
            continue
        fi
        out_dir="$BUILD_DIR/stdlib_io_$base"
        rm -rf "$out_dir"
        if ! $LIVAC build "$f" --output "$out_dir" > /dev/null 2>&1; then
            echo -e "  ${RED}✗${NC} $name (build failed)"
            ((FAIL++))
            ERRORS+=("stdlib-io/$name: build failed")
            continue
        fi
        local binary
        binary=$(find "$out_dir/target/debug" -maxdepth 1 -type f -executable ! -name "*.d" 2>/dev/null | head -1)
        if [[ -z "$binary" ]]; then
            echo -e "  ${RED}✗${NC} $name (no binary found)"
            ((FAIL++))
            ERRORS+=("stdlib-io/$name: no binary found")
            continue
        fi
        actual=$("$binary" 2>&1 || true)
        local expected_content
        expected_content=$(cat "$expected")
        if [[ "$actual" == "$expected_content" ]]; then
            echo -e "  ${GREEN}✓${NC} $name"
            ((PASS++))
        else
            echo -e "  ${RED}✗${NC} $name (output mismatch)"
            ((FAIL++))
            ERRORS+=("stdlib-io/$name: output mismatch")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 6: Errors — livac check MUST fail
#───────────────────────────────────────────────────────
run_error_tests() {
    local dir="$SCRIPT_DIR/errors"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 6: Error Tests (must fail with expected error) ═══${NC}"
    for f in "$dir"/*.liva; do
        [[ ! -f "$f" ]] && continue
        local name base
        name="$(basename "$f")"
        base="${name%.liva}"
        # First line can have // EXPECT: <error code or substring>
        local expect_pattern
        expect_pattern=$(head -1 "$f" | grep -oP '// EXPECT: \K.*' || echo "")
        local output
        output=$($LIVAC check "$f" 2>&1 || true)
        local exit_code
        $LIVAC check "$f" > /dev/null 2>&1 && exit_code=0 || exit_code=$?
        if [[ $exit_code -eq 0 ]]; then
            echo -e "  ${RED}✗${NC} $name (should have failed but passed)"
            ((FAIL++))
            ERRORS+=("errors/$name: should have failed but passed check")
        elif [[ -n "$expect_pattern" ]]; then
            if echo "$output" | grep -qi "$expect_pattern"; then
                echo -e "  ${GREEN}✓${NC} $name (error: $expect_pattern)"
                ((PASS++))
            else
                echo -e "  ${RED}✗${NC} $name (expected '$expect_pattern' in error output)"
                ((FAIL++))
                ERRORS+=("errors/$name: expected '$expect_pattern' not found in output")
            fi
        else
            echo -e "  ${GREEN}✓${NC} $name (correctly fails)"
            ((PASS++))
        fi
    done
}

#───────────────────────────────────────────────────────
# Main
#───────────────────────────────────────────────────────
echo -e "${BOLD}🧪 Liva Test Suite${NC}"
echo -e "   livac: $($LIVAC --version 2>&1 | head -1)"
echo -e "   dir:   $SCRIPT_DIR"

case "$ONLY" in
    syntax)     run_syntax_tests ;;
    compile)    run_compile_tests ;;
    e2e)        run_e2e_tests ;;
    stdlib)     run_stdlib_tests ;;
    stdlib-io)  run_stdlib_io_tests ;;
    errors)     run_error_tests ;;
    "")
        run_syntax_tests
        run_compile_tests
        run_e2e_tests
        run_stdlib_tests
        $INCLUDE_IO && run_stdlib_io_tests
        run_error_tests
        ;;
    *)
        echo -e "${RED}Unknown layer: $ONLY${NC}"
        echo "Valid: syntax, compile, e2e, stdlib, stdlib-io, errors"
        exit 1
        ;;
esac

# Summary
echo -e "\n${BOLD}═══════════════════════════════════════════${NC}"
echo -e "  ${GREEN}✓ $PASS passed${NC}  ${RED}✗ $FAIL failed${NC}  ${YELLOW}⊘ $SKIP skipped${NC}"

if [[ ${#ERRORS[@]} -gt 0 ]]; then
    echo -e "\n${RED}Failures:${NC}"
    for err in "${ERRORS[@]}"; do
        echo -e "  ${RED}•${NC} $err"
    done
fi

echo -e "${BOLD}═══════════════════════════════════════════${NC}"

[[ $FAIL -eq 0 ]] && exit 0 || exit 1
