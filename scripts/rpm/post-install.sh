#!/bin/bash
# post-install.sh — RPM post-install script for livac
# Installs AI skill symlinks for all coding agents
set -e

if [ -x /usr/share/livac/scripts/install-skills.sh ]; then
    /usr/share/livac/scripts/install-skills.sh
fi

exit 0
