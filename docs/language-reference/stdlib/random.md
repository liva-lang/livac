# Random Module

The `Random` module provides functions for generating random numbers, selecting random elements, and creating UUIDs.

**Crates auto-injected:** `rand = "0.8"`, `uuid = { version = "1", features = ["v4"] }`

---

## Functions

### Random.nextInt(min, max) → `number`

Generates a random integer in the range `[min, max]` (inclusive).

```liva
let dice = Random.nextInt(1, 6)
print(dice)  // 1-6

let index = Random.nextInt(0, 99)
```

### Random.nextFloat() → `float`

Generates a random float in the range `[0.0, 1.0)`.

```liva
let f = Random.nextFloat()
print(f)  // e.g., 0.7234...
```

### Random.nextFloat(min, max) → `float`

Generates a random float in the range `[min, max)`.

```liva
let temp = Random.nextFloat(36.0, 42.0)
print($"Temperature: {temp}")
```

### Random.choice(array) → `T`

Selects a random element from an array. Panics on empty arrays.

```liva
let colors = ["red", "blue", "green", "yellow"]
let pick = Random.choice(colors)
print($"Selected: {pick}")
```

### Random.shuffle(array) → `[T]`

Returns a new array with elements in random order (Fisher-Yates shuffle).

```liva
let cards = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let deck = Random.shuffle(cards)
print(deck)
```

### Random.uuid() → `string`

Generates a UUID v4 string.

```liva
let id = Random.uuid()
print(id)  // e.g., "550e8400-e29b-41d4-a716-446655440000"
```

---

## Complete Example

```liva
main() {
    // Dice game
    let roll1 = Random.nextInt(1, 6)
    let roll2 = Random.nextInt(1, 6)
    print($"You rolled: {roll1} + {roll2} = {roll1 + roll2}")

    // Random password
    let chars = "abcdefghijklmnopqrstuvwxyz0123456789"
    let password = ""
    for i in 0..12 {
        let idx = Random.nextInt(0, chars.length - 1)
        password = password + chars.charAt(idx)
    }
    print($"Password: {password}")

    // Unique ID
    let sessionId = Random.uuid()
    print($"Session: {sessionId}")
}
```
