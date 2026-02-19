#!/bin/bash
# Liva Compiler Installer
# Usage:
#   curl -sSf https://raw.githubusercontent.com/liva-lang/livac/main/scripts/install.sh | bash
#   curl -sSf ... | bash -s -- --version v1.3.0
#   curl -sSf ... | bash -s -- --nightly
#
# Environment variables:
#   LIVA_VERSION  - specific version to install (e.g. v1.3.0)
#   LIVA_DIR      - installation directory (default: ~/.liva)

set -euo pipefail

REPO="liva-lang/livac"
INSTALL_DIR="${LIVA_DIR:-$HOME/.liva}"
BIN_DIR="$INSTALL_DIR/bin"
VERSION_FILE="$INSTALL_DIR/version"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

info()  { echo -e "${BLUE}→${NC} $1"; }
ok()    { echo -e "${GREEN}✓${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠${NC} $1"; }
err()   { echo -e "${RED}✗${NC} $1" >&2; }
bold()  { echo -e "${BOLD}$1${NC}"; }

# --- Detect platform ---
detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)   os="linux" ;;
        Darwin)  os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*)
            err "Windows detected. Please use PowerShell installer or Scoop:"
            echo "  scoop bucket add liva-lang https://github.com/liva-lang/livac"
            echo "  scoop install livac"
            exit 1
            ;;
        *)
            err "Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)
            err "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    # Only darwin has arm64 builds
    if [[ "$os" == "linux" && "$arch" == "arm64" ]]; then
        err "Linux ARM64 is not yet supported. Only x64 is available."
        exit 1
    fi

    echo "${os}-${arch}"
}

# --- Get version to install ---
get_version() {
    local version="${LIVA_VERSION:-}"

    # CLI argument overrides env var
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --version)  version="$2"; shift 2 ;;
            --nightly)  version="nightly"; shift ;;
            --help|-h)
                echo "Liva Compiler Installer"
                echo ""
                echo "Usage:"
                echo "  curl -sSf https://raw.githubusercontent.com/liva-lang/livac/main/scripts/install.sh | bash"
                echo "  curl -sSf ... | bash -s -- --version v1.3.0"
                echo ""
                echo "Options:"
                echo "  --version VERSION  Install a specific version (e.g. v1.3.0, v1.3.0-rc7)"
                echo "  --help             Show this help"
                echo ""
                echo "Environment:"
                echo "  LIVA_DIR           Installation directory (default: ~/.liva)"
                echo "  LIVA_VERSION       Version to install (overridden by --version)"
                exit 0
                ;;
            *) shift ;;
        esac
    done

    if [[ -z "$version" ]]; then
        # Fetch latest release from GitHub API
        info "Checking latest version..."
        version=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "\(.*\)".*/\1/')

        if [[ -z "$version" ]]; then
            # Fallback: try listing all releases (in case latest is a prerelease)
            version=$(curl -sSf "https://api.github.com/repos/${REPO}/releases" \
                | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "\(.*\)".*/\1/')
        fi

        if [[ -z "$version" ]]; then
            err "Could not determine latest version. Use --version to specify one."
            exit 1
        fi
    fi

    echo "$version"
}

# --- Check currently installed version ---
get_installed_version() {
    if [[ -f "$VERSION_FILE" ]]; then
        cat "$VERSION_FILE"
    elif [[ -x "$BIN_DIR/livac" ]]; then
        "$BIN_DIR/livac" --version 2>/dev/null | awk '{print "v"$2}' || echo ""
    else
        echo ""
    fi
}

# --- Download and install ---
install() {
    local platform="$1"
    local version="$2"

    local artifact="livac-${platform}"
    local ext="tar.gz"
    local url="https://github.com/${REPO}/releases/download/${version}/${artifact}.${ext}"

    local tmp_dir
    tmp_dir="$(mktemp -d)"
    local archive="${tmp_dir}/${artifact}.${ext}"

    echo ""
    bold "🧩 Liva Compiler Installer"
    echo ""

    local installed
    installed="$(get_installed_version)"
    if [[ -n "$installed" ]]; then
        info "Installed: ${installed}"
        if [[ "$installed" == "$version" ]]; then
            ok "Already up to date (${version})"
            rm -rf "$tmp_dir"
            exit 0
        fi
        info "Updating: ${installed} → ${version}"
    else
        info "Installing: ${version}"
    fi

    info "Platform:   ${platform}"
    info "Target:     ${BIN_DIR}/livac"
    echo ""

    # Download
    info "Downloading ${artifact}.${ext}..."
    local http_code
    http_code=$(curl -sSL -w "%{http_code}" -o "$archive" "$url")

    if [[ "$http_code" != "200" ]]; then
        err "Download failed (HTTP ${http_code})"
        err "URL: ${url}"
        err ""
        err "Available versions: https://github.com/${REPO}/releases"
        rm -rf "$tmp_dir"
        exit 1
    fi

    ok "Downloaded ($(du -h "$archive" | awk '{print $1}'))"

    # Extract
    info "Extracting..."
    mkdir -p "$BIN_DIR"
    tar xzf "$archive" -C "$tmp_dir"

    # Find the binary (it might be in a subdirectory or at root)
    local binary
    binary=$(find "$tmp_dir" -name "livac" -type f -not -path "*/\.*" | head -1)

    if [[ -z "$binary" ]]; then
        err "Could not find livac binary in archive"
        rm -rf "$tmp_dir"
        exit 1
    fi

    # Install binary
    # Remove old binary first (handles "text file busy")
    rm -f "$BIN_DIR/livac"
    cp "$binary" "$BIN_DIR/livac"
    chmod +x "$BIN_DIR/livac"
    ok "Binary installed: ${BIN_DIR}/livac"

    # Install skills and docs if present
    if [[ -d "$tmp_dir/skills" ]]; then
        mkdir -p "$INSTALL_DIR/skills"
        cp -r "$tmp_dir/skills/"* "$INSTALL_DIR/skills/"
        ok "Skills installed"
    fi

    if [[ -d "$tmp_dir/docs" ]]; then
        mkdir -p "$INSTALL_DIR/docs"
        cp -r "$tmp_dir/docs/"* "$INSTALL_DIR/docs/"
        ok "Docs installed"
    fi

    # Run skills installer if present
    if [[ -f "$tmp_dir/install-skills.sh" ]]; then
        bash "$tmp_dir/install-skills.sh" 2>/dev/null || true
    fi

    # Save version
    echo "$version" > "$VERSION_FILE"
    ok "Version saved: ${version}"

    # Verify
    if ! "$BIN_DIR/livac" --version >/dev/null 2>&1; then
        warn "Binary installed but verification failed"
    else
        local actual_ver
        actual_ver=$("$BIN_DIR/livac" --version 2>&1)
        ok "Verified: ${actual_ver}"
    fi

    # Cleanup
    rm -rf "$tmp_dir"

    # Setup PATH
    ensure_path

    echo ""
    ok "${BOLD}Liva ${version} installed successfully!${NC}"
    echo ""

    # Check if livac is accessible
    if command -v livac >/dev/null 2>&1; then
        ok "livac is available in your PATH"
    else
        warn "Restart your terminal or run:"
        echo "    export PATH=\"\$HOME/.liva/bin:\$PATH\""
    fi

    echo ""
}

# --- Ensure ~/.liva/bin is in PATH ---
ensure_path() {
    # Already in PATH?
    if echo "$PATH" | tr ':' '\n' | grep -q "$BIN_DIR"; then
        return
    fi

    local shell_name profile_file export_line

    shell_name="$(basename "${SHELL:-bash}")"

    case "$shell_name" in
        zsh)  profile_file="$HOME/.zshrc" ;;
        fish) profile_file="$HOME/.config/fish/config.fish" ;;
        *)    profile_file="$HOME/.bashrc" ;;
    esac

    # Already configured?
    if [[ -f "$profile_file" ]] && grep -q '.liva/bin' "$profile_file" 2>/dev/null; then
        return
    fi

    case "$shell_name" in
        fish) export_line=$'\n# Liva compiler\nfish_add_path $HOME/.liva/bin\n' ;;
        *)    export_line=$'\n# Liva compiler\nexport PATH="$HOME/.liva/bin:$PATH"\n' ;;
    esac

    echo "$export_line" >> "$profile_file"
    ok "Added ~/.liva/bin to PATH in ${profile_file}"
}

# --- Main ---
main() {
    local platform version

    platform="$(detect_platform)"
    version="$(get_version "$@")"

    install "$platform" "$version"
}

main "$@"
