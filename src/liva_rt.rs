use std::future::Future;
use tokio::task::spawn as tokio_spawn;

// Placeholder types for advanced parallelism features
pub enum ThreadOption {
    Auto,
    Count(usize),
}

pub enum SimdWidthOption {
    Auto,
    Width(usize),
}

pub struct ParallelForOptions {
    pub chunk: Option<usize>,
    pub threads: Option<ThreadOption>,
    pub prefetch: Option<usize>,
    pub reduction: Option<ReductionOption>,
    pub schedule: Option<ScheduleOption>,
    pub detect: Option<DetectOption>,
    pub simd_width: Option<SimdWidthOption>,
}

pub enum ReductionOption {
    Safe,
    Fast,
}

pub enum ScheduleOption {
    Static,
    Dynamic,
}

pub enum DetectOption {
    Auto,
}

pub trait SequenceCount {
    type Output;
    fn count(&self) -> Self::Output;
}

impl<T> SequenceCount for Vec<T> {
    type Output = usize;
    fn count(&self) -> usize {
        self.len()
    }
}

impl<T> SequenceCount for &[T] {
    type Output = usize;
    fn count(&self) -> usize {
        self.len()
    }
}

pub fn normalize_size(size: usize, _default: usize) -> usize {
    size
}

pub fn for_par<T, F>(_iter: Vec<T>, _func: F)
where
    F: Fn(T) + Send + Sync,
{
    // Placeholder - not implemented
}

pub fn for_vec<T, F>(_iter: Vec<T>, _func: F)
where
    F: Fn(T) + Send + Sync,
{
    // Placeholder - not implemented
}

pub fn for_parvec<T, F>(_iter: Vec<T>, _func: F)
where
    F: Fn(T) + Send + Sync,
{
    // Placeholder - not implemented
}

/// Spawn an async task
pub fn spawn_async<F, T>(future: F) -> tokio::task::JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(future)
}

/// Spawn a parallel task (for simplicity, executes synchronously and returns JoinHandle)
pub fn spawn_parallel<F, T>(f: F) -> tokio::task::JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    // For simplicity, just execute synchronously and wrap in JoinHandle
    // In a real implementation, this would use rayon or std::thread
    let result = f();
    tokio::spawn(async move { result })
}

/// Fire and forget async task
pub fn fire_async<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio_spawn(future);
}

/// Fire and forget parallel task
pub fn fire_parallel<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    // For simplicity, just spawn a thread
    std::thread::spawn(f);
}

/// Runtime error type for fallible operations
#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn from<S: Into<String>>(message: S) -> Self {
        Error {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

/// String multiplication helper
/// Supports both String*int and int*String patterns
pub fn string_mul<L, R>(left: L, right: R) -> String
where
    L: StringOrInt,
    R: StringOrInt,
{
    match (left.as_string_or_int(), right.as_string_or_int()) {
        (StringOrIntValue::String(s), StringOrIntValue::Int(n)) => {
            if n <= 0 {
                String::new()
            } else {
                s.repeat(n as usize)
            }
        }
        (StringOrIntValue::Int(n), StringOrIntValue::String(s)) => {
            if n <= 0 {
                String::new()
            } else {
                s.repeat(n as usize)
            }
        }
        (StringOrIntValue::Int(a), StringOrIntValue::Int(b)) => {
            // Fallback to numeric multiplication
            (a * b).to_string()
        }
        (StringOrIntValue::String(_), StringOrIntValue::String(_)) => {
            // String * String is not supported
            panic!("Cannot multiply two strings")
        }
    }
}

pub enum StringOrIntValue {
    String(String),
    Int(i64),
}

pub trait StringOrInt {
    fn as_string_or_int(self) -> StringOrIntValue;
}

impl StringOrInt for String {
    fn as_string_or_int(self) -> StringOrIntValue {
        StringOrIntValue::String(self)
    }
}

impl StringOrInt for &str {
    fn as_string_or_int(self) -> StringOrIntValue {
        StringOrIntValue::String(self.to_string())
    }
}

impl StringOrInt for i32 {
    fn as_string_or_int(self) -> StringOrIntValue {
        StringOrIntValue::Int(self as i64)
    }
}

impl StringOrInt for i64 {
    fn as_string_or_int(self) -> StringOrIntValue {
        StringOrIntValue::Int(self)
    }
}

impl StringOrInt for f64 {
    fn as_string_or_int(self) -> StringOrIntValue {
        StringOrIntValue::Int(self as i64)
    }
}

impl StringOrInt for usize {
    fn as_string_or_int(self) -> StringOrIntValue {
        StringOrIntValue::Int(self as i64)
    }
}
