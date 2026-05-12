#!/usr/bin/env python3
"""Codemod: collapse `let _ = switch X { ...; 0 }, _ => { 0 } }` patterns
into bare statement-position switches now that Cycle 28 made `switch` usable
as a statement directly.

Strategy:
- Find each `let _ = switch ` token and rewrite the immediate prefix.
- Locate the opening `{` after the discriminant and walk forward, tracking
  brace depth, until the matching `}`.
- Inside that range:
  * `=> { 0 },`           -> `=> {},`           (empty dummy arm)
  * `=> { 0 }`            -> `=> {}`            (empty dummy arm, last)
  * `; 0 },`              -> ` },`              (single-line arm trailing `; 0`)
  * `; 0 }`               -> ` }`               (single-line, last)
  * Lone line `^\s*0$` immediately before `}` / `},` -> drop the line.

We do NOT touch nested `switch` expressions that are NOT preceded by `let _ =`
— those are real value-producing matches and the `0` may be load-bearing.
"""
from __future__ import annotations
import re
import sys
from pathlib import Path


def find_matching_brace(text: str, open_idx: int) -> int:
    """Given index of `{`, return index of its matching `}`.
    Handles Liva `"..."` strings and `//` line comments."""
    assert text[open_idx] == "{"
    depth = 0
    i = open_idx
    in_str = False
    in_comment = False
    while i < len(text):
        ch = text[i]
        if in_comment:
            if ch == "\n":
                in_comment = False
            i += 1
            continue
        if in_str:
            if ch == "\\" and i + 1 < len(text):
                i += 2
                continue
            if ch == '"':
                in_str = False
            i += 1
            continue
        if ch == "/" and i + 1 < len(text) and text[i + 1] == "/":
            in_comment = True
            i += 2
            continue
        if ch == '"':
            in_str = True
            i += 1
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError("no matching brace")


# Match `let _ = switch ` (with optional leading whitespace handled by caller).
LET_SWITCH = re.compile(r"\blet\s+_\s*=\s*switch\b")


def rewrite_block(body: str) -> str:
    """Rewrite the inside of a switch block to drop `; 0` and `0` fillers."""
    # Single-line arms with trailing `; 0 }` (followed by `,` or end-of-block).
    body = re.sub(r";\s*0\s*\}\s*,", " },", body)
    body = re.sub(r";\s*0\s*\}(\s*[\)\n])", r" }\1", body)
    # Empty dummy arms `{ 0 }`.
    body = re.sub(r"=>\s*\{\s*0\s*\}\s*,", "=> {},", body)
    body = re.sub(r"=>\s*\{\s*0\s*\}", "=> {}", body)
    # Lone `0` line preceding `}` or `},`.
    body = re.sub(r"\n(\s*)0\s*\n(\s*\})", r"\n\2", body)
    return body


def process_file(path: Path) -> tuple[int, str]:
    src = path.read_text()
    total = 0
    # Iterate until fixpoint — handles nested `let _ = switch` patterns.
    while True:
        out_parts: list[str] = []
        i = 0
        n_rewrites = 0
        while i < len(src):
            m = LET_SWITCH.search(src, i)
            if not m:
                out_parts.append(src[i:])
                break
            out_parts.append(src[i:m.start()])
            # Find the `switch` keyword start, then walk to its `{`.
            brace_idx = src.find("{", m.end())
            if brace_idx < 0:
                out_parts.append(src[m.start():])
                break
            try:
                close_idx = find_matching_brace(src, brace_idx)
            except ValueError:
                out_parts.append(src[m.start():])
                break
            switch_decl = "switch" + src[m.end():brace_idx + 1]
            block_inner = src[brace_idx + 1:close_idx]
            block_inner_new = rewrite_block(block_inner)
            out_parts.append(switch_decl + block_inner_new + "}")
            n_rewrites += 1
            i = close_idx + 1
        new_src = "".join(out_parts)
        if n_rewrites == 0:
            break
        total += n_rewrites
        src = new_src
    return total, src


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: codemod_switch.py <file>...", file=sys.stderr)
        return 2
    total = 0
    for arg in sys.argv[1:]:
        p = Path(arg)
        n, new_src = process_file(p)
        if n:
            p.write_text(new_src)
            print(f"{arg}: {n} rewrites")
            total += n
        else:
            print(f"{arg}: 0 rewrites")
    print(f"total: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
