#!/bin/bash

# Liva Compiler Installation Script
# Version 0.6.0

set -e

# Get the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go to the project root (parent of scripts/)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root directory
cd "$PROJECT_ROOT"

echo "🧩 Liva Compiler Installation Script"
echo "======================================"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✓ Rust toolchain found"
echo ""

# Check Rust version
RUST_VERSION=$(rustc --version | awk '{print $2}')
echo "  Rust version: $RUST_VERSION"
echo ""

# Build the compiler
echo "🔨 Building Liva compiler..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Build successful!"
    echo ""
else
    echo ""
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi

# Determine installation directory
INSTALL_DIR="${HOME}/.local/bin"

# Create installation directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating installation directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi

# Copy binary
echo "📦 Installing livac to $INSTALL_DIR..."
cp target/release/livac "$INSTALL_DIR/"

if [ $? -eq 0 ]; then
    echo "✓ Installation successful!"
    echo ""
else
    echo "❌ Installation failed."
    exit 1
fi

# Check if installation directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Warning: $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add the following line to your shell configuration file:"
    echo "  (e.g., ~/.bashrc, ~/.zshrc, ~/.profile)"
    echo ""
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then run: source ~/.bashrc (or your shell config file)"
    echo ""
else
    echo "✓ $INSTALL_DIR is in your PATH"
    echo ""
fi

# Verify installation
if command -v livac &> /dev/null; then
    echo "🎉 Liva compiler installed successfully!"
    echo ""
    echo "Usage:"
    echo "  livac <input.liva> [OPTIONS]"
    echo ""
    echo "Examples:"
    echo "  livac run hello.liva"
    echo "  livac build --verbose my_app.liva"
    echo "  livac check program.liva"
    echo ""
    echo "For more information, run: livac --help"
else
    echo "⚠️  livac command not found in PATH"
    echo "You may need to add $INSTALL_DIR to your PATH"
fi

echo ""
echo "📚 Documentation available in docs/ directory"
echo "💡 Example programs available in examples/ directory"
echo ""
echo "Happy coding with Liva! 🚀"
