# Benchmark Results — 2026-04-28 14:41

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
| Line processing | 152ms | 142ms | 1,07x |
| CSV building | 99ms | 99ms | 1,00x |
| Word counting | 112ms | 91ms | 1,23x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 159ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 114ms (1000 iterations)
Line processing: 151ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 112ms (1000 iterations)
Line processing: 152ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 112ms (1000 iterations)
Line processing: 161ms (1000 iterations x 1000 lines)
CSV building: 105ms (1000 iterations x 1000 rows)
Word counting: 121ms (1000 iterations)
Line processing: 152ms (1000 iterations x 1000 lines)
CSV building: 98ms (1000 iterations x 1000 rows)
Word counting: 111ms (1000 iterations)
```

**Rust**
```

Line processing: 141ms (1000 iterations x 1000 lines)
CSV building: 98ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
Line processing: 141ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 90ms (1000 iterations)
Line processing: 144ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 90ms (1000 iterations)
Line processing: 148ms (1000 iterations x 1000 lines)
CSV building: 100ms (1000 iterations x 1000 rows)
Word counting: 95ms (1000 iterations)
Line processing: 142ms (1000 iterations x 1000 lines)
CSV building: 103ms (1000 iterations x 1000 rows)
Word counting: 91ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 3ms | 0ms | (rust ≈ 0ms) |
| Filter+Map | 3ms | 2ms | 1,50x |
| Map build+lookup | 159ms | 146ms | 1,09x |
| Sort | 5ms | 2ms | 2,50x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 159ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 158ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 161ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 160ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 159ms (1000 x 1000)
Sort: 4ms (1000 x 5000)
```

**Rust**
```

Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 145ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 146ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 145ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 154ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 149ms (1000 x 1000)
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
Shape compute: 0ms (1000 x 3000 shapes)
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
