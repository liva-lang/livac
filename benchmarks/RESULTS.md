# Benchmark Results — 2026-04-28 14:23

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
| Line processing | 209ms | 186ms | 1,12x |
| CSV building | 149ms | 127ms | 1,17x |
| Word counting | 152ms | 119ms | 1,28x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 218ms (1000 iterations x 1000 lines)
CSV building: 150ms (1000 iterations x 1000 rows)
Word counting: 152ms (1000 iterations)
Line processing: 204ms (1000 iterations x 1000 lines)
CSV building: 143ms (1000 iterations x 1000 rows)
Word counting: 156ms (1000 iterations)
Line processing: 225ms (1000 iterations x 1000 lines)
CSV building: 146ms (1000 iterations x 1000 rows)
Word counting: 156ms (1000 iterations)
Line processing: 209ms (1000 iterations x 1000 lines)
CSV building: 149ms (1000 iterations x 1000 rows)
Word counting: 149ms (1000 iterations)
Line processing: 209ms (1000 iterations x 1000 lines)
CSV building: 149ms (1000 iterations x 1000 rows)
Word counting: 149ms (1000 iterations)
```

**Rust**
```

Line processing: 186ms (1000 iterations x 1000 lines)
CSV building: 127ms (1000 iterations x 1000 rows)
Word counting: 118ms (1000 iterations)
Line processing: 183ms (1000 iterations x 1000 lines)
CSV building: 136ms (1000 iterations x 1000 rows)
Word counting: 120ms (1000 iterations)
Line processing: 205ms (1000 iterations x 1000 lines)
CSV building: 127ms (1000 iterations x 1000 rows)
Word counting: 125ms (1000 iterations)
Line processing: 190ms (1000 iterations x 1000 lines)
CSV building: 137ms (1000 iterations x 1000 rows)
Word counting: 119ms (1000 iterations)
Line processing: 183ms (1000 iterations x 1000 lines)
CSV building: 127ms (1000 iterations x 1000 rows)
Word counting: 118ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 3ms | 0ms | (rust ≈ 0ms) |
| Filter+Map | 3ms | 2ms | 1,50x |
| Map build+lookup | 207ms | 181ms | 1,14x |
| Sort | 6ms | 2ms | 3,00x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 4ms (1000 x 5000)
Map build+lookup: 194ms (1000 x 1000)
Sort: 5ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 199ms (1000 x 1000)
Sort: 6ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 217ms (1000 x 1000)
Sort: 6ms (1000 x 5000)
Array fill+sum: 4ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 237ms (1000 x 1000)
Sort: 9ms (1000 x 5000)
Array fill+sum: 3ms (1000 x 5000)
Filter+Map: 4ms (1000 x 5000)
Map build+lookup: 207ms (1000 x 1000)
Sort: 6ms (1000 x 5000)
```

**Rust**
```

Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 181ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 186ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 3ms (1000 x 5000)
Map build+lookup: 190ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 174ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
Array fill+sum: 0ms (1000 x 5000)
Filter+Map: 2ms (1000 x 5000)
Map build+lookup: 171ms (1000 x 1000)
Sort: 2ms (1000 x 5000)
```

</details>

## Benchmark: classes

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Shape compute | 0ms | 0ms | (rust ≈ 0ms) |
| Vec2 ops | 0ms | 0ms | (rust ≈ 0ms) |
| Particle sim | 0ms | 5ms | 0,00x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 0ms (1000 x 100 particles x 100 steps)
Shape compute: 1ms (1000 x 3000 shapes)
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
Particle sim: 5ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 5ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 5ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 5ms (1000 x 100 particles x 100 steps)
Shape compute: 0ms (1000 x 3000 shapes)
Vec2 ops: 0ms (1000 x 10000 ops)
Particle sim: 5ms (1000 x 100 particles x 100 steps)
```

</details>
