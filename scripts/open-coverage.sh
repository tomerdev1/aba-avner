#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
backend_report="${repo_root}/coverage/backend/html/index.html"
ui_report="${repo_root}/ui/coverage/index.html"

for report in "${backend_report}" "${ui_report}"; do
  if [[ ! -f "${report}" ]]; then
    printf 'Missing coverage report: %s\n' "${report}" >&2
    printf 'Run ./scripts/coverage.sh first.\n' >&2
    exit 1
  fi
done

if command -v xdg-open >/dev/null 2>&1; then
  xdg-open "${backend_report}" >/dev/null 2>&1 &
  xdg-open "${ui_report}" >/dev/null 2>&1 &
elif command -v open >/dev/null 2>&1; then
  open "${backend_report}"
  open "${ui_report}"
else
  printf 'No supported opener found. Open these files manually:\n'
  printf '  %s\n' "${backend_report}"
  printf '  %s\n' "${ui_report}"
  exit 1
fi
