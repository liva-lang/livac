# Liva Language Server - User Guide
## Getting Started with LSP Features

> **Version:** 0.12.0  
> **Audience:** Liva developers using VS Code

---

## 🚀 Quick Start

### Installation

1. **Install VS Code Extension:**
   ```bash
   # From VS Code Marketplace
   code --install-extension liva-lang.liva-vscode
   
   # Or from VSIX
   code --install-extension liva-vscode-0.14.0.vsix
   ```

2. **Verify Installation:**
   - Open a `.liva` file
   - Check status bar: "Liva LSP: Ready"
   - Try autocomplete (Ctrl+Space)

### Requirements
- VS Code 1.80.0 or higher
- Liva compiler v0.12.0 or higher
- 100MB free RAM

---

## ✨ Features Overview

### 1. **Intelligent Autocompletion**

**Trigger:** Type any character or press `Ctrl+Space`

**What You Get:**
- 🔤 **Keywords:** if, let, const, switch, etc.
- 🔢 **Types:** int, string, bool, float, etc.
- 📦 **Variables:** All variables in current scope
- 🔧 **Functions:** Available functions with signatures
- 🏗️ **Classes/Interfaces:** Type names
- 📝 **Snippets:** Common code patterns

**Example:**
```liva
let x = 10
let y = 20

// Type 'x' and see:
// - x (variable)
// - y (variable)  
// - if (keyword)
// - let (keyword)
```

**Smart Context:**
```liva
class Person {
    name: string
    age: int
}

let p = Person("Alice", 25)
p.  // Shows: name, age (fields only)
```

---

### 2. **Go to Definition**

**Trigger:** 
- Right-click → "Go to Definition"
- F12
- Ctrl+Click (Cmd+Click on Mac)

**Works For:**
- ✅ Variables
- ✅ Functions
- ✅ Classes
- ✅ Interfaces
- ✅ Type aliases
- ✅ Imports

**Example:**
```liva
type UserId = int

fn getUser(id: UserId) {  // Ctrl+Click on UserId
    // Jumps to type alias definition
}
```

**Cross-File:**
```liva
// math.liva
export fn add(a: int, b: int) -> int {
    return a + b
}

// main.liva
import { add } from "./math"
add(1, 2)  // F12 jumps to math.liva
```

---

### 3. **Go to Implementation**

**Trigger:**
- Right-click → "Go to Implementation"
- Ctrl+F12 (Cmd+F12 on Mac)

**Works For:**
- ✅ Interface names → lists all classes that `implement` the interface
- ✅ Interface method names → lists overrides across implementing classes

**Example:**
```liva
type Drawable {
    draw(): void
}

Cat : Drawable {
    draw() => print("meow")
}

Dog : Drawable {
    draw() => print("woof")
}

// Ctrl+F12 on `Drawable`  → jumps between Cat and Dog
// Ctrl+F12 on `draw()`    → lists Cat::draw and Dog::draw
```

If no implementations are recorded, falls back to "Go to Definition" behavior.

---

### 4. **Find All References**

**Trigger:**
- Right-click → "Find All References"
- Shift+F12
- Alt+Shift+F12 (Peek References)

**Shows:**
- All usages of a symbol
- Definition location (optional)
- Grouped by file

**Example:**
```liva
let counter = 0  // Definition

counter += 1     // Reference 1
print(counter)   // Reference 2
return counter   // Reference 3

// Shift+F12 on 'counter' shows all 4 locations
```

**Use Cases:**
- 🔍 **Impact Analysis:** See where a variable is used
- 🔧 **Refactoring:** Understand dependencies before changes
- 📊 **Code Navigation:** Jump between usages

---

### 4. **Hover Information**

**Trigger:** Hover mouse over any symbol

**Shows:**
- 📘 **Type Information**
- 📝 **Documentation**
- 🔧 **Function Signatures**
- 🎯 **Quick Reference**

**Example:**
```liva
let name: string = "Alice"
// Hover over 'name':
// ┌─────────────────┐
// │ let name: string│
// └─────────────────┘

fn add(a: int, b: int) -> int {
    return a + b
}
// Hover over 'add':
// ┌──────────────────────────────┐
// │ fn add(a: int, b: int) -> int│
// │                              │
// │ Adds two numbers             │
// └──────────────────────────────┘
```

**Union Types:**
```liva
let value: int | string = 42
// Hover shows: int | string
```

---

### 5. **Real-time Diagnostics**

**Automatic:** Errors appear as you type

**Error Types:**
- 🔴 **Syntax Errors:** Red squiggles
- 🟡 **Warnings:** Yellow squiggles
- 🔵 **Info:** Blue squiggles

**Example:**
```liva
let x = 10
let y: string = x  // Error: Type mismatch
                   // Expected: string
                   // Got: int
```

**Quick Fixes:** (Coming in v0.13.0)
```liva
let unused = 10  // Warning: Unused variable
                 // 💡 Quick fix: Remove variable
```

---

### 6. **Rename Symbol**

**Trigger:**
- Right-click → "Rename Symbol"
- F2
- Double-click to select + F2

**Features:**
- ✅ **All References:** Updates everywhere
- ✅ **Preview:** See changes before applying
- ✅ **Cross-File:** Works across multiple files
- ✅ **Undo:** Can revert

**Example:**
```liva
let oldName = 10
print(oldName)
let result = oldName * 2

// F2 on 'oldName' → type 'newName'
// All 3 locations updated instantly:

let newName = 10
print(newName)
let result = newName * 2
```

**Safe Refactoring:**
- Won't rename unrelated symbols
- Preserves comments
- Maintains code structure

---

## ⚙️ Configuration

### VS Code Settings

**File:** `.vscode/settings.json`

```json
{
  // Enable/disable LSP
  "liva.lsp.enabled": true,
  
  // Trace level (off, messages, verbose)
  "liva.lsp.trace.server": "off",
  
  // Max diagnostics per file
  "liva.lsp.maxNumberOfProblems": 100,
  
  // Completion trigger characters
  "liva.lsp.completionTriggerCharacters": [".", ":"],
  
  // Debounce time for diagnostics (ms)
  "liva.lsp.diagnostics.debounceMs": 300,
  
  // Show hover on mouse move
  "editor.hover.enabled": true,
  
  // Completion suggestion mode
  "editor.suggest.snippetsPreventQuickSuggestions": false
}
```

### Keyboard Shortcuts

**Default shortcuts:**
- `Ctrl+Space` - Trigger completion
- `F12` - Go to definition
- `Shift+F12` - Find all references
- `F2` - Rename symbol
- `Ctrl+.` - Quick fix (coming soon)
- `Alt+F12` - Peek definition

**Custom shortcuts:** File → Preferences → Keyboard Shortcuts

---

## 🐛 Troubleshooting

### LSP Not Starting

**Symptom:** No completion, no diagnostics

**Solutions:**
1. Check status bar: Should say "Liva LSP: Ready"
2. Check Output panel: View → Output → "Liva Language Server"
3. Restart extension: Cmd+Shift+P → "Reload Window"
4. Verify livac version: `livac --version` (should be ≥ 0.12.0)

**Logs:**
```bash
# Enable verbose logging
"liva.lsp.trace.server": "verbose"

# Check logs
# View → Output → Liva Language Server
```

---

### Slow Completions

**Symptom:** Completion menu takes >1 second

**Solutions:**
1. **Large files:** Split into smaller modules
2. **Too many symbols:** Reduce scope
3. **Memory:** Close unused documents
4. **Restart:** Reload window

**Performance Tips:**
- Keep files under 2000 lines
- Use modules for organization
- Close unused tabs
- Increase debounce time:
  ```json
  "liva.lsp.diagnostics.debounceMs": 500
  ```

---

### Diagnostics Not Updating

**Symptom:** Old errors stay after fixing code

**Solutions:**
1. **Save file:** Ctrl+S (triggers reparse)
2. **Manual refresh:** Close and reopen file
3. **Clear cache:** Restart VS Code
4. **Check syntax:** Ensure code is valid

---

### Go to Definition Not Working

**Symptom:** F12 does nothing or goes to wrong place

**Solutions:**
1. **Cursor position:** Ensure cursor is on symbol name
2. **Symbol type:** Some symbols not yet supported
3. **File not saved:** Save file first
4. **Cross-file:** Ensure both files are in workspace

---

### Extension Crashes

**Symptom:** "Liva language server has crashed"

**Solutions:**
1. **Check logs:** View → Output → Liva Language Server
2. **Report bug:** Include logs and steps to reproduce
3. **Temporary fix:** Disable LSP:
   ```json
   "liva.lsp.enabled": false
   ```
4. **Restart:** Reload window

---

## 💡 Tips & Tricks

### 1. **Peek Definition**
Instead of jumping to definition, peek inline:
- `Alt+F12` - Peek definition
- Navigate without losing context
- Edit definition in-place

### 2. **Multi-Cursor Rename**
Rename multiple symbols at once:
- Select first symbol
- `Ctrl+D` to select next occurrence
- Type new name
- All selected instances update

### 3. **Quick Documentation**
Add documentation that appears in hover:
```liva
/// This function adds two numbers
/// 
/// Parameters:
///   a - First number
///   b - Second number
/// 
/// Returns: Sum of a and b
fn add(a: int, b: int) -> int {
    return a + b
}
```

### 4. **Completion Shortcuts**
- `Tab` - Accept suggestion
- `Enter` - Accept and add newline
- `Esc` - Dismiss
- `↑↓` - Navigate suggestions
- `Ctrl+Space` - Re-trigger

### 5. **Breadcrumbs**
Enable breadcrumb navigation:
```json
"breadcrumbs.enabled": true
```
Shows: File → Function → Current position

---

## 📊 Feature Comparison

| Feature | Without LSP | With LSP |
|---------|-------------|----------|
| **Completion** | None | ✅ Keywords, variables, functions |
| **Error Detection** | On compile only | ✅ Real-time |
| **Navigation** | Manual search | ✅ Go to def, find refs |
| **Refactoring** | Manual find/replace | ✅ Safe rename |
| **Documentation** | External docs | ✅ Inline hover |
| **Code Understanding** | Read all files | ✅ Quick symbol lookup |

---

## 🎯 Best Practices

### 1. **Keep Files Organized**
```
project/
├── src/
│   ├── main.liva
│   ├── utils.liva
│   └── models.liva
└── tests/
    └── test_main.liva
```

### 2. **Use Type Annotations**
Better completion and hover:
```liva
// Good
let name: string = "Alice"
fn process(data: [int]) -> int { ... }

// Also works but less informative
let name = "Alice"
fn process(data) { ... }
```

### 3. **Document Public APIs**
```liva
/// User authentication service
class AuthService {
    /// Logs in a user with credentials
    login(username: string, password: string) -> bool {
        ...
    }
}
```

### 4. **Save Frequently**
- Ctrl+S to trigger reparse
- Auto-save: `"files.autoSave": "afterDelay"`

### 5. **Use Workspace**
- Open folder (not individual files)
- Better cross-file features
- Faster symbol lookup

---

## 🔮 Coming Soon (v0.13.0)

### Code Actions
- 💡 Quick fixes for common errors
- 🔧 Extract function/variable
- 📝 Add missing imports
- 🎨 Format code

### Advanced Navigation
- 📚 Document symbols
- 🔍 Workspace-wide search
- 🌲 Call hierarchy
- 📊 Type hierarchy

### Semantic Highlighting
- Different colors for:
  - Variables vs constants
  - Functions vs methods
  - Classes vs interfaces
  - Mutable vs immutable

---

## 📚 Additional Resources

- [Liva Documentation](https://liva-lang.org/docs)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [VS Code LSP Guide](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)
- [Report Issues](https://github.com/liva-lang/livac/issues)

---

**Version:** 0.12.0  
**Last Updated:** 2025-10-27  
**Need Help?** Open an issue on GitHub or join our Discord
