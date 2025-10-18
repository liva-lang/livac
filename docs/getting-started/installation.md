# 🚀 Installation

This guide will help you install the Liva compiler on your system.

## Prerequisites

- **Rust** 1.70 or later
- **Cargo** (comes with Rust)
- **Git** (for cloning the repository)

## Installing Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit [https://rustup.rs/](https://rustup.rs/) for platform-specific instructions.

## Installing Liva

### From Source (Recommended)

1. **Clone the repository:**

```bash
git clone https://github.com/liva-lang/livac.git
cd livac
```

2. **Build the compiler:**

```bash
cargo build --release
```

This will create the `livac` binary in `target/release/livac`.

3. **Install globally (optional):**

```bash
cargo install --path .
```

This installs `livac` to `~/.cargo/bin/`, which should be in your PATH.

### Verify Installation

Check that Liva is correctly installed:

```bash
livac --version
```

You should see:
```
livac 0.6.0
```

## Setting Up Your First Project

1. **Create a new directory:**

```bash
mkdir my-liva-project
cd my-liva-project
```

2. **Create a Liva file:**

```bash
touch main.liva
```

3. **Write some code:**

```liva
main() {
  print("Hello, Liva!")
}
```

4. **Compile and run:**

```bash
livac main.liva --run
```

## IDE Support

### VS Code

Install the Liva VS Code extension for syntax highlighting, IntelliSense, and more:

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Liva"
4. Click Install

Or install from VSIX:

```bash
cd vscode-extension
npm install
npm run compile
code --install-extension liva-vscode-0.1.0.vsix
```

### Vim/Neovim

Syntax highlighting files are available in `editors/vim/`.

### Emacs

Emacs mode is available in `editors/emacs/`.

## Environment Variables

### Optional Configuration

- `LIVAC_SKIP_CARGO` - Skip Cargo build step (useful for testing code generation)
- `LIVAC_OUTPUT` - Default output directory (overrides `--output`)
- `RUST_LOG` - Enable debug logging (`RUST_LOG=debug livac file.liva`)

## Troubleshooting

### "livac: command not found"

Make sure `~/.cargo/bin` is in your PATH:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Cargo Build Fails

Ensure you have the latest Rust version:

```bash
rustup update
```

### Permission Denied

On Linux/macOS, you might need to make the binary executable:

```bash
chmod +x target/release/livac
```

## Updating Liva

To update to the latest version:

```bash
cd livac
git pull origin main
cargo build --release
```

Or if installed via `cargo install`:

```bash
cargo install --path . --force
```

## Uninstalling

If you used `cargo install`:

```bash
cargo uninstall livac
```

Otherwise, simply delete the `livac` directory.

## Next Steps

- **[Quick Start Guide](quick-start.md)** - Build your first Liva program
- **[Basic Concepts](basic-concepts.md)** - Learn core Liva concepts
- **[Examples](examples.md)** - Explore example programs

## Platform-Specific Notes

### Windows

On Windows, use PowerShell or WSL (Windows Subsystem for Linux) for the best experience.

### macOS

macOS users may need to install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Linux

Most Linux distributions work out of the box. Ensure you have `gcc` or `clang` installed:

```bash
# Debian/Ubuntu
sudo apt install build-essential

# Fedora/RHEL
sudo dnf install gcc

# Arch
sudo pacman -S base-devel
```

## Getting Help

- 📚 **Documentation**: See the [main docs](../README.md)
- 💬 **Community**: Join our GitHub Discussions
- 🐛 **Issues**: Report bugs on GitHub Issues

---

**You're all set! Let's write some Liva code! 🎉**
