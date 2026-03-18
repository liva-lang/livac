# File & Directory I/O Operations

> **Version**: 1.3.0  
> **Status**: Stable

## Overview

Liva provides built-in File and Directory I/O operations through the `File` and `Dir` namespaces. All fallible operations use the **error binding pattern** to handle potential failures gracefully.

## API Reference

### File.read()

Reads the entire contents of a file as a string.

**Signature:**
```liva
File.read(path: string): (string?, Error?)
```

**Parameters:**
- `path`: Path to the file (relative or absolute)

**Returns:**
- Success: `(Some(content), None)` where `content` is the file contents as a string
- Failure: `(None, Some(Error))` if file doesn't exist or can't be read

**Example:**
```liva
let content, err = File.read("config.json")

if err {
    console.log("Failed to read file: " + err.message)
} else {
    console.log("File content: " + content)
}
```

**Error Scenarios:**
- File doesn't exist: `"File read error: No such file or directory (os error 2)"`
- Permission denied: `"File read error: Permission denied (os error 13)"`
- Directory instead of file: `"File read error: Is a directory (os error 21)"`

---

### File.write()

Writes content to a file, creating it if it doesn't exist, or overwriting if it does.

**Signature:**
```liva
File.write(path: string, content: string): (bool?, Error?)
```

**Parameters:**
- `path`: Path to the file
- `content`: String content to write

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if write operation fails

**Example:**
```liva
let success, err = File.write("output.txt", "Hello, Liva!")

if err {
    console.log("Write failed: " + err.message)
} else {
    console.log("File written successfully")
}
```

**Behavior:**
- Creates the file if it doesn't exist
- Overwrites existing file contents
- Creates parent directories if they don't exist (Unix behavior)

**Error Scenarios:**
- Permission denied: `"File write error: Permission denied (os error 13)"`
- Disk full: `"File write error: No space left on device (os error 28)"`
- Read-only filesystem: `"File write error: Read-only file system (os error 30)"`

---

### File.append()

Appends content to the end of a file. Creates the file if it doesn't exist.

**Signature:**
```liva
File.append(path: string, content: string): (bool?, Error?)
```

**Parameters:**
- `path`: Path to the file
- `content`: String content to append

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if append operation fails

**Example:**
```liva
let success, err = File.append("log.txt", "\n[INFO] New log entry")

if err {
    console.log("Append failed: " + err.message)
} else {
    console.log("Log entry added")
}
```

**Behavior:**
- Creates the file if it doesn't exist
- Appends to the end without modifying existing content
- Ideal for logging and incremental writes

**Error Scenarios:**
- Same as `File.write()` (permission, disk space, etc.)

---

### File.exists()

Checks whether a file or directory exists at the specified path.

**Signature:**
```liva
File.exists(path: string): bool
```

**Parameters:**
- `path`: Path to check

**Returns:**
- `true` if the path exists (file or directory)
- `false` if the path doesn't exist

**Example:**
```liva
if File.exists("config.json") {
    let content, err = File.read("config.json")
    // Process config...
} else {
    console.log("Config file not found, using defaults")
}
```

**Note:**
- This is the **only** File operation that doesn't use error binding
- Returns `false` for broken symbolic links
- Cannot distinguish between files and directories (both return `true`)

---

### File.delete()

Deletes a file from the filesystem.

**Signature:**
```liva
File.delete(path: string): (bool?, Error?)
```

**Parameters:**
- `path`: Path to the file to delete

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if deletion fails

**Example:**
```liva
let success, err = File.delete("temp.txt")

if err {
    console.log("Delete failed: " + err.message)
} else {
    console.log("File deleted successfully")
}
```

**Error Scenarios:**
- File doesn't exist: `"File delete error: No such file or directory (os error 2)"`
- Permission denied: `"File delete error: Permission denied (os error 13)"`
- File is a directory: `"File delete error: Is a directory (os error 21)"`

**Warning:**
- This operation is **irreversible**
- Does not work on directories (use OS-specific commands)
- No confirmation prompt - file is deleted immediately

---

## Directory Operations

The `Dir` namespace provides directory traversal capabilities. Added in v1.3.0.

### Dir.list()

Lists the entries (files and subdirectories) in a directory, sorted alphabetically.

**Signature:**
```liva
Dir.list(path: string): ([string]?, Error?)
```

**Parameters:**
- `path`: Path to the directory

**Returns:**
- Success: `(Some(entries), None)` where `entries` is a sorted array of filenames
- Failure: `(None, Some(Error))` if directory can't be read

**Example:**
```liva
let entries, err = Dir.list("./src")

if err {
    console.log("Cannot list directory: " + err.message)
} else {
    for entry in entries {
        print(entry)
    }
    print($"Found {entries.length} entries")
}
```

**Error Scenarios:**
- Directory doesn't exist: `"Dir.list error: No such file or directory (os error 2)"`
- Permission denied: `"Dir.list error: Permission denied (os error 13)"`
- Not a directory: `"Dir.list error: Not a directory (os error 20)"`

**Notes:**
- Returns only the entry names (not full paths) — combine with the directory path manually
- Entries are sorted alphabetically
- Does not include `.` or `..`
- Uses error binding pattern (like `File.read`)

---

### Dir.isDir()

Checks whether a path is a directory.

**Signature:**
```liva
Dir.isDir(path: string): bool
```

**Parameters:**
- `path`: Path to check

**Returns:**
- `true` if the path exists and is a directory
- `false` if the path doesn't exist or is a file

**Example:**
```liva
if Dir.isDir("./src") {
    let entries, err = Dir.list("./src")
    // Process directory...
} else {
    print("Not a directory")
}
```

**Notes:**
- Does not use error binding (returns simple bool, like `File.exists`)
- Returns `false` for broken symbolic links
- Useful for recursive directory traversal

---

### Directory Traversal Pattern

```liva
searchDir(query: string, dirPath: string) {
    let entries, err = Dir.list(dirPath)
    if err {
        return
    }

    for i in 0..entries.length {
        let entry = entries[i]
        let fullPath = dirPath + "/" + entry

        if Dir.isDir(fullPath) {
            searchDir(query, fullPath)
        } else {
            // Process file
            let content, readErr = File.read(fullPath)
            if !readErr {
                if content.contains(query) {
                    print($"Found in: {fullPath}")
                }
            }
        }
    }
}
```

---

## Common Patterns

### 1. Safe Read with Fallback

```liva
let config, err = File.read("config.json")

if err {
    console.log("Using default configuration")
    config = "{\"default\": true}"
}

let parsed, parseErr = JSON.parse(config)
```

### 2. Write-Then-Verify

```liva
let written, writeErr = File.write("output.txt", data)

if writeErr {
    console.log("Write failed!")
} else {
    // Verify
    let content, readErr = File.read("output.txt")
    if readErr {
        console.log("Write succeeded but verification failed")
    } else {
        console.log("Write verified: " + content)
    }
}
```

### 3. Conditional Write (Check Before Overwrite)

```liva
if File.exists("important.txt") {
    console.log("File exists - refusing to overwrite")
} else {
    File.write("important.txt", newData)
}
```

### 4. Logging Pattern

```liva
fn log(message: string) {
    let timestamp = "2024-01-15 10:30:00" // Get from system
    let entry = "\n[" + timestamp + "] " + message
    
    let success, err = File.append("app.log", entry)
    
    if err {
        console.log("Failed to write log: " + err.message)
    }
}
```

### 5. Backup Before Modify

```liva
fn updateConfig(newConfig: string) {
    // Read original
    let original, readErr = File.read("config.json")
    
    if readErr {
        console.log("No existing config found")
    } else {
        // Backup
        File.write("config.json.backup", original)
    }
    
    // Write new config
    let success, writeErr = File.write("config.json", newConfig)
    
    if writeErr {
        console.log("Failed to update config: " + writeErr.message)
        // Could restore backup here
    }
}
```

### 6. Temporary File Cleanup

```liva
fn processWithTempFile() {
    let tempFile = "temp_processing.txt"
    
    // Create temp file
    File.write(tempFile, intermediateData)
    
    // Process...
    let data, err = File.read(tempFile)
    
    // Cleanup
    File.delete(tempFile)
    
    return data
}
```

---

## Implementation Details

### Rust Backend

All File operations are implemented using Rust's standard library:

- **`File.read()`** → `std::fs::read_to_string()`
- **`File.write()`** → `std::fs::write()`
- **`File.append()`** → `std::fs::OpenOptions::new().create(true).append(true).open().write_all()`
- **`File.exists()`** → `std::path::Path::new().exists()`
- **`File.delete()`** → `std::fs::remove_file()`

### Error Binding Integration

File operations (except `exists()`) return tuples compatible with Liva's error binding:

```rust
// Generated Rust code for File.read()
let (content, err) = match std::fs::read_to_string(&path) {
    Ok(data) => (Some(data), None),
    Err(e) => (None, Some(liva_rt::Error::from(format!("File read error: {}", e))))
};
```

### String Concatenation Handling

When error binding variables are used in string concatenation, they are automatically unwrapped:

```liva
let content, err = File.read("test.txt")
console.log("Content: " + content)  // ✓ Works correctly
```

Generated Rust:
```rust
println!("{}", format!("{}{}", "Content: ", content.as_ref().map(|v| v.to_string()).unwrap_or_default()));
```

---

## Limitations

1. **No Directory Operations**
   - Cannot create, list, or remove directories
   - `File.exists()` returns `true` for directories but other operations fail

2. **No Metadata Access**
   - Cannot read file size, permissions, timestamps
   - Cannot set file attributes

3. **Synchronous Only**
   - All operations block until completion
   - No async/await for file I/O (yet)

4. **No Streaming**
   - Entire file is read/written at once
   - Not suitable for very large files (>100MB)

5. **UTF-8 Only**
   - Files are treated as UTF-8 text
   - Binary files not supported

6. **No Path Manipulation**
   - Cannot join paths, resolve relatives, normalize
   - Use string concatenation for path building

---

## Error Handling Best Practices

### ✅ Good: Always Check Errors

```liva
let content, err = File.read("data.txt")

if err {
    console.log("Error: " + err.message)
    return
}

// Use content safely
processData(content)
```

### ❌ Bad: Ignoring Errors

```liva
let content, err = File.read("data.txt")
// Proceeding without checking err
processData(content)  // Might crash if content is None!
```

### ✅ Good: Graceful Degradation

```liva
let customConfig, err = File.read("custom.config")

let config = ""
if err {
    console.log("Custom config not found, using defaults")
    config = "{\"mode\": \"default\"}"
} else {
    config = customConfig
}
```

### ✅ Good: Check Existence Before Critical Operations

```liva
if File.exists("important.db") {
    File.write("important.db.backup", originalData)
    File.delete("important.db")
} else {
    console.log("Database file missing - cannot proceed")
}
```

---

## Performance Considerations

### File Read Performance

- **Small files (<1KB)**: ~0.1ms
- **Medium files (1-100KB)**: ~1-10ms
- **Large files (1-10MB)**: ~100-1000ms

### Optimization Tips

1. **Batch Writes**: Combine multiple small writes into one large write
2. **Avoid Repeated Reads**: Cache file contents if reading multiple times
3. **Use Append for Logs**: More efficient than read-modify-write for logs
4. **Check Existence First**: Avoid expensive operations on non-existent files

---

## Version History

### v1.3.0 (Current)
- ✨ Added `Dir.list()` — List directory entries with error binding
- ✨ Added `Dir.isDir()` — Check if path is a directory
- ✅ Directory traversal support for recursive file operations

### v0.9.4
- ✨ Initial implementation of File I/O operations
- ✅ All 5 File operations: `read`, `write`, `append`, `exists`, `delete`
- ✅ Error binding integration
- ✅ Comprehensive test coverage (27 test cases)

---

## See Also

- [Error Handling](./error-handling.md) - Learn about error binding patterns
- [JSON API](./json.md) - Parsing and serializing JSON files
- [String Operations](./types.md#strings) - Working with file contents

---

## Examples Repository

Find more examples in:
- `examples/manual-tests/test_file_simple.liva` - Basic File operations
- `examples/manual-tests/test_file_complex.liva` - 27 comprehensive test cases
