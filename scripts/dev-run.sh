#!/bin/sh
# Local dev runner for serval — test the bundled config/agents without cutting a
# release. Assembles a throwaway install root whose `bundle/` points at the repo's
# assets/default-bundle and whose `opencode` points at your system opencode, then
# runs the freshly-built serval against it via SERVAL_INSTALL_ROOT.
#
# Usage:  scripts/dev-run.sh [serval args...]
#   scripts/dev-run.sh status
#   scripts/dev-run.sh code            # needs a token first: scripts/dev-run.sh auth
#
# Tip: to check ONLY that agents load (no serval/token), point opencode straight
# at the bundle:  OPENCODE_CONFIG_DIR="$PWD/assets/default-bundle" opencode
set -eu

repo="$(cd "$(dirname "$0")/.." && pwd)"
root="${SERVAL_DEV_ROOT:-${TMPDIR:-/tmp}/serval-dev}"

opencode_bin="$(command -v opencode || true)"
if [ -z "$opencode_bin" ]; then
  echo "opencode not found on PATH — install it, or set OPENCODE_CONFIG_DIR and run opencode directly." >&2
  exit 1
fi

cargo build --quiet --manifest-path "$repo/Cargo.toml"

mkdir -p "$root"
ln -sfn "$repo/assets/default-bundle" "$root/bundle"
ln -sfn "$opencode_bin" "$root/opencode"

echo "dev root: $root  (bundle → assets/default-bundle, opencode → $opencode_bin)" >&2
SERVAL_INSTALL_ROOT="$root" "$repo/target/debug/serval" "$@"
