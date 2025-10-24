# Phase 6.2: File I/O API Design & Specification

**Version:** v0.9.4  
**Date:** 2025-01-21  
**Status:** Design Phase  
**Estimated Time:** 3 hours total

---

## 1. Overview

This phase adds file system operations to Liva, enabling reading, writing, and manipulating files with proper error handling.

### Goals
- ✅ Read text files with error handling
- ✅ Write and append to files
- ✅ Check file existence and delete files
- ✅ Proper error handling with error binding
- ✅ Cross-platform path handling

### Non-Goals (Future Phases)
- ❌ Binary file operations (Phase 6.7)
- ❌ File watching/monitoring (Phase 7+)
- ❌ Advanced file metadata (Phase 7+)
- ❌ Directory recursion/walking (Phase 7+)

---

## 2. API Design

### 2.1 File.read() - Read Text File

**Signature:**
```liva
File.read(path: string): (string?, Error?)
```

**Description:**  
Reads the entire contents of a text file and returns it as a string.

**Usage:**
```liva
let content, err = File.read("config.txt")

if err {
    print("Failed to read file: ${err}")
    fail err
}

print("File content: ${content}")
```

**Return Type:**
- `(string?, Error?)` - File content or error

**Error Cases:**
- File not found
- Permission denied
- Invalid UTF-8 content
- I/O errors

---

### 2.2 File.write() - Write Text File

**Signature:**
```liva
File.write(path: string, content: string): (bool?, Error?)
```

**Description:**  
Writes content to a file, creating it if it doesn't exist. Overwrites existing content.

**Usage:**
```liva
let success, err = File.write("output.txt", "Hello, World!")

if err {
    print("Failed to write file: ${err}")
} else {
    print("File written successfully")
}
```

**Return Type:**
- `(bool?, Error?)` - Success flag or error

**Behavior:**
- Creates parent directories if needed
- Overwrites existing file
- Creates new file if doesn't exist

---

### 2.3 File.append() - Append to File

**Signature:**
```liva
File.append(path: string, content: string): (bool?, Error?)
```

**Description:**  
Appends content to the end of a file. Creates file if it doesn't exist.

**Usage:**
```liva
let success, err = File.append("log.txt", "New log entry\n")

if err {
    print("Failed to append: ${err}")
}
```

**Return Type:**
- `(bool?, Error?)` - Success flag or error

---

### 2.4 File.exists() - Check File Existence

**Signature:**
```liva
File.exists(path: string): bool
```

**Description:**  
Checks if a file exists at the given path. Does not use error binding (always succeeds).

**Usage:**
```liva
if File.exists("config.txt") {
    print("Config file found")
} else {
    print("Config file missing")
}
```

**Return Type:**
- `bool` - true if file exists, false otherwise

---

### 2.5 File.delete() - Delete File

**Signature:**
```liva
File.delete(path: string): (bool?, Error?)
```

**Description:**  
Deletes a file from the file system.

**Usage:**
```liva
let success, err = File.delete("temp.txt")

if err {
    print("Failed to delete: ${err}")
} else {
    print("File deleted")
}
```

**Return Type:**
- `(bool?, Error?)` - Success flag or error

**Error Cases:**
- File not found
- Permission denied
- File is directory

---

## 3. Implementation Strategy

### 3.1 Dependencies

**Rust Standard Library:**
- `std::fs` - File system operations
- `std::path::Path` - Path handling
- `std::io` - I/O operations

No external crates needed!

### 3.2 Code Generation Approach

Similar to JSON, implement as method calls on `File` object:

```rust
// In generate_method_call_expr()
if name == "File" {
    return self.generate_file_function_call(method_call);
}
```

### 3.3 Rust Code Mapping

**File.read():**
```rust
// Liva: let content, err = File.read("file.txt")
// Rust:
match std::fs::read_to_string("file.txt") {
    Ok(content) => (Some(content), None),
    Err(e) => (None, Some(liva_rt::Error::from(format!("File read error: {}", e))))
}
```

**File.write():**
```rust
// Liva: let ok, err = File.write("file.txt", "content")
// Rust:
match std::fs::write("file.txt", "content") {
    Ok(_) => (Some(true), None),
    Err(e) => (Some(false), Some(liva_rt::Error::from(format!("File write error: {}", e))))
}
```

**File.append():**
```rust
// Liva: let ok, err = File.append("file.txt", "content")
// Rust:
match std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open("file.txt")
    .and_then(|mut file| std::io::Write::write_all(&mut file, "content".as_bytes()))
{
    Ok(_) => (Some(true), None),
    Err(e) => (Some(false), Some(liva_rt::Error::from(format!("File append error: {}", e))))
}
```

**File.exists():**
```rust
// Liva: if File.exists("file.txt")
// Rust:
std::path::Path::new("file.txt").exists()
```

**File.delete():**
```rust
// Liva: let ok, err = File.delete("file.txt")
// Rust:
match std::fs::remove_file("file.txt") {
    Ok(_) => (Some(true), None),
    Err(e) => (Some(false), Some(liva_rt::Error::from(format!("File delete error: {}", e))))
}
```

---

## 4. Error Handling

### 4.1 Common Error Types

**File Operations:**
- `NotFound` - File doesn't exist
- `PermissionDenied` - Insufficient permissions
- `AlreadyExists` - File already exists (write mode)
- `InvalidInput` - Invalid path or filename
- `Other` - I/O errors

**Error Messages:**
```liva
let content, err = File.read("missing.txt")
if err {
    // err.message: "File read error: No such file or directory (os error 2)"
}
```

### 4.2 Error Binding Pattern

All file operations (except `exists()`) use error binding:

```liva
let result, err = File.operation(args)

if err {
    // Handle error
} else {
    // Use result
}
```

---

## 5. Testing Strategy

### 5.1 Test Cases

**Read Tests:**
1. Read existing file
2. Read non-existent file (error)
3. Read file with special characters
4. Read empty file

**Write Tests:**
1. Write new file
2. Overwrite existing file
3. Write to non-existent directory (should create)
4. Write with permission error

**Append Tests:**
1. Append to existing file
2. Append to new file (creates it)
3. Multiple appends

**Exists Tests:**
1. Check existing file (true)
2. Check non-existent file (false)

**Delete Tests:**
1. Delete existing file
2. Delete non-existent file (error)
3. Delete after write

**Integration Tests:**
1. Write → Read → Verify content
2. Write → Append → Read → Verify
3. Write → Delete → Exists → Verify gone
4. Read → Modify → Write back

---

## 6. Example Usage

### Example 1: Read Configuration File

```liva
main() {
    let config, err = File.read("config.json")
    
    if err {
        print("Config file not found, using defaults")
        return
    }
    
    // Parse config
    let data, parseErr = JSON.parse(config)
    if parseErr {
        print("Invalid config file")
        fail parseErr
    }
    
    print("Config loaded successfully")
}
```

### Example 2: Write Log File

```liva
function log(message: string) {
    let timestamp = "2025-01-21 10:30:00"  // Placeholder
    let entry = "${timestamp}: ${message}\n"
    
    let ok, err = File.append("app.log", entry)
    
    if err {
        print("Failed to write log: ${err}")
    }
}

main() {
    log("Application started")
    log("Processing data")
    log("Application finished")
}
```

### Example 3: Backup System

```liva
main() {
    let source = "important.txt"
    let backup = "important.txt.backup"
    
    // Check if source exists
    if !File.exists(source) {
        print("Source file not found")
        return
    }
    
    // Read source
    let content, readErr = File.read(source)
    if readErr {
        print("Failed to read source: ${readErr}")
        fail readErr
    }
    
    // Write backup
    let ok, writeErr = File.write(backup, content)
    if writeErr {
        print("Failed to create backup: ${writeErr}")
        fail writeErr
    }
    
    print("Backup created successfully")
}
```

### Example 4: File Processing Pipeline

```liva
main() {
    // Read input
    let input, err1 = File.read("input.txt")
    if err1 {
        fail err1
    }
    
    // Process (example: convert to uppercase)
    let processed = input  // Would call .toUpperCase() when available
    
    // Write output
    let ok, err2 = File.write("output.txt", processed)
    if err2 {
        fail err2
    }
    
    print("File processed successfully")
}
```

---

## 7. Documentation Plan

### 7.1 Files to Create

1. **`docs/language-reference/file-io.md`** (400 lines)
   - Complete API reference
   - Error handling guide
   - Path handling notes
   - Examples

2. **`examples/file_io_demo.liva`** (100 lines)
   - Read/write examples
   - Error handling
   - Real-world use cases

3. **Update `CHANGELOG.md`**
   - Add v0.9.4 entry
   - List new features

4. **Update `ROADMAP.md`**
   - Mark Phase 6.2 as complete
   - Update version to v0.9.4

---

## 8. Iteration Plan (3 hours)

### Iteration 1: Design & API (20 min) ✅ CURRENT
- ✅ Create this design document
- ✅ Define function signatures
- ✅ Plan error handling

### Iteration 2: Implement File.read() (40 min)
- [ ] Add to codegen
- [ ] Handle errors
- [ ] Test basic read

### Iteration 3: Implement File.write() & File.append() (40 min)
- [ ] Add write operation
- [ ] Add append operation
- [ ] Test both operations

### Iteration 4: Implement File.exists() & File.delete() (30 min)
- [ ] Add exists check
- [ ] Add delete operation
- [ ] Test utilities

### Iteration 5: Integration & Testing (40 min)
- [ ] Comprehensive test suite
- [ ] Integration tests (read→write→delete)
- [ ] Error handling tests

### Iteration 6: Documentation (30 min)
- [ ] Create file-io.md reference
- [ ] Example programs
- [ ] Update CHANGELOG & ROADMAP

---

## 9. Success Criteria

- ✅ Read text files successfully
- ✅ Write and append to files
- ✅ Check file existence
- ✅ Delete files
- ✅ Error binding works correctly
- ✅ All tests pass
- ✅ Documentation complete
- ✅ Examples demonstrate real usage

---

## 10. Future Enhancements (Post-v0.9.4)

### Phase 6.7: Binary File Operations
```liva
let bytes = File.readBytes("image.png")
File.writeBytes("copy.png", bytes)
```

### Phase 7+: Directory Operations
```liva
Dir.create("new_folder")
Dir.list("path")
Dir.delete("folder")
```

### Phase 7+: File Metadata
```liva
let info = File.info("file.txt")
print("Size: ${info.size}")
print("Modified: ${info.modified}")
```

### Phase 7+: File Watching
```liva
File.watch("config.txt", (event) => {
    print("File changed: ${event}")
})
```

---

**Next Step:** Start Iteration 2 - Implement File.read()
