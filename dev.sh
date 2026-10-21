#!/bin/sh
# Runs ToneSnap in dev mode on macOS from any folder: `./dev.sh`.
# Needs Node and Rust (~/.cargo/bin). Tauri's own CLI comes from node_modules and starts Vite.
set -eu
cd "$(dirname "$0")"
PATH="$HOME/.cargo/bin:$PATH"
for tool in node cargo; do
  command -v "$tool" >/dev/null 2>&1 || { echo "$tool is missing" >&2; exit 1; }
done
[ -x node_modules/.bin/tauri ] || { echo "Run pnpm install first" >&2; exit 1; }
exec node_modules/.bin/tauri dev
