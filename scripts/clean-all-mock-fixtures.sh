#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/clean-all-mock-fixtures.sh <target-dir>

Arguments:
  target-dir   Base directory where fixture folders will be removed.

This script removes:
  <target-dir>/env-fixtures
  <target-dir>/file-fixtures
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 1
fi

target_dir=$1

if [[ ! -d "$target_dir" ]]; then
  echo "Directory not found: $target_dir" >&2
  exit 1
fi

env_target="$target_dir/env-fixtures"
file_target="$target_dir/file-fixtures"

echo "Removing mock fixtures from: $target_dir"

if [[ -d "$env_target" ]]; then
  echo "Removing $env_target..."
  rm -rf "$env_target"
fi

if [[ -d "$file_target" ]]; then
  echo "Removing $file_target..."
  rm -rf "$file_target"
fi

echo "Done!"
