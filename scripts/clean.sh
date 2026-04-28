#!/usr/bin/env bash
# clean.sh — Remove all regenerable build artifacts from the Liva project.
#
# Wipes:
#   - cargo target dirs (livac/target, livac-project/target, nested target/)
#   - liva build artifacts (.liva_build/, liva_build/, *.liva.rs)
#   - generational/self-hosting scratch dirs (gen*_build, bootstrap_test, etc.)
#   - node_modules in vscode-extension/ and website/
#
# Safe: does NOT touch .git/, source files, configs, fixtures, or .env files.
#
# Usage:
#   bash scripts/clean.sh           # interactive (lists, asks)
#   bash scripts/clean.sh --yes     # non-interactive
#   bash scripts/clean.sh --dry-run # show what would be deleted

set -euo pipefail

# Resolve repo roots: this script lives at <livac>/scripts/clean.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$(cd "$LIVAC_DIR/.." && pwd)"

DRY_RUN=0
ASSUME_YES=0
for arg in "$@"; do
  case "$arg" in
    --dry-run|-n) DRY_RUN=1 ;;
    --yes|-y)     ASSUME_YES=1 ;;
    -h|--help)
      sed -n '2,18p' "$0"; exit 0 ;;
    *) echo "Unknown arg: $arg" >&2; exit 2 ;;
  esac
done

run() {
  if [[ $DRY_RUN -eq 1 ]]; then
    echo "[dry-run] $*"
  else
    eval "$@"
  fi
}

human_du() { du -sh "$1" 2>/dev/null | awk '{print $1}'; }

# Collect deletion targets (only paths that exist)
TARGETS=()
add() { [[ -e "$1" ]] && TARGETS+=("$1") || true; }

# 1) Cargo target dirs
add "$LIVAC_DIR/target"
add "$PROJECT_DIR/target"

# 2) Nested target dirs under examples and fixtures (regenerable)
while IFS= read -r -d '' d; do TARGETS+=("$d"); done < <(
  find "$LIVAC_DIR/examples" "$LIVAC_DIR/tests/fixtures" "$LIVAC_DIR/compiler/tests" \
       -type d \( -name target -o -name '.liva_build' -o -name 'liva_build' -o -name '.liva_test_build' \) \
       -prune -print0 2>/dev/null || true
)

# 3) node_modules (only the two known locations)
add "$PROJECT_DIR/vscode-extension/node_modules"
add "$PROJECT_DIR/website/node_modules"
add "$PROJECT_DIR/website/dist"
add "$PROJECT_DIR/website/.astro"
add "$PROJECT_DIR/vscode-extension/out"

if [[ ${#TARGETS[@]} -eq 0 ]]; then
  echo "Nothing to clean. Tree is already tidy."
  exit 0
fi

echo "Will delete the following (${#TARGETS[@]} paths):"
TOTAL_KB=0
for t in "${TARGETS[@]}"; do
  size_kb=$(du -sk "$t" 2>/dev/null | awk '{print $1}')
  TOTAL_KB=$((TOTAL_KB + ${size_kb:-0}))
  printf "  %8s  %s\n" "$(human_du "$t")" "$t"
done
printf "  --------\n  total ~%s\n\n" "$(numfmt --to=iec --suffix=B $((TOTAL_KB * 1024)) 2>/dev/null || echo "${TOTAL_KB}K")"

if [[ $DRY_RUN -eq 1 ]]; then
  echo "(dry-run, nothing removed)"
  exit 0
fi

if [[ $ASSUME_YES -ne 1 ]]; then
  read -r -p "Proceed? [y/N] " ans
  [[ "$ans" =~ ^[yY]$ ]] || { echo "Aborted."; exit 1; }
fi

# Prefer cargo clean for the official target dir (faster + integrates with cargo)
if command -v cargo >/dev/null 2>&1 && [[ -f "$LIVAC_DIR/Cargo.toml" ]]; then
  ( cd "$LIVAC_DIR" && cargo clean ) || true
fi

for t in "${TARGETS[@]}"; do
  [[ -e "$t" ]] || continue
  run "rm -rf -- \"$t\""
done

echo "✓ Clean complete."
