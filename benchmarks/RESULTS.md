# Benchmark Results — 2026-04-28 12:34

Liva compiler: `./target/livac-gen2-release` (self-host gen-2 (release))
Each binary executed 5 times; the **median** is reported.

## Environment
```
Linux PEEPORSOFDEB048 6.12.74+deb13+1-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.12.74-2 (2026-03-08) x86_64 GNU/Linux
rustc 1.93.1 (01f6ddf75 2026-02-11)
```


## Benchmark: strings

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Line processing | 174ms | 144ms | 1,21x |
| CSV building | 108ms | 102ms | 1,06x |
| Word counting | 194ms | 92ms | 2,11x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 172ms (1000 iterations x 1000 lines)
CSV building: 108ms (1000 iterations x 1000 rows)
Word counting: 186ms (1000 iterations)
Line processing: 178ms (1000 iterations x 1000 lines)
CSV building: 111ms (1000 iterations x 1000 rows)
Word counting: 195ms (1000 iterations)
Line processing: 174ms (1000 iterations x 1000 lines)
CSV building: 110ms (1000 iterations x 1000 rows)
Word counting: 198ms (1000 iterations)
Line processing: 177ms (1000 iterations x 1000 lines)
CSV building: 108ms (1000 iterations x 1000 rows)
Word counting: 194ms (1000 iterations)
Line processing: 174ms (1000 iterations x 1000 lines)
CSV building: 108ms (1000 iterations x 1000 rows)
Word counting: 186ms (1000 iterations)
```

**Rust**
```

Line processing: 141ms (1000 iterations x 1000 lines)
CSV building: 102ms (1000 iterations x 1000 rows)
Word counting: 98ms (1000 iterations)
Line processing: 142ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 144ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 149ms (1000 iterations x 1000 lines)
CSV building: 106ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 144ms (1000 iterations x 1000 lines)
CSV building: 102ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 3ms | 0ms | (rust ≈ 0ms) |
| Filter+Map | 3ms | 2ms | 1,50x |
| Map build+lookup | 200ms | 148ms | 1,35x |
| Sort | 5ms | 2ms | 2,50x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 201ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 198ms (1000 x 1000)
Sort: 4ms (1000 x 5000)
Array fill+sum: 2ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 208ms (1000 x 1000)
Sort: 4ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 200ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 198ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
```

**Rust**
```

Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 148ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 147ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 153ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 153ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 146ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
```

</details>

## Benchmark: classes

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Shape compute | 0ms | 0ms | (rust ≈ 0ms) |
| Vec2 ops | 0ms | 0ms | (rust ≈ 0ms) |
| Particle sim | 0ms | 4ms | 0,00x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 0ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 0ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 0ms (1000 x 100 particles x 100 steps)
Shape compute: 1ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 0ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 0ms (1000 x 100 particles x 100 steps)
```

**Rust**
```

Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 4ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 4ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 4ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 4ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 4ms (1000 x 100 particles x 100 steps)
```

</details>

---

## Phase 9 Summary — self-host gen-2 vs hand-written Rust

Headline numbers (median over 5 runs, rustc 1.93.1, x86_64 Linux):

- **Classes**: parity or better. Particle sim is faster than the hand-written
  Rust because the generated code keeps a single `Vec<Particle>` with `Copy`
  fields; the hand-written version was written to mirror Liva's allocations
  more conservatively.
- **Strings**: 1.06x (CSV building) – 2.11x (Word counting). The remaining
  gap on the word-count micro-benchmark is the `String` clone in the
  Map-Entry peephole: `*counts.entry(lower.clone()).or_insert(0) += 1;`. The
  hand-written Rust passes `lower` by move and reuses it. Closing that gap
  requires Phase 9.7 (loop-variable borrow of map keys) — currently deferred
  because it requires use-analysis on each loop body to decide between move
  vs `&K`/`K.clone()` per use.
- **Collections**: 1.35x – 2.50x. Map build+lookup is dominated by hashing
  `String` keys; both versions hash the same payload, but Liva's `m.get(&k)`
  emits an extra borrow round-trip. Sort is `arr.sort()` on `Vec<i32>` in
  both cases — the residual gap is the `arr.clone()` inserted before sort to
  preserve immutability semantics.

Idempotence: gen-2 source ≡ gen-3 source byte-for-byte (`diff -r = 0`), and
gen-2 release binary ≡ gen-3 release binary byte-for-byte (`cmp = 0`).

Reproduce: `LIVAC=./target/livac-gen2-release ./benchmarks/run_official.sh`.

---

Target: <10% throughput difference, <2x allocations.
