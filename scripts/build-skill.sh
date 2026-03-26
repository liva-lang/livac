#!/bin/bash
# build-skill.sh — Build the liva-lang AI skill from docs/ + skills/
#
# Assembles a self-contained skill directory at dist/skill/liva-lang/
# by combining the compact SKILL.md with docs/ as references/.
#
# Usage:
#   scripts/build-skill.sh [--output DIR]
#
# The generated skill follows the Agent Skills specification (agentskills.io)
# and can be installed to ~/.agents/skills/ for use with any compatible agent.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default output directory
OUTPUT_DIR="${PROJECT_ROOT}/dist/skill/liva-lang"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --output) OUTPUT_DIR="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--output DIR]"
            echo "  Builds the liva-lang skill from docs/ into a self-contained directory."
            echo "  Default output: dist/skill/liva-lang/"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "🔧 Building liva-lang skill..."
echo "   Source: ${PROJECT_ROOT}"
echo "   Output: ${OUTPUT_DIR}"

# Files excluded from skill (dangerous wrong syntax, human-only tutorials, indexes)
EXCLUDE_LANG_REF=(
    "generics-advanced.md"    # Wrong syntax (fn, class, speculative stdlib)
    "json-advanced.md"        # Migration guide v0.9→v0.10, not for AI
    "syntax-overview.md"      # Just an index to other files
    "stdlib/README.md"        # Index duplicating individual module docs
    "stdlib/response.md"      # Merged into server.md
)

EXCLUDE_GUIDES=(
    "tuples.md"               # Duplicated by types-advanced.md
    "json-typed-parsing.md"   # Duplicated by json-basics.md
    "generics-quick-start.md" # Wrong syntax (fn, !, manual Option/Result)
)

is_excluded_lang() {
    local rel="$1"
    for ex in "${EXCLUDE_LANG_REF[@]}"; do
        [[ "$rel" == "$ex" ]] && return 0
    done
    return 1
}

is_excluded_guide() {
    local name="$1"
    for ex in "${EXCLUDE_GUIDES[@]}"; do
        [[ "$name" == "$ex" ]] && return 0
    done
    return 1
}

# Clean previous build
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/references"

# 1. Copy SKILL.md
cp "$PROJECT_ROOT/skills/liva-lang/SKILL.md" "$OUTPUT_DIR/SKILL.md"

# 2. Copy language-reference docs (filtered)
LANG_REF="$PROJECT_ROOT/docs/language-reference"
REFS="$OUTPUT_DIR/references"

if [ -d "$LANG_REF" ]; then
    find "$LANG_REF" -name "*.md" -type f | while read -r file; do
        rel="${file#$LANG_REF/}"
        if is_excluded_lang "$rel"; then
            echo "   ⊘ Excluded: language-reference/$rel"
            continue
        fi
        target_dir="$REFS/$(dirname "$rel")"
        mkdir -p "$target_dir"
        cp "$file" "$REFS/$rel"
    done
else
    echo "   ⚠ Missing: docs/language-reference/"
fi

# 3. Copy guides (filtered)
GUIDES_SRC="$PROJECT_ROOT/docs/guides"
GUIDES_DST="$REFS/guides"

if [ -d "$GUIDES_SRC" ]; then
    mkdir -p "$GUIDES_DST"
    find "$GUIDES_SRC" -name "*.md" -type f | while read -r file; do
        local_name="$(basename "$file")"
        if is_excluded_guide "$local_name"; then
            echo "   ⊘ Excluded: guides/$local_name"
            continue
        fi
        cp "$file" "$GUIDES_DST/$local_name"
    done
else
    echo "   ⚠ Missing: docs/guides/"
fi

# 4. No getting-started/ — installation/tutorial docs are for humans, not AI

# 5. Copy selected top-level doc files (only those useful for AI code generation)
for file in QUICK_REFERENCE.md ERROR_CODES.md; do
    if [ -f "$PROJECT_ROOT/docs/$file" ]; then
        cp "$PROJECT_ROOT/docs/$file" "$REFS/$file"
    fi
done

# 6. Count and report
TOTAL_FILES=$(find "$OUTPUT_DIR" -name "*.md" | wc -l)
TOTAL_LINES=$(find "$OUTPUT_DIR" -name "*.md" -exec cat {} + | wc -l)
SKILL_LINES=$(wc -l < "$OUTPUT_DIR/SKILL.md")

echo ""
echo "✓ Skill built successfully!"
echo "   SKILL.md: ${SKILL_LINES} lines"
echo "   References: $((TOTAL_FILES - 1)) files"
echo "   Total: ${TOTAL_LINES} lines across ${TOTAL_FILES} files"
echo "   Output: ${OUTPUT_DIR}"
