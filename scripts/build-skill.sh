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
mkdir -p "$OUTPUT_DIR/references/stdlib"

# 1. Copy SKILL.md
cp "$PROJECT_ROOT/skills/liva-lang/SKILL.md" "$OUTPUT_DIR/SKILL.md"

# 2. Copy language-reference docs as references/
LANG_REF="$PROJECT_ROOT/docs/language-reference"
REFS="$OUTPUT_DIR/references"

# Core language files
for file in \
    variables.md \
    types-primitives.md \
    operators.md \
    functions-basics.md \
    functions-advanced.md \
    control-flow.md \
    pattern-matching.md \
    classes-basics.md \
    classes-data.md \
    classes-interfaces.md \
    enums.md \
    visibility.md \
    error-handling.md \
    collections.md \
    concurrency.md \
    modules.md \
    string-templates.md \
    ; do
    if [ -f "$LANG_REF/$file" ]; then
        cp "$LANG_REF/$file" "$REFS/$file"
    else
        echo "   ⚠ Missing: docs/language-reference/$file"
    fi
done

# 3. Copy stdlib docs
STDLIB_SRC="$LANG_REF/stdlib"
STDLIB_DST="$REFS/stdlib"

for file in arrays.md strings.md io.md math.md conversions.md system.md; do
    if [ -f "$STDLIB_SRC/$file" ]; then
        cp "$STDLIB_SRC/$file" "$STDLIB_DST/$file"
    else
        echo "   ⚠ Missing: docs/language-reference/stdlib/$file"
    fi
done

# 4. Copy guides
cp "$PROJECT_ROOT/docs/QUICK_REFERENCE.md" "$REFS/quick-reference.md"
cp "$PROJECT_ROOT/docs/PROJECT_STRUCTURE.md" "$REFS/project-structure.md"

# 5. Count and report
TOTAL_FILES=$(find "$OUTPUT_DIR" -name "*.md" | wc -l)
TOTAL_LINES=$(find "$OUTPUT_DIR" -name "*.md" -exec cat {} + | wc -l)
SKILL_LINES=$(wc -l < "$OUTPUT_DIR/SKILL.md")

echo ""
echo "✓ Skill built successfully!"
echo "   SKILL.md: ${SKILL_LINES} lines"
echo "   References: $((TOTAL_FILES - 1)) files"
echo "   Total: ${TOTAL_LINES} lines across ${TOTAL_FILES} files"
echo "   Output: ${OUTPUT_DIR}"
