#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

paths=(
  "dist"
  "src-tauri/target"
  "node_modules/.vite"
  "coverage"
)

removed_any=0

for rel_path in "${paths[@]}"; do
  abs_path="$REPO_ROOT/$rel_path"
  if [ -e "$abs_path" ]; then
    rm -rf "$abs_path"
    echo "[clean:heavy] removed $rel_path"
    removed_any=1
  else
    echo "[clean:heavy] not present $rel_path"
  fi
done

if [ "$removed_any" -eq 0 ]; then
  echo "[clean:heavy] nothing to remove"
fi
