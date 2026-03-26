# Date Module

> **Since:** v1.6.0 — `chrono = "0.4"` auto-injected when `Date.*` is used.

## Static Constructors

### `Date.now()` → `Date`

Returns current local date and time.

```liva
let now = Date.now()
```

### `Date.new(year, month, day [, hour, minute, second])` → `Date`

Creates a date. 3 args → midnight, 6 args → full datetime.

```liva
let birthday = Date.new(1990, 6, 15)
let meeting = Date.new(2026, 3, 15, 14, 30, 0)
```

### `Date.parse(str, pattern)` → `(Date, string)`

**Fallible** — returns `(Date, error)` tuple. On failure returns epoch (1970-01-01) with error message.

| Liva Pattern | Meaning |
|-------------|---------|
| `YYYY` | 4-digit year |
| `MM` | 2-digit month |
| `DD` | 2-digit day |
| `HH` | 24-hour hour |
| `mm` | Minute |
| `ss` | Second |

```liva
let date, err = Date.parse("2026-03-11", "YYYY-MM-DD")
if !err {
    print($"Parsed: {date}")
}
```

If parsing as datetime fails, attempts date-only parse (adds midnight).

### `Date.timestamp()` → `int`

Current Unix epoch in milliseconds.

## Properties

| Property | Type | Range |
|----------|------|-------|
| `d.year` | `int` | e.g., 2026 |
| `d.month` | `int` | 1–12 |
| `d.day` | `int` | 1–31 |
| `d.hour` | `int` | 0–23 |
| `d.minute` | `int` | 0–59 |
| `d.second` | `int` | 0–59 |

```liva
let now = Date.now()
print(now.year)    // 2026
print(now.month)   // 3
```

## Instance Methods

### `d.format(pattern)` → `string`

Uses same pattern tokens as `Date.parse`.

```liva
print(now.format("DD/MM/YYYY"))           // "23/03/2026"
print(now.format("YYYY-MM-DD HH:mm:ss")) // "2026-03-23 14:30:00"
```

### `d.add(n, unit)` → `Date`

Returns new Date with time added. Units: `"days"`, `"hours"`, `"minutes"`, `"seconds"`, `"weeks"`.

```liva
let tomorrow = now.add(1, "days")
let inTwoHours = now.add(2, "hours")
```

### `d.diff(other, unit)` → `int`

Difference between two dates. Units: `"days"`, `"hours"`, `"minutes"`, `"seconds"`, `"weeks"`, `"years"` (approx ÷365), `"months"` (approx ÷30).

```liva
let ageDays = now.diff(birthday, "days")
let ageYears = now.diff(birthday, "years")
```

### `d.toString()` → `string`

ISO 8601 format: `"2026-03-23T14:30:00"`.

## Comparisons

All comparison operators work: `>`, `<`, `>=`, `<=`, `==`, `!=`.

```liva
if future > now { print("Future!") }
```

## String Interpolation

Date in `$"..."` auto-formats as ISO 8601:

```liva
print($"Today is {now}")  // "Today is 2026-03-23T14:30:00"
```

## Type Annotation

```liva
let now: Date = Date.now()
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| `Date.parse` invalid string | Returns epoch + error message |
| `Date.parse` date-only string | Auto-adds midnight |
| `Date.new` invalid values | Rust panic |
| Unknown unit in `add`/`diff` | Defaults to `"days"` |
