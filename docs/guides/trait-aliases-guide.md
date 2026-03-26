# Trait Aliases Guide

Liva generics support **trait aliases** (simple, recommended) and **granular traits** (precise control). You can mix both.

## Trait Aliases (Recommended)

| Alias | Expands To | Use Case |
|-------|-----------|----------|
| `Numeric` | `Add + Sub + Mul + Div + Rem + Neg` | Any arithmetic |
| `Comparable` | `Ord + Eq` | Comparisons, sorting |
| `Number` | `Numeric + Comparable` | Full number operations |
| `Printable` | `Display + Debug` | Formatting, debugging |

```liva
sum<T: Numeric>(a: T, b: T): T => a + b
max<T: Comparable>(a: T, b: T): T {
    if a > b { return a }
    return b
}
clamp<T: Number>(value: T, min: T, max: T): T {
    if value < min { return min }
    if value > max { return max }
    return value
}
show<T: Printable>(value: T) {
    print(value)
}
```

## Granular Traits

**Arithmetic:** `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`
**Comparison:** `Eq`, `Ord`
**Utilities:** `Display`, `Debug`, `Clone`, `Copy`, `Not`

Use when you need precise control:

```liva
addOnly<T: Add>(a: T, b: T): T => a + b
isLessThan<T: Ord>(a: T, b: T): bool => a < b
areEqual<T: Eq>(a: T, b: T): bool => a == b
```

## Mixing Approaches

```liva
formatComparison<T: Comparable + Display>(a: T, b: T): string {
    if a == b { return $"Equal: {a}" }
    if a > b { return $"{a} > {b}" }
    return $"{a} < {b}"
}

formatRange<T: Number + Display>(min: T, max: T): string {
    return $"Range: [{min}, {max}]"
}
```

## Decision Tree

| Need | Use |
|------|-----|
| Arithmetic only | `Numeric` |
| Comparisons only | `Comparable` |
| Both arithmetic + comparisons | `Number` |
| Need to format/print | Add `Printable` |
| Very specific operation (e.g., only `+`) | Granular: `Add` |
| Complex requirements | Mix aliases + granular |

## Best Practices

- **Start with aliases** — simpler, covers most cases
- **Use granular when precision matters** — e.g., `Add` when only `+` is needed
- **Don't over-constrain** — `increment<T: Add>` not `increment<T: Numeric>`
- **Don't under-constrain** — `clamp<T: Number>` not `clamp<T: Numeric>` (needs comparison)

## Summary

| Scenario | Constraint | Example |
|----------|-----------|---------|
| General arithmetic | `Numeric` | `sum<T: Numeric>` |
| General comparison | `Comparable` | `max<T: Comparable>` |
| Arithmetic + comparison | `Number` | `clamp<T: Number>` |
| Only addition | `Add` | `increment<T: Add>` |
| Only ordering | `Ord` | `isGreater<T: Ord>` |
| Display + comparison | `Comparable + Display` | `formatCompare<T: Comparable + Display>` |
| All ops + formatting | `Number + Printable` | `debugCalc<T: Number + Printable>` |
