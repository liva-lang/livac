# Operators

> Arithmetic, comparison, logical, and string operators are in SKILL.md. This file covers precedence, compound assignment, ternary, ranges, and method reference operator.

## Operator Precedence (Highest → Lowest)

| Prec | Operator | Description | Assoc |
|------|----------|-------------|-------|
| 1 | `()` `[]` `.` `::` | Grouping, index, member, method ref | L→R |
| 2 | `-` `!` `not` `await` | Unary | R→L |
| 3 | `*` `/` `%` | Multiply, divide, modulo | L→R |
| 4 | `+` `-` | Add, subtract | L→R |
| 5 | `..` | Range | L→R |
| 6 | `<` `<=` `>` `>=` | Comparison | L→R |
| 7 | `==` `!=` | Equality | L→R |
| 8 | `and` `&&` | Logical AND | L→R |
| 9 | `or` `\|\|` | Logical OR | L→R |
| 10 | `? :` | Ternary | R→L |
| 11 | `=` `+=` `-=` `*=` `/=` `%=` | Assignment | R→L |

## Compound Assignment

```liva
x += 5      // x = x + 5
x -= 3      // x = x - 3
x *= 2      // x = x * 2
x /= 4      // x = x / 4
x %= 5      // x = x % 5
```

Works with member access and array indexing:

```liva
c.count += 1        // Member access
arr[0] += 10        // Array index
```

> **No `++` or `--`** — use `x += 1`.

## Ternary Operator

```liva
let status = age >= 18 ? "Adult" : "Minor"
let max = a > b ? a : b

// Nested (avoid for readability)
let grade = score >= 90 ? "A" : score >= 80 ? "B" : score >= 70 ? "C" : "F"

// With fail (in fallible functions)
let discount = age >= 65 ? 0.2 : fail "No discount"
```

## Range Operators

```liva
// Exclusive end (..)
for i in 1..6 { }       // 1, 2, 3, 4, 5

// Inclusive end (..=)
for i in 1..=5 { }      // 1, 2, 3, 4, 5

// With variables
for i in start..end { }
```

## Method Reference Operator (`::`)

Binds an instance method as a callback:

```liva
let greetings = names.map(fmt::format)
// Equivalent to: names.map(x => fmt.format(x))

names.forEach(logger::log)
names.filter(validator::isValid)
```

## Logical Operators

Both word and symbol forms supported:

| Word | Symbol | Operation |
|------|--------|-----------|
| `and` | `&&` | Logical AND |
| `or` | `\|\|` | Logical OR |
| `not` | `!` | Logical NOT |

Short-circuit evaluation applies to both forms.
