# Date Module

> **Since:** v1.6.0  
> **Crate:** `chrono` (auto-injected when `Date.*` is used)  
> **Rust type:** `chrono::NaiveDateTime`

The `Date` module provides first-class date/time support. When any `Date.*` function is used, the `chrono = "0.4"` crate is automatically added to the generated `Cargo.toml`.

---

## Static Constructors

### `Date.now()` → `Date`

Returns the current local date and time.

```liva
let now = Date.now()
print($"Current time: {now}")
```

**Rust codegen:**
```rust
let now = chrono::Local::now().naive_local();
```

---

### `Date.new(year, month, day)` → `Date`

Creates a date at midnight (00:00:00). Also accepts 6 arguments for full datetime.

```liva
let birthday = Date.new(1990, 6, 15)
let meeting = Date.new(2026, 3, 15, 14, 30, 0)
```

**Rust codegen:**
```rust
let birthday = chrono::NaiveDate::from_ymd_opt(1990, 6, 15).unwrap().and_hms_opt(0, 0, 0).unwrap();
let meeting = chrono::NaiveDate::from_ymd_opt(2026, 3, 15).unwrap().and_hms_opt(14, 30, 0).unwrap();
```

---

### `Date.parse(str, pattern)` → `Date, error`

Parses a string into a Date. **Fallible** — returns `(Date, error)` tuple.

Supports Liva-style patterns (auto-converted to chrono strftime):

| Liva Pattern | chrono Pattern | Meaning |
|-------------|---------------|---------|
| `YYYY` | `%Y` | 4-digit year |
| `MM` | `%m` | 2-digit month |
| `DD` | `%d` | 2-digit day |
| `HH` | `%H` | 24-hour hour |
| `mm` | `%M` | Minute |
| `ss` | `%S` | Second |

```liva
let date, err = Date.parse("2026-03-11", "YYYY-MM-DD")
if err == "" {
    print($"Parsed: {date}")
}

let dt, err2 = Date.parse("15/06/1990 14:30:00", "DD/MM/YYYY HH:mm:ss")
```

**Fallback behavior:** If parsing as `NaiveDateTime` fails, attempts `NaiveDate` parse (adds midnight). If both fail, returns epoch (1970-01-01) with error message.

---

### `Date.timestamp()` → `int`

Returns the current Unix epoch in milliseconds.

```liva
let ts = Date.timestamp()
print($"Epoch ms: {ts}")
```

---

## Properties

Access date components as `int` values:

| Property | Type | Description | Range |
|----------|------|-------------|-------|
| `d.year` | `int` | Year | e.g., 2026 |
| `d.month` | `int` | Month | 1–12 |
| `d.day` | `int` | Day of month | 1–31 |
| `d.hour` | `int` | Hour | 0–23 |
| `d.minute` | `int` | Minute | 0–59 |
| `d.second` | `int` | Second | 0–59 |

```liva
let now = Date.now()
print(now.year)    // 2026
print(now.month)   // 3
print(now.day)     // 23
print(now.hour)    // 14
```

**Rust codegen:** Uses trait methods `chrono::Datelike::year()`, `chrono::Timelike::hour()`, etc.

---

## Instance Methods

### `d.format(pattern)` → `string`

Formats the date using a pattern string. Supports the same Liva-style patterns as `Date.parse`.

```liva
let now = Date.now()
print(now.format("DD/MM/YYYY"))           // "23/03/2026"
print(now.format("YYYY-MM-DD HH:mm:ss")) // "2026-03-23 14:30:00"
```

---

### `d.add(n, unit)` → `Date`

Returns a new Date with time added. Original date is unchanged.

| Unit | Description |
|------|-------------|
| `"days"` | Calendar days |
| `"hours"` | Hours |
| `"minutes"` | Minutes |
| `"seconds"` | Seconds |
| `"weeks"` | Weeks (7 days) |

```liva
let now = Date.now()
let tomorrow = now.add(1, "days")
let nextWeek = now.add(7, "days")
let inTwoHours = now.add(2, "hours")
```

---

### `d.diff(other, unit)` → `int`

Returns the difference between two dates in the specified unit.

| Unit | Description |
|------|-------------|
| `"days"` | Calendar days |
| `"hours"` | Hours |
| `"minutes"` | Minutes |
| `"seconds"` | Seconds |
| `"weeks"` | Weeks |
| `"years"` | Approximate years (days / 365) |
| `"months"` | Approximate months (days / 30) |

```liva
let now = Date.now()
let birthday = Date.new(1990, 6, 15)

let ageDays = now.diff(birthday, "days")
let ageYears = now.diff(birthday, "years")
print($"Age: {ageYears} years ({ageDays} days)")
```

**Note:** `years` and `months` are approximate (integer division by 365/30).

---

### `d.toString()` → `string`

Returns the date in ISO 8601 format (`YYYY-MM-DDTHH:MM:SS`).

```liva
let now = Date.now()
let iso = now.toString()    // "2026-03-23T14:30:00"
```

---

## Comparisons

Date supports all comparison operators:

```liva
let now = Date.now()
let future = now.add(7, "days")
let past = Date.new(2020, 1, 1)

if future > now { print("Future!") }
if past < now { print("Past!") }
if now >= past { print("Now or after") }
if now != past { print("Different dates") }
```

**Rust:** Works natively — `chrono::NaiveDateTime` implements `PartialOrd` and `PartialEq`.

---

## String Interpolation

Date variables in string templates are auto-formatted as ISO 8601:

```liva
let now = Date.now()
print($"Today is {now}")  // "Today is 2026-03-23T14:30:00"
```

**Rust codegen:** `now.format("%Y-%m-%dT%H:%M:%S")`

---

## Type Annotation

Use `Date` as a type annotation:

```liva
let now: Date = Date.now()
```

**Rust type:** `chrono::NaiveDateTime`

---

## Complete Example

```liva
main() {
    let now = Date.now()
    let birthday = Date.new(1990, 6, 15)
    
    // Properties
    print($"Year: {now.year}, Month: {now.month}, Day: {now.day}")
    
    // Formatting
    print(now.format("DD/MM/YYYY HH:mm:ss"))
    
    // Arithmetic
    let nextWeek = now.add(7, "days")
    let yesterday = now.add(-1, "days")
    
    // Differences
    let age = now.diff(birthday, "years")
    print($"Age: {age} years")
    
    // Parsing (fallible)
    let parsed, err = Date.parse("2026-12-25", "YYYY-MM-DD")
    if err == "" {
        let daysUntil = parsed.diff(now, "days")
        print($"Christmas in {daysUntil} days")
    }
    
    // Comparisons
    if nextWeek > now {
        print("Time moves forward")
    }
    
    // Interpolation
    print($"Current: {now}")
}
```

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| `Date.parse` with invalid string | Returns epoch (1970-01-01) + error message |
| `Date.parse` with date-only string | Auto-adds midnight (00:00:00) |
| `Date.new` with invalid values | Rust panic (unwrap on None) |
| `Date.add` with unknown unit | Defaults to `"days"` |
| `Date.diff` with unknown unit | Defaults to `"days"` |

---

## Crate Auto-Injection

When `Date.*` is detected during desugaring, the `has_date` flag is set, and `chrono = "0.4"` is automatically added to the generated `Cargo.toml`. If `Log.*` is also used, chrono is shared (not duplicated).
