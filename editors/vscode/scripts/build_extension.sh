#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
EXT_DIR="$REPO_ROOT/editors/vscode"
RESOURCES_DIR="$EXT_DIR/resources"
BINARY_DEST="$RESOURCES_DIR/lex-lsp"

BUILD_PROFILE="release"
TARGET_DIR="$REPO_ROOT/target/$BUILD_PROFILE"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build lex-lsp" >&2
  exit 1
fi

pushd "$REPO_ROOT" >/dev/null
cargo build --bin lex-lsp --release
popd >/dev/null

BINARY_SRC="$TARGET_DIR/lex-lsp"
if [[ ! -f "$BINARY_SRC" && -f "$BINARY_SRC.exe" ]]; then
  BINARY_SRC="$BINARY_SRC.exe"
fi

if [[ ! -f "$BINARY_SRC" ]]; then
  echo "lex-lsp binary not found at $BINARY_SRC" >&2
  exit 1
fi

mkdir -p "$RESOURCES_DIR"
cp "$BINARY_SRC" "$BINARY_DEST"
chmod +x "$BINARY_DEST"

echo "lex-lsp copied to $BINARY_DEST"

pushd "$EXT_DIR" >/dev/null
npm ci
npm run bundle
popd >/dev/null

echo "Extension bundle written to $EXT_DIR/dist/extension.js"
