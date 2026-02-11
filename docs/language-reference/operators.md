# Operators

Complete reference for all operators in Liva: arithmetic, logical, comparison, and special operators.

## Table of Contents
- [Arithmetic Operators](#arithmetic-operators)
- [Comparison Operators](#comparison-operators)
- [Logical Operators](#logical-operators)
- [Assignment Operators](#assignment-operators)
- [Range Operator](#range-operator)
- [Ternary Operator](#ternary-operator)
- [Member Access](#member-access)
- [Index Access](#index-access)
- [Operator Precedence](#operator-precedence)

---

## Arithmetic Operators

### Basic Arithmetic

| Operator | Operation | Example | Result |
|----------|-----------|---------|--------|
| `+` | Addition | `10 + 5` | `15` |
| `-` | Subtraction | `20 - 8` | `12` |
| `*` | Multiplication | `6 * 7` | `42` |
| `/` | Division | `100 / 4` | `25` |
| `%` | Modulo | `17 % 3` | `2` |

```liva
let sum = 10 + 5        // 15
let diff = 20 - 8       // 12
let product = 6 * 7     // 42
let quotient = 100 / 4  // 25
let remainder = 17 % 3  // 2
```

### Unary Minus

```liva
let positive = 10
let negative = -positive  // -10
```

### Operator Chaining

```liva
let result = 10 + 5 * 2  // 20 (multiplication first)
let result2 = (10 + 5) * 2  // 30 (parentheses first)
```

---

## Comparison Operators

### Comparison Table

| Operator | Meaning | Example | Result |
|----------|---------|---------|--------|
| `==` | Equal | `5 == 5` | `true` |
| `!=` | Not equal | `5 != 3` | `true` |
| `<` | Less than | `3 < 5` | `true` |
| `<=` | Less or equal | `5 <= 5` | `true` |
| `>` | Greater than | `7 > 5` | `true` |
| `>=` | Greater or equal | `5 >= 5` | `true` |

```liva
let isEqual = 5 == 5          // true
let notEqual = 5 != 3         // true
let lessThan = 3 < 5          // true
let lessOrEqual = 5 <= 5      // true
let greaterThan = 7 > 5       // true
let greaterOrEqual = 5 >= 5   // true
```

### String Comparison

```liva
let name1 = "Alice"
let name2 = "Alice"
let same = name1 == name2  // true

let different = "Bob" != "Alice"  // true
```

### Boolean Comparison

```liva
let a = true
let b = false

let same = a == true        // true
let different = a != b      // true
```

---

## Logical Operators

### Word-Based Operators

Liva uses **English keywords** for logical operations:

| Operator | Symbol Alternative | Operation | Example |
|----------|-------------------|-----------|---------|
| `and` | `&&` | Logical AND | `a and b` |
| `or` | `\|\|` | Logical OR | `a or b` |
| `not` | `!` | Logical NOT | `not a` |

```liva
let canVote = age >= 18 and isRegistered
let shouldShow = isPremium or isTrial
let isInvalid = not isValid
```

### Symbol-Based Operators (Also Supported)

```liva
let canAccess = isLoggedIn && hasPermission
let showBanner = isNewUser || hasDiscount
let isHidden = !isVisible
```

### Logical AND (`and` / `&&`)

```liva
let age = 25
let hasLicense = true

let canDrive = age >= 18 and hasLicense  // true

if age >= 18 and hasLicense {
  print("Can drive")
}
```

### Logical OR (`or` / `||`)

```liva
let isWeekend = day == "Saturday" or day == "Sunday"
let canEnter = isVIP or hasPaidTicket

if isAdmin or isModerator {
  print("Has moderation powers")
}
```

### Logical NOT (`not` / `!`)

```liva
let isActive = true
let isInactive = not isActive  // false

if not hasErrors {
  proceed()
}
```

### Short-Circuit Evaluation

```liva
// AND: stops if first is false
let result = expensiveCheck() and anotherCheck()

// OR: stops if first is true
let result2 = quickCheck() or slowCheck()
```

---

## Assignment Operators

### Simple Assignment

```liva
let x = 10
x = 20  // Reassignment
```

### No Compound Assignment

Liva **does not support** `+=`, `-=`, etc. Use explicit assignment:

```liva
// ❌ Not supported
x += 5
counter++

// ✅ Use explicit assignment
x = x + 5
counter = counter + 1
```

---

## Range Operator

### Basic Range (`..`)

```liva
for i in 1..10 {
  print(i)  // 1, 2, 3, 4, 5, 6, 7, 8, 9
}
```

### Range is Exclusive on End

```liva
// 1..6 means [1, 2, 3, 4, 5] (6 is excluded)
for i in 1..6 {
  print(i)  // Prints 1, 2, 3, 4, 5
}
```

### Range with Variables

```liva
let start = 0
let end = 5

for i in start..end {
  print(i)  // 0, 1, 2, 3, 4
}
```

---

## Ternary Operator

### Basic Ternary

```liva
let status = age >= 18 ? "Adult" : "Minor"
let max = a > b ? a : b
```

### With Fail

```liva
let discount = age >= 65 ? 0.2 : age < 18 ? fail "No discount for minors" : 0.0
```

### Nested Ternary

```liva
let grade = score >= 90 ? "A" : score >= 80 ? "B" : score >= 70 ? "C" : "F"
```

**Warning**: Nested ternaries can reduce readability. Consider using `if-else-if` for complex logic.

---

## Member Access

### Dot Notation (`.`)

```liva
let user = { name: "Alice", age: 25 }

let userName = user.name     // "Alice"
let userAge = user.age       // 25
```

### Method Calls

```liva
let person = Person("Alice", 30)
let greeting = person.greet()  // Call method
```

### Chaining

```liva
let fullName = user.profile.name.toUpperCase()
let firstItem = items[0].name.toLowerCase()
```

### Method Reference Operator (`::`)

**New in v1.1.0.** The `::` operator creates a reference to an instance method, binding it to a specific object:

```liva
let fmt = Formatter("Hello")

// :: binds fmt.format as a callback
let greetings = names.map(fmt::format)
// Equivalent to: names.map(x => fmt.format(x))

// Works with forEach, filter, find, some, every
names.forEach(logger::log)
names.filter(validator::isValid)
```

---

## Index Access

### Array Indexing (`[]`)

```liva
let numbers = [10, 20, 30, 40]

let first = numbers[0]   // 10
let second = numbers[1]  // 20
```

### Object/Map Access

```liva
let user = { name: "Alice", age: 25 }
let name = user["name"]  // "Alice"
```

### Computed Index

```liva
let index = 2
let value = numbers[index]  // 30
```

---

## Operator Precedence

### Precedence Table (Highest to Lowest)

| Precedence | Operator | Description | Associativity |
|------------|----------|-------------|---------------|
| 1 | `()` `[]` `.` `::` | Grouping, indexing, member access, method ref | Left-to-right |
| 2 | `-` `!` `not` `await` | Unary operators | Right-to-left |
| 3 | `*` `/` `%` | Multiplication, division, modulo | Left-to-right |
| 4 | `+` `-` | Addition, subtraction | Left-to-right |
| 5 | `..` | Range | Left-to-right |
| 6 | `<` `<=` `>` `>=` | Comparison | Left-to-right |
| 7 | `==` `!=` | Equality | Left-to-right |
| 8 | `and` `&&` | Logical AND | Left-to-right |
| 9 | `or` `\|\|` | Logical OR | Left-to-right |
| 10 | `? :` | Ternary | Right-to-left |
| 11 | `=` | Assignment | Right-to-left |

### Examples

```liva
// Multiplication before addition
let result = 10 + 5 * 2  // 20, not 30

// Comparison before logical
let check = age > 18 and hasLicense  // (age > 18) and hasLicense

// Parentheses override
let result2 = (10 + 5) * 2  // 30
```

### Best Practice: Use Parentheses

```liva
// ✅ Good: Explicit with parentheses
let result = (a + b) * c
let check = (age >= 18) and (hasLicense == true)

// ⚠️ Works but less clear
let result = a + b * c
let check = age >= 18 and hasLicense == true
```

---

## Best Practices

### Prefer Word Operators

```liva
// ✅ Good: Readable word operators
if age >= 18 and isRegistered {
  print("Can vote")
}

// ⚠️ Acceptable: Symbol operators
if age >= 18 && isRegistered {
  print("Can vote")
}
```

### Use Parentheses for Clarity

```liva
// ✅ Good: Clear intent
let result = (a + b) * c

// ❌ Bad: Ambiguous
let result = a + b * c
```

### Avoid Deep Nesting

```liva
// ❌ Bad: Hard to read
let result = a > b ? c > d ? e : f : g > h ? i : j

// ✅ Good: Use if-else-if
let result = ""
if a > b {
  result = c > d ? e : f
} else {
  result = g > h ? i : j
}
```

### Explicit Comparison

```liva
// ✅ Good: Explicit
if isActive == true {
  // ...
}

// ⚠️ Acceptable: Implicit (for booleans)
if isActive {
  // ...
}
```

---

## Summary

### Arithmetic
```liva
+ - * / %
-x  // unary minus
```

### Comparison
```liva
== != < <= > >=
```

### Logical
```liva
and or not
&&  ||  !
```

### Other
```liva
..      // range
? :     // ternary
.       // member access
[]      // index access
=       // assignment
```

### Quick Reference

```liva
// Arithmetic
let sum = 10 + 5
let product = 6 * 7
let remainder = 17 % 3

// Comparison
let isEqual = a == b
let isGreater = a > b

// Logical
let canVote = age >= 18 and isRegistered
let showBanner = isVIP or hasCoupon
let isInvalid = not isValid

// Range
for i in 1..10 { }

// Ternary
let status = age >= 18 ? "Adult" : "Minor"

// Member/Index
let name = user.name
let first = items[0]
```

---

**Next**: [String Templates →](string-templates.md)

**See Also**:
- [Variables](variables.md)
- [Control Flow](control-flow.md)
- [Functions](functions.md)
