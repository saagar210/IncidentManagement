#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TMP_BASE="${TMPDIR:-/tmp}"
LEAN_TMP_ROOT="$(mktemp -d "$TMP_BASE/incidentmgmt-lean-dev.XXXXXX")"
LEAN_CARGO_TARGET="$LEAN_TMP_ROOT/cargo-target"
LEAN_VITE_CACHE="$LEAN_TMP_ROOT/vite-cache"

cleanup() {
  rm -rf "$LEAN_TMP_ROOT"
}

trap cleanup EXIT INT TERM

mkdir -p "$LEAN_CARGO_TARGET" "$LEAN_VITE_CACHE"

export CARGO_TARGET_DIR="$LEAN_CARGO_TARGET"
export VITE_CACHE_DIR="$LEAN_VITE_CACHE"

echo "[lean-dev] Using temporary CARGO_TARGET_DIR: $CARGO_TARGET_DIR"
echo "[lean-dev] Using temporary VITE_CACHE_DIR: $VITE_CACHE_DIR"
echo "[lean-dev] Temporary build artifacts will be removed when dev exits."

cd "$REPO_ROOT"
pnpm tauri dev "$@"
