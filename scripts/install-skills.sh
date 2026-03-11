#!/bin/bash
# install-skills.sh — Install Liva AI skills for all supported coding agents
# Called by post-install scripts (.deb/.rpm) and can be run manually.
#
# Usage:
#   install-skills.sh [--uninstall] [--user USERNAME]
#
# Installs to ~/.agents/skills/ (Agent Skills standard, agentskills.io)
# plus legacy compatibility symlinks for agents that don't yet support
# the standard directory.
#
# Source priority:
#   1. dist/skill/liva-lang/ (build output from build-skill.sh)
#   2. /usr/share/livac/skills/liva-lang/ (system package install)

set -euo pipefail

SKILL_NAME="liva-lang"
UNINSTALL=false

# Determine skill source: prefer local build, fall back to system package
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_SOURCE="${PROJECT_ROOT}/dist/skill/${SKILL_NAME}"
SYSTEM_SOURCE="/usr/share/livac/skills/${SKILL_NAME}"

if [ -d "$BUILD_SOURCE" ]; then
    SKILL_SOURCE="$BUILD_SOURCE"
elif [ -d "$SYSTEM_SOURCE" ]; then
    SKILL_SOURCE="$SYSTEM_SOURCE"
else
    echo "⚠ No skill found. Run 'make skill' first or install the livac package."
    exit 1
fi

# Standard directory (agentskills.io) + legacy compatibility
AGENT_DIRS=(
    ".agents/skills"        # Agent Skills standard (all compatible agents)
    ".copilot/skills"       # GitHub Copilot (legacy)
    ".claude/skills"        # Claude Code (legacy)
)

usage() {
    echo "Usage: $0 [--uninstall] [--user USERNAME]"
    echo "  --uninstall   Remove skill instead of installing"
    echo "  --user USER   Install for a specific user (default: all users with /home/*)"
    exit 1
}

# Parse arguments
TARGET_USER=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --uninstall) UNINSTALL=true; shift ;;
        --user) TARGET_USER="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

install_for_user() {
    local user_home="$1"
    local username="$2"

    for agent_dir in "${AGENT_DIRS[@]}"; do
        local target_dir="${user_home}/${agent_dir}"
        local skill_path="${target_dir}/${SKILL_NAME}"

        if $UNINSTALL; then
            if [ -L "$skill_path" ] || [ -d "$skill_path" ]; then
                rm -rf "$skill_path"
                echo "  Removed: ${skill_path}"
            fi
        else
            # Create parent directory
            mkdir -p "$target_dir"
            chown "$username:$username" "$target_dir" 2>/dev/null || true

            # Remove existing (symlink or directory)
            if [ -L "$skill_path" ] || [ -d "$skill_path" ]; then
                rm -rf "$skill_path"
            fi

            # Copy skill directory
            cp -r "$SKILL_SOURCE" "$skill_path"
            chown -R "$username:$username" "$skill_path" 2>/dev/null || true
            echo "  Installed: ${skill_path} (from ${SKILL_SOURCE})"
        fi
    done
}

# Determine which users to install for
get_users() {
    if [ -n "$TARGET_USER" ]; then
        local home_dir
        home_dir=$(eval echo "~${TARGET_USER}")
        if [ -d "$home_dir" ]; then
            echo "${TARGET_USER}:${home_dir}"
        fi
    else
        # All real users with home directories
        for home_dir in /home/*; do
            if [ -d "$home_dir" ]; then
                local username
                username=$(basename "$home_dir")
                echo "${username}:${home_dir}"
            fi
        done
        # Also handle root if running as root
        if [ -d /root ] && [ "$(id -u)" = "0" ]; then
            echo "root:/root"
        fi
    fi
}

# Check that source exists
if [ ! -d "$SKILL_SOURCE" ] && ! $UNINSTALL; then
    echo "⚠ Skill source not found at ${SKILL_SOURCE}."
    echo "  Run 'make skill' to build it first."
    exit 1
fi

action=$($UNINSTALL && echo "Removing" || echo "Installing")
echo "${action} Liva AI skills for coding agents..."

while IFS=':' read -r username home_dir; do
    echo "User: ${username} (${home_dir})"
    install_for_user "$home_dir" "$username"
done < <(get_users)

echo "Done."
