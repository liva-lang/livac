# Concurrency

> Basic `async`, `par`, `task`, `await` syntax is in SKILL.md. This file covers fire-and-forget rules, parallel array execution policies, data-parallel for loops, and error handling with tasks.

## Keyword Summary

| Keyword | Type | Use Case | Returns |
|---------|------|----------|---------|
| `async` | Asynchronous | I/O-bound | Value (auto-awaited on use) |
| `par` | Parallel | CPU-bound | Value (auto-joined on use) |
| `task` | Handle | Explicit control | Task handle |
| `await` | Wait | Wait for handle | Task result |

## Auto-Await on First Use

`async` and `par` start execution immediately but the result is **lazily awaited/joined when the variable is first used**. This allows overlapping work:

```liva
main() {
    let x = async slowOperation()  // Spawns tokio task NOW, doesn't block
    let y = par heavyCalc()        // Spawns thread NOW, doesn't block

    doOtherWork()                  // Runs concurrently with x and y

    print(x)                       // Awaits here — blocks until slowOperation() completes
    print(y)                       // Joins here — blocks until heavyCalc() completes
}
```

- `async` → `tokio::spawn()` under the hood, `.await` inserted at first use of the variable
- `par` → `std::thread::spawn()`, `.join()` inserted at first use of the variable
- The variable holds the **result value** (not a future/handle) — no manual `.await` needed
- If used multiple times, only the first use triggers the await; subsequent uses see the cached value

## Fire-and-Forget (Auto-Inferred)

When `async` or `par` call is **not assigned to a variable**, it runs as fire-and-forget:

```liva
async logEvent("login")       // Not assigned → fire-and-forget
par backgroundCleanup()       // Not assigned → fire-and-forget

// Multiple fire-and-forget
async logEvent("Event 1")
async logEvent("Event 2")
par backgroundTask1()
```

No special keyword needed — detection is purely by whether the call is assigned.

## Task Error Handling

```liva
// Error binding with task + await
let calcTask = task par processData(-10)
let result, err = await calcTask

if err {
    print($"Task failed: {err}")
} else {
    print($"Task succeeded: {result}")
}

// Error binding with direct async/par
let data, err = async fetchData("https://example.com")
let result, err = par processData(50)
```

## Data-Parallel For Loops

### Parallel For (`for par`)

```liva
let workloads = [1, 2, 3, 4, 5, 6, 7, 8]

for par item in workloads with chunk 2 threads 4 {
    print($"Processing {item}")
}
```

**Policies:**
- `chunk N` — process N items per thread
- `threads N` — maximum N threads
- `ordered` — preserve iteration order

### ParVec / SIMD (`for parvec`)

```liva
let data = [1, 2, 3, 4, 5, 6, 7, 8]

for parvec lane in data with simdWidth 4 ordered {
    print($"Vector lane: {lane}")
}
```

**Policies:**
- `simdWidth N` — SIMD vector width
- `ordered` — preserve order
- `unordered` — allow reordering for performance

## Array Execution Policies

Adapter-style parallel/vectorized processing on collections:

```liva
// Sequential (default)
let doubled = numbers.map(x => x * 2)

// Parallel — multi-threading via Rayon
let doubled = numbers.par().map(x => x * 2)

// Parallel with options
let doubled = numbers.par({threads: 4, chunk: 2}).map(x => heavyCompute(x))

// Vectorized (SIMD planned, sequential fallback)
let doubled = numbers.vec().map(x => x * 2)

// Parallel + Vectorized combined
let doubled = numbers.parvec().map(x => x * 2)
```

### Supported Methods

All array methods support all policies: `map`, `filter`, `reduce`, `forEach`, `find`, `some`, `every`, `indexOf`, `includes`.

Parallel adapters use Rayon's ordered variants (`find_first`, `position_first`) for deterministic results.

```liva
let first = items.par().find(x => x > threshold)   // Leftmost match
let sum = numbers.par().reduce(0, (acc, x) => acc + x)
let idx = numbers.par().indexOf(42)                 // Leftmost position
```

## Async Propagation (Transitive)

```liva
// Base async
fetchData(url: string): string {
    return async httpGet(url)
}

// Auto-async: calls fetchData
processData(url: string): string {
    let data = fetchData(url)       // fetchData is async → processData becomes async
    return data.toUpperCase()
}
```

## Runtime Behavior

- **Async**: Tokio runtime, `tokio::spawn()`, auto-awaited on first variable use
- **Par**: `std::thread::spawn()`, auto-joined on first variable use
- Both are **lazy** — execution starts immediately but result is obtained on first use

### Choosing

| Workload | Use | Scale |
|----------|-----|-------|
| I/O (network, files, DB) | `async` | 100s–1000s concurrent |
| CPU-intensive | `par` | Limited by CPU cores |
