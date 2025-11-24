#!/usr/bin/env bash
# Search for version = "<foo>" in lex-*/Cargo.toml files only

set -euo pipefail

# Find Cargo.toml files matching lex-*/Cargo.toml pattern and search for version definitions
find . -path "./lex-*/Cargo.toml" -type f -exec grep -Hn '^version\s*=\s*"' {} \;
