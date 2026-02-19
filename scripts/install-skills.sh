#!/bin/bash
# install-skills.sh — Install Liva AI skills for all supported coding agents
# Called by post-install scripts (.deb/.rpm) and can be run manually.
#
# Usage:
#   install-skills.sh [--uninstall] [--user USERNAME]
#
# Creates symlinks from each agent's skills directory to
# /usr/share/livac/skills/liva-lang/ so that updating livac
# automatically updates the skills for all agents.

set -euo pipefail

SKILL_SOURCE="/usr/share/livac/skills/liva-lang"
SKILL_NAME="liva-lang"
UNINSTALL=false

# Agent directories (relative to $HOME)
AGENT_DIRS=(
    ".copilot/skills"
    ".claude/skills"
    ".codex/skills"
    ".cursor/skills"
    ".codeium/windsurf/skills"
    ".gemini/skills"
    ".gemini/antigravity/skills"
    ".continue/skills"
    ".openclaw/skills"
)

usage() {
    echo "Usage: $0 [--uninstall] [--user USERNAME]"
    echo "  --uninstall   Remove symlinks instead of creating them"
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
        local link_path="${target_dir}/${SKILL_NAME}"

        if $UNINSTALL; then
            if [ -L "$link_path" ]; then
                rm -f "$link_path"
                echo "  Removed: ${link_path}"
            fi
        else
            # Create parent directory
            mkdir -p "$target_dir"
            chown "$username:$username" "$target_dir" 2>/dev/null || true

            # Create or update symlink
            if [ -L "$link_path" ]; then
                rm -f "$link_path"
            elif [ -d "$link_path" ]; then
                # If a real directory exists, skip (user may have custom content)
                echo "  Skip (real dir): ${link_path}"
                continue
            fi

            ln -s "$SKILL_SOURCE" "$link_path"
            chown -h "$username:$username" "$link_path" 2>/dev/null || true
            echo "  Linked: ${link_path} → ${SKILL_SOURCE}"
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

# Check that source exists (skip if not installed yet — e.g. during build)
if [ ! -d "$SKILL_SOURCE" ] && ! $UNINSTALL; then
    echo "Warning: ${SKILL_SOURCE} not found. Skills will be linked on next install."
    exit 0
fi

action=$($UNINSTALL && echo "Removing" || echo "Installing")
echo "${action} Liva AI skills for coding agents..."

while IFS=':' read -r username home_dir; do
    echo "User: ${username} (${home_dir})"
    install_for_user "$home_dir" "$username"
done < <(get_users)

echo "Done."
