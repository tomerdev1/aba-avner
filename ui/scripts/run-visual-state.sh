#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ui_dir="$(cd "${script_dir}/.." && pwd)"

state="${1:-idle}"
port="${UI_VISUAL_PORT:-4173}"
host="${UI_VISUAL_HOST:-127.0.0.1}"

case "${state}" in
  idle|settings|progress|canceling|complete|review|warning-flood|warning-flood-rtl|failure-flood|error)
    ;;
  *)
    cat <<EOF
Unknown visual test state: ${state}

Allowed states:
  idle
  settings
  progress
  canceling
  complete
  review
  warning-flood
  warning-flood-rtl
  failure-flood
  error
EOF
    exit 1
    ;;
esac

encoded_state="${state// /%20}"
url="http://${host}:${port}/index.html?visual-test-state=${encoded_state}"

cd "${ui_dir}"

echo "Serving UI from ${ui_dir}"
echo "Visual state: ${state}"
echo "Open:"
echo "  ${url}"
echo
echo "Press Ctrl+C to stop the server."

exec python3 -m http.server "${port}" --bind "${host}"
