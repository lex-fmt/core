#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $(basename "$0") --version <vX.Y.Z> [--output-name <name>]

Builds the Lex Neovim plugin bundle. The provided version should match the
release tag (e.g. v0.1.10) and is embedded as the default lex-lsp binary
version. The resulting archive is written to editors/nvim/dist/.
USAGE
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PLUGIN_DIR="$REPO_ROOT/editors/nvim"
DIST_DIR="$PLUGIN_DIR/dist"

VERSION="${LEX_RELEASE_VERSION:-}"
OUTPUT_NAME="lex-nvim"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v)
      VERSION="$2"
      shift 2
      ;;
    --output-name)
      OUTPUT_NAME="$2"
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

if [[ -z "$VERSION" ]]; then
  if VERSION=$(cd "$REPO_ROOT" && git describe --tags --abbrev=0 2>/dev/null); then
    echo "Detected version $VERSION from git tags"
  else
    echo "Error: --version flag is required when no git tag is available" >&2
    exit 1
  fi
fi

PLUGIN_VERSION="${VERSION#v}"
LEX_LSP_VERSION="$VERSION"

if [[ -z "$PLUGIN_VERSION" ]]; then
  echo "Error: derived plugin version is empty" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t lexnvim)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

PACKAGE_ROOT="$TMP_DIR/$OUTPUT_NAME"
mkdir -p "$PACKAGE_ROOT"
cp -R "$PLUGIN_DIR"/. "$PACKAGE_ROOT"/

rm -rf \
  "$PACKAGE_ROOT/test" \
  "$PACKAGE_ROOT/dist" \
  "$PACKAGE_ROOT/scripts" \
  "$PACKAGE_ROOT/test_phase1_interactive.sh"

INIT_FILE="$PACKAGE_ROOT/lua/lex/init.lua"
if [[ ! -f "$INIT_FILE" ]]; then
  echo "Error: could not find init.lua at $INIT_FILE" >&2
  exit 1
fi

INIT_FILE="$INIT_FILE" \
PLUGIN_VERSION="$PLUGIN_VERSION" \
LEX_LSP_VERSION="$LEX_LSP_VERSION" \
python3 <<'PY'
import os
import pathlib
import re

init_path = pathlib.Path(os.environ['INIT_FILE'])
plugin_version = os.environ['PLUGIN_VERSION']
lex_version = os.environ['LEX_LSP_VERSION']
text = init_path.read_text()

ver_pattern = re.compile(r'(M\.version\s*=\s*")[^"]+(")')
lsp_pattern = re.compile(r'(M\.lex_lsp_version\s*=\s*")[^"]+(")')

if not ver_pattern.search(text):
    raise SystemExit('Unable to find M.version assignment in init.lua')
if not lsp_pattern.search(text):
    raise SystemExit('Unable to find M.lex_lsp_version assignment in init.lua')

text = ver_pattern.sub(f'M.version = "{plugin_version}"', text, count=1)
text = lsp_pattern.sub(f'M.lex_lsp_version = "{lex_version}"', text, count=1)

init_path.write_text(text)
PY

mkdir -p "$DIST_DIR"
TAR_PATH="$DIST_DIR/$OUTPUT_NAME.tar.gz"
tar -czf "$TAR_PATH" -C "$TMP_DIR" "$OUTPUT_NAME"

echo "Neovim plugin packaged at $TAR_PATH"
