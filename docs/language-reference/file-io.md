# File & Directory I/O Operations

> **Version**: 1.6.0  
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
- Does not work on directories (use `Dir.delete()`)
- No confirmation prompt - file is deleted immediately

---

### File.copy() *(v1.6.0)*

Copies a file from one location to another.

**Signature:**
```liva
File.copy(src: string, dest: string): (bool?, Error?)
```

**Parameters:**
- `src`: Path to the source file
- `dest`: Path to the destination file

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if copy fails

**Example:**
```liva
let ok, err = File.copy("data.csv", "data_backup.csv")

if err {
    print($"Copy failed: {err}")
} else {
    print("Backup created successfully")
}
```

**Behavior:**
- Overwrites the destination if it already exists
- Preserves file permissions on Unix
- Does not copy directories (use `Dir.create()` + recursive copy)

**Error Scenarios:**
- Source doesn't exist: `"File copy error: No such file or directory (os error 2)"`
- Permission denied: `"File copy error: Permission denied (os error 13)"`

---

### File.move() *(v1.6.0)*

Moves or renames a file.

**Signature:**
```liva
File.move(src: string, dest: string): (bool?, Error?)
```

**Parameters:**
- `src`: Current path of the file
- `dest`: New path for the file

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if move fails

**Example:**
```liva
let ok, err = File.move("draft.txt", "final.txt")

if err {
    print($"Move failed: {err}")
}
```

**Behavior:**
- Works as rename when source and dest are in the same directory
- Overwrites the destination if it already exists
- May fail across different filesystems/mount points

**Error Scenarios:**
- Source doesn't exist: `"File move error: No such file or directory (os error 2)"`
- Cross-device move: `"File move error: Invalid cross-device link (os error 18)"`

---

### File.size() *(v1.6.0)*

Returns the size of a file in bytes.

**Signature:**
```liva
File.size(path: string): (int?, Error?)
```

**Parameters:**
- `path`: Path to the file

**Returns:**
- Success: `(Some(bytes), None)` where `bytes` is the file size as an integer
- Failure: `(None, Some(Error))` if metadata can't be read

**Example:**
```liva
let bytes, err = File.size("data.bin")

if err == "" {
    print($"File size: {bytes} bytes")
    if bytes > 1048576 {
        print("File is larger than 1MB")
    }
}
```

**Error Scenarios:**
- File doesn't exist: `"File size error: No such file or directory (os error 2)"`
- Permission denied: `"File size error: Permission denied (os error 13)"`

---

### File.extension() *(v1.6.0)*

Returns the file extension (without the dot).

**Signature:**
```liva
File.extension(path: string): string
```

**Parameters:**
- `path`: Path to the file

**Returns:**
- The extension as a string (e.g., `"txt"`, `"json"`, `"rs"`)
- Empty string `""` if the file has no extension

**Example:**
```liva
let ext = File.extension("photo.jpg")
print($"Extension: {ext}")  // "Extension: jpg"

let noExt = File.extension("Makefile")
print($"Extension: '{noExt}'")  // "Extension: ''"

let dotfile = File.extension(".gitignore")
print($"Extension: '{dotfile}'")  // "Extension: 'gitignore'"
```

**Note:**
- Like `File.exists()`, this does **not** use error binding (never fails)
- Returns the last extension only: `"archive.tar.gz"` → `"gz"`
- Does not include the leading dot

---

### File.readLines() *(v1.6.0)*

Reads a file and returns its contents as an array of lines.

**Signature:**
```liva
File.readLines(path: string): ([string]?, Error?)
```

**Parameters:**
- `path`: Path to the file

**Returns:**
- Success: `(Some(lines), None)` where `lines` is an array of strings (one per line)
- Failure: `(None, Some(Error))` if file can't be read

**Example:**
```liva
let lines, err = File.readLines("data.txt")

if err == "" {
    print($"Read {lines.length} lines")
    for line in lines {
        print(line)
    }
}
```

**Behavior:**
- Splits on `\n` (handles both `\n` and `\r\n`)
- Empty file returns an array with one empty string `[""]`
- Trailing newline does not produce an extra empty element

**Error Scenarios:**
- Same as `File.read()` (file not found, permission denied, etc.)

---

### File.writeLines() *(v1.6.0)*

Writes an array of strings to a file, one per line.

**Signature:**
```liva
File.writeLines(path: string, lines: [string]): (bool?, Error?)
```

**Parameters:**
- `path`: Path to the file
- `lines`: Array of strings to write

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if write fails

**Example:**
```liva
let lines = ["name,age,city", "Alice,30,NYC", "Bob,25,LAX"]
let ok, err = File.writeLines("output.csv", lines)

if err != "" {
    print($"Write error: {err}")
}
```

**Behavior:**
- Joins lines with `\n` separator
- Creates the file if it doesn't exist
- Overwrites existing content
- No trailing newline after the last line

**Error Scenarios:**
- Same as `File.write()` (permission denied, disk full, etc.)

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

### Dir.exists() *(v1.6.0)*

Checks whether a path exists **and is a directory**.

**Signature:**
```liva
Dir.exists(path: string): bool
```

**Parameters:**
- `path`: Path to check

**Returns:**
- `true` if the path exists and is a directory
- `false` if the path doesn't exist or is a regular file

**Example:**
```liva
if Dir.exists("./output") {
    print("Output directory exists")
} else {
    let ok, err = Dir.create("./output")
}
```

**Notes:**
- Unlike `File.exists()` (which returns `true` for both files and dirs), `Dir.exists()` only returns `true` for directories
- Does not use error binding

---

### Dir.create() *(v1.6.0)*

Creates a directory (and any missing parent directories).

**Signature:**
```liva
Dir.create(path: string): (bool?, Error?)
```

**Parameters:**
- `path`: Path to the directory to create

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if creation fails

**Example:**
```liva
let ok, err = Dir.create("./output/reports/2026")

if err != "" {
    print($"Create failed: {err}")
} else {
    print("Directory created (including parents)")
}
```

**Behavior:**
- Creates all intermediate directories (like `mkdir -p`)
- Succeeds silently if the directory already exists
- Fails if a file exists at the given path

**Error Scenarios:**
- Permission denied: `"Dir.create error: Permission denied (os error 13)"`
- Path is a file: `"Dir.create error: File exists (os error 17)"`

---

### Dir.delete() *(v1.6.0)*

Deletes a directory and all its contents recursively.

**Signature:**
```liva
Dir.delete(path: string): (bool?, Error?)
```

**Parameters:**
- `path`: Path to the directory to delete

**Returns:**
- Success: `(Some(true), None)`
- Failure: `(Some(false), Some(Error))` if deletion fails

**Example:**
```liva
let ok, err = Dir.delete("./temp")

if err != "" {
    print($"Delete failed: {err}")
} else {
    print("Temp directory removed")
}
```

**Warning:**
- This operation is **irreversible** and removes all contents recursively
- Equivalent to `rm -rf` on Unix

**Error Scenarios:**
- Directory doesn't exist: `"Dir.delete error: No such file or directory (os error 2)"`
- Permission denied: `"Dir.delete error: Permission denied (os error 13)"`

---

### Dir.listRecursive() / Dir.walk() *(v1.6.0)*

Lists all files and directories recursively under a given path.

**Signature:**
```liva
Dir.listRecursive(path: string): ([string]?, Error?)
Dir.walk(path: string): ([string]?, Error?)
```

**Parameters:**
- `path`: Root path to traverse

**Returns:**
- Success: `(Some(paths), None)` where `paths` is a sorted array of relative paths
- Failure: `(None, Some(Error))` if traversal fails

**Example:**
```liva
let files, err = Dir.listRecursive("./src")

if err == "" {
    print($"Found {files.length} entries")
    for f in files {
        print(f)
    }
}
// Output:
// Found 5 entries
// main.liva
// utils/helpers.liva
// utils/math.liva
// tests/test_main.liva
// tests/test_utils.liva
```

**Behavior:**
- Returns relative paths from the given root
- Includes both files and subdirectories
- Results are sorted alphabetically
- `Dir.walk()` is an alias for `Dir.listRecursive()` — identical behavior

**Error Scenarios:**
- Directory doesn't exist: `"Dir.listRecursive error: No such file or directory (os error 2)"`
- Permission denied inside tree: `"Dir.listRecursive error: Permission denied (os error 13)"`

---

### Directory Traversal Pattern

```liva
// Simple: using Dir.listRecursive (v1.6+)
let files, err = Dir.listRecursive("./src")
if err == "" {
    for f in files {
        if File.extension(f) == "liva" {
            print($"Liva source: {f}")
        }
    }
}

// Manual: using Dir.list + Dir.isDir for custom logic
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
- **`File.copy()`** → `std::fs::copy()` *(v1.6)*
- **`File.move()`** → `std::fs::rename()` *(v1.6)*
- **`File.size()`** → `std::fs::metadata().len()` *(v1.6)*
- **`File.extension()`** → `std::path::Path::new().extension()` *(v1.6)*
- **`File.readLines()`** → `std::fs::read_to_string().lines().collect()` *(v1.6)*
- **`File.writeLines()`** → `std::fs::write(lines.join("\n"))` *(v1.6)*
- **`Dir.list()`** → `std::fs::read_dir()`
- **`Dir.isDir()`** → `std::path::Path::new().is_dir()`
- **`Dir.exists()`** → `Path::exists() && Path::is_dir()` *(v1.6)*
- **`Dir.create()`** → `std::fs::create_dir_all()` *(v1.6)*
- **`Dir.delete()`** → `std::fs::remove_dir_all()` *(v1.6)*
- **`Dir.listRecursive()` / `Dir.walk()`** → recursive `std::fs::read_dir()` *(v1.6)*

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

1. **Synchronous Only**
   - All operations block until completion
   - No async/await for file I/O (yet)

2. **No Streaming**
   - Entire file is read/written at once
   - Not suitable for very large files (>100MB)

3. **UTF-8 Only**
   - Files are treated as UTF-8 text
   - Binary files not supported

4. **No Path Manipulation**
   - Cannot join paths, resolve relatives, normalize
   - Use string concatenation for path building

5. **No File Permissions/Timestamps**
   - Cannot read or set file permissions
   - Cannot read modification timestamps

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

### v1.6.0 (Current)
- ✨ Added `File.copy()` — Copy files
- ✨ Added `File.move()` — Move/rename files
- ✨ Added `File.size()` — Get file size in bytes
- ✨ Added `File.extension()` — Get file extension
- ✨ Added `File.readLines()` — Read file as array of lines
- ✨ Added `File.writeLines()` — Write array of lines to file
- ✨ Added `Dir.exists()` — Check if path is a directory
- ✨ Added `Dir.create()` — Create directory (recursive)
- ✨ Added `Dir.delete()` — Delete directory (recursive)
- ✨ Added `Dir.listRecursive()` / `Dir.walk()` — List files recursively
- 🔧 Parser: Allow `move` keyword as method name for `File.move()`

### v1.3.0
- ✨ Added `Dir.list()` — List directory entries with error binding
- ✨ Added `Dir.isDir()` — Check if path is a directory
- ✅ Directory traversal support for recursive file operations

### v0.9.4
- ✨ Initial implementation of File I/O operations
- ✅ All 5 File operations: `read`, `write`, `append`, `exists`, `delete`
- ✅ Error binding integration
- ✅ Comprehensive test coverage

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
