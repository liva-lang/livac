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

Liva uses **identifier-based visibility** instead of explicit keywords (`public`, `private`).

Visibility is determined by a **leading underscore**:

| Prefix | Visibility | Access Scope |
|--------|-----------|--------------|
| None | **Public** | Anywhere |
| `_` (underscore) | **Private** | Same class/module only |

**Note**: Since Liva doesn't have class inheritance, there's no need for `protected` visibility.

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

### Private (Single Underscore `_`)

```liva
// Private function
_validateInput(data) => data != null && data.length > 0

// Private field
Person {
  _password: string  // Private
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

### Private Functions

```liva
// Private: Internal implementation details
_secretAlgorithm(data) {
  // Complex internal logic
  return result
}

_generateToken(userId: number): string {
  // Private helper
}
```

### Example

```liva
// Public API
createUser(name: string, email: string, password: string): User {
  _validateEmail(email)         // Private helper
  let hashed = _hashPassword(password)  // Private helper
  
  return User {
    name: name,
    email: email,
    password: hashed
  }
}

// Private helper
_validateEmail(email: string) {
  if email == "" fail "Email required"
  if not email.contains("@") fail "Invalid email"
}

// Private helper
_hashPassword(password: string): string {
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
    this.name = name             // Public
    this.email = email           // Public
    this._password = password    // Private
  }
  
  name: string      // Public
  email: string     // Public
  _password: string // Private
}
```

### Accessing Fields

```liva
let user = User("Alice", "alice@example.com", "secret123")

print(user.name)       // ✅ Public: OK
print(user.email)      // ✅ Public: OK
print(user._password)  // ❌ Private: Compile error (outside class)
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
    _validateAmount(amount)  // Call private helper
    this.balance = this.balance + amount
    this._logTransaction("deposit", amount)  // Call private method
  }
  
  // Private helper function
  _validateAmount(amount: number) {
    if amount <= 0 fail "Amount must be positive"
  }
  
  // Private method
  _logTransaction(type: string, amount: number) {
    print($"[{type}] ${amount}")
  }
}
```

### Method Calls

```liva
let account = BankAccount(100)

account.getBalance()           // ✅ Public: OK
account.deposit(50)            // ✅ Public: OK
account._logTransaction(...)   // ❌ Private: Compile error
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

### Use Private for Implementation Details

```liva
// ✅ Good: Private internal logic
_encryptData(data) { }
_generateRandomId() { }
_connectToDatabase() { }
```

### Keep Sensitive Fields Private

```liva
// ✅ Good: Encapsulation
User {
  constructor(name: string, balance: number) {
    this.name = name        // Public (readable)
    this._balance = balance // Private
  }
  
  name: string
  _balance: number
  
  getBalance() => this._balance
  
  deposit(amount: number) {
    this._balance = this._balance + amount
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

// Private: Validate email format (internal)
_validateEmail(email: string) { }

// Private: Internal password hashing
_hashPassword(password: string) { }
```

### Consistency

```liva
// ✅ Good: Consistent visibility pattern
class UserService {
  // Public interface
  createUser() { }
  deleteUser() { }
  
  // Private internals
  _validateUser() { }
  _formatUser() { }
  _saveToDatabase() { }
  _generateId() { }
}
```

---

## Summary

### Visibility Levels

| Level | Prefix | Access | Use Case |
|-------|--------|--------|----------|
| **Public** | (none) | Everywhere | Public APIs, exported functions |
| **Private** | `_` | Same file/class | Implementation details, secrets |

### Examples

```liva
// Public
calculatePrice(quantity, price) => quantity * price

// Private
_validateInput(data) => data != null
```

### Class Visibility

```liva
User {
  constructor(name: string, password: string) {
    this.name = name            // Public
    this._password = password   // Private
  }
  
  name: string        // Public
  _password: string   // Private
  
  // Public method
  getName() => this.name
  
  // Private method
  _checkPassword(input: string): bool {
    return input == this._password
  }
}
```

### Quick Reference

```liva
// Functions
publicFunction() { }
_privateFunction() { }

// Fields
class Example {
  publicField: string
  _privateField: string
}

// Methods
class Example {
  publicMethod() { }
  _privateMethod() { }
}
```

---

**See Also**:
- [Functions](functions.md)
- [Classes](classes.md)
- [Variables](variables.md)
