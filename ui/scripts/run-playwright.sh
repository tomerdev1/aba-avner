#!/usr/bin/env bash
set -euo pipefail

# Playwright forces colored worker and webServer output internally.
# If the parent shell exports NO_COLOR, Node warns about the conflict.
unset NO_COLOR

exec playwright "$@"
