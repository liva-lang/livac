#!/bin/bash
# pre-uninstall.sh — RPM pre-uninstall script for livac
# Removes AI skill symlinks
set -e

if [ -x /usr/share/livac/scripts/install-skills.sh ]; then
    /usr/share/livac/scripts/install-skills.sh --uninstall
fi

exit 0
