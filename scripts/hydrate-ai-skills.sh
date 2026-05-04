#!/usr/bin/env bash
# Hydrate examples/ai/*/.copilot/skills/liva-lang/ from the canonical skill.
# Run after `git clone` (or after editing skills/liva-lang/) to sync the
# Liva skill into each AI demo project.
#
# Why: skills/liva-lang/ is the source of truth; previously each AI
# example shipped its own 624KB copy (~2.5MB total, 4× duplication).
# Now examples/ai/*/.copilot/ is git-ignored and rebuilt from this script.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SKILL_SRC="$LIVAC_ROOT/skills/liva-lang"
AI_DIR="$LIVAC_ROOT/examples/ai"

if [[ ! -d "$SKILL_SRC" ]]; then
    echo "skills/liva-lang/ not found at $SKILL_SRC" >&2
    exit 1
fi

# Build the assembled skill (SKILL.md + docs/ → references/) once.
TMP_BUILT="$(mktemp -d)"
trap "rm -rf '$TMP_BUILT'" EXIT
cp -r "$SKILL_SRC"/. "$TMP_BUILT/"
mkdir -p "$TMP_BUILT/references"
if [[ -d "$LIVAC_ROOT/docs" ]]; then
    cp -r "$LIVAC_ROOT/docs"/. "$TMP_BUILT/references/" 2>/dev/null || true
fi

count=0
for proj in "$AI_DIR"/*/; do
    [[ -d "$proj" ]] || continue
    target="$proj.copilot/skills/liva-lang"
    rm -rf "$proj.copilot"
    mkdir -p "$target"
    cp -r "$TMP_BUILT"/. "$target/"
    count=$((count + 1))
done

echo "✓ Hydrated $count AI example project(s) at examples/ai/*/.copilot/skills/liva-lang/"
