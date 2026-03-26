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
