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

# Clean previous build
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/references"

# 1. Copy SKILL.md
cp "$PROJECT_ROOT/skills/liva-lang/SKILL.md" "$OUTPUT_DIR/SKILL.md"

# 2. Copy all language-reference docs as references/ (preserving subdirectories)
LANG_REF="$PROJECT_ROOT/docs/language-reference"
REFS="$OUTPUT_DIR/references"

if [ -d "$LANG_REF" ]; then
    find "$LANG_REF" -name "*.md" -type f | while read -r file; do
        rel="${file#$LANG_REF/}"
        target_dir="$REFS/$(dirname "$rel")"
        mkdir -p "$target_dir"
        cp "$file" "$REFS/$rel"
    done
else
    echo "   ⚠ Missing: docs/language-reference/"
fi

# 3. Copy guides
GUIDES_SRC="$PROJECT_ROOT/docs/guides"
GUIDES_DST="$REFS/guides"

if [ -d "$GUIDES_SRC" ]; then
    mkdir -p "$GUIDES_DST"
    find "$GUIDES_SRC" -name "*.md" -type f | while read -r file; do
        cp "$file" "$GUIDES_DST/$(basename "$file")"
    done
else
    echo "   ⚠ Missing: docs/guides/"
fi

# 4. Copy getting-started
GETTING_STARTED_SRC="$PROJECT_ROOT/docs/getting-started"
GETTING_STARTED_DST="$REFS/getting-started"

if [ -d "$GETTING_STARTED_SRC" ]; then
    mkdir -p "$GETTING_STARTED_DST"
    find "$GETTING_STARTED_SRC" -name "*.md" -type f | while read -r file; do
        cp "$file" "$GETTING_STARTED_DST/$(basename "$file")"
    done
else
    echo "   ⚠ Missing: docs/getting-started/"
fi

# 5. Copy top-level doc files
for file in QUICK_REFERENCE.md PROJECT_STRUCTURE.md ERROR_CODES.md \
            ERROR_HANDLING_GUIDE.md TROUBLESHOOTING.md README.md; do
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
