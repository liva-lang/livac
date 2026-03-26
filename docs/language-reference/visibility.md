# Visibility

> SKILL.md covers: `_` prefix = private, no `public`/`private` keywords.
> This file: what gets exported from modules, cross-module access rules.

## Rule

| Prefix | Visibility | Scope |
|--------|-----------|-------|
| None | **Public** | Anywhere (importable) |
| `_` | **Private** | Same file/class only |

No `protected` — Liva has no inheritance.

## What Gets Exported

When another file does `import { X } from "./myfile"`, these are importable:

| Item | Exportable? |
|------|------------|
| Functions (no `_` prefix) | Yes |
| Classes (no `_` prefix) | Yes |
| Enums (no `_` prefix) | Yes |
| Type aliases (no `_` prefix) | Yes |
| `_` prefixed anything | **No** — private to defining file |
| Constants (`const`) | Yes (if no `_` prefix) |

## Cross-Module Access

```liva
// math.liva
add(a: number, b: number) => a + b         // Public — importable
_validate(n: number) => n >= 0              // Private — NOT importable

// main.liva
import { add } from "./math"
// import { _validate } from "./math"    // ❌ Cannot import private
```

## Class Member Visibility

Private members are enforced at the class boundary:

```liva
BankAccount {
    _balance: number

    constructor(balance: number) { this._balance = balance }

    getBalance() => this._balance           // Public method
    _log(msg: string) { print(msg) }       // Private method

    deposit(amount: number) {
        this._log($"Depositing {amount}")   // ✅ Same class — OK
        this._balance = this._balance + amount
    }
}

// External code:
let acc = BankAccount(100)
acc.getBalance()        // ✅ Public
acc.deposit(50)         // ✅ Public
// acc._balance         // ❌ Private field
// acc._log("test")     // ❌ Private method
```

## Naming Convention

```liva
// Public API
createUser(name: string) { }
processOrder(order: Order) { }

// Internal helpers
_validateEmail(email: string) { }
_hashPassword(pw: string): string { }
_formatCurrency(amount: float): string { }
```
