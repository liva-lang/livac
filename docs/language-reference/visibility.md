# Visibility

Complete reference for visibility modifiers and access control in Liva.

## Table of Contents
- [Overview](#overview)
- [Identifier-Based Visibility](#identifier-based-visibility)
- [Function Visibility](#function-visibility)
- [Field and Method Visibility](#field-and-method-visibility)
- [Best Practices](#best-practices)

---

## Overview

Liva uses **identifier-based visibility** instead of explicit keywords (`public`, `private`, `protected`).

Visibility is determined by the **number of leading underscores**:

| Prefix | Visibility | Access Scope |
|--------|-----------|--------------|
| None | **Public** | Anywhere |
| `_` (single) | **Protected** | Same module + subclasses |
| `__` (double) | **Private** | Same class/file only |

---

## Identifier-Based Visibility

### Public (No Prefix)

```liva
// Public function
calculatePrice(quantity, price) => quantity * price

// Public field
Person {
  name: string  // Public
}
```

**Accessible**: Everywhere (internal and external modules)

### Protected (Single Underscore `_`)

```liva
// Protected function
_validateInput(data) => data != null && data.length > 0

// Protected field
Person {
  _email: string  // Protected
}
```

**Accessible**: Same module and subclasses

### Private (Double Underscore `__`)

```liva
// Private function
__internalHelper(value) => value * 2 + 1

// Private field
Person {
  __password: string  // Private
}
```

**Accessible**: Same file/class only

---

## Function Visibility

### Public Functions

```liva
// Public: Available to all consumers
calculateTax(amount: number, rate: float): float {
  return amount * rate
}

processOrder(order) {
  // Can be called from anywhere
}
```

### Protected Functions

```liva
// Protected: Internal to module, available to subclasses
_validateUser(user) {
  return user.name != "" && user.age >= 0
}

_logTransaction(type: string, amount: number) {
  print($"[{type}] ${amount}")
}
```

### Private Functions

```liva
// Private: Internal implementation details
__secretAlgorithm(data) {
  // Complex internal logic
  return result
}

__generateToken(userId: number): string {
  // Private helper
}
```

### Example

```liva
// Public API
createUser(name: string, email: string, password: string): User {
  _validateEmail(email)       // Protected helper
  let hashed = __hashPassword(password)  // Private helper
  
  return User {
    name: name,
    email: email,
    password: hashed
  }
}

// Protected helper (available to subclasses/module)
_validateEmail(email: string) {
  if email == "" fail "Email required"
  if not email.contains("@") fail "Invalid email"
}

// Private helper (internal only)
__hashPassword(password: string): string {
  // Secret hashing algorithm
  return hashedValue
}
```

---

## Field and Method Visibility

### Class Fields

```liva
User {
  constructor(name: string, email: string, password: string) {
    this.name = name           // Public
    this._email = email        // Protected
    this.__password = password // Private
  }
  
  name: string        // Public
  _email: string      // Protected
  __password: string  // Private
}
```

### Accessing Fields

```liva
let user = User("Alice", "alice@example.com", "secret123")

print(user.name)          // ✅ Public: OK
print(user._email)        // ⚠️ Protected: OK in same module
print(user.__password)    // ❌ Private: Compile error (outside class)
```

### Class Methods

```liva
BankAccount {
  constructor(balance: number) {
    this.balance = balance
  }
  
  balance: number
  
  // Public method
  getBalance() => this.balance
  
  // Public method
  deposit(amount: number) {
    this._validateAmount(amount)  // Call protected
    this.balance = this.balance + amount
    this.__logTransaction("deposit", amount)  // Call private
  }
  
  // Protected method
  _validateAmount(amount: number) {
    if amount <= 0 fail "Amount must be positive"
  }
  
  // Private method
  __logTransaction(type: string, amount: number) {
    print($"[{type}] ${amount}")
  }
}
```

### Method Calls

```liva
let account = BankAccount(100)

account.getBalance()           // ✅ Public: OK
account.deposit(50)            // ✅ Public: OK
account._validateAmount(10)    // ⚠️ Protected: OK in same module
account.__logTransaction(...)  // ❌ Private: Compile error
```

---

## Best Practices

### Use Public for APIs

```liva
// ✅ Good: Public API functions
calculateTotal(items) { }
processPayment(order) { }
fetchUserData(id) { }
```

### Use Protected for Module Internals

```liva
// ✅ Good: Protected helpers shared within module
_validateInput(data) { }
_formatCurrency(amount) { }
_logDebug(message) { }
```

### Use Private for Implementation Details

```liva
// ✅ Good: Private internal logic
__encryptData(data) { }
__generateRandomId() { }
__connectToDatabase() { }
```

### Keep Fields Private or Protected

```liva
// ✅ Good: Encapsulation
User {
  constructor(name: string, balance: number) {
    this.name = name              // Public (immutable via methods)
    this.__balance = balance      // Private
  }
  
  name: string
  __balance: number
  
  getBalance() => this.__balance
  
  deposit(amount: number) {
    this.__balance = this.__balance + amount
  }
}

// ❌ Bad: Public mutable field
User {
  balance: number  // Anyone can modify directly
}
```

### Document Visibility Intent

```liva
// ✅ Good: Clear naming and comments
// Public API: Create a new user account
createUser(name: string, email: string) { }

// Protected: Validate email format (module-internal)
_validateEmail(email: string) { }

// Private: Internal password hashing
__hashPassword(password: string) { }
```

### Consistency

```liva
// ✅ Good: Consistent visibility pattern
class UserService {
  // Public interface
  createUser() { }
  deleteUser() { }
  
  // Protected helpers
  _validateUser() { }
  _formatUser() { }
  
  // Private internals
  __saveToDatabase() { }
  __generateId() { }
}
```

---

## Summary

### Visibility Levels

| Level | Prefix | Access | Use Case |
|-------|--------|--------|----------|
| **Public** | (none) | Everywhere | Public APIs, exported functions |
| **Protected** | `_` | Module + subclasses | Internal helpers, shared utilities |
| **Private** | `__` | Same file/class | Implementation details, secrets |

### Examples

```liva
// Public
calculatePrice(quantity, price) => quantity * price

// Protected
_validateInput(data) => data != null

// Private
__secretKey() => "..."
```

### Class Visibility

```liva
User {
  constructor(name: string, password: string) {
    this.name = name            // Public
    this._verified = false      // Protected
    this.__password = password  // Private
  }
  
  name: string        // Public
  _verified: bool     // Protected
  __password: string  // Private
  
  // Public method
  getName() => this.name
  
  // Protected method
  _setVerified(value: bool) {
    this._verified = value
  }
  
  // Private method
  __checkPassword(input: string): bool {
    return input == this.__password
  }
}
```

### Quick Reference

```liva
// Functions
publicFunction() { }
_protectedFunction() { }
__privateFunction() { }

// Fields
class Example {
  publicField: string
  _protectedField: string
  __privateField: string
}

// Methods
class Example {
  publicMethod() { }
  _protectedMethod() { }
  __privateMethod() { }
}
```

---

**See Also**:
- [Functions](functions.md)
- [Classes](classes.md)
- [Variables](variables.md)
