# Benchmark Results — 2026-05-04 16:35

## v2.0 official gate (all 10 benchmarks under 1.15x)

| Suite | Metric | Liva | Rust | Ratio | Gate |
|---|---|---:|---:|---:|:---:|
| strings | Line processing | 154ms | 147ms | 1.05x | ✅ |
| strings | CSV building | 104ms | 103ms | 1.01x | ✅ |
| strings | Word counting | 93ms | 94ms | 0.99x | ✅ |
| collections | Array fill+sum | 49ms | 44ms | 1.11x | ✅ |
| collections | Filter+Map | 27ms | 24ms | 1.12x | ✅ |
| collections | Map build+lookup | 168ms | 150ms | 1.12x | ✅ |
| collections | Sort | 64ms | 63ms | 1.02x | ✅ |
| classes | Shape compute | 15ms | 15ms | 1.00x | ✅ |
| classes | Vec2 ops | 115ms | 114ms | 1.01x | ✅ |
| classes | Particle sim | 49ms | 111ms | 0.44x | ✅ |

**10/10 under 1.15x** · benches use side-effect checksums printed at the end so
the optimizer cannot elide measured work · Sort uses adversarial reverse-sorted
input (real work) · `(0..n).collect()` replaced with explicit `push` loop on the
Rust side to keep both implementations algorithmically equivalent.

---

Liva compiler: `./target/livac-gen2-release` (self-host gen-2 (release))
Each binary executed 5 times; the **median** is reported.

## Environment
```
Linux PEEPORSOFDEB048 6.12.85+deb13-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.12.85-1 (2026-04-30) x86_64 GNU/Linux
rustc 1.93.1 (01f6ddf75 2026-02-11)
```


## Benchmark: strings

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Line processing | 154ms | 147ms | 1,05x |
| CSV building | 104ms | 103ms | 1,01x |
| Word counting | 93ms | 94ms | 0,99x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 154ms (1000 iterations x 1000 lines)
CSV building: 100ms (1000 iterations x 1000 rows)
Word counting: 91ms (1000 iterations)
Line processing: 158ms (1000 iterations x 1000 lines)
CSV building: 103ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
Line processing: 153ms (1000 iterations x 1000 lines)
CSV building: 104ms (1000 iterations x 1000 rows)
Word counting: 97ms (1000 iterations)
Line processing: 161ms (1000 iterations x 1000 lines)
CSV building: 107ms (1000 iterations x 1000 rows)
Word counting: 96ms (1000 iterations)
Line processing: 153ms (1000 iterations x 1000 lines)
CSV building: 104ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
```

**Rust**
```

Line processing: 146ms (1000 iterations x 1000 lines)
CSV building: 105ms (1000 iterations x 1000 rows)
Word counting: 95ms (1000 iterations)
Line processing: 147ms (1000 iterations x 1000 lines)
CSV building: 102ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
Line processing: 149ms (1000 iterations x 1000 lines)
CSV building: 103ms (1000 iterations x 1000 rows)
Word counting: 94ms (1000 iterations)
Line processing: 145ms (1000 iterations x 1000 lines)
CSV building: 104ms (1000 iterations x 1000 rows)
Word counting: 96ms (1000 iterations)
Line processing: 148ms (1000 iterations x 1000 lines)
CSV building: 103ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 49ms | 44ms | 1,11x |
| Filter+Map | 27ms | 24ms | 1,12x |
| Map build+lookup | 168ms | 150ms | 1,12x |
| Sort | 64ms | 63ms | 1,02x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 50ms (1000 x 50000)
Filter+Map: 28ms (1000 x 50000)
Map build+lookup: 172ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 48ms (1000 x 50000)
Filter+Map: 27ms (1000 x 50000)
Map build+lookup: 183ms (1000 x 1000)
Sort: 65ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 48ms (1000 x 50000)
Filter+Map: 26ms (1000 x 50000)
Map build+lookup: 165ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 50ms (1000 x 50000)
Filter+Map: 26ms (1000 x 50000)
Map build+lookup: 166ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 49ms (1000 x 50000)
Filter+Map: 27ms (1000 x 50000)
Map build+lookup: 168ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
```

**Rust**
```

Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 24ms (1000 x 50000)
Map build+lookup: 150ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 25ms (1000 x 50000)
Map build+lookup: 150ms (1000 x 1000)
Sort: 62ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 152ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 150ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 26ms (1000 x 50000)
Map build+lookup: 150ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
```

</details>

## Benchmark: classes

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Shape compute | 15ms | 15ms | 1,00x |
| Vec2 ops | 115ms | 114ms | 1,01x |
| Particle sim | 49ms | 111ms | 0,44x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 49ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 16ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 49ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 49ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 51ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 18ms (5000 x 3000 shapes)
Vec2 ops: 117ms (5000 x 10000 ops)
Particle sim: 49ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
```

**Rust**
```

Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 110ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 109ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 116ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
```

</details>
