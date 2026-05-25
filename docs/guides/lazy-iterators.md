# Lazy Iterator Chains (v2.3+)

Liva's array adapter methods (`filter`, `map`, `flatMap`, `take`, `drop`)
**fuse into a single lazy pipeline** when they appear consecutively.
That means no intermediate `Vec` allocations, and the work stops as soon
as the downstream consumer is satisfied.

This guide explains what gets fused, what doesn't, and how to read the
generated Rust to verify.

---

## Quick rules

1. **A chain is fused** when `filter`, `map`, `flatMap`, `take`, `drop`
   appear back-to-back on an array (or on the result of another adapter).
2. **The materialization point** is the final non-adapter call:
   `collect`, `forEach`, `reduce`, indexing, a `for` loop, etc.
3. **Standalone** `xs.take(n)` / `xs.drop(n)` (no upstream adapter)
   keep their eager slice semantics for backwards compatibility.
4. **Order matters**: pruning adapters (`filter`, `take`) before
   transforming ones (`map`) save work.

---

## What fuses

```liva
let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Single fused pipeline — one .collect at the end
let result = nums
    .filter(n => n % 2 == 0)
    .map(n => n * n)
    .take(2)
```

Generated Rust (approximately):

```rust
nums.iter().copied()
    .filter(|n| n % 2 == 0)
    .map(|n| n * n)
    .take(2usize)
    .collect::<Vec<_>>()
```

Two key wins:

- **No intermediate `Vec`s** between adapters.
- **`take(2)` short-circuits**: once the second element is produced the
  iterator stops, so the `map` closure runs at most 2 ×–4 × times rather
  than 5 (number of evens in the input).

### Patterns covered by the regression probe

| Chain                              | Behavior                                       |
| ---------------------------------- | ---------------------------------------------- |
| `filter().map().take(n)`           | Stops after `n` matches.                       |
| `filter().take(n)`                 | Stops after `n` matches.                       |
| `drop(k).map().take(n)`            | Skips `k`, emits next `n`.                     |
| `take(n).drop(k)`                  | Same as `[k..n]` semantically, fully lazy.     |

---

## What does **not** fuse (yet)

These calls force materialization right where they appear:

- **Terminal adapters**: `reduce`, `sum`, `min`, `max`, `count`,
  `forEach`, `every`, `some`, `find`.
- **Index access**: `arr.filter(...).at(0)` — the `at(0)` materializes
  the filter into a `Vec` first.
- **Two-pass methods**: `sort`, `reversed`, `groupBy`, `unique`.
- **Mixed parallel/sequential**: as soon as `.par()` enters the chain,
  the fused-sequential path stops; parallel pipelines use Rayon, which
  has its own (different) fusion rules.

If you see an intermediate `Vec` that you didn't expect, look for one
of the operators above as the boundary.

---

## Reading the generated Rust

To inspect what Liva actually emits, compile with `LIVA_DEBUG=1`:

```bash
$ LIVA_DEBUG=1 livac build pipeline.liva
[codegen] emitting iter chain: filter→map→take
```

Or open the generated crate directly:

```bash
$ livac build --emit-rust pipeline.liva
$ cat target/build/src/main.rs   # search for your function name
```

A correctly fused chain shows exactly one `.iter()` / `.iter().copied()`
followed by adapter calls and a single terminal (`collect`, `for_each`,
`fold`, …).

---

## Pitfalls

### 1. Binding kills fusion

```liva
let evens = nums.filter(n => n % 2 == 0)   // materializes here
let squared = evens.map(n => n * n)        // starts a new chain
```

The intermediate `let` materializes a `Vec`. Inline the chain if you
care about the allocation:

```liva
let squared = nums.filter(n => n % 2 == 0).map(n => n * n)
```

### 2. Reusing a result is fine

If you actually need the intermediate value twice, **keep the `let`** —
running the chain twice would be slower than one allocation.

### 3. `take` before `filter` is rarely what you want

```liva
nums.take(3).filter(n => n > 5)     // takes first 3, then filters → maybe 0
nums.filter(n => n > 5).take(3)     // filters, then takes first 3 matches
```

Both fuse, but the semantics differ. Prefer the second form for
"first N matching".

---

## Performance notes

The compiler's own bench gate (`compiler/tests/bench/`) covers four
representative pipelines; expect roughly 1.5×–3× speedups on long
chains versus a pre-v2.3 build, with the largest wins on
`filter→map→take(small_n)` patterns where short-circuiting kicks in
early.

There is no per-build heuristic to disable fusion; if a chain
behaves unexpectedly, file a bug with a reproducer.

---

## Related

- [`cli-tools.md`](./cli-tools.md) — `livac bench` for measuring impact
- [`../language-reference/collections.md`](../language-reference/collections.md) — array method reference
- [`../CHANGELOG.md`](../../CHANGELOG.md) — release entry: "Lazy iterator chain fusion (`take`/`drop`)"
