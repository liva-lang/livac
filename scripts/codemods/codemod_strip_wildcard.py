#!/usr/bin/env python3
"""Cycle 34: strip redundant `_ => {}` arms from stmt-position switches.

Heuristic:
- Find lines that are exactly `<indent>_ => {}` (optionally followed by `,`).
- If the previous non-blank line ends with a trailing `,`, drop the wildcard
  line and remove that trailing comma (the wildcard was the last arm).
- If the previous non-blank line does NOT end with `,` (so the wildcard is
  the only arm), leave the file alone — codegen still needs *some* arm.

After codemod we rebuild + run the full gauntlet. If anything regresses we
revert.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

WILDCARD = re.compile(r"^(\s*)_\s*=>\s*\{\s*\},?\s*$")


def process(path: Path) -> int:
    lines = path.read_text().splitlines()
    out: list[str] = []
    removed = 0
    i = 0
    while i < len(lines):
        line = lines[i]
        m = WILDCARD.match(line)
        if not m:
            out.append(line)
            i += 1
            continue
        # Find previous non-blank line in `out`.
        j = len(out) - 1
        while j >= 0 and out[j].strip() == "":
            j -= 1
        if j < 0:
            out.append(line)
            i += 1
            continue
        prev = out[j].rstrip()
        if prev.endswith(","):
            out[j] = prev[:-1] + ("\n" if False else "")
            # preserve original trailing whitespace? simplest: replace.
            out[j] = prev[:-1]
            removed += 1
            i += 1
            continue
        out.append(line)
        i += 1
    if removed:
        path.write_text("\n".join(out) + "\n")
    return removed


def main() -> int:
    root = Path("compiler/src")
    total = 0
    for p in sorted(root.glob("*.liva")):
        n = process(p)
        if n:
            print(f"  {p}: removed {n}")
            total += n
    print(f"Total removed: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
