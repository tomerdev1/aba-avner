#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_step() {
  local label="$1"
  shift

  printf '\n[%s]\n' "${label}"
  "$@"
}

detect_release_platform() {
  case "$(uname -s)" in
    Linux)
      printf 'linux'
      ;;
    Darwin)
      printf 'macos'
      ;;
    MINGW*|MSYS*|CYGWIN*)
      printf 'windows'
      ;;
    *)
      printf 'unsupported'
      ;;
  esac
}

release_bundles_for_platform() {
  local platform="$1"

  case "${platform}" in
    linux)
      printf 'appimage,deb,rpm'
      ;;
    macos)
      printf 'app,dmg'
      ;;
    windows)
      printf 'nsis'
      ;;
    *)
      return 1
      ;;
  esac
}
