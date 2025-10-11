.PHONY: build test clean install run fmt clippy doc examples help

# Default target
all: build

# Build the compiler in release mode
build:
	@echo "🔨 Building Liva compiler..."
	@cargo build --release
	@echo "✓ Build complete: target/release/livac"

# Build in debug mode
debug:
	@echo "🔨 Building in debug mode..."
	@cargo build
	@echo "✓ Debug build complete: target/debug/livac"

# Run tests
test:
	@echo "🧪 Running tests..."
	@cargo test
	@echo "✓ All tests passed"

# Run tests with output
test-verbose:
	@echo "🧪 Running tests (verbose)..."
	@cargo test -- --nocapture
	@echo "✓ All tests passed"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/liva_build
	@echo "✓ Clean complete"

# Install to ~/.local/bin
install: build
	@echo "📦 Installing livac..."
	@mkdir -p ~/.local/bin
	@cp target/release/livac ~/.local/bin/
	@echo "✓ Installed to ~/.local/bin/livac"
	@echo "   Make sure ~/.local/bin is in your PATH"

# Uninstall
uninstall:
	@echo "🗑️  Uninstalling livac..."
	@rm -f ~/.local/bin/livac
	@echo "✓ Uninstalled"

# Run example
run-example:
	@echo "▶️  Running hello.liva example..."
	@cargo run -- examples/hello.liva --run

# Format code
fmt:
	@echo "🎨 Formatting code..."
	@cargo fmt
	@echo "✓ Code formatted"

# Run clippy linter
clippy:
	@echo "📎 Running clippy..."
	@cargo clippy -- -D warnings
	@echo "✓ Clippy passed"

# Check code (fast)
check:
	@echo "✓ Checking code..."
	@cargo check
	@echo "✓ Check complete"

# Generate documentation
doc:
	@echo "📚 Generating documentation..."
	@cargo doc --no-deps --open
	@echo "✓ Documentation generated"

# Create examples directory
examples:
	@mkdir -p examples
	@echo "sum(a: number, b: number): number = a + b\n\nmain() {\n  print(sum(5, 3))\n}" > examples/hello.liva
	@echo "✓ Examples directory created"

# Run benchmarks
bench:
	@echo "⏱️  Running benchmarks..."
	@cargo bench
	@echo "✓ Benchmarks complete"

# Watch for changes and rebuild
watch:
	@echo "👀 Watching for changes..."
	@cargo watch -x build

# Full pipeline: format, clippy, test, build
ci: fmt clippy test build
	@echo "✓ CI pipeline complete"

# Help
help:
	@echo "Liva Compiler - Available Commands:"
	@echo ""
	@echo "  make build         - Build release version"
	@echo "  make debug         - Build debug version"
	@echo "  make test          - Run tests"
	@echo "  make test-verbose  - Run tests with output"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make install       - Install to ~/.local/bin"
	@echo "  make uninstall     - Remove from ~/.local/bin"
	@echo "  make run-example   - Run example program"
	@echo "  make fmt           - Format code"
	@echo "  make clippy        - Run linter"
	@echo "  make check         - Fast code check"
	@echo "  make doc           - Generate documentation"
	@echo "  make examples      - Create examples directory"
	@echo "  make bench         - Run benchmarks"
	@echo "  make watch         - Watch and rebuild"
	@echo "  make ci            - Full CI pipeline"
	@echo "  make help          - Show this help"
	@echo ""
