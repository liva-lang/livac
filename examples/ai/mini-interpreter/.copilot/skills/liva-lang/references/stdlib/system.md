# System Functions (Sys)

> **Status:** ✅ Complete (3/3 functions)  
> **Version:** v1.3.0

The `Sys` namespace provides system-level functions for command-line arguments, environment variables, and process control.

---

## 📋 Table of Contents

- [Sys.args()](#sysargs)
- [Sys.env()](#sysenv)
- [Sys.exit()](#sysexit)

---

## `Sys.args()`

Get command-line arguments passed to the program.

**Signature:**
```liva
Sys.args(): [string]
```

**Returns:**
- An array of strings containing the command-line arguments
- The first element is the program name/path

**Example:**
```liva
main() {
    let args = Sys.args()
    
    if args.length < 3 {
        print("Usage: program <input> <output>")
        Sys.exit(1)
    }
    
    let inputFile = args[1]
    let outputFile = args[2]
    print($"Processing {inputFile} -> {outputFile}")
}
```

**Rust Codegen:**
```rust
std::env::args().collect::<Vec<String>>()
```

**Notes:**
- Returns a native `Vec<String>` (not the standard Liva array)
- `args[0]` is the program path/name
- User arguments start at `args[1]`
- Empty array is never returned (at minimum contains program name)

---

## `Sys.env()`

Read an environment variable.

**Signature:**
```liva
Sys.env(name: string): string
```

**Parameters:**
- `name`: The environment variable name

**Returns:**
- The value of the environment variable as a string
- Empty string `""` if the variable is not set

**Example:**
```liva
let home = Sys.env("HOME")
print($"Home directory: {home}")

let apiKey = Sys.env("API_KEY")
if apiKey == "" {
    print("Warning: API_KEY not set")
}
```

**Rust Codegen:**
```rust
std::env::var("HOME").unwrap_or_default()
```

**Notes:**
- Returns empty string (not an error) if variable is not set
- Variable names are case-sensitive on Linux/macOS, case-insensitive on Windows
- Common variables: `HOME`, `PATH`, `USER`, `PWD`, `SHELL`

---

## `Sys.exit()`

Terminate the program with a specific exit code.

**Signature:**
```liva
Sys.exit(code: number): never
```

**Parameters:**
- `code`: Exit code (0 = success, non-zero = error)

**Example:**
```liva
main() {
    let args = Sys.args()
    
    if args.length < 2 {
        print("Error: missing argument")
        Sys.exit(1)
    }
    
    // ... process ...
    print("Done!")
    Sys.exit(0)
}
```

**Rust Codegen:**
```rust
std::process::exit(1)
```

**Notes:**
- This function never returns (terminates the process immediately)
- Exit code 0 conventionally means success
- Exit codes 1-255 indicate errors
- Destructors and cleanup code will **not** run after `Sys.exit()`

---

## Common Patterns

### CLI Tool with Arguments

```liva
main() {
    let args = Sys.args()
    
    if args.length < 3 {
        print("Usage: tool <command> <path>")
        print("Commands: search, list, count")
        Sys.exit(1)
    }
    
    let command = args[1]
    let path = args[2]
    
    if command == "list" {
        let entries, err = Dir.list(path)
        if err {
            print($"Error: {err}")
            Sys.exit(1)
        }
        for entry in entries {
            print(entry)
        }
    }
}
```

### Configuration from Environment

```liva
let debug = Sys.env("DEBUG")
let logLevel = Sys.env("LOG_LEVEL")

if debug != "" {
    print("Debug mode enabled")
}

if logLevel == "" {
    logLevel = "info"  // Default
}
```

---

## See Also

- [File I/O](../file-io.md) - File and directory operations
- [Console I/O](./io.md) - Console input/output
- [Standard Library Overview](./README.md)
