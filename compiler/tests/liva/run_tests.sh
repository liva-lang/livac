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


#───────────────────────────────────────────────────────
# Layer 1: Syntax — livac test (*.test.liva with assertions)
#───────────────────────────────────────────────────────
run_syntax_tests() {
    local dir="$SCRIPT_DIR/syntax"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 1: Syntax Tests (livac test) ═══${NC}"
    for f in "$dir"/*.test.liva; do
        [[ ! -f "$f" ]] && continue
        local name
        name="$(basename "$f")"
        if $LIVAC test "$f" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            PASS=$((PASS + 1))
        else
            echo -e "  ${RED}✗${NC} $name"
            FAIL=$((FAIL + 1))
            ERRORS+=("syntax/$name: livac test failed")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 2: Compile — livac test (*.test.liva with assertions)
#───────────────────────────────────────────────────────
run_compile_tests() {
    local dir="$SCRIPT_DIR/compile"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 2: Compile Tests (livac test) ═══${NC}"
    for f in "$dir"/*.test.liva; do
        [[ ! -f "$f" ]] && continue
        local name
        name="$(basename "$f")"
        if $LIVAC test "$f" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            PASS=$((PASS + 1))
        else
            echo -e "  ${RED}✗${NC} $name"
            FAIL=$((FAIL + 1))
            ERRORS+=("compile/$name: livac test failed")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 3: E2E — livac test (*.test.liva with assertions)
#───────────────────────────────────────────────────────
run_e2e_tests() {
    local dir="$SCRIPT_DIR/e2e"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 3: E2E Tests (livac test) ═══${NC}"
    for f in "$dir"/*.test.liva; do
        [[ ! -f "$f" ]] && continue
        local name
        name="$(basename "$f")"
        if $LIVAC test "$f" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            PASS=$((PASS + 1))
        else
            echo -e "  ${RED}✗${NC} $name"
            FAIL=$((FAIL + 1))
            ERRORS+=("e2e/$name: livac test failed")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 4: Stdlib — livac test (*.test.liva with assertions)
#───────────────────────────────────────────────────────
run_stdlib_tests() {
    local dir="$SCRIPT_DIR/stdlib"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 4: Stdlib Tests (livac test) ═══${NC}"
    for f in "$dir"/*.test.liva; do
        [[ ! -f "$f" ]] && continue
        local name
        name="$(basename "$f")"
        if $LIVAC test "$f" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            PASS=$((PASS + 1))
        else
            echo -e "  ${RED}✗${NC} $name"
            FAIL=$((FAIL + 1))
            ERRORS+=("stdlib/$name: livac test failed")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 5: Stdlib-IO — opt-in (livac test)
#───────────────────────────────────────────────────────
run_stdlib_io_tests() {
    local dir="$SCRIPT_DIR/stdlib-io"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 5: Stdlib-IO Tests (opt-in, livac test) ═══${NC}"
    for f in "$dir"/*.test.liva; do
        [[ ! -f "$f" ]] && continue
        local name
        name="$(basename "$f")"
        if $LIVAC test "$f" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            PASS=$((PASS + 1))
        else
            echo -e "  ${RED}✗${NC} $name"
            FAIL=$((FAIL + 1))
            ERRORS+=("stdlib-io/$name: livac test failed")
        fi
    done
}

#───────────────────────────────────────────────────────
# Layer 6: Errors — livac test (errors.test.liva invokes livac check)
#───────────────────────────────────────────────────────
run_error_tests() {
    local dir="$SCRIPT_DIR/errors"
    [[ ! -d "$dir" ]] && return
    echo -e "\n${CYAN}${BOLD}═══ Layer 6: Error Tests (livac test) ═══${NC}"
    for f in "$dir"/*.test.liva; do
        [[ ! -f "$f" ]] && continue
        local name
        name="$(basename "$f")"
        if $LIVAC test "$f" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name"
            PASS=$((PASS + 1))
        else
            echo -e "  ${RED}✗${NC} $name"
            FAIL=$((FAIL + 1))
            ERRORS+=("errors/$name: livac test failed")
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
