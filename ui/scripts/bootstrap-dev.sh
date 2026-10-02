#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ui_dir="$(cd "${script_dir}/.." && pwd)"

(
  cd "${ui_dir}"
  npm install
  npx playwright install chromium
)
