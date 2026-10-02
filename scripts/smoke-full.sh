#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/verification-common.sh"

run_step "Capability review" \
  python3 "${repo_root}/src-tauri/scripts/check_tauri_capabilities.py"

run_step "Rust dependency audit" \
  bash -lc "cd \"${repo_root}/src-tauri\" && cargo audit"

run_step "Production npm audit" \
  bash -lc "cd \"${repo_root}/ui\" && npm audit --omit=dev"

run_step "Backend tests without GUI features" \
  bash -lc "cd \"${repo_root}/src-tauri\" && cargo test --no-default-features"

run_step "Ignored dataset smoke test" \
  bash -lc "cd \"${repo_root}/src-tauri\" && cargo test synthetic_dataset_full_hash_smoke -- --ignored --exact --nocapture"

run_step "Backend tests with GUI features" \
  bash -lc "cd \"${repo_root}/src-tauri\" && cargo test"

run_step "UI tests" \
  bash -lc "cd \"${repo_root}/ui\" && npm test"

run_step "Visual regression tests" \
  bash -lc "cd \"${repo_root}/ui\" && npm run test:visual"
