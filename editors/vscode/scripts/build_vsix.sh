#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
EXT_DIR="$REPO_ROOT/editors/vscode"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [--target <vsce-target>]

Builds a VSIX package for the VS Code extension.

Options:
  --target <vsce-target>  Platform-specific target (e.g., darwin-arm64, linux-x64, win32-x64)
                          If not specified, builds a universal VSIX.

Examples:
  $(basename "$0")                       # Build universal VSIX
  $(basename "$0") --target darwin-arm64 # Build for Apple Silicon macOS
USAGE
}

TARGET=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

echo "Building VS Code extension..."

pushd "$EXT_DIR" >/dev/null

# Install dependencies
npm ci

# Build TypeScript
npm run build

# Bundle extension
npm run bundle

# Package VSIX
if [[ -n "$TARGET" ]]; then
  echo "Packaging for target: $TARGET"
  npx vsce package --no-dependencies --target "$TARGET"
else
  echo "Packaging universal VSIX"
  npx vsce package --no-dependencies
fi

popd >/dev/null

VSIX_FILE=$(find "$EXT_DIR" -name "*.vsix" -type f -print -quit)

if [[ -n "$VSIX_FILE" ]]; then
  echo "✓ VSIX package created: $VSIX_FILE"
else
  echo "✗ Failed to create VSIX package" >&2
  exit 1
fi
