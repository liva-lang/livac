# Process Module

The `Process` module provides functions for executing system commands, spawning background processes, and process management.

**No external crates** — uses `std::process` from Rust's standard library.

---

## Functions

### Process.exec(command) → `string, error`

Executes a shell command and returns its stdout output. **Fallible** — returns error if the command fails or returns a non-zero exit code.

```liva
let output, err = Process.exec("echo hello")
if err {
    print($"Error: {err}")
} else {
    print(output)  // "hello"
}
```

```liva
// More complex commands
let files, err = Process.exec("ls -la /tmp")
let date, err2 = Process.exec("date +%Y-%m-%d")
let count, err3 = Process.exec("wc -l < file.txt")
```

### Process.spawn(command) → `number, error`

Spawns a command as a background process and returns its PID. **Fallible**.

```liva
let pid, err = Process.spawn("sleep 60")
if err {
    print($"Spawn error: {err}")
} else {
    print($"Started process with PID: {pid}")
}
```

### Process.pid() → `number`

Returns the PID of the current process.

```liva
let myPid = Process.pid()
print($"My PID: {myPid}")
```

### Process.exit(code)

Terminates the process immediately with the given exit code.

```liva
if criticalError {
    print("Fatal error, exiting")
    Process.exit(1)
}
```

---

## Complete Example

```liva
main() {
    print($"Process PID: {Process.pid()}")

    // Run a command
    let hostname, err = Process.exec("hostname")
    if !err {
        print($"Running on: {hostname}")
    }

    // Check disk space
    let disk, err2 = Process.exec("df -h / | tail -1")
    if !err2 {
        print($"Disk: {disk}")
    }

    // Spawn background task
    let pid, err3 = Process.spawn("sleep 10")
    if !err3 {
        print($"Background task PID: {pid}")
    }
}
```

---

## Error Handling

| Function | Fallible? | Error pattern |
|----------|-----------|---------------|
| `Process.exec` | Yes | `let output, err = Process.exec(cmd)` |
| `Process.spawn` | Yes | `let pid, err = Process.spawn(cmd)` |
| `Process.pid` | No | `let pid = Process.pid()` |
| `Process.exit` | No | `Process.exit(code)` |

---

## Platform Notes

Commands are executed via `sh -c` (Unix shell). On Linux/macOS this works with any shell command including pipes, redirects, and shell builtins.
