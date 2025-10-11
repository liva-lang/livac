#!/bin/bash

# Liva Compiler - Setup and Commit Script
# This script creates the project structure and commits everything

set -e

echo "🧩 Setting up Liva Compiler project..."
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo -e "${YELLOW}Not a git repository. Initializing...${NC}"
    git init
    echo -e "${GREEN}✓ Git repository initialized${NC}"
fi

# Create directory structure
echo "📁 Creating directory structure..."
mkdir -p src
mkdir -p docs
mkdir -p examples
mkdir -p .github/workflows
mkdir -p .vscode

echo -e "${GREEN}✓ Directories created${NC}"
echo ""

# Reminder to copy files
echo -e "${YELLOW}⚠️  IMPORTANT: You need to copy the following files:${NC}"
echo ""
echo "From the artifacts, copy to:"
echo "  - Cargo.toml"
echo "  - README.md"
echo "  - QUICKSTART.md"
echo "  - Makefile"
echo "  - .gitignore"
echo "  - install.sh"
echo ""
echo "  src/:"
echo "    - main.rs"
echo "    - lib.rs"
echo "    - ast.rs"
echo "    - lexer.rs"
echo "    - parser.rs"
echo "    - semantic.rs"
echo "    - desugaring.rs"
echo "    - codegen.rs"
echo "    - error.rs"
echo ""
echo "  examples/:"
echo "    - examples.liva"
echo ""
echo "  .github/workflows/:"
echo "    - ci.yml"
echo ""
echo "  .vscode/:"
echo "    - settings.json"
echo ""

read -p "Have you copied all the files? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}Please copy all files first, then run this script again.${NC}"
    exit 1
fi

# Check if critical files exist
if [ ! -f Cargo.toml ]; then
    echo -e "${RED}❌ Cargo.toml not found!${NC}"
    exit 1
fi

if [ ! -f src/main.rs ]; then
    echo -e "${RED}❌ src/main.rs not found!${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Files verified${NC}"
echo ""

# Make install.sh executable
if [ -f install.sh ]; then
    chmod +x install.sh
    echo -e "${GREEN}✓ Made install.sh executable${NC}"
fi

# Create or switch to feature branch
BRANCH_NAME="feature/compiler-implementation"
echo "🌿 Creating branch: $BRANCH_NAME"

if git show-ref --verify --quiet refs/heads/$BRANCH_NAME; then
    echo -e "${YELLOW}Branch already exists. Switching to it...${NC}"
    git checkout $BRANCH_NAME
else
    git checkout -b $BRANCH_NAME
fi

echo -e "${GREEN}✓ On branch: $BRANCH_NAME${NC}"
echo ""

# Stage all files
echo "📦 Staging files..."
git add .

# Show status
echo ""
echo "📊 Git status:"
git status --short

echo ""
read -p "Proceed with commit? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Commit cancelled.${NC}"
    exit 0
fi

# Commit
echo "💾 Committing changes..."
git commit -m "feat: implement Liva v0.6 compiler

Complete implementation of the Liva programming language compiler that
transpiles to Rust code.

Components:
- Lexer: Token generation with Logos
- Parser: Complete AST construction
- Semantic Analyzer: Type checking and async inference
- Desugaring: Liva to Rust concept transformation
- Code Generator: Rust code and Cargo.toml emission

Language Features:
- Variables and constants with type inference
- Functions (one-liner and block syntax)
- Classes with inheritance support
- Visibility levels (public, protected, private)
- Async/await with Tokio runtime
- Parallel computing with OS threads
- Task-based concurrency control
- Fire-and-forget execution
- String templates
- Logical operators (and/or/not and &&/||/!)
- Complete control flow (if/while/for/switch)
- Error handling (try/catch/throw)
- Rust crate interoperability

Documentation:
- Complete language specification
- EBNF grammar and AST definition
- Desugaring rules reference
- README with examples
- Quickstart guide
- 14 example programs

Build System:
- Makefile for common tasks
- GitHub Actions CI/CD pipeline
- VSCode configuration
- Installation script

Testing:
- Unit tests for all modules
- Integration test examples
- CI pipeline with multiple platforms
"

echo -e "${GREEN}✓ Changes committed${NC}"
echo ""

# Ask about remote
echo "🌐 Remote repository setup"
read -p "Do you want to push to a remote repository? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Check if remote exists
    if git remote | grep -q origin; then
        echo "Remote 'origin' already configured:"
        git remote get-url origin
        echo ""
        read -p "Push to this remote? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "📤 Pushing to remote..."
            git push -u origin $BRANCH_NAME
            echo -e "${GREEN}✓ Pushed to remote!${NC}"
        fi
    else
        echo "No remote configured."
        read -p "Enter remote URL (or press Enter to skip): " remote_url
        if [ ! -z "$remote_url" ]; then
            git remote add origin "$remote_url"
            echo "📤 Pushing to remote..."
            git push -u origin $BRANCH_NAME
            echo -e "${GREEN}✓ Pushed to remote!${NC}"
        fi
    fi
fi

echo ""
echo -e "${GREEN}🎉 Setup complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Test the build: make build"
echo "  2. Run tests: make test"
echo "  3. Create a pull request on GitHub"
echo ""
echo "Current branch: $BRANCH_NAME"
echo ""
