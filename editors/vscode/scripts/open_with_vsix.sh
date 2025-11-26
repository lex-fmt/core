#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$EXT_DIR/../.." && pwd)"

cd "$EXT_DIR"

echo "Building extension bundle..."
npm run bundle

echo "Packaging VSIX..."
npx vsce package --no-dependencies

# Find the latest VSIX file
VSIX_FILE=$(ls -t "$EXT_DIR"/*.vsix 2>/dev/null | head -1)

if [[ -z "$VSIX_FILE" ]]; then
  echo "Error: No VSIX file found after packaging" >&2
  exit 1
fi

echo "Installing $VSIX_FILE..."
code --install-extension "$VSIX_FILE" --force

echo "Opening VS Code with sample workspace..."
WORKSPACE_FILE="$EXT_DIR/test/fixtures/sample-workspace.code-workspace"

if [[ -f "$WORKSPACE_FILE" ]]; then
  exec code "$WORKSPACE_FILE"
else
  exec code "$EXT_DIR/test/fixtures"
fi
