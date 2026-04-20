// Hand-written Rust equivalent of bench_collections.liva
use std::collections::HashMap;
use std::time::Instant;

fn fill_array(n: i32) -> Vec<i32> {
    (0..n).collect()
}

fn sum_array(arr: &[i32]) -> i32 {
    arr.iter().sum()
}

fn filter_and_map(arr: &[i32]) -> Vec<i32> {
    arr.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 3)
        .collect()
}

fn build_lookup(n: i32) -> HashMap<String, i32> {
    let mut m = HashMap::new();
    for i in 0..n {
        m.insert(format!("key_{}", i), i * 7);
    }
    m
}

fn lookup_all(m: &HashMap<String, i32>, n: i32) -> i32 {
    let mut total = 0;
    for i in 0..n {
        let key = format!("key_{}", i);
        if let Some(&v) = m.get(&key) {
            total += v;
        }
    }
    total
}

fn main() {
    let iterations = 1000;
    let size = 5000;

    // Benchmark 1: Array fill + sum
    let t1 = Instant::now();
    for _ in 0..iterations {
        let arr = fill_array(size);
        let _ = sum_array(&arr);
    }
    println!("Array fill+sum: {}ms ({} x {})", t1.elapsed().as_millis(), iterations, size);

    // Benchmark 2: Filter + Map
    let base_arr = fill_array(size);
    let t2 = Instant::now();
    for _ in 0..iterations {
        let _ = filter_and_map(&base_arr);
    }
    println!("Filter+Map: {}ms ({} x {})", t2.elapsed().as_millis(), iterations, size);

    // Benchmark 3: Map build + lookup
    let t3 = Instant::now();
    for _ in 0..iterations {
        let lookup = build_lookup(1000);
        let _ = lookup_all(&lookup, 1000);
    }
    println!("Map build+lookup: {}ms ({} x 1000)", t3.elapsed().as_millis(), iterations);

    // Benchmark 4: Sort
    let t4 = Instant::now();
    for _ in 0..iterations {
        let mut arr = fill_array(size);
        arr.sort();
    }
    println!("Sort: {}ms ({} x {})", t4.elapsed().as_millis(), iterations, size);
}
