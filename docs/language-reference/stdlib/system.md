# System Functions (Sys)

The `Sys` namespace provides system-level functions for CLI args, environment variables, and process control.

## `Sys.args()` → `[string]`

Get command-line arguments. `args[0]` is the program name; user args start at `args[1]`.

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

## `Sys.env(name: string)` → `string`

Read an environment variable. Returns empty string `""` if not set.

```liva
let home = Sys.env("HOME")
let apiKey = Sys.env("API_KEY")
if apiKey == "" {
    print("Warning: API_KEY not set")
}
```

Case-sensitive on Linux/macOS, case-insensitive on Windows.

## `Sys.exit(code: number)` → `never`

Terminate the program. Exit code 0 = success, non-zero = error. Destructors/cleanup will **not** run.

```liva
if args.length < 2 {
    print("Error: missing argument")
    Sys.exit(1)
}
```

## `Sys.exe()` → `string` *(v2.7.0+)*

Absolute path of the currently running executable. Returns empty string `""` if the path cannot be determined (extremely rare). Resolves symlinks on Unix; on Windows returns the canonical path.

```liva
let me = Sys.exe()
print($"Running from {me}")
// e.g. /usr/local/bin/livac
```

Useful for self-update flows, locating sibling files next to the binary, and writing portable launchers. Maps to `std::env::current_exe()` in the generated Rust.

## `Sys.os()` → `string` *(v2.7.0+)*

Operating system identifier. Returns Rust's `std::env::consts::OS` value:

| Value | Platform |
|-------|----------|
| `"linux"` | Linux |
| `"macos"` | macOS |
| `"windows"` | Windows |
| `"freebsd"`, `"netbsd"`, `"openbsd"`, `"dragonfly"` | BSDs |
| `"android"`, `"ios"` | Mobile targets |

```liva
if Sys.os() == "windows" {
    print("Use backslashes in paths")
}
```

## `Sys.arch()` → `string` *(v2.7.0+)*

CPU architecture identifier. Returns Rust's `std::env::consts::ARCH` value:

| Value | Architecture |
|-------|--------------|
| `"x86_64"` | 64-bit Intel/AMD |
| `"aarch64"` | 64-bit ARM (Apple Silicon, ARM servers) |
| `"x86"` | 32-bit Intel |
| `"arm"` | 32-bit ARM |
| `"riscv64"` | 64-bit RISC-V |

```liva
let target = $"{Sys.os()}-{Sys.arch()}"
// e.g. "linux-x86_64", "macos-aarch64"
```
