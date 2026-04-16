// Hand-written idiomatic Rust equivalent of the Liva benchmark
// This is the TARGET — what Liva's codegen should produce
use std::collections::HashMap;

pub struct WordCounter {
    pub counts: HashMap<String, i32>,
    pub total_words: i32,
    pub unique_words: i32,
}

impl WordCounter {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            total_words: 0,
            unique_words: 0,
        }
    }

    // Takes &str instead of String — no allocation needed for lookup/insert
    pub fn add_word(&mut self, word: &str) {
        let current = self.counts.get(word).copied().unwrap_or(0);
        if current == 0 {
            self.unique_words += 1;
        }
        self.counts.insert(word.to_string(), current + 1);
        self.total_words += 1;
    }

    pub fn get_count(&self, word: &str) -> i32 {
        self.counts.get(word).copied().unwrap_or(0)
    }
}

fn generate_lines(n: i32) -> Vec<String> {
    let words = ["the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
                 "hello", "world", "foo", "bar", "baz", "liva", "rust", "code"];
    let mut lines: Vec<String> = Vec::with_capacity(n as usize);
    for i in 0..n {
        let mut line = String::new();
        for j in 0..10 {
            let idx = (i + j * 7) % (words.len() as i32);
            let word = words[idx as usize]; // &str, no clone needed
            if j > 0 {
                line.push(' ');
                line.push_str(word);
            } else {
                line.push_str(word);
            }
        }
        lines.push(line); // move, no clone
    }
    lines
}

// Takes &[String] — borrows the vec, no clone
fn process_lines(lines: &[String]) -> WordCounter {
    let mut counter = WordCounter::new();
    for line in lines {                     // &String, no clone of vec
        let words: Vec<&str> = line.split(' ').collect(); // &str, no to_string
        for word in &words {                // &str, no clone of vec
            let lower = word.to_lowercase();
            let trimmed = lower.trim();
            if !trimmed.is_empty() {
                counter.add_word(trimmed);  // &str, no clone
            }
        }
    }
    counter
}

fn transform_numbers(n: i32) -> i32 {
    let nums: Vec<i32> = (0..n).collect();
    nums.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 2)
        .sum()  // No intermediate collections
}

// Takes &WordCounter — borrows, no clone
fn build_report(counter: &WordCounter, n: i32) -> String {
    let mut report = String::from("=== Word Count Report ===\n");
    report.push_str(&format!("Total words: {}\n", counter.total_words));
    report.push_str(&format!("Unique words: {}\n", counter.unique_words));
    report.push_str(&format!("Lines processed: {}\n", n));
    let the_count = counter.get_count("the");
    let fox_count = counter.get_count("fox");
    let hello_count = counter.get_count("hello");
    report.push_str(&format!("  'the': {}\n", the_count));
    report.push_str(&format!("  'fox': {}\n", fox_count));
    report.push_str(&format!("  'hello': {}\n", hello_count));
    report
}

fn main() {
    let iterations = 50;
    let lines_per_iter = 1000;
    let num_transform = 5000;
    println!("Running benchmark: {} iterations", iterations);
    println!("  Lines per iteration: {}", lines_per_iter);
    println!("  Numbers to transform: {}", num_transform);
    println!();
    let mut total_words = 0;
    let mut total_sum = 0;
    for i in 0..iterations {
        let lines = generate_lines(lines_per_iter);
        let counter = process_lines(&lines);  // borrow, no clone
        total_words += counter.total_words;
        let sum = transform_numbers(num_transform);
        total_sum += sum;
        let report = build_report(&counter, lines_per_iter); // borrow, no clone
        if i == 0 {
            println!("{}", report);
        }
    }
    println!("Total words processed: {}", total_words);
    println!("Total sum: {}", total_sum);
    println!("Done.");
}
