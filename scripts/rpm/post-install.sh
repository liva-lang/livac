#!/bin/bash
# post-install.sh — RPM post-install script for livac
# 1. Ensures Rust toolchain is available
# 2. Installs AI skill symlinks for all coding agents
set -e

# --- Rust dependency ---
if command -v cargo >/dev/null 2>&1; then
    echo "livac: Rust toolchain found ($(cargo --version))"
else
    echo "livac: Rust toolchain not found. Installing via rustup..."
    if [ "$(id -u)" = "0" ]; then
        REAL_USER=$(stat -c '%U' /home/* 2>/dev/null | head -1 || echo "")
        if [ -n "$REAL_USER" ]; then
            su - "$REAL_USER" -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y' || {
                echo "livac: WARNING - Could not install Rust automatically."
                echo "livac: Please install manually: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            }
        fi
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y || {
            echo "livac: WARNING - Could not install Rust automatically."
            echo "livac: Please install manually: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        }
    fi
fi

# --- AI Skills ---
if [ -x /usr/share/livac/scripts/install-skills.sh ]; then
    /usr/share/livac/scripts/install-skills.sh
fi

exit 0
