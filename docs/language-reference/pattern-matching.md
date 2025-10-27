# Pattern Matching

> **Version:** v0.10.5  
> **Status:** Production Ready  
> **Last Updated:** 2025-01-24  
> **New in v0.10.5:** Or-patterns, Enhanced exhaustiveness checking

Pattern matching provides a powerful way to inspect and destructure values in Liva. The `switch` expression allows you to match values against patterns and execute different code paths.

---

## Table of Contents

- [Overview](#overview)
- [Switch Expressions](#switch-expressions)
- [Pattern Types](#pattern-types)
- [Pattern Guards](#pattern-guards)
- [Exhaustiveness](#exhaustiveness)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Limitations](#limitations)

---

## Overview

Pattern matching in Liva combines the simplicity of traditional switch statements with the power of modern pattern matching systems. Key features:

- **Switch as Expression** - Returns a value, can be used anywhere an expression is valid
- **Multiple Pattern Types** - Literals, wildcards, bindings, ranges, or-patterns
- **Or-Patterns** - Match multiple values with a single arm: `1 | 2 | 3 => ...` (v0.10.5)
- **Pattern Guards** - Add conditional logic with `if` clauses
- **Type Safety** - All arms must return the same type
- **Exhaustiveness Checking** - Compiler ensures all cases are handled for bool, int, and string types (v0.10.5)

---

## Switch Expressions

### Basic Syntax

```liva
let result = switch value {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression
};
```

**Key Points:**
- Switch expressions **must** have at least one arm
- Each arm has a pattern, optional guard, and body
- Bodies can be single expressions or blocks
- Trailing commas are optional

### Expression vs Statement

Switch can be used as an expression (returns a value):

```liva
let grade = switch score {
    90..=100 => "A",
    80..=89 => "B",
    70..=79 => "C",
    _ => "F"
};
```

Or with block bodies for side effects:

```liva
let message = switch status {
    200 => {
        print("Success!");
        "OK"
    },
    404 => {
        print("Not found");
        "Error"
    },
    _ => "Unknown"
};
```

---

## Pattern Types

### 1. Literal Patterns

Match exact values:

```liva
let result = switch x {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"
};
```

**Supported Types:**
- `int`: `42`, `-10`, `0`
- `float`: `3.14`, `-0.5`
- `bool`: `true`, `false`
- `string`: `"hello"`, `"world"`
- `char`: `'a'`, `'Z'`

### 2. Wildcard Pattern

Matches anything (catch-all):

```liva
let result = switch value {
    1 => "one",
    2 => "two",
    _ => "something else"  // Wildcard catches everything
};
```

**Best Practice:** Always include a wildcard arm as the last pattern to ensure exhaustiveness.

### 3. Binding Pattern

Captures the matched value in a variable:

```liva
let doubled = switch num {
    0 => 0,
    n => n * 2  // 'n' binds to the value
};
```

The bound variable can be used in the arm's body:

```liva
let description = switch age {
    x if x < 13 => $"Child: {x} years old",
    x if x < 20 => $"Teenager: {x} years old",
    x => $"Adult: {x} years old"
};
```

### 4. Range Patterns

Match ranges of values:

```liva
let category = switch score {
    90..=100 => "A",    // Inclusive range (90 to 100)
    80..=89 => "B",     // 80 to 89
    70..=79 => "C",     // 70 to 79
    60..=69 => "D",     // 60 to 69
    0..=59 => "F",      // 0 to 59
    _ => "Invalid"
};
```

**Range Syntax:**

| Syntax | Meaning | Example | Matches |
|--------|---------|---------|---------|
| `a..=b` | Inclusive range | `1..=10` | 1, 2, ..., 10 |
| `a..b` | Exclusive end | `1..10` | 1, 2, ..., 9 |
| `a..` | Open end | `18..` | 18, 19, 20, ... |
| `..=b` | Up to and including | `..=100` | ..., 99, 100 |
| `..b` | Up to but not including | `..18` | ..., 16, 17 |

**Example: Age Categories**

```liva
let category = switch age {
    ..13 => "child",       // Less than 13
    13..20 => "teenager",  // 13 to 19
    20..65 => "adult",     // 20 to 64
    65.. => "senior"       // 65 and above
};
```

---

## Pattern Guards

Add conditional logic to patterns with `if` clauses:

```liva
let status = switch value {
    x if x < 0 => "negative",
    x if x == 0 => "zero",
    x if x > 0 => "positive"
};
```

**Multiple Conditions:**

```liva
let category = switch (age, hasLicense) {
    (a, true) if a >= 18 => "can drive",
    (a, false) if a >= 18 => "adult without license",
    (a, _) if a < 18 => "too young",
    _ => "invalid"
};
```

---

### 5. Or-Patterns ✅ Implemented (v0.10.5)

Match multiple patterns with the same action using the `|` operator:

```liva
let category = switch num {
    1 | 2 | 3 => "small",
    4 | 5 | 6 => "medium",
    7 | 8 | 9 => "large",
    _ => "out of range"
};
```

**String Or-Patterns:**

```liva
let isWeekend = switch day {
    "Saturday" | "Sunday" => true,
    _ => false
};
```

**Status Codes:**

```liva
let statusType = switch httpCode {
    200 | 201 | 204 => "success",
    400 | 401 | 403 | 404 => "client error",
    500 | 502 | 503 => "server error",
    _ => "other"
};
```

**Benefits:**
- More concise than multiple arms with the same body
- Reduces code duplication
- Makes patterns more readable

**Limitations (v0.10.5):**
- Or-patterns with bindings must bind the same variables in all alternatives
- Cannot mix wildcard `_` with specific patterns in an or-pattern (use separate arms instead)

**Example: Valid Or-Patterns with Exhaustiveness:**

```liva
// ✅ Good: All paths have wildcard
let result = switch status {
    "active" | "running" | "started" => "operational",
    "stopped" | "paused" | "idle" => "not operational",
    _ => "unknown"  // Required for string exhaustiveness
};
```

**Note:** Or-patterns still require exhaustiveness checking - they don't automatically make a switch exhaustive unless combined with a wildcard or binding pattern.

---

## Pattern Guards

Add conditional logic to patterns with `if` clauses:

```liva
let status = switch value {
    x if x < 0 => "negative",
    x if x == 0 => "zero",
    x if x > 0 => "positive"
};
```

**Multiple Conditions:**

```liva
let category = switch (age, hasLicense) {
    (a, true) if a >= 18 => "can drive",
    (a, false) if a >= 18 => "adult without license",
    (a, _) if a < 18 => "too young",
    _ => "invalid"
};
```

**Guard Evaluation:**
- Guards are evaluated in order
- First matching guard wins
- Guards can use any boolean expression
- Guards have access to bound variables

---

## Exhaustiveness

> **Status:** ✅ Implemented for `bool`, `int`, and `string` types (v0.10.5)  
> **Future:** Array/tuple patterns, or-patterns (v0.10.6+)

The compiler checks that all possible values are covered in pattern matching. This prevents runtime errors from unhandled cases.

### Boolean Exhaustiveness ✅ Implemented (v0.9.5)

For `bool` type, both values must be covered:

```liva
// ✅ Exhaustive - both cases covered
let result = switch flag {
    true => "yes",
    false => "no"
};

// ✅ Exhaustive - wildcard catches remaining case
let result = switch flag {
    true => "yes",
    _ => "no"
};

// ✅ Exhaustive - binding pattern catches all
let result = switch flag {
    false => "no",
    x => "other"
};

// ❌ Non-exhaustive - missing 'false' case
let result = switch flag {
    true => "yes"
    // Compiler error: E0901 - Non-exhaustive pattern match
};
```

**Error Example:**

```
● E0901: Non-exhaustive Pattern Matching [Semantic]
────────────────────────────────────────────────────────────

  ⓘ Pattern matching on bool is not exhaustive - missing case(s): false

  � Hint: Add pattern `false` or use wildcard `_` to catch remaining cases

  �📚 Learn more: https://liva-lang.org/docs/pattern-matching#exhaustiveness
────────────────────────────────────────────────────────────
```

### Integer Exhaustiveness ✅ Implemented (v0.10.5)

For integer types, the compiler requires a wildcard pattern unless you have a very small set of explicit values (≤ 20):

```liva
// ✅ Exhaustive - wildcard catches all other integers
let result = switch num {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"  // Required
};

// ✅ Exhaustive - with ranges and wildcard
let grade = switch score {
    0..=59 => "F",
    60..=69 => "D",
    70..=79 => "C",
    80..=89 => "B",
    90..=100 => "A",
    _ => "Invalid"  // Required
};

// ✅ Exhaustive - binding pattern catches all
let category = switch num {
    0 => "zero",
    1 => "one",
    other => "something else"
};

// ❌ Non-exhaustive - missing wildcard
let result = switch num {
    0 => "zero",
    1 => "one",
    2 => "two"
    // Compiler error: E0902 - Non-exhaustive pattern match
};
```

**Error Example:**

```
● E0902: Non-exhaustive Pattern Matching [Semantic]
────────────────────────────────────────────────────────────

  ⓘ Pattern matching on integers is not exhaustive - 3 value(s) explicitly
    covered, but no wildcard for other integers

  💡 Hint: Add wildcard pattern `_` to catch all other integer values

  📝 Example:
    switch num {
        0 => "zero",
        1 => "one",
        _ => "other"  // Required
    }

  📚 Learn more: https://liva-lang.org/docs/pattern-matching#exhaustiveness
────────────────────────────────────────────────────────────
```

**Why Require Wildcard?**
- Integers have infinite possible values (or very large ranges like i32: -2³¹ to 2³¹-1)
- Explicitly listing all values is impractical
- Wildcard ensures no runtime panics from unhandled cases

**Small Sets Exception:**
If you have ≤ 20 explicit integer literals without ranges, the compiler still requires a wildcard to be safe.

### String Exhaustiveness ✅ Implemented (v0.10.5)

For string types, a wildcard or binding pattern is **always required** since strings have infinite possible values:

```liva
// ✅ Exhaustive - wildcard catches all other strings
let code = switch status {
    "active" => 1,
    "inactive" => 2,
    "pending" => 3,
    _ => 0  // Required
};

// ✅ Exhaustive - binding pattern catches all
let message = switch text {
    "yes" => "affirmative",
    "no" => "negative",
    other => $"unknown: {other}"
};

// ❌ Non-exhaustive - missing wildcard
let code = switch status {
    "active" => 1,
    "inactive" => 2
    // Compiler error: E0903 - Non-exhaustive pattern match
};
```

**Error Example:**

```
● E0903: Non-exhaustive Pattern Matching [Semantic]
────────────────────────────────────────────────────────────

  ⓘ Pattern matching on strings requires a wildcard or binding pattern

  💡 Hint: Add wildcard pattern `_` or binding pattern to catch all string
    values not explicitly matched

  📝 Example:
    switch text {
        "active" => 1,
        "inactive" => 2,
        _ => 0  // Required
    }

  📚 Learn more: https://liva-lang.org/docs/pattern-matching#exhaustiveness
────────────────────────────────────────────────────────────
```

### Wildcard for Completeness

Use `_` to handle remaining cases:

```liva
let result = switch day {
    "Monday" => "Start of week",
    "Friday" => "End of week",
    _ => "Middle of week"  // Catches all other strings
};
```

### Exhaustiveness Summary

**Checked Types:**
- ✅ `bool` - Both `true` and `false` must be covered (v0.9.5)
- ✅ `int`, `i8`-`i128`, `u8`-`u128` - Requires wildcard (v0.10.5)
- ✅ `string`, `String` - Requires wildcard (v0.10.5)
- ✅ **Or-patterns** - Supported with exhaustiveness checking (v0.10.5)

**Not Checked Yet:**
- ⏳ `float`, `f32`, `f64` - Not recommended for pattern matching
- ⏳ `char` - Coming soon
- ⏳ Enum variants - Coming in future versions
- ⏳ Tuple/array destructuring patterns - Coming in v0.10.6+

**Recommendation:** Always include a wildcard `_` or binding pattern as the last arm to ensure exhaustiveness.

**Or-Pattern Note:** Or-patterns (`1 | 2 | 3 => ...`) don't automatically make a switch exhaustive. You still need a wildcard or binding pattern to catch remaining values.

### Future: Full Exhaustiveness (v0.10.6+)

Coming soon:
- Array/tuple destructuring patterns: `[x, y] => ...`, `(x, y) => ...`
- Enum variant exhaustiveness
- Custom type exhaustiveness

---

## Examples

### Example 1: HTTP Status Codes

```liva
main() {
    let statusCode = 404;
    
    let message = switch statusCode {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown Status"  // Required for exhaustiveness
    };
    
    print($"Status: {statusCode} - {message}");
    // Output: Status: 404 - Not Found
}
```

### Example 2: Grade Calculator

```liva
main() {
    let scores = [95, 82, 71, 58];
    
    for score in scores {
        let grade = switch score {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F"
        };
        
        print($"Score {score} = Grade {grade}");
    }
}
// Output:
// Score 95 = Grade A
// Score 82 = Grade B
// Score 71 = Grade C
// Score 58 = Grade F
```

### Example 3: Age Categories with Guards

```liva
main() {
    let ages = [5, 15, 25, 70];
    
    for age in ages {
        let category = switch age {
            x if x < 13 => "child",
            x if x < 20 => "teenager",
            x if x < 65 => "adult",
            _ => "senior"
        };
        
        print($"Age {age}: {category}");
    }
}
// Output:
// Age 5: child
// Age 15: teenager
// Age 25: adult
// Age 70: senior
```

### Example 4: Nested Switch

```liva
main() {
    let x = 5;
    let y = 10;
    
    let result = switch x {
        0 => "x is zero",
        n if n > 0 => switch y {
            0 => "x positive, y zero",
            m if m > n => "y greater than x",
            _ => "x greater or equal to y"
        },
        _ => "x is negative"
    };
    
    print(result);
    // Output: y greater than x
}
```

### Example 5: Binding with Computation

```liva
main() {
    let numbers = [0, 1, 2, 5, 10];
    
    for num in numbers {
        let result = switch num {
            0 => 0,
            1 => 1,
            n => n * n  // Square any other number
        };
        
        print($"{num} => {result}");
    }
}
// Output:
// 0 => 0
// 1 => 1
// 2 => 4
// 5 => 25
// 10 => 100
```

---

## Best Practices

### ✅ Do's

1. **Use Descriptive Binding Names**
   ```liva
   // ✅ Good
   switch age {
       childAge if childAge < 13 => "child",
       teenAge if teenAge < 20 => "teenager",
       _ => "adult"
   }
   
   // ❌ Avoid
   switch age {
       x if x < 13 => "child",
       x if x < 20 => "teenager",
       _ => "adult"
   }
   ```

2. **Order Patterns from Specific to General**
   ```liva
   // ✅ Good - specific cases first
   switch value {
       0 => "zero",
       1 => "one",
       n if n < 10 => "single digit",
       _ => "large number"
   }
   
   // ❌ Bad - wildcard too early
   switch value {
       _ => "any number",  // This catches everything!
       0 => "zero"         // Never reached
   }
   ```

3. **Use Ranges for Continuous Values**
   ```liva
   // ✅ Good
   switch score {
       90..=100 => "A",
       80..=89 => "B",
       _ => "C or below"
   }
   
   // ❌ Verbose
   switch score {
       90 => "A", 91 => "A", 92 => "A", /* ... */
   }
   ```

4. **Prefer Switch Over If-Else Chains**
   ```liva
   // ✅ Good - clear intent
   let grade = switch score {
       90..=100 => "A",
       80..=89 => "B",
       _ => "F"
   };
   
   // ❌ Less clear
   let grade = if score >= 90 and score <= 100 {
       "A"
   } else if score >= 80 and score <= 89 {
       "B"
   } else {
       "F"
   };
   ```

### ❌ Don'ts

1. **Don't Forget Wildcard for Non-Exhaustive Types**
   ```liva
   // ❌ May miss cases
   switch status {
       "active" => handleActive(),
       "pending" => handlePending()
       // What about "inactive", "deleted", etc.?
   }
   
   // ✅ Safe
   switch status {
       "active" => handleActive(),
       "pending" => handlePending(),
       _ => handleUnknown()
   }
   ```

2. **Don't Mix Types in Switch Arms**
   ```liva
   // ❌ Type error
   let result = switch x {
       0 => "zero",      // string
       1 => 1,           // int - ERROR!
       _ => "other"
   };
   
   // ✅ All same type
   let result = switch x {
       0 => "zero",
       1 => "one",
       _ => "other"
   };
   ```

3. **Don't Use Complex Logic in Guards - Extract to Functions**
   ```liva
   // ❌ Hard to read
   switch user {
       u if u.age >= 18 and u.hasLicense and u.hasInsurance and !u.isSuspended => "can drive",
       _ => "cannot drive"
   }
   
   // ✅ Clear intent
   canDrive(user: User): bool {
       return user.age >= 18 and user.hasLicense 
              and user.hasInsurance and !user.isSuspended;
   }
   
   switch user {
       u if canDrive(u) => "can drive",
       _ => "cannot drive"
   }
   ```

---

## Limitations

### Current Limitations (v0.9.5)

1. **No Tuple/Array Destructuring**
   ```liva
   // ❌ Not yet supported
   switch point {
       (0, 0) => "origin",
       (x, 0) => "on x-axis",
       (0, y) => "on y-axis",
       (x, y) => "in quadrant"
   }
   ```

2. **No Enum Patterns**
   ```liva
   // ❌ Not yet supported (enums coming in future)
   switch result {
       Ok(value) => handleSuccess(value),
       Err(error) => handleError(error)
   }
   ```

3. **Limited Exhaustiveness Checking**
   - Currently only checks `bool` exhaustiveness
   - Integer and string exhaustiveness coming in v0.9.6

4. **No Or Patterns**
   ```liva
   // ❌ Not yet supported
   switch value {
       1 | 2 | 3 => "small",
       _ => "large"
   }
   
   // ✅ Current workaround
   switch value {
       1 => "small",
       2 => "small",
       3 => "small",
       _ => "large"
   }
   ```

### Planned Features (v0.9.6+)

- Full exhaustiveness checking for all types
- Tuple and array destructuring patterns
- Enum variant patterns
- Or patterns (`|` operator)
- Guard variables (naming in guards)
- As patterns (binding + subpattern)

---

## Error Codes

### E6001: Non-Exhaustive Pattern Match

Pattern matching must cover all possible values.

```liva
// Error: E6001
let result = switch flag {
    true => "yes"
    // Missing: false case
};
```

**Solution:** Add a `false` case or wildcard:
```liva
let result = switch flag {
    true => "yes",
    false => "no"
};
```

### E6002: Type Mismatch in Switch Arms

All arms must return the same type.

```liva
// Error: E6002
let result = switch x {
    0 => "zero",    // string
    1 => 1,         // int - ERROR
    _ => "other"
};
```

**Solution:** Make all arms return the same type:
```liva
let result = switch x {
    0 => "zero",
    1 => "one",
    _ => "other"
};
```

### E6003: Invalid Range Pattern

Range patterns must have valid bounds.

```liva
// Error: E6003
let result = switch x {
    10..5 => "invalid",  // Start > end
    _ => "ok"
};
```

**Solution:** Use correct range bounds:
```liva
let result = switch x {
    5..10 => "valid range",
    _ => "outside"
};
```

---

## See Also

- [Control Flow](control-flow.md) - Traditional switch statements
- [Error Handling](error-handling.md) - Error patterns
- [Types](types.md) - Type system overview
- [Operators](operators.md) - Comparison and range operators

---

**Next:** [String Templates](string-templates.md)  
**Previous:** [Operators](operators.md)
