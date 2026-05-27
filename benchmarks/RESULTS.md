# Benchmark Results — 2026-05-27 16:13

Liva compiler: `./target/livac-gen2-release` (self-host gen-2 (release))
Each binary executed 5 times; the **median** is reported.

## Environment
```
Linux PEEPORSOFDEB048 6.12.88+deb13-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.12.88-1 (2026-05-15) x86_64 GNU/Linux
rustc 1.93.1 (01f6ddf75 2026-02-11)
```


## Benchmark: strings

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Line processing | 167ms | 143ms | 1,17x |
| CSV building | 98ms | 101ms | 0,97x |
| Word counting | 87ms | 91ms | 0,96x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 169ms (1000 iterations x 1000 lines)
CSV building: 98ms (1000 iterations x 1000 rows)
Word counting: 86ms (1000 iterations)
Line processing: 167ms (1000 iterations x 1000 lines)
CSV building: 101ms (1000 iterations x 1000 rows)
Word counting: 90ms (1000 iterations)
Line processing: 165ms (1000 iterations x 1000 lines)
CSV building: 98ms (1000 iterations x 1000 rows)
Word counting: 87ms (1000 iterations)
Line processing: 165ms (1000 iterations x 1000 lines)
CSV building: 97ms (1000 iterations x 1000 rows)
Word counting: 87ms (1000 iterations)
Line processing: 169ms (1000 iterations x 1000 lines)
CSV building: 102ms (1000 iterations x 1000 rows)
Word counting: 90ms (1000 iterations)
```

**Rust**
```

Line processing: 144ms (1000 iterations x 1000 lines)
CSV building: 108ms (1000 iterations x 1000 rows)
Word counting: 91ms (1000 iterations)
Line processing: 143ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 91ms (1000 iterations)
Line processing: 143ms (1000 iterations x 1000 lines)
CSV building: 99ms (1000 iterations x 1000 rows)
Word counting: 90ms (1000 iterations)
Line processing: 143ms (1000 iterations x 1000 lines)
CSV building: 101ms (1000 iterations x 1000 rows)
Word counting: 90ms (1000 iterations)
Line processing: 143ms (1000 iterations x 1000 lines)
CSV building: 102ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 43ms | 43ms | 1,00x |
| Filter+Map | 22ms | 21ms | 1,05x |
| Map build+lookup | 165ms | 148ms | 1,11x |
| Sort | 62ms | 61ms | 1,02x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 169ms (1000 x 1000)
Sort: 62ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 162ms (1000 x 1000)
Sort: 61ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 164ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 168ms (1000 x 1000)
Sort: 62ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 165ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
```

**Rust**
```

Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 148ms (1000 x 1000)
Sort: 61ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 148ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 149ms (1000 x 1000)
Sort: 61ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 153ms (1000 x 1000)
Sort: 61ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 43ms (1000 x 50000)
Filter+Map: 21ms (1000 x 50000)
Map build+lookup: 147ms (1000 x 1000)
Sort: 62ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
```

</details>

## Benchmark: classes

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Shape compute | 15ms | 14ms | 1,07x |
| Vec2 ops | 113ms | 113ms | 1,00x |
| Particle sim | 48ms | 108ms | 0,44x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 119ms (5000 x 10000 ops)
Particle sim: 48ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 118ms (5000 x 10000 ops)
Particle sim: 48ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 113ms (5000 x 10000 ops)
Particle sim: 48ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 112ms (5000 x 10000 ops)
Particle sim: 48ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 112ms (5000 x 10000 ops)
Particle sim: 48ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
```

**Rust**
```

Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 111ms (5000 x 10000 ops)
Particle sim: 108ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 14ms (5000 x 3000 shapes)
Vec2 ops: 112ms (5000 x 10000 ops)
Particle sim: 109ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 108ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 14ms (5000 x 3000 shapes)
Vec2 ops: 118ms (5000 x 10000 ops)
Particle sim: 107ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 14ms (5000 x 3000 shapes)
Vec2 ops: 113ms (5000 x 10000 ops)
Particle sim: 108ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
```

</details>
