#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_DIR="${ROOT_DIR}/../ui"
NVM_DIR="${NVM_DIR:-${HOME}/.nvm}"
NODE_BIN=""

has_supported_node() {
  local node_bin="${1:-node}"
  command -v "${node_bin}" >/dev/null 2>&1 || return 1
  "${node_bin}" -e 'process.exit(Number(process.versions.node.split(".")[0]) >= 18 ? 0 : 1)'
}

find_supported_nvm_node() {
  local versions_dir="${NVM_DIR}/versions/node"

  [ -d "${versions_dir}" ] || return 1

  find "${versions_dir}" -mindepth 3 -maxdepth 3 -path '*/bin/node' -type f 2>/dev/null \
    | while IFS= read -r candidate; do
        local version_dir
        version_dir="$(basename "$(dirname "$(dirname "${candidate}")")")"
        local version_number="${version_dir#v}"
        if "${candidate}" -e 'process.exit(Number(process.versions.node.split(".")[0]) >= 18 ? 0 : 1)' >/dev/null 2>&1; then
          printf '%s %s\n' "${version_number}" "${candidate}"
        fi
      done \
    | sort -V \
    | tail -n 1 \
    | awk '{ print $2 }'
}

if has_supported_node node; then
  NODE_BIN="$(command -v node)"
else
  NODE_BIN="$(find_supported_nvm_node || true)"
fi

if [ -z "${NODE_BIN}" ]; then
  echo "Aba Avner requires Node.js 18+ to build the UI. Current node: $(command -v node || echo missing)" >&2
  exit 1
fi

"${NODE_BIN}" "${UI_DIR}/scripts/build-ui.mjs"
