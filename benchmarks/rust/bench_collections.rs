// Hand-written Rust equivalent of bench_collections.liva
use std::collections::HashMap;
use std::time::Instant;

fn fill_array(n: i32) -> Vec<i32> {
    let mut arr = Vec::new();
    let mut i = 0;
    while i < n {
        arr.push(i);
        i += 1;
    }
    arr
}

fn fill_reversed(n: i32) -> Vec<i32> {
    let mut arr = Vec::new();
    let mut i = 0;
    while i < n {
        arr.push(n - i);
        i += 1;
    }
    arr
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
    let size = 50000;

    let mut chk1: i64 = 0;
    let mut chk2: i64 = 0;
    let mut chk3: i64 = 0;
    let mut chk4: i64 = 0;

    let t1 = Instant::now();
    for _ in 0..iterations {
        let arr = fill_array(size);
        chk1 += sum_array(&arr) as i64;
    }
    println!("Array fill+sum: {}ms ({} x {})", t1.elapsed().as_millis(), iterations, size);

    let base_arr = fill_array(size);
    let t2 = Instant::now();
    for _ in 0..iterations {
        let result = filter_and_map(&base_arr);
        chk2 += result.len() as i64;
    }
    println!("Filter+Map: {}ms ({} x {})", t2.elapsed().as_millis(), iterations, size);

    let t3 = Instant::now();
    for _ in 0..iterations {
        let lookup = build_lookup(1000);
        chk3 += lookup_all(&lookup, 1000) as i64;
    }
    println!("Map build+lookup: {}ms ({} x 1000)", t3.elapsed().as_millis(), iterations);

    let t4 = Instant::now();
    for _ in 0..iterations {
        let mut arr = fill_reversed(size);
        arr.sort();
        chk4 += arr[0] as i64;
    }
    println!("Sort: {}ms ({} x {})", t4.elapsed().as_millis(), iterations, size);

    println!("checksums: {} {} {} {}", chk1, chk2, chk3, chk4);
}
