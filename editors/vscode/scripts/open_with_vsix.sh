#!/usr/bin/env bash
# Opens a clean VS Code instance with the extension installed via VSIX.
# Unlike open_dev_vscode.sh (which uses --extensionDevelopmentPath), this
# tests the actual packaged extension as users would install it.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
EXT_DIR="$REPO_ROOT/editors/vscode"
WORKSPACE_FILE="$EXT_DIR/test/fixtures/sample-workspace.code-workspace"
USER_DATA_DIR="$EXT_DIR/.vscode-vsix-test"
EXTENSIONS_DIR="$USER_DATA_DIR/extensions"

if ! command -v code >/dev/null 2>&1; then
  echo "VS Code CLI (code) not found on PATH. Install VS Code and ensure 'code' is available."
  exit 1
fi

cd "$EXT_DIR"

echo "Building extension bundle..."
npm run bundle

echo "Packaging VSIX..."
npx vsce package --no-dependencies

# Find the latest VSIX file
VSIX_FILE=$(ls -t "$EXT_DIR"/*.vsix 2>/dev/null | head -1)

if [[ -z "$VSIX_FILE" || ! -f "$VSIX_FILE" ]]; then
  echo "Error: No VSIX file found after packaging" >&2
  exit 1
fi

echo "VSIX: $VSIX_FILE"

# Create clean user data and extensions directories
mkdir -p "$USER_DATA_DIR"
mkdir -p "$EXTENSIONS_DIR"

echo "Installing VSIX to clean extensions directory..."
code --extensions-dir="$EXTENSIONS_DIR" --install-extension "$VSIX_FILE" --force

echo "Opening VS Code with clean test configuration..."
exec code \
  --user-data-dir="$USER_DATA_DIR" \
  --extensions-dir="$EXTENSIONS_DIR" \
  "$WORKSPACE_FILE"
