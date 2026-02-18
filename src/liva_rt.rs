use std::future::Future;
use std::time::Duration;
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

// ============================================================================
// HTTP Client
// ============================================================================

/// HTTP Response structure
#[derive(Debug, Clone, Default)]
pub struct LivaHttpResponse {
    pub status: i32,
    pub status_text: String,
    pub body: String,
    pub headers: Vec<String>,
}

impl LivaHttpResponse {
    pub fn empty() -> Self {
        LivaHttpResponse {
            status: 0,
            status_text: String::new(),
            body: String::new(),
            headers: Vec::new(),
        }
    }

    /// Parse response body as JSON
    /// Returns (JsonValue, error_string) tuple for error binding
    pub fn json(&self) -> (JsonValue, String) {
        match serde_json::from_str(&self.body) {
            Ok(value) => (JsonValue(value), String::new()),
            Err(e) => (
                JsonValue(serde_json::Value::Null),
                format!("JSON parse error: {}", e),
            ),
        }
    }
}

/// HTTP GET request
pub async fn liva_http_get(url: String) -> (Option<LivaHttpResponse>, String) {
    liva_http_request("GET", url, None).await
}

/// HTTP POST request
pub async fn liva_http_post(url: String, body: String) -> (Option<LivaHttpResponse>, String) {
    liva_http_request("POST", url, Some(body)).await
}

/// HTTP PUT request
pub async fn liva_http_put(url: String, body: String) -> (Option<LivaHttpResponse>, String) {
    liva_http_request("PUT", url, Some(body)).await
}

/// HTTP DELETE request
pub async fn liva_http_delete(url: String) -> (Option<LivaHttpResponse>, String) {
    liva_http_request("DELETE", url, None).await
}

/// Internal HTTP request implementation
async fn liva_http_request(
    method: &str,
    url: String,
    body: Option<String>,
) -> (Option<LivaHttpResponse>, String) {
    // Validate URL format
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return (
            None,
            format!(
                "Invalid URL format: '{}'. URLs must start with http:// or https://",
                url
            ),
        );
    }

    // Create reqwest client with 30s timeout
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => return (None, format!("Failed to create HTTP client: {}", e)),
    };

    // Build request
    let request_builder = match method {
        "GET" => client.get(&url),
        "POST" => {
            let mut builder = client.post(&url);
            if let Some(body_content) = body {
                builder = builder
                    .header("Content-Type", "application/json")
                    .body(body_content);
            }
            builder
        }
        "PUT" => {
            let mut builder = client.put(&url);
            if let Some(body_content) = body {
                builder = builder
                    .header("Content-Type", "application/json")
                    .body(body_content);
            }
            builder
        }
        "DELETE" => client.delete(&url),
        _ => return (None, format!("Unknown HTTP method: {}", method)),
    };

    // Execute request
    let response = match request_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            // Handle different error types
            let error_msg = if e.is_timeout() {
                "Request timeout (30s)".to_string()
            } else if e.is_connect() {
                format!("Connection error: {}", e)
            } else if e.is_request() {
                format!("Request error: {}", e)
            } else {
                format!("Network error: {}", e)
            };
            return (None, error_msg);
        }
    };

    // Extract status
    let status = response.status();
    let status_code = status.as_u16() as i32;
    let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

    // Extract headers
    let mut headers = Vec::new();
    for (key, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.push(format!("{}: {}", key.as_str(), value_str));
        }
    }

    // Read body as text
    let body = match response.text().await {
        Ok(text) => text,
        Err(e) => return (None, format!("Failed to read response body: {}", e)),
    };

    // Create response object
    let liva_response = LivaHttpResponse {
        status: status_code,
        status_text,
        body,
        headers,
    };

    (Some(liva_response), String::new())
}

// ============================================================================
// JSON Support - Wrapper around serde_json::Value
// ============================================================================

/// Wrapper for JSON values to provide Liva-friendly interface
#[derive(Debug, Clone)]
pub struct JsonValue(pub serde_json::Value);

impl JsonValue {
    /// Create from serde_json::Value
    pub fn new(value: serde_json::Value) -> Self {
        JsonValue(value)
    }

    /// Get length of array or object
    pub fn length(&self) -> usize {
        match &self.0 {
            serde_json::Value::Array(arr) => arr.len(),
            serde_json::Value::Object(obj) => obj.len(),
            serde_json::Value::String(s) => s.len(),
            _ => 0,
        }
    }

    /// Get element by index (for arrays)
    pub fn get(&self, index: usize) -> Option<JsonValue> {
        match &self.0 {
            serde_json::Value::Array(arr) => arr.get(index).map(|v| JsonValue(v.clone())),
            _ => None,
        }
    }

    /// Get field by key (for objects)
    pub fn get_field(&self, key: &str) -> Option<JsonValue> {
        match &self.0 {
            serde_json::Value::Object(obj) => obj.get(key).map(|v| JsonValue(v.clone())),
            _ => None,
        }
    }

    /// Convert to i32 if possible
    pub fn as_i32(&self) -> Option<i32> {
        self.0.as_i64().map(|n| n as i32)
    }

    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    /// Convert to String
    pub fn as_string(&self) -> Option<String> {
        match &self.0 {
            serde_json::Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Convert to bool
    pub fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    /// Check if null
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// Check if array
    pub fn is_array(&self) -> bool {
        self.0.is_array()
    }

    /// Check if object
    pub fn is_object(&self) -> bool {
        self.0.is_object()
    }

    /// Convert entire value to string (JSON representation)
    pub fn to_json_string(&self) -> String {
        self.0.to_string()
    }

    /// Get as vector for iteration (if array)
    pub fn as_array(&self) -> Option<Vec<JsonValue>> {
        match &self.0 {
            serde_json::Value::Array(arr) => {
                Some(arr.iter().map(|v| JsonValue(v.clone())).collect())
            }
            _ => None,
        }
    }

    /// Convert to Vec for array operations (unwraps to empty vec if not array)
    pub fn to_vec(&self) -> Vec<JsonValue> {
        self.as_array().unwrap_or_else(Vec::new)
    }

    /// Iterator method for array operations (forEach, map, filter)
    /// Always returns a Vec iterator for consistency
    pub fn iter(&self) -> std::vec::IntoIter<JsonValue> {
        self.to_vec().into_iter()
    }
}

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement IntoIterator for JsonValue to support for...in loops
impl IntoIterator for JsonValue {
    type Item = JsonValue;
    type IntoIter = std::vec::IntoIter<JsonValue>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            serde_json::Value::Array(arr) => arr
                .into_iter()
                .map(|v| JsonValue(v))
                .collect::<Vec<_>>()
                .into_iter(),
            _ => Vec::new().into_iter(), // Empty iterator for non-arrays
        }
    }
}

// Legacy helper functions (kept for backward compatibility)
pub fn json_value_length(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(arr) => arr.len(),
        serde_json::Value::Object(obj) => obj.len(),
        serde_json::Value::String(s) => s.len(),
        _ => 0,
    }
}

pub fn json_value_get(value: &serde_json::Value, index: usize) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Array(arr) => arr.get(index).cloned(),
        _ => None,
    }
}

pub fn json_value_get_field(value: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Object(obj) => obj.get(key).cloned(),
        _ => None,
    }
}
