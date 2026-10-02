#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

(
  cd "${repo_root}/src-tauri"
  cargo llvm-cov --html --output-dir ../coverage/backend
)

(
  cd "${repo_root}/ui"
  npm run coverage
)

printf '\nCoverage reports generated:\n'
printf '  Backend: %s\n' "${repo_root}/coverage/backend/html/index.html"
printf '  UI: %s\n' "${repo_root}/ui/coverage/index.html"
