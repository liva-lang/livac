// Hand-written Rust equivalent of bench_strings.liva
use std::collections::HashMap;
use std::time::Instant;

fn process_lines(lines: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let upper = trimmed.to_uppercase();
            let replaced = upper.replace("FOO", "BAR");
            let tagged = format!("[PROCESSED] {}", replaced);
            results.push(tagged);
        }
    }
    results
}

fn build_csv(rows: i32) -> String {
    let mut output = String::from("id,name,value\n");
    for i in 0..rows {
        output.push_str(&format!("{},item_{},{}\n", i, i, i * 10));
    }
    output
}

fn count_words(text: &str) -> HashMap<String, i32> {
    let mut counts = HashMap::new();
    for word in text.split(' ') {
        let lower = word.to_lowercase();
        *counts.entry(lower).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let iterations = 1000;

    // Benchmark 1: Line processing
    let mut lines = Vec::new();
    for j in 0..1000 {
        lines.push(format!("  foo item number {}  ", j));
    }

    let t1 = Instant::now();
    for _ in 0..iterations {
        let _ = process_lines(&lines);
    }
    let t1_ms = t1.elapsed().as_millis();
    println!("Line processing: {}ms ({} iterations x 1000 lines)", t1_ms, iterations);

    // Benchmark 2: CSV building
    let t2 = Instant::now();
    for _ in 0..iterations {
        let _ = build_csv(1000);
    }
    let t2_ms = t2.elapsed().as_millis();
    println!("CSV building: {}ms ({} iterations x 1000 rows)", t2_ms, iterations);

    // Benchmark 3: Word counting
    let mut words = Vec::new();
    for _ in 0..200 {
        words.push("the quick brown fox jumps over the lazy dog and the fox runs fast");
    }
    let big_text = words.join(" ");

    let t3 = Instant::now();
    for _ in 0..iterations {
        let _ = count_words(&big_text);
    }
    let t3_ms = t3.elapsed().as_millis();
    println!("Word counting: {}ms ({} iterations)", t3_ms, iterations);
}
