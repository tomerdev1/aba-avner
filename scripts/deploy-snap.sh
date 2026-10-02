#!/usr/bin/env bash
# Build the snap on this host (destructive mode - no lxd/multipass needed
# since this host already matches the core24 base) and push it to the
# Snap Store.
#
# Usage: sudo ./scripts/deploy-snap.sh [channel]
#   channel defaults to "edge". Pass "stable" once you've smoke-tested.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
channel="${1:-edge}"

if [[ "${EUID}" -ne 0 ]]; then
  echo "Run with sudo: sudo $0 ${channel}" >&2
  exit 1
fi

# ponytail: root has no keyring, so `snapcraft upload` can't stash/read store
# creds there. One-time fix as your normal user:
#   snapcraft export-login .snapcraft-credentials
creds_file="${repo_root}/.snapcraft-credentials"
if [[ -z "${SNAPCRAFT_STORE_CREDENTIALS:-}" && -f "${creds_file}" ]]; then
  export SNAPCRAFT_STORE_CREDENTIALS="$(cat "${creds_file}")"
fi

cd "${repo_root}"
snapcraft --destructive-mode

snap_file="$(ls -t ./*.snap | head -n1)"
snapcraft upload --release="${channel}" "${snap_file}"

# ponytail: leftover build dirs are root-owned; wipe them so `git status`
# stays clean instead of chowning them back.
rm -rf parts prime stage .craft ./*.snap
