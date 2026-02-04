# Generics & Parallel Dogfooding Tests

Testing advanced generics and parallel execution in Liva.

## Tests

1. **test1_stack.liva** - Generic Stack<T> data structure
2. **test2_pair.liva** - Pair<K,V> with methods
3. **test3_result.liva** - Result<T,E> error handling pattern
4. **test4_parallel_arrays.liva** - Parallel array operations
5. **test5_parallel_generics.liva** - Combining generics with parallel
6. **test6_multifile/** - Multi-file generic imports

## Running

```bash
cd examples/generics_parallel_test/src
../../../target/release/livac test1_stack.liva --run
```
