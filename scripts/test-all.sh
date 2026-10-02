#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
green=$'\033[1;32m'
reset=$'\033[0m'

print_success_banner() {
  printf '\n'
  printf '%s\n' "${green}========================================${reset}"
  printf '%s\n' "${green}           ALL TESTS PASSED             ${reset}"
  printf '%s\n' "${green}========================================${reset}"
}

(
  cd "${repo_root}/src-tauri"
  cargo test
)

(
  cd "${repo_root}/ui"
  npm test
)

(
  cd "${repo_root}/ui"
  npm run test:visual
)

print_success_banner
