# Benchmark Results — 2026-05-05 15:38

Liva compiler: `./target/livac-gen2-release` (self-host gen-2 (release))
Each binary executed 5 times; the **median** is reported.

> ⚠️ **Load notice (2026-05-05):** This run was captured with the host
> under sustained load (load avg 18–22 from concurrent test suites and
> editor processes); per-bench variance is several × baseline. Particle
> sim went from 0.44× (vacuous, see B157) to 0.53× **with checksums
> matching hand-written Rust** — the fix is correctness-validated.
> The other ratios (1.5–2.7×) reflect noise, not a regression: the
> previous quiet baseline (2026-05-04) had Line 1.07×, CSV 1.00×,
> Word 0.98×, Map 1.09×. A clean re-run on an idle host is pending.

## Environment
```
Linux PEEPORSOFDEB048 6.12.85+deb13-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.12.85-1 (2026-04-30) x86_64 GNU/Linux
rustc 1.93.1 (01f6ddf75 2026-02-11)
```


## Benchmark: strings

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Line processing | 506ms | 296ms | 1,71x |
| CSV building | 317ms | 200ms | 1,58x |
| Word counting | 301ms | 169ms | 1,78x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 558ms (1000 iterations x 1000 lines)
CSV building: 275ms (1000 iterations x 1000 rows)
Word counting: 196ms (1000 iterations)
Line processing: 457ms (1000 iterations x 1000 lines)
CSV building: 443ms (1000 iterations x 1000 rows)
Word counting: 339ms (1000 iterations)
Line processing: 1000ms (1000 iterations x 1000 lines)
CSV building: 317ms (1000 iterations x 1000 rows)
Word counting: 301ms (1000 iterations)
Line processing: 506ms (1000 iterations x 1000 lines)
CSV building: 369ms (1000 iterations x 1000 rows)
Word counting: 321ms (1000 iterations)
Line processing: 401ms (1000 iterations x 1000 lines)
CSV building: 282ms (1000 iterations x 1000 rows)
Word counting: 192ms (1000 iterations)
```

**Rust**
```

Line processing: 316ms (1000 iterations x 1000 lines)
CSV building: 203ms (1000 iterations x 1000 rows)
Word counting: 177ms (1000 iterations)
Line processing: 399ms (1000 iterations x 1000 lines)
CSV building: 239ms (1000 iterations x 1000 rows)
Word counting: 185ms (1000 iterations)
Line processing: 254ms (1000 iterations x 1000 lines)
CSV building: 189ms (1000 iterations x 1000 rows)
Word counting: 169ms (1000 iterations)
Line processing: 296ms (1000 iterations x 1000 lines)
CSV building: 200ms (1000 iterations x 1000 rows)
Word counting: 136ms (1000 iterations)
Line processing: 212ms (1000 iterations x 1000 lines)
CSV building: 154ms (1000 iterations x 1000 rows)
Word counting: 131ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 231ms | 84ms | 2,75x |
| Filter+Map | 95ms | 47ms | 2,02x |
| Map build+lookup | 641ms | 373ms | 1,72x |
| Sort | 153ms | 124ms | 1,23x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 448ms (1000 x 50000)
Filter+Map: 128ms (1000 x 50000)
Map build+lookup: 1305ms (1000 x 1000)
Sort: 341ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 269ms (1000 x 50000)
Filter+Map: 201ms (1000 x 50000)
Map build+lookup: 641ms (1000 x 1000)
Sort: 152ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 129ms (1000 x 50000)
Filter+Map: 80ms (1000 x 50000)
Map build+lookup: 360ms (1000 x 1000)
Sort: 153ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 231ms (1000 x 50000)
Filter+Map: 95ms (1000 x 50000)
Map build+lookup: 762ms (1000 x 1000)
Sort: 154ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 150ms (1000 x 50000)
Filter+Map: 77ms (1000 x 50000)
Map build+lookup: 478ms (1000 x 1000)
Sort: 137ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
```

**Rust**
```

Array fill+sum: 102ms (1000 x 50000)
Filter+Map: 77ms (1000 x 50000)
Map build+lookup: 442ms (1000 x 1000)
Sort: 145ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 96ms (1000 x 50000)
Filter+Map: 47ms (1000 x 50000)
Map build+lookup: 368ms (1000 x 1000)
Sort: 133ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 78ms (1000 x 50000)
Filter+Map: 48ms (1000 x 50000)
Map build+lookup: 382ms (1000 x 1000)
Sort: 111ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 74ms (1000 x 50000)
Filter+Map: 42ms (1000 x 50000)
Map build+lookup: 322ms (1000 x 1000)
Sort: 124ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 84ms (1000 x 50000)
Filter+Map: 47ms (1000 x 50000)
Map build+lookup: 373ms (1000 x 1000)
Sort: 122ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
```

</details>

## Benchmark: classes

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Shape compute | 73ms | 38ms | 1,92x |
| Vec2 ops | 275ms | 190ms | 1,45x |
| Particle sim | 99ms | 188ms | 0,53x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Shape compute: 82ms (5000 x 3000 shapes)
Vec2 ops: 338ms (5000 x 10000 ops)
Particle sim: 115ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 83ms (5000 x 3000 shapes)
Vec2 ops: 403ms (5000 x 10000 ops)
Particle sim: 151ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 73ms (5000 x 3000 shapes)
Vec2 ops: 275ms (5000 x 10000 ops)
Particle sim: 99ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 50ms (5000 x 3000 shapes)
Vec2 ops: 233ms (5000 x 10000 ops)
Particle sim: 85ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 49ms (5000 x 3000 shapes)
Vec2 ops: 246ms (5000 x 10000 ops)
Particle sim: 84ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
```

**Rust**
```

Shape compute: 68ms (5000 x 3000 shapes)
Vec2 ops: 266ms (5000 x 10000 ops)
Particle sim: 230ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 38ms (5000 x 3000 shapes)
Vec2 ops: 202ms (5000 x 10000 ops)
Particle sim: 189ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 38ms (5000 x 3000 shapes)
Vec2 ops: 184ms (5000 x 10000 ops)
Particle sim: 188ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 30ms (5000 x 3000 shapes)
Vec2 ops: 190ms (5000 x 10000 ops)
Particle sim: 186ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 32ms (5000 x 3000 shapes)
Vec2 ops: 178ms (5000 x 10000 ops)
Particle sim: 169ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
```

</details>
