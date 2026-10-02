#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/verification-common.sh"

platform="${1:-$(detect_release_platform)}"
if ! bundles="$(release_bundles_for_platform "${platform}")"; then
  printf 'Unsupported release platform: %s\n' "${platform}" >&2
  printf 'Use one of: linux, macos, windows\n' >&2
  exit 1
fi

run_step "Full smoke verification" \
  "${repo_root}/scripts/smoke-full.sh"

run_step "UI production build" \
  bash -lc "cd \"${repo_root}/ui\" && npm run build"

run_step "Tauri bundle build (${platform})" \
  bash -lc "cd \"${repo_root}/src-tauri\" && cargo tauri build --bundles ${bundles}"

run_step "Bundle artifact verification (${platform})" \
  python3 "${repo_root}/src-tauri/scripts/verify_bundle_outputs.py" "${platform}"
