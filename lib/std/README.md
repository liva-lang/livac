# `lib/std/` — Liva Standard Library (Liva-side)

> **Status:** scaffold (v2.0). First reusable Liva modules.

## Purpose

This is the **Liva-side stdlib**, written entirely in `.liva` and
compiled by every program that imports it. It complements the **FFI
stdlib** (built into the compiler — `String`, `Math`, `File`, `HTTP`,
`DB`, …) by providing reusable, composable helpers that don't need to
go through Rust.

Modules here are:

- 100 % `.liva` source — readable and patchable.
- Self-hosted: gen-2 compiles them just like user code.
- Imported with relative or `lib/std/` paths (TBD on package layout).

## Modules

| Module | Purpose |
|--------|---------|
| `validators.liva` | Common predicates: `isEmail`, `isUrl`, `isNumeric`, `isBlank` |

## Status & Roadmap

This is intentionally minimal for v2.0. Tier C8 of Phase 11 only
required a working scaffold + 1 reusable module; further modules
(parsers, functional helpers, format, collections-extra) are tracked
under v2.1 in `BACKLOG.md`.

## How to import (today)

Until package resolution lands, copy the file you need into your
project, or use a relative import:

```liva
import { isEmail, isBlank } from "../../lib/std/validators"

main() {
    print(isEmail("foo@bar.com").toString())  // true
    print(isBlank("   ").toString())          // true
}
```
