# Classes: Interfaces

> SKILL.md covers: `Printable { display(): string }`, `Dog : Printable { ... }`.
> This file: auto-detection rules, multiple interfaces, signature rules, semantic validation.

## How Interfaces Are Auto-Detected

The compiler distinguishes interfaces from classes by body content:

| Feature | Interface | Class |
|---------|-----------|-------|
| **Fields** | None | Has fields |
| **Method bodies** | Only signatures (no `=>` or `{ }`) | Has implementations |
| **Constructor** | Never | May have one |

```liva
// INTERFACE — only method signatures, no fields
Animal {
    makeSound(): string
    getName(): string
}

// CLASS — has fields and implementations
Dog : Animal {
    name: string
    constructor(name: string) { this.name = name }
    makeSound() => "Woof!"
    getName() => this.name
}
```

## Method Signature Rules

Interface method signatures declare name, parameters, and return type — no body:

```liva
Serializable {
    toJSON(): string
    fromJSON(json: string): Self     // Self = implementing type
    validate(): bool
}
```

- Return type is required in signatures
- Parameter types are required
- `Self` refers to the implementing class
- No default implementations

## Multiple Interfaces

Comma-separated after `:`:

```liva
Drawable { draw(): void }
Named { getName(): string }
Comparable { compareTo(other: Self): int }

Circle : Drawable, Named, Comparable {
    radius: float
    name: string

    constructor(name: string, radius: float) {
        this.name = name
        this.radius = radius
    }

    draw() => print($"Drawing {this.name}")
    getName() => this.name
    compareTo(other: Circle) => (this.radius - other.radius) as int
}
```

## Semantic Validation

The compiler validates interface implementations at compile time:

### All methods must be implemented

```liva
Animal { makeSound(): string; getName(): string }

// ❌ Error: Missing getName() implementation
Dog : Animal {
    makeSound() => "Woof!"
}
```

### Return types must match

```liva
Comparable { compareTo(other: Self): int }

// ❌ Error: Wrong return type (int expected, got bool)
Point : Comparable {
    compareTo(other: Point): bool => true
}
```

### Parameter types must match

```liva
Processor { process(data: string): string }

// ❌ Error: Wrong parameter type (string expected, got int)
DataProcessor : Processor {
    process(data: int): string => ""
}
```

## Polymorphism with Interfaces

Functions can accept interface types for polymorphic behavior:

```liva
Shape { area(): float; perimeter(): float }

Circle : Shape {
    radius: float
    constructor(radius: float) { this.radius = radius }
    area() => 3.14159 * this.radius * this.radius
    perimeter() => 2.0 * 3.14159 * this.radius
}

Rectangle : Shape {
    width: float
    height: float
    constructor(width: float, height: float) { this.width = width; this.height = height }
    area() => this.width * this.height
    perimeter() => 2.0 * (this.width + this.height)
}

// Accept any Shape
totalArea(shapes: [Shape]): float {
    let total = 0.0
    for shape in shapes { total = total + shape.area() }
    return total
}
```
