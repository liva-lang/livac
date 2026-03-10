# GitHub Dashboard CLI

A comprehensive CLI dashboard for GitHub, written in **Liva** to validate all language features.

## Features Used

This project is designed as a **language validation tool**, using every major Liva feature:

### ✅ Core Language Features
- [x] **Variables** - `let`, `const`
- [x] **Type annotations** - `name: string`, `count: number`
- [x] **Functions** - One-liner (`=>`) and block syntax
- [x] **Classes** - Constructors, fields, methods
- [x] **Interfaces** - `Displayable`, `Fetchable` with multiple implementation
- [x] **Visibility** - Public fields and `_private` fields

### ✅ Control Flow
- [x] **If/else** - Conditionals with multiple branches
- [x] **Ternary** - `condition ? then : else`
- [x] **Switch/Pattern matching** - Literals, ranges (`0..=10`), or-patterns (`|`), guards (`if`), wildcards (`_`)
- [x] **Loops** - `while`, `for..in`, `for..in range`
- [x] **Break/Continue** - Loop control

### ✅ Error Handling
- [x] **fail** - Explicit error throwing
- [x] **Error binding** - `let result, err = fallibleFunc()`
- [x] **Error propagation** - Throughout API calls

### ✅ Concurrency
- [x] **async** - Async HTTP calls with auto-await
- [x] **task** - Explicit task handles
- [x] **await** - Manual task waiting
- [x] **fire** - Fire-and-forget async
- [x] **par** - Parallel computation (stats calculation)

### ✅ Collections
- [x] **Arrays** - `[1, 2, 3]`
- [x] **map** - Transform elements
- [x] **filter** - Filter elements
- [x] **reduce** - Aggregate values
- [x] **some/every** - Boolean checks
- [x] **find** - Find first match
- [x] **concat** - Array concatenation

### ✅ Standard Library
- [x] **print/console.log** - Output
- [x] **console.error/warn/success** - Colored output
- [x] **File.read/write/exists** - File I/O
- [x] **JSON.parse/stringify** - JSON handling
- [x] **HTTP.get** - HTTP client
- [x] **Math.sqrt/pow/floor/round/min/max** - Math operations

### ✅ Modules
- [x] **import { x, y } from** - Named imports
- [x] **Multi-file project** - 7 source files
- [x] **Private functions** - `_helper` prefix

### ✅ Advanced Features
- [x] **String templates** - `$"Hello {name}"`
- [x] **Tuples** - `(min, max)` return values
- [x] **null keyword** - Nullable values
- [x] **Method chaining** - `array.filter().map().reduce()`

## Project Structure

```
github-dashboard/
├── src/
│   ├── main.liva              # Entry point, CLI, App class
│   ├── api/
│   │   ├── users.liva         # User/Repo API client
│   │   └── issues.liva        # Issues/PRs API client
│   ├── models/
│   │   ├── entities.liva      # User, Repo, Issue, PR classes
│   │   └── stats.liva         # Stats calculation
│   ├── display/
│   │   └── output.liva        # Console output formatting
│   └── utils/
│       ├── config.liva        # Configuration management
│       └── format.liva        # String/number formatting
├── config.json                # Default configuration
└── README.md                  # This file
```

## Running

```bash
# From livac-project root
cd livac
cargo build --release

# Compile and run the dashboard
./target/release/livac ../github-dashboard/src/main.liva --run
```

## Commands

Once running, the dashboard supports:

| Command | Description |
|---------|-------------|
| `user [name]` | Show user profile |
| `repos [name]` | List user's repositories |
| `stats` | Show aggregated statistics |
| `issues` | Show open issues |
| `prs` | Show pull requests |
| `config` | Show current configuration |
| `help` | Show available commands |
| `quit` | Exit the application |

## Classes Implemented

### Models
- **User** - GitHub user profile
- **Repo** - Repository information
- **Issue** - Issue details
- **PullRequest** - PR information
- **Stats** - Aggregated statistics
- **LanguageStat** - Language breakdown

### Interfaces
- **Displayable** - `display()`, `summary()`
- **Fetchable** - `getId()`, `getUrl()`

### Utility Classes
- **Config** - Application configuration
- **Command** - CLI command parsing
- **App** - Main application state
- **QueryParam** - URL query parameters

## API Endpoints Used

- `GET /users/{username}` - User profile
- `GET /users/{username}/repos` - User repositories
- `GET /search/issues` - Search issues/PRs
- `GET /repos/{owner}/{repo}/issues` - Repo issues
- `GET /repos/{owner}/{repo}/pulls` - Repo PRs
- `GET /rate_limit` - Rate limit status

## Purpose

This project serves as:

1. **Language Test Suite** - Validates all Liva features compile correctly
2. **Dogfooding** - Uses Liva to build something real
3. **Documentation** - Shows idiomatic Liva patterns
4. **Bug Discovery** - Finds edge cases in the compiler

## Feature Coverage Summary

| Category | Features | Status |
|----------|----------|--------|
| Variables | let, const, types | ✅ |
| Functions | one-liner, block, typed | ✅ |
| Classes | constructor, fields, methods | ✅ |
| Interfaces | signatures, multi-impl | ✅ |
| Control Flow | if, switch, loops | ✅ |
| Error Handling | fail, error binding | ✅ |
| Concurrency | async, par, task, fire | ✅ |
| Arrays | map, filter, reduce, etc | ✅ |
| Strings | templates, methods | ✅ |
| Modules | import, visibility | ✅ |
| Stdlib | File, JSON, HTTP, Math | ✅ |
| Advanced | tuples, null, chaining | ✅ |

**Total: 100% feature coverage**
