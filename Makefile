.PHONY: build build-bootstrap build-selfhost livac test test-full clean install run fmt clippy doc examples skill help

# Default target — produce the full release set:
#   target/release/livac           Rust bootstrap (gen-0 seed; still the
#                                  user-facing binary for now — dispatches
#                                  fmt/lint/lsp to liva-tools)
#   target/release/liva-tools      Rust developer tools (fmt/lint/lsp)
#   target/livac-gen2-release      Liva-self-hosted gen-2 (canonical compiler
#                                  reference, used by self-host test gates)
# Idempotence (gen-2 ≡ gen-3) is verified by rebuild_selfhost.sh.
all: livac

# `make livac` post-Phase-F build flow:
#   1. Build the Rust bootstrap + liva-tools (workspace build).
#   2. Drive bootstrap -> gen-1 -> gen-2 -> gen-3 and assert idempotence.
#   3. Stage gen-2 at target/livac-gen2-release for the gate scripts.
livac: build-bootstrap build-selfhost

# Step 1 — cargo workspace build (livac-bootstrap + liva-tools).
build-bootstrap:
	@echo "🔨 Building Rust bootstrap (livac-bootstrap + liva-tools)..."
	@cargo build --release --workspace
	@echo "✓ Bootstrap + tools at target/release/{livac,liva-tools}"

# Step 2 — self-host gen-2 build + idempotence check via rebuild_selfhost.sh.
# Stages gen-2 at target/livac-gen2-release so all run_all.sh gates pick it up.
build-selfhost: build-bootstrap
	@echo "🚀 Building self-host gen-2 (canonical Liva compiler)..."
	@TMPDIR=$${TMPDIR:-/tmp} bash compiler/tests/rebuild_selfhost.sh
	@cp $${TMPDIR:-/tmp}/gen2_build/target/release/main target/livac-gen2-release
	@echo "✓ Canonical gen-2 staged at target/livac-gen2-release"

# Compatibility alias for pre-F.4 muscle memory — Rust-only build.
build: build-bootstrap

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

# Run all 5 self-host gates + cargo tests (full validation)
test-full:
	@bash compiler/tests/run_all.sh

# Quick gates (skip rebuild_selfhost — useful for inner dev loop)
test-quick:
	@bash compiler/tests/run_all.sh --quick

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

# Deep clean: cargo + nested target/, .liva_build/, node_modules, dist, .astro
clean-all:
	@bash scripts/clean.sh --yes

# Prune stale Cargo build artifacts (keeps recent, deletes old).
# Requires: cargo install cargo-sweep --locked
sweep:
	@echo "🧹 Sweeping stale build artifacts (>30 days)..."
	@cargo sweep --time 30 || echo "  (install cargo-sweep first: cargo install cargo-sweep --locked)"
	@echo "✓ Sweep complete"

# Build AI skill from docs/
skill:
	@scripts/build-skill.sh

# Install to ~/.local/bin
install: build skill
	@echo "📦 Installing livac..."
	@mkdir -p ~/.local/bin
	@cp target/release/livac ~/.local/bin/
	@echo "✓ Installed to ~/.local/bin/livac"
	@echo "   Make sure ~/.local/bin is in your PATH"
	@scripts/install-skills.sh --user $$(whoami)

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

# Coverage report (cargo-llvm-cov). Requires:
#   cargo install cargo-llvm-cov --locked
#   rustup component add llvm-tools-preview
coverage:
	@echo "📊 Generating coverage report..."
	@cargo llvm-cov --summary-only --quiet
	@echo ""
	@echo "  → HTML report: cargo llvm-cov --open"

coverage-html:
	@echo "📊 Generating HTML coverage report..."
	@cargo llvm-cov --html --quiet
	@echo "  ✓ Opened in browser via target/llvm-cov/html/index.html"

# Watch for changes and rebuild
watch:
	@echo "👀 Watching for changes..."
	@cargo watch -x build

# Full pipeline: format, clippy, test, build, skill
ci: fmt clippy test build skill
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
	@echo "  make doc           - Generate documentation"	@echo "  make skill         - Build AI skill from docs/"	@echo "  make examples      - Create examples directory"
	@echo "  make bench         - Run benchmarks"
	@echo "  make watch         - Watch and rebuild"
	@echo "  make ci            - Full CI pipeline"
	@echo "  make help          - Show this help"
	@echo ""
