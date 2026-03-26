# Classes: Basics

> SKILL.md covers: declaration syntax, constructor, fields, methods, `object::method` references.
> This file: field defaults, constructor validation, `this` binding, body ordering, factory patterns.

## Body Ordering (Required)

1. **Fields** (first)
2. **Constructor** (second)
3. **Methods** (last)

Violating this order → parse error.

## Field Defaults

Fields can have default values. When a class has an explicit constructor, the constructor doesn't need to set fields that have defaults:

```liva
AppSettings {
    host: string = "localhost"
    port: int = 8080
    debug: bool = false

    constructor() {}    // explicit — even if empty
}

let settings = AppSettings()
// host = "localhost", port = 8080, debug = false
```

Any type supports field defaults (not just primitives) — as long as you provide an init expression.

**Data classes** (no constructor) also support defaults: if ALL fields have default values, the auto-generated constructor takes no arguments:

```liva
AppConfig {
    host: string = "localhost"
    port: int = 8080
    debug: bool = false
}

let config = AppConfig()    // ✅ Works — all fields use defaults
```

> **Note:** If a data class has a mix of fields with and without defaults, ALL fields are required as positional args.

### Mixing Required + Default Fields

```liva
User {
    name: string
    age: int = 18
    role: string = "user"
    active: bool = true

    constructor(name: string) {
        this.name = name
        // age, role, active use their defaults
    }
}

let user = User("Alice")
```

### Optional Fields with Defaults

Combine `?` (nullable) with `=` (default):

```liva
Settings {
    theme: string = "dark"
    fontSize: int = 14
    autoSave?: bool = true    // Optional + default → Some(true)
}
```

## Constructor Validation

Use `fail` in constructors to reject invalid state — caller must use error binding:

```liva
User {
    username: string
    password: string

    constructor(username: string, password: string) {
        if username == "" { fail "Username cannot be empty" }
        if password.length < 8 { fail "Password must be at least 8 characters" }
        this.username = username
        this.password = password
    }
}

let user, err = User("", "short")
if err { print(err) }
```

## `this` Binding

- All field/method access inside a class requires `this.fieldName`
- `this` is implicit — no parameter needed
- Works in both arrow (`=>`) and block (`{ }`) methods

```liva
Person {
    firstName: string
    lastName: string

    constructor(firstName: string, lastName: string) {
        this.firstName = firstName
        this.lastName = lastName
    }

    fullName() => $"{this.firstName} {this.lastName}"
}
```

## One Constructor Only

For variants use default parameters or factory functions outside the class:

```liva
Rectangle {
    width: number
    height: number

    constructor(width: number = 1, height: number = 1) {
        this.width = width
        this.height = height
    }
}

createSquare(size: number) => Rectangle(size, size)
```

## Computed Properties

Use methods — Liva has no computed field syntax:

```liva
Circle {
    radius: float
    constructor(radius: float) { this.radius = radius }
    area() => 3.14159 * this.radius * this.radius
}
```

## Async Methods

Methods are **automatically async** if they call async functions — no annotation needed:

```liva
UserService {
    apiUrl: string
    constructor(apiUrl: string) { this.apiUrl = apiUrl }

    fetchUser(id: number): string {
        let response, err = async HTTP.get($"{this.apiUrl}/users/{id}")
        if err { fail $"Fetch failed: {err}" }
        return response.body
    }
}
