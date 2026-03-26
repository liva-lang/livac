# File & Directory I/O

## File Methods

| Method | Signature | Notes |
|--------|-----------|-------|
| `File.read(path)` | `(string?, Error?)` | Returns file content as string |
| `File.write(path, content)` | `(bool?, Error?)` | Creates/overwrites file; creates parent dirs |
| `File.append(path, content)` | `(bool?, Error?)` | Appends to end; creates file if not exists |
| `File.exists(path)` | `bool` | **No error binding** — infallible |
| `File.delete(path)` | `(bool?, Error?)` | Irreversible; does NOT work on directories |
| `File.copy(src, dest)` | `(bool?, Error?)` | Overwrites dest if exists; preserves perms on Unix |
| `File.move(src, dest)` | `(bool?, Error?)` | Moves/renames; may fail across mount points |
| `File.size(path)` | `(int?, Error?)` | Size in bytes |
| `File.extension(path)` | `string` | **No error binding**; returns ext without dot (`"archive.tar.gz"` → `"gz"`) |
| `File.readLines(path)` | `([string]?, Error?)` | Splits on `\n`/`\r\n`; empty file → `[""]` |
| `File.writeLines(path, lines)` | `(bool?, Error?)` | Joins with `\n`; no trailing newline |

All params are `string` except `File.writeLines` which takes `(path: string, lines: [string])`.

## Dir Methods

| Method | Signature | Notes |
|--------|-----------|-------|
| `Dir.list(path)` | `([string]?, Error?)` | Sorted entry names only (not full paths); excludes `.`/`..` |
| `Dir.isDir(path)` | `bool` | **No error binding** |
| `Dir.exists(path)` | `bool` | **No error binding**; `true` only for directories (unlike `File.exists` which matches both) |
| `Dir.create(path)` | `(bool?, Error?)` | Creates all parents (like `mkdir -p`); succeeds if already exists |
| `Dir.delete(path)` | `(bool?, Error?)` | ⚠️ Recursive delete — equivalent to `rm -rf` |
| `Dir.listRecursive(path)` | `([string]?, Error?)` | All relative paths recursively, sorted alphabetically |
| `Dir.walk(path)` | `([string]?, Error?)` | Alias for `Dir.listRecursive` — identical behavior |

## Error Handling Pattern

All fallible operations return `(value?, Error?)` — use error binding:

```liva
let content, err = File.read("config.json")
if err {
    print($"Failed: {err}")
    return
}
// content is safe to use
```

Infallible operations (return simple `bool` or `string`, no error binding):
`File.exists`, `File.extension`, `Dir.isDir`, `Dir.exists`.

### Common Error Messages

| Error | Cause |
|-------|-------|
| `No such file or directory (os error 2)` | Path doesn't exist |
| `Permission denied (os error 13)` | Insufficient permissions |
| `Is a directory (os error 21)` | Used `File.read/delete` on a directory |
| `Not a directory (os error 20)` | Used `Dir.list` on a file |
| `No space left on device (os error 28)` | Disk full |
| `Invalid cross-device link (os error 18)` | `File.move` across filesystems |
| `File exists (os error 17)` | `Dir.create` on a path that is a file |

## Patterns

### Safe read with fallback

```liva
let config, err = File.read("config.json")
if err {
    config = "{\"default\": true}"
}
```

### Check before overwrite

```liva
if File.exists("important.txt") {
    print("File exists — refusing to overwrite")
} else {
    File.write("important.txt", data)
}
```

### Backup before modify

```liva
updateConfig(newConfig: string) {
    let original, readErr = File.read("config.json")
    if !readErr {
        File.copy("config.json", "config.json.backup")
    }
    let ok, writeErr = File.write("config.json", newConfig)
    if writeErr {
        print($"Failed to update config: {writeErr}")
    }
}
```

### Logging (append)

```liva
log(message: string) {
    let entry = $"\n[LOG] {message}"
    let ok, err = File.append("app.log", entry)
    if err {
        print($"Failed to write log: {err}")
    }
}
```

### Read lines and process

```liva
let lines, err = File.readLines("data.csv")
if !err {
    print($"Read {lines.length} lines")
    for line in lines {
        print(line)
    }
}
```

### Write array of lines

```liva
let rows = ["name,age,city", "Alice,30,NYC", "Bob,25,LAX"]
let ok, err = File.writeLines("output.csv", rows)
if err {
    print($"Write error: {err}")
}
```

### Ensure directory exists before write

```liva
if !Dir.exists("./output") {
    let ok, err = Dir.create("./output")
    if err {
        print($"Cannot create dir: {err}")
        return
    }
}
File.write("./output/result.txt", data)
```

### Recursive file search

```liva
let files, err = Dir.listRecursive("./src")
if !err {
    for f in files {
        if File.extension(f) == "liva" {
            print($"Found: {f}")
        }
    }
}
```

### Manual recursive traversal

```liva
searchDir(query: string, dirPath: string) {
    let entries, err = Dir.list(dirPath)
    if err { return }

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

### Temporary file cleanup

```liva
processWithTemp() {
    let tempFile = "temp_processing.txt"
    File.write(tempFile, intermediateData)
    let data, err = File.read(tempFile)
    File.delete(tempFile)
    return data
}
```

## Limitations

- **Synchronous only** — no async file I/O
- **UTF-8 only** — no binary file support
- **No streaming** — entire file read/written at once
- **No path manipulation** — use string concatenation for paths
- **No permissions/timestamps** — cannot read or set file metadata
