# Benchmark Results — 2026-05-06 08:45

Liva compiler: `./target/livac-gen2-release` (self-host gen-2 (release))
Each binary executed 5 times; the **median** is reported.

> ✅ **10/10 benchmarks under the 1.15× release gate** (idle host).
> Particle sim 0.45× ≪ 1.0× with checksums matching hand-written Rust —
> B157 fix validated end-to-end. See `BUGS.md § B157`.

## Environment
```
Linux PEEPORSOFDEB048 6.12.85+deb13-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.12.85-1 (2026-04-30) x86_64 GNU/Linux
rustc 1.93.1 (01f6ddf75 2026-02-11)
```


## Benchmark: strings

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Line processing | 153ms | 149ms | 1,03x |
| CSV building | 102ms | 110ms | 0,93x |
| Word counting | 92ms | 96ms | 0,96x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 158ms (1000 iterations x 1000 lines)
CSV building: 102ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 183ms (1000 iterations x 1000 lines)
CSV building: 110ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 148ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 89ms (1000 iterations)
Line processing: 153ms (1000 iterations x 1000 lines)
CSV building: 102ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 149ms (1000 iterations x 1000 lines)
CSV building: 98ms (1000 iterations x 1000 rows)
Word counting: 90ms (1000 iterations)
```

**Rust**
```

Line processing: 144ms (1000 iterations x 1000 lines)
CSV building: 101ms (1000 iterations x 1000 rows)
Word counting: 94ms (1000 iterations)
Line processing: 143ms (1000 iterations x 1000 lines)
CSV building: 101ms (1000 iterations x 1000 rows)
Word counting: 96ms (1000 iterations)
Line processing: 204ms (1000 iterations x 1000 lines)
CSV building: 167ms (1000 iterations x 1000 rows)
Word counting: 151ms (1000 iterations)
Line processing: 163ms (1000 iterations x 1000 lines)
CSV building: 110ms (1000 iterations x 1000 rows)
Word counting: 103ms (1000 iterations)
Line processing: 149ms (1000 iterations x 1000 lines)
CSV building: 114ms (1000 iterations x 1000 rows)
Word counting: 94ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 49ms | 44ms | 1,11x |
| Filter+Map | 26ms | 23ms | 1,13x |
| Map build+lookup | 172ms | 157ms | 1,10x |
| Sort | 63ms | 63ms | 1,00x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 49ms (1000 x 50000)
Filter+Map: 27ms (1000 x 50000)
Map build+lookup: 174ms (1000 x 1000)
Sort: 65ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 48ms (1000 x 50000)
Filter+Map: 27ms (1000 x 50000)
Map build+lookup: 172ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 49ms (1000 x 50000)
Filter+Map: 26ms (1000 x 50000)
Map build+lookup: 167ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 52ms (1000 x 50000)
Filter+Map: 26ms (1000 x 50000)
Map build+lookup: 168ms (1000 x 1000)
Sort: 62ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 48ms (1000 x 50000)
Filter+Map: 26ms (1000 x 50000)
Map build+lookup: 202ms (1000 x 1000)
Sort: 69ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
```

**Rust**
```

Array fill+sum: 51ms (1000 x 50000)
Filter+Map: 29ms (1000 x 50000)
Map build+lookup: 180ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 25ms (1000 x 50000)
Map build+lookup: 155ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 45ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 157ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 23ms (1000 x 50000)
Map build+lookup: 152ms (1000 x 1000)
Sort: 62ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 159ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
```

</details>

## Benchmark: classes

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Shape compute | 16ms | 15ms | 1,07x |
| Vec2 ops | 115ms | 115ms | 1,00x |
| Particle sim | 50ms | 111ms | 0,45x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Shape compute: 16ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 50ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 50ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 17ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 49ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 116ms (5000 x 10000 ops)
Particle sim: 52ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 17ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 50ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
```

**Rust**
```

Shape compute: 16ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 110ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 110ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
```

</details>
