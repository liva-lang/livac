# ⚡ Quick Start

Get up and running with Liva in 5 minutes!

## Your First Program

Create a file called `hello.liva`:

```liva
main() {
  print("Hello, Liva!")
}
```

Compile and run it:

```bash
livac hello.liva --run
```

**Output:**
```
Hello, Liva!
```

🎉 **Congratulations!** You just ran your first Liva program!

## Understanding the Basics

### Functions

Functions in Liva are simple and clean:

```liva
// One-liner function
add(a, b) => a + b

// Block function
greet(name) {
  print($"Hello, {name}!")
}

main() {
  let sum = add(5, 3)
  print($"Sum: {sum}")
  
  greet("World")
}
```

### Variables

```liva
main() {
  // Mutable variable
  let x = 10
  x = 20
  
  // Immutable constant
  const PI = 3.1416
  
  // Type annotations (optional)
  let age: number = 25
  let name: string = "Liva"
  
  print($"x = {x}, PI = {PI}")
}
```

### String Templates

Liva supports elegant string interpolation with `$"..."`:

```liva
main() {
  let name = "Fran"
  let age = 41
  
  print($"My name is {name} and I'm {age} years old")
}
```

### Control Flow

```liva
main() {
  let age = 25
  
  // If statement
  if age >= 18 {
    print("Adult")
  } else {
    print("Minor")
  }
  
  // For loop with range
  for i in 1..5 {
    print($"Count: {i}")
  }
  
  // While loop
  let counter = 0
  while counter < 3 {
    print($"Counter: {counter}")
    counter = counter + 1
  }
}
```

### Arrays and Objects

```liva
main() {
  // Array
  let numbers = [1, 2, 3, 4, 5]
  print($"First: {numbers[0]}")
  print($"Length: {numbers.length}")
  
  // Object
  let person = {
    name: "Alice",
    age: 30
  }
  
  print($"{person.name} is {person.age} years old")
  
  // Array of objects
  let users = [
    { name: "Bob", age: 25 },
    { name: "Charlie", age: 35 }
  ]
  
  for user in users {
    print($"User: {user.name}")
  }
}
```

### Functions with Types

```liva
// Function with explicit types
sum(a: number, b: number): number => a + b

// Function with multiple parameters
calculateArea(width: float, height: float): float {
  return width * height
}

main() {
  let result = sum(10, 20)
  let area = calculateArea(5.5, 3.2)
  
  print($"Sum: {result}")
  print($"Area: {area}")
}
```

### Classes

```liva
// Define a class
Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  greet() {
    print($"Hi, I'm {this.name}!")
  }
  
  isAdult() => this.age >= 18
}

main() {
  // Create instance with constructor
  let person1 = Person("Alice", 30)
  person1.greet()
  
  // Create instance with object literal
  let person2 = Person {
    name: "Bob",
    age: 25
  }
  
  print($"Is adult: {person2.isAdult()}")
}
```

### Concurrency - Async

```liva
// Async function
fetchData(url: string): string {
  // Simulated async operation
  return $"Data from {url}"
}

main() {
  // Run asynchronously
  let data = async fetchData("https://api.example.com")
  print($"Got: {data}")
}
```

### Concurrency - Parallel

```liva
// CPU-intensive function
heavyCalc(n: number): number {
  return n * n
}

main() {
  // Run in parallel thread
  let result1 = par heavyCalc(100)
  let result2 = par heavyCalc(200)
  
  print($"Results: {result1}, {result2}")
}
```

### Error Handling

```liva
// Fallible function
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

main() {
  // Error binding
  let result, err = divide(10, 2)
  
  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Result: {result}")
  }
  
  // Error case
  let result2, err2 = divide(10, 0)
  print($"Error: {err2}")  // "Division by zero"
}
```

## Complete Example

Here's a complete program demonstrating multiple features:

```liva
// Define a class
User {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  canVote() => this.age >= 18
}

// Async function
fetchUserData(id: number): string {
  return $"User data for ID {id}"
}

// Fallible function
validateAge(age: number): string {
  if age < 0 fail "Age cannot be negative"
  if age > 150 fail "Age too high"
  return "Valid age"
}

main() {
  print("🚀 Liva Demo Program\n")
  
  // Create users
  let users = [
    User("Alice", 25),
    User("Bob", 17),
    User("Charlie", 30)
  ]
  
  // Process users
  for user in users {
    print($"👤 {user.name}, age {user.age}")
    
    if user.canVote() {
      print("   ✅ Can vote")
    } else {
      print("   ❌ Cannot vote")
    }
  }
  
  // Async operation
  print("\n📡 Fetching user data...")
  let data = async fetchUserData(123)
  print($"   {data}")
  
  // Error handling
  print("\n🛡️ Validating ages...")
  let result1, err1 = validateAge(25)
  print($"   Age 25: {result1}")
  
  let result2, err2 = validateAge(-5)
  if err2 != "" {
    print($"   Age -5: Error - {err2}")
  }
  
  print("\n✨ Demo complete!")
}
```

**Output:**
```
🚀 Liva Demo Program

👤 Alice, age 25
   ✅ Can vote
👤 Bob, age 17
   ❌ Cannot vote
👤 Charlie, age 30
   ✅ Can vote

📡 Fetching user data...
   User data for ID 123

🛡️ Validating ages...
   Age 25: Valid age
   Age -5: Error - Age cannot be negative

✨ Demo complete!
```

## Compiler Options

### Basic Usage

```bash
# Just compile
livac program.liva

# Compile and run
livac program.liva --run

# Check syntax only
livac program.liva --check

# Show generated Rust code
livac program.liva --verbose
```

### Advanced Options

```bash
# Custom output directory
livac program.liva --output ./build

# JSON error output (for IDEs)
livac program.liva --check --json

# Help
livac --help
```

## Project Structure

For larger projects, organize your code:

```
my-project/
├── main.liva           # Entry point
├── models/
│   ├── user.liva
│   └── product.liva
├── utils/
│   └── helpers.liva
└── target/
    └── liva_build/     # Generated by compiler
```

## Next Steps

Now that you know the basics, explore more:

- **[Basic Concepts](basic-concepts.md)** - Deeper dive into core concepts
- **[Examples](examples.md)** - More example programs
- **[Language Reference](../language-reference/syntax-overview.md)** - Complete syntax guide
- **[Concurrency Guide](../guides/async-programming.md)** - Master async and parallel
- **[Error Handling](../guides/error-handling-patterns.md)** - Best practices

## Common Patterns

### Reading User Input

```liva
main() {
  print("Enter your name: ")
  let name = readLine()
  print($"Hello, {name}!")
}
```

### Working with Arrays

```liva
main() {
  let numbers = [1, 2, 3, 4, 5]
  
  // Map
  let doubled = numbers.map(x => x * 2)
  
  // Filter
  let evens = numbers.filter(x => x % 2 == 0)
  
  // Reduce
  let sum = numbers.reduce(0, (acc, x) => acc + x)
  
  print($"Doubled: {doubled}")
  print($"Evens: {evens}")
  print($"Sum: {sum}")
}
```

### File Operations

```liva
main() {
  // Write file
  writeFile("output.txt", "Hello, file!")
  
  // Read file
  let content = readFile("output.txt")
  print($"Content: {content}")
}
```

## Tips for Success

1. **Start Simple** - Begin with basic programs and gradually add complexity
2. **Use Type Annotations** - When you need clarity or precision
3. **Embrace Concurrency** - Liva makes async/parallel easy
4. **Handle Errors** - Use error binding for robust programs
5. **Check the Docs** - Reference documentation when stuck

## Getting Help

- 📚 **Full Documentation**: [docs/README.md](../README.md)
- 💬 **Community**: GitHub Discussions
- 🐛 **Bug Reports**: GitHub Issues
- 📧 **Contact**: maintainers@liva-lang.org

---

**Happy Coding! 🎉**

*You're now ready to explore the full power of Liva!*
