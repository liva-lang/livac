# DB Module

The `DB` module provides SQLite database operations for persistent data storage.

**Crate auto-injected:** `rusqlite = { version = "0.32", features = ["bundled"] }`

> The `bundled` feature includes SQLite itself, so no system-level SQLite installation is needed.

---

## Functions

### DB.open(path) → `connection, error`

Opens (or creates) a SQLite database file. **Fallible** — returns error if the file cannot be opened.

```liva
let db, err = DB.open("myapp.db")
if err {
    print("Cannot open database: " + err)
}
```

### DB.exec(db, sql) → `_, error`

Executes one or more SQL statements that don't return rows (CREATE, INSERT, UPDATE, DELETE). **Fallible**.

```liva
let _, err = DB.exec(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
```

### DB.exec(db, sql, params) → `_, error`

Executes a parameterized SQL statement. Parameters are passed as an array and bind to `?` placeholders. **Fallible**.

```liva
let _, err = DB.exec(db, "INSERT INTO users (name) VALUES (?)", ["Alice"])
```

### DB.query(db, sql) → `rows, error`

Queries the database and returns all matching rows. Each row is a `Map<string, string>` where keys are column names. **Fallible**.

```liva
let rows, err = DB.query(db, "SELECT * FROM users")
for row in rows {
    print(row.get("name") or "")
}
```

### DB.query(db, sql, params) → `rows, error`

Queries with parameterized SQL. **Fallible**.

```liva
let results, err = DB.query(db, "SELECT * FROM users WHERE name = ?", ["Alice"])
for row in results {
    print("Found: " + row.get("name"))
}
```

### DB.close(db)

Closes the database connection. After closing, the connection can no longer be used.

```liva
DB.close(db)
```

---

## Row Access

Query results are `Array<Map<string, string>>`. Each row is a map of column name → value (as string).

- `row.get("column")` — returns the column value as a string
- All SQL types (INTEGER, REAL, TEXT, BLOB, NULL) are converted to strings
- NULL values become empty strings

In string concatenation, `row.get("key")` automatically unwraps to a string:

```liva
print("Name: " + row.get("name"))  // works directly
```

For explicit defaults, use `or`:

```liva
let name = row.get("name") or "unknown"
```

---

## Complete Example

```liva
main() {
    // Open database
    let db, err = DB.open("demo.db")
    if err {
        print("Failed to open DB: " + err)
    }

    // Create table
    let _, err2 = DB.exec(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, email TEXT)")

    // Insert data
    let _, err3 = DB.exec(db, "INSERT INTO users (name, email) VALUES (?, ?)", ["Alice", "alice@example.com"])
    let _, err4 = DB.exec(db, "INSERT INTO users (name, email) VALUES (?, ?)", ["Bob", "bob@example.com"])

    // Query all
    let rows, err5 = DB.query(db, "SELECT * FROM users")
    for row in rows {
        print("ID: " + row.get("id") + " | Name: " + row.get("name") + " | Email: " + row.get("email"))
    }

    // Query with params
    let results, err6 = DB.query(db, "SELECT * FROM users WHERE name = ?", ["Alice"])
    for row in results {
        print("Found: " + row.get("name") + " (" + row.get("email") + ")")
    }

    // Close
    DB.close(db)
}
```

---

## Error Handling

All DB operations except `close` are fallible. Use the two-binding pattern:

```liva
let result, err = DB.exec(db, "INSERT ...")
if err {
    print("Error: " + err)
}
```

Common errors:
- `DB.open`: file permission denied, invalid path
- `DB.exec`: SQL syntax error, constraint violation
- `DB.query`: SQL syntax error, no such table/column

---

## Notes

- SQLite runs in-process — no server needed
- Database files are created automatically by `DB.open` if they don't exist
- All operations are synchronous
- Column values are always returned as strings — use `toInt()` / `toFloat()` for numeric operations
- Use `?` placeholders for parameterized queries to prevent SQL injection
