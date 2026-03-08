#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/generate-all-mock-fixtures.sh <target-dir> [count] [env-prefix] [file-prefix]

Arguments:
  target-dir   Base directory where fixture folders will be created.
  count        Number of .env files and regular files to generate. Defaults to 50.
  env-prefix   Prefix for generated .env files. Defaults to vault_env.
  file-prefix  Prefix for generated regular files. Defaults to vault_file.

This wrapper creates:
  <target-dir>/env-fixtures
  <target-dir>/file-fixtures
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 || $# -gt 4 ]]; then
  usage >&2
  exit 1
fi

target_dir=$1
count=${2:-50}
env_prefix=${3:-vault_env}
file_prefix=${4:-vault_file}

if ! [[ "$count" =~ ^[0-9]+$ ]] || [[ "$count" -eq 0 ]]; then
  echo "count must be a positive integer" >&2
  exit 1
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
project_root=$(cd "$script_dir/.." && pwd)

env_target="$target_dir/env-fixtures"
file_target="$target_dir/file-fixtures"

mkdir -p "$env_target" "$file_target"

cd "$project_root"

echo "Generating .env fixtures into $env_target"
cargo run -p vault-cli --bin generate-mock-env-fixtures -- "$env_target" "$count" "$env_prefix"

echo
echo "Generating hardcoded file fixtures into $file_target"
cargo run -p vault-cli --bin generate-mock-file-fixtures -- "$file_target" "$count" "$file_prefix"

echo
echo "Done."
echo "env fixtures:  $env_target"
echo "file fixtures: $file_target"
