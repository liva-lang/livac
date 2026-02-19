# ⚡ Concurrency in Liva

Liva provides a **hybrid concurrency model** that combines both **asynchronous** (I/O-bound) and **parallel** (CPU-bound) execution primitives.

## Overview

Liva offers four main concurrency keywords:

| Keyword | Type | Use Case | Blocks | Returns |
|---------|------|----------|--------|---------|
| `async` | Asynchronous | I/O-bound tasks | No (lazy) | Value directly |
| `par` | Parallel | CPU-bound tasks | No (lazy) | Value directly |
| `task` | Handle | Need explicit control | No | Task handle |
| `fire` | Fire-and-forget | Background work | No | Nothing |

## Async - Asynchronous Execution

Use `async` for **I/O-bound operations** like network requests, file I/O, database queries.

### Basic Async

```liva
fetchUser(id: number): string {
  // Simulated network call
  return $"User {id} data"
}

main() {
  // Runs asynchronously, auto-awaited on first use
  let user = async fetchUser(123)
  print($"Got: {user}")  // Awaits here
}
```

### Multiple Async Calls

```liva
main() {
  // Start all async operations
  let user1 = async fetchUser(1)
  let user2 = async fetchUser(2)
  let user3 = async fetchUser(3)
  
  // All run concurrently, await happens on use
  print($"Users: {user1}, {user2}, {user3}")
}
```

### Async with Error Handling

```liva
fetchData(url: string): string {
  if url == "" fail "Empty URL"
  return $"Data from {url}"
}

main() {
  let data, err = async fetchData("https://api.example.com")
  
  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Success: {data}")
  }
}
```

## Par - Parallel Execution

Use `par` for **CPU-bound computations** that benefit from multi-threading.

### Basic Parallel

```liva
heavyComputation(n: number): number {
  // CPU-intensive work
  let result = n * n
  return result
}

main() {
  // Runs in parallel thread, auto-joined on first use
  let result = par heavyComputation(1000)
  print($"Result: {result}")  // Joins here
}
```

### Multiple Parallel Tasks

```liva
main() {
  // Start all parallel tasks
  let calc1 = par heavyComputation(100)
  let calc2 = par heavyComputation(200)
  let calc3 = par heavyComputation(300)
  
  // All run in parallel threads, join happens on use
  print($"Results: {calc1}, {calc2}, {calc3}")
}
```

### Parallel with Error Handling

```liva
processData(data: number): number {
  if data < 0 fail "Negative data"
  return data * 2
}

main() {
  let result, err = par processData(50)
  
  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Processed: {result}")
  }
}
```

## Task - Explicit Task Handles

Use `task` when you need **explicit control** over when to await/join.

### Async Task

```liva
main() {
  // Create task handle
  let userTask = task async fetchUser(123)
  
  // Do other work here...
  print("Doing other work...")
  
  // Explicitly await
  let user = await userTask
  print($"User: {user}")
}
```

### Parallel Task

```liva
main() {
  // Create task handle
  let calcTask = task par heavyComputation(500)
  
  // Do other work here...
  print("Doing other work...")
  
  // Explicitly join
  let result = await calcTask
  print($"Result: {result}")
}
```

### Multiple Tasks

```liva
main() {
  let task1 = task async fetchUser(1)
  let task2 = task async fetchUser(2)
  let task3 = task par heavyComputation(100)
  
  print("All tasks started")
  
  // Await all explicitly
  let user1 = await task1
  let user2 = await task2
  let result = await task3
  
  print($"Done: {user1}, {user2}, {result}")
}
```

### Task with Error Handling

```liva
main() {
  let calcTask = task par processData(-10)
  
  // Do other work...
  
  let result, err = await calcTask
  
  if err != "" {
    print($"Task failed: {err}")
  } else {
    print($"Task succeeded: {result}")
  }
}
```

## Fire - Fire and Forget

Use `fire` for **background operations** where you don't care about the result.

### Async Fire and Forget

```liva
logEvent(message: string) {
  print($"[LOG] {message}")
}

main() {
  // Starts async, result ignored
  fire async logEvent("Application started")
  
  // Continue immediately
  print("Main continues...")
}
```

### Parallel Fire and Forget

```liva
backgroundCleanup() {
  print("🧹 Running cleanup...")
}

main() {
  // Starts in parallel, result ignored
  fire par backgroundCleanup()
  
  // Continue immediately
  print("Main continues...")
}
```

### Multiple Fire Calls

```liva
main() {
  fire async logEvent("Event 1")
  fire async logEvent("Event 2")
  fire par backgroundTask1()
  fire par backgroundTask2()
  
  print("All background tasks started")
}
```

## Hybrid Concurrency

Combine `async` and `par` for optimal performance:

```liva
// I/O-bound: Use async
fetchFromAPI(endpoint: string): string {
  return $"API data from {endpoint}"
}

// CPU-bound: Use par
processData(data: string): string {
  // Heavy computation
  return $"Processed: {data}"
}

main() {
  // Step 1: Fetch data asynchronously
  let rawData = async fetchFromAPI("/users")
  
  // Step 2: Process data in parallel
  let processed = par processData(rawData)
  
  print($"Final result: {processed}")
}
```

## Data-Parallel For Loops

Liva supports **data-parallel for loops** for processing collections:

### Parallel For

```liva
main() {
  let workloads = [1, 2, 3, 4, 5, 6, 7, 8]
  
  // Process items in parallel threads
  for par item in workloads with chunk 2 threads 4 {
    print($"Processing {item} in parallel")
  }
}
```

**Policies:**
- `chunk N` - Process N items per thread
- `threads N` - Use N threads maximum
- `ordered` - Preserve iteration order

### ParVec (SIMD)

```liva
main() {
  let data = [1, 2, 3, 4, 5, 6, 7, 8]
  
  // SIMD vectorization
  for parvec lane in data with simdWidth 4 ordered {
    print($"Vector lane: {lane}")
  }
}
```

**Policies:**
- `simdWidth N` - SIMD vector width
- `ordered` - Preserve iteration order
- `unordered` - Allow reordering for performance

## Array Execution Policies

Liva array methods support **adapter-style execution policies** for parallel and vectorized processing. These complement the `par`/`async` keywords by providing data-parallel operations directly on collections.

### Adapters

```liva
// Sequential (default)
let doubled = numbers.map(x => x * 2)

// Parallel - multi-threading via Rayon
let doubled = numbers.par().map(x => x * 2)

// Parallel with options
let doubled = numbers.par({threads: 4, chunk: 2}).map(x => heavyCompute(x))

// Vectorized (SIMD planned, sequential fallback)
let doubled = numbers.vec().map(x => x * 2)

// Parallel + Vectorized combined
let doubled = numbers.parvec().map(x => x * 2)
```

### Supported Methods

All array methods (`map`, `filter`, `reduce`, `forEach`, `find`, `some`, `every`, `indexOf`, `includes`) support all execution policies. Parallel adapters use Rayon's ordered variants (`find_first`, `position_first`) to guarantee deterministic results.

```liva
// Parallel find: finds the leftmost match
let first = items.par().find(x => x > threshold)

// Parallel reduce: requires associative operation
let sum = numbers.par().reduce(0, (acc, x) => acc + x)
let product = numbers.par().reduce((acc, x) => acc * x, 1)

// Parallel indexOf: finds leftmost position
let idx = numbers.par().indexOf(42)
```

> For full API reference, adapter options, and the support matrix, see **[Array Methods](stdlib/arrays.md)**.

## Auto-Async Inference

Functions automatically become `async` if they contain `async` calls:

```liva
// This function is automatically async
fetchAndProcess(id: number): string {
  let data = async fetchData(id)  // Contains async call
  return $"Processed: {data}"
}

main() {
  // Must use async when calling
  let result = async fetchAndProcess(123)
  print($"Result: {result}")
}
```

## Best Practices

### 1. Choose the Right Primitive

```liva
// ✅ Good: async for I/O
let user = async fetchFromDatabase(id)

// ✅ Good: par for CPU
let result = par complexCalculation(data)

// ❌ Bad: par for I/O (wastes threads)
let user = par fetchFromDatabase(id)

// ❌ Bad: async for CPU (doesn't utilize cores)
let result = async complexCalculation(data)
```

### 2. Use Task for Complex Orchestration

```liva
main() {
  // Start tasks early
  let dbTask = task async queryDatabase()
  let apiTask = task async fetchFromAPI()
  let calcTask = task par heavyComputation()
  
  // Do other work...
  prepareOutput()
  
  // Await only when needed
  let dbResult = await dbTask
  let apiResult = await apiTask
  let calcResult = await calcTask
  
  combine(dbResult, apiResult, calcResult)
}
```

### 3. Fire and Forget for Side Effects

```liva
main() {
  // Don't wait for logging
  fire async sendAnalytics(event)
  
  // Don't wait for cleanup
  fire par cleanupTempFiles()
  
  // Continue with main flow
  processUserRequest()
}
```

### 4. Error Handling with Concurrency

```liva
main() {
  // Always handle errors for critical operations
  let data, err = async fetchCriticalData()
  
  if err != "" {
    print($"Critical error: {err}")
    return
  }
  
  // Fire and forget can ignore errors
  fire async sendOptionalNotification()
  
  processData(data)
}
```

## Runtime Behavior

### Async Runtime (Tokio)

When you use `async`, Liva automatically:
1. Adds Tokio dependency to generated Cargo.toml
2. Creates `liva_rt::run_async()` helper
3. Wraps calls in `tokio::spawn()`
4. Auto-awaits on first variable use

### Parallel Runtime (Threads)

When you use `par`, Liva automatically:
1. Creates `liva_rt::run_parallel()` helper
2. Wraps calls in `std::thread::spawn()`
3. Auto-joins on first variable use

### Lazy Evaluation

Both `async` and `par` are **lazy**:

```liva
main() {
  let x = async slowOperation()  // Started but not awaited yet
  
  // Do work here...
  doOtherWork()
  
  print(x)  // Awaits here on first use
}
```

## Performance Considerations

### Async Overhead

- **Lightweight**: Minimal overhead for I/O
- **Many tasks**: Can handle thousands of concurrent async operations
- **Scheduler**: Uses Tokio's efficient work-stealing scheduler

### Parallel Overhead

- **Thread creation**: More expensive than async tasks
- **CPU cores**: Limited by available CPU cores
- **Use for**: Heavy computations that justify thread cost

### Choosing Concurrency Level

```liva
main() {
  // CPU-bound: Use par, limited by cores (e.g., 8)
  let tasks = [1, 2, 3, 4, 5, 6, 7, 8]
  for item in tasks {
    let result = par heavyCalc(item)  // Max ~8 parallel
  }
  
  // I/O-bound: Use async, can handle thousands
  let urls = [/* hundreds of URLs */]
  for url in urls {
    let data = async fetch(url)  // Can handle 100s-1000s
  }
}
```

## Advanced Patterns

### Pipeline Pattern

```liva
main() {
  // Stage 1: Fetch (async)
  let rawData = async fetchData()
  
  // Stage 2: Transform (par)
  let transformed = par transformData(rawData)
  
  // Stage 3: Save (async)
  let saved = async saveToDatabase(transformed)
  
  print($"Pipeline complete: {saved}")
}
```

### Fan-Out Fan-In

```liva
main() {
  let ids = [1, 2, 3, 4, 5]
  
  // Fan out: Start all async operations
  let tasks = []
  for id in ids {
    tasks.push(task async fetchUser(id))
  }
  
  // Fan in: Wait for all results
  let users = []
  for t in tasks {
    users.push(await t)
  }
  
  print($"Fetched {users.length} users")
}
```

### Circuit Breaker Pattern

```liva
fetchWithRetry(url: string, maxRetries: number): string {
  for i in 0..maxRetries {
    let data, err = async fetch(url)
    
    if err == "" {
      return data
    }
    
    print($"Retry {i + 1}/{maxRetries}")
  }
  
  fail "Max retries exceeded"
}

main() {
  let data, err = fetchWithRetry("https://api.example.com", 3)
  
  if err != "" {
    print($"Failed: {err}")
  } else {
    print($"Success: {data}")
  }
}
```

## See Also

- **[Array Methods](stdlib/arrays.md)** - Execution policies for array methods (`.par()`, `.vec()`, `.parvec()`)
- **[Error Handling](error-handling.md)** - Combining concurrency with fallibility
- **[Async Programming Guide](../guides/async-programming.md)** - Deep dive into async
- **[Parallel Computing Guide](../guides/parallel-computing.md)** - Deep dive into parallel
- **[Hybrid Concurrency Guide](../guides/hybrid-concurrency.md)** - Best practices

---

**Next:** [Error Handling](error-handling.md)
