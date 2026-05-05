# Compile-speed benchmark

- Date: 2026-05-05T12:26:19Z
- Compiler: gen-2 (2058744 bytes)
- Runs per program: 5
- Mode: `check` (front-end) and `build --release` (full pipeline + rustc)


## bootstrap_apps (check)

| Program | LOC | min | median | max |
|---|---:|---:|---:|---:|
| app10_stats | 82 | 3ms | **3ms** | 4ms |
| app11_words | 31 | 2ms | **3ms** | 3ms |
| app12_tree | 51 | 2ms | **3ms** | 3ms |
| app13_calc | 36 | 2ms | **3ms** | 3ms |
| app14_setops | 26 | 2ms | **3ms** | 3ms |
| app15_library | 68 | 3ms | **3ms** | 3ms |
| app16_fsm | 92 | 3ms | **3ms** | 3ms |
| app17_pipeline | 67 | 3ms | **3ms** | 3ms |
| app18_template | 61 | 3ms | **3ms** | 3ms |
| app19_pq | 104 | 3ms | **4ms** | 5ms |
| app20_shapes | 102 | 3ms | **3ms** | 4ms |
| app21_hashmap | 159 | 4ms | **4ms** | 5ms |
| app22_glob | 92 | 3ms | **3ms** | 4ms |
| app23_stack | 83 | 3ms | **3ms** | 4ms |
| app24_lexer | 106 | 4ms | **4ms** | 5ms |
| app25_parser | 204 | 5ms | **5ms** | 6ms |
| app26_window | 55 | 2ms | **3ms** | 3ms |
| app27_b148 | 44 | 3ms | **3ms** | 4ms |
| app28_closures | 30 | 2ms | **2ms** | 3ms |
| app8_orders | 136 | 3ms | **4ms** | 5ms |
| app9_graph | 79 | 3ms | **3ms** | 4ms |

**Sum of medians:** 68ms across 21 programs

Done. Save this output to `benchmarks/COMPILE_SPEED.md` to update the baseline.
