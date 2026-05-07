# Benchmark Results — 2026-05-07 08:11

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
| Line processing | 157ms | 150ms | 1,05x |
| CSV building | 105ms | 105ms | 1,00x |
| Word counting | 93ms | 96ms | 0,97x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Line processing: 158ms (1000 iterations x 1000 lines)
CSV building: 105ms (1000 iterations x 1000 rows)
Word counting: 97ms (1000 iterations)
Line processing: 155ms (1000 iterations x 1000 lines)
CSV building: 107ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 165ms (1000 iterations x 1000 lines)
CSV building: 106ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
Line processing: 155ms (1000 iterations x 1000 lines)
CSV building: 104ms (1000 iterations x 1000 rows)
Word counting: 92ms (1000 iterations)
Line processing: 157ms (1000 iterations x 1000 lines)
CSV building: 103ms (1000 iterations x 1000 rows)
Word counting: 93ms (1000 iterations)
```

**Rust**
```

Line processing: 149ms (1000 iterations x 1000 lines)
CSV building: 103ms (1000 iterations x 1000 rows)
Word counting: 96ms (1000 iterations)
Line processing: 150ms (1000 iterations x 1000 lines)
CSV building: 105ms (1000 iterations x 1000 rows)
Word counting: 97ms (1000 iterations)
Line processing: 151ms (1000 iterations x 1000 lines)
CSV building: 104ms (1000 iterations x 1000 rows)
Word counting: 94ms (1000 iterations)
Line processing: 150ms (1000 iterations x 1000 lines)
CSV building: 105ms (1000 iterations x 1000 rows)
Word counting: 94ms (1000 iterations)
Line processing: 151ms (1000 iterations x 1000 lines)
CSV building: 107ms (1000 iterations x 1000 rows)
Word counting: 97ms (1000 iterations)
```

</details>

## Benchmark: collections

| Metric | Liva (median) | Rust (median) | Liva/Rust |
|---|---:|---:|---:|
| Array fill+sum | 50ms | 45ms | 1,11x |
| Filter+Map | 28ms | 22ms | 1,27x |
| Map build+lookup | 171ms | 156ms | 1,10x |
| Sort | 64ms | 64ms | 1,00x |

<details><summary>raw output (5 runs each)</summary>

**Liva**
```

Array fill+sum: 50ms (1000 x 50000)
Filter+Map: 28ms (1000 x 50000)
Map build+lookup: 174ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 50ms (1000 x 50000)
Filter+Map: 28ms (1000 x 50000)
Map build+lookup: 171ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 49ms (1000 x 50000)
Filter+Map: 28ms (1000 x 50000)
Map build+lookup: 174ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 49ms (1000 x 50000)
Filter+Map: 27ms (1000 x 50000)
Map build+lookup: 168ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
Array fill+sum: 50ms (1000 x 50000)
Filter+Map: 27ms (1000 x 50000)
Map build+lookup: 167ms (1000 x 1000)
Sort: 63ms (1000 x 50000)
checksums: 139516864 25000000 -798467296 1000
```

**Rust**
```

Array fill+sum: 45ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 156ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 152ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 45ms (1000 x 50000)
Filter+Map: 23ms (1000 x 50000)
Map build+lookup: 156ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 45ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 154ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
checksums: 1249975000000 25000000 3496500000 1000
Array fill+sum: 44ms (1000 x 50000)
Filter+Map: 22ms (1000 x 50000)
Map build+lookup: 161ms (1000 x 1000)
Sort: 64ms (1000 x 50000)
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
Particle sim: 49ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 16ms (5000 x 3000 shapes)
Vec2 ops: 117ms (5000 x 10000 ops)
Particle sim: 50ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 16ms (5000 x 3000 shapes)
Vec2 ops: 116ms (5000 x 10000 ops)
Particle sim: 51ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 16ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 50ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
Shape compute: 16ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 49ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.93524 10064579568.140549 1578125000
```

**Rust**
```

Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 112ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 119ms (5000 x 10000 ops)
Particle sim: 112ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 114ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
Shape compute: 15ms (5000 x 3000 shapes)
Vec2 ops: 115ms (5000 x 10000 ops)
Particle sim: 111ms (5000 x 100 particles x 100 steps)
checksums: 192537385008.8916 10064579568.140549 1578125000
```

</details>

---

## Notes

### Particle sim — defendible (B157 fix validated, 2026-05-07)

After the fix landed in commit `3463ce5` (`_suppressIndexElemClone` flag in
codegen), `arr[i].mutMethod()` no longer emits a redundant `.clone()`. The
generated Rust is now `particles[(pi) as usize].step(0.01);` — calling `step`
on the live mutable slot, so mutation propagates correctly.

Verification:
- Generated Rust audit (`/tmp/pcheck2/target/liva_build/src/main.rs:167`)
  confirms no `.clone()` between index and method.
- Checksums match hand-written Rust (`chk_p = 1578125000` on both sides).
- Resulting ratio **0.45×** (Liva 50 ms vs Rust 111 ms): Liva is genuinely
  faster than the hand-written `iter_mut()` version — likely because the
  index-based access pattern lets LLVM unroll/vectorise the inner step loop
  more aggressively. Both versions do the same arithmetic and produce the
  same checksums.

The ratio **is defensible** — the earlier "no defendible" label is removed.

### Binary size (2026-05-07)

Measured with `./benchmarks/binary_size.sh` on Linux x86-64 (default
`cargo build --release`, no extra strip/LTO flags). Raw = as built.
Stripped = `strip` applied.

| Binary | Raw | Stripped |
|--------|----:|---------:|
| bootstrap (Rust, full compiler + LSP + fmt + lint) | 6.79 MB | 6.79 MB |
| gen-1 (self-host built by bootstrap)               | 2.14 MB | 1.91 MB |
| gen-2 (self-host built by gen-1)                   | 2.03 MB | 1.80 MB |
| gen-3 (self-host built by gen-2)                   | 2.03 MB | 1.80 MB |

- gen-2 and gen-3 are **byte-identical** when stripped — confirms self-host
  idempotency at the binary level (not just at the source level).
- gen-2 is ~3.7× smaller than the bootstrap because the self-host currently
  ships only the codegen pipeline; the bootstrap also includes LSP, fmt,
  lint, hints, suggestions and their dependencies (`tower-lsp`, `tokio`).
