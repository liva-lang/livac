# CSV Module

The `CSV` module provides functions for reading, writing, and manipulating CSV (Comma-Separated Values) data. It includes a `Table` concept for working with header-based tabular data.

**No external crates** — implemented with pure Rust `std`.

---

## Functions

### CSV.read(path) → `[[string]], error`

Reads a CSV file and returns a 2D array of strings.

```liva
let data, err = CSV.read("data.csv")
if err {
    print($"Error: {err}")
} else {
    for row in data {
        print(row)
    }
}
```

Supports quoted fields, escaped quotes (`""`), and multiline awareness.

### CSV.read(path, separator) → `[[string]], error`

Reads a delimited file with a custom separator (e.g., TSV).

```liva
let data, err = CSV.read("data.tsv", "\t")
```

### CSV.write(path, data) → `bool, error`

Writes a 2D array of strings to a CSV file. Automatically quotes fields containing commas, quotes, or newlines.

```liva
let data = [
    ["name", "age", "city"],
    ["Alice", "30", "Madrid"],
    ["Bob", "25", "Barcelona"]
]
let ok, err = CSV.write("output.csv", data)
if err {
    print($"Write error: {err}")
}
```

### CSV.readTable(path) → `Table, error`

Reads a CSV file where the **first row is treated as headers**. Returns a `Table` (array of maps), where each row is a `Map<string, string>` keyed by header names.

```liva
let table, err = CSV.readTable("ventas.csv")
if !err {
    for row in table {
        print($"{row.get(\"producto\")}: {row.get(\"ventas\")}")
    }
}
```

### CSV.writeTable(path, table) → `bool, error`

Writes a `Table` back to a CSV file. Headers are extracted from the first row's keys (sorted alphabetically for determinism).

```liva
CSV.writeTable("resultado.csv", table)
```

### CSV.parse(text) → `[[string]]`

Parses a CSV string into a 2D array. **Pure function** — no error binding needed.

```liva
let text = "name,age\nAlice,30\nBob,25"
let rows = CSV.parse(text)
// rows = [["name", "age"], ["Alice", "30"], ["Bob", "25"]]
```

### CSV.stringify(rows) → `string`

Converts a 2D array back to a CSV string. **Pure function** — automatically quotes fields as needed.

```liva
let rows = [["name", "age"], ["Alice", "30"]]
let csv = CSV.stringify(rows)
print(csv)
// name,age
// Alice,30
```

---

## Table Operations

A `Table` is internally `[Map<string, string>]` — an array of maps where each map represents a row keyed by column headers.

### CSV.headers(table) → `[string]`

Returns the sorted list of column names from the table.

```liva
let table, err = CSV.readTable("data.csv")
let headers = CSV.headers(table)
print(headers)  // ["age", "city", "name"]
```

### CSV.column(table, colName) → `[string]`

Extracts all values from a specific column.

```liva
let names = CSV.column(table, "name")
print(names)  // ["Alice", "Bob"]

// Combine with array methods for calculations
let ventas = CSV.column(table, "ventas").map(x => parseInt(x) or 0)
let total = ventas.sum()
print($"Total: {total}")
```

### Table filtering and sorting

Since `Table` is `[Map<string, string>]`, you can use standard array methods:

```liva
// Filter rows
let europeos = table.filter(row => row.get("region") == "Europa")

// Sort by column (ascending)
let sorted = table.sortBy(row => row.get("ventas"))

// Group by column
let grupos = table.groupBy(row => row.get("producto"))
```

---

## Complete Example

```liva
main() {
    // Read CSV with headers
    let table, err = CSV.readTable("ventas.csv")
    if err {
        print($"Error: {err}")
        return
    }

    // Inspect structure
    let headers = CSV.headers(table)
    print($"Columns: {headers}")
    print($"Rows: {table.length}")

    // Extract and calculate
    let ventas = CSV.column(table, "ventas").map(x => parseInt(x) or 0)
    let total = ventas.sum()
    print($"Total ventas: {total}")

    // Filter and export
    let altas = table.filter(row => parseInt(row.get("ventas") or "0") or 0 > 100)
    let ok, writeErr = CSV.writeTable("altas.csv", altas)
    if writeErr {
        print($"Write error: {writeErr}")
    }
}
```

---

## Error Handling

| Function | Fallible? | Error pattern |
|----------|-----------|---------------|
| `CSV.read` | Yes | `let data, err = CSV.read(path)` |
| `CSV.write` | Yes | `let ok, err = CSV.write(path, data)` |
| `CSV.readTable` | Yes | `let table, err = CSV.readTable(path)` |
| `CSV.writeTable` | Yes | `let ok, err = CSV.writeTable(path, table)` |
| `CSV.parse` | No | `let rows = CSV.parse(text)` |
| `CSV.stringify` | No | `let csv = CSV.stringify(rows)` |
| `CSV.headers` | No | `let h = CSV.headers(table)` |
| `CSV.column` | No | `let col = CSV.column(table, name)` |

---

## CSV Parsing Details

The built-in parser handles:
- **Comma-separated** fields (default) or custom separators
- **Quoted fields**: `"Hello, World"` → `Hello, World`
- **Escaped quotes**: `"She said ""hello"""` → `She said "hello"`
- **Empty lines**: automatically filtered
- **Whitespace trimming**: leading/trailing spaces in fields are trimmed
