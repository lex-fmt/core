#!/usr/bin/env bash
# Coverage by Directory Report
#
# Generates a unit test coverage report broken down by directory.
# Excludes test utilities and integration tests to focus on production code coverage.
#
# Usage: ./scripts/coverage-by-directory.sh

set -euo pipefail

echo "Running unit test coverage analysis (excluding testing utilities)..."
echo

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Run tarpaulin and save to temp file
TEMP_FILE=$(mktemp)
trap "rm -f $TEMP_FILE" EXIT

cargo tarpaulin \
    --lib \
    --package lex-parser \
    --exclude-files 'tests/*' \
    --exclude-files 'src/lex/testing/*' \
    --out Stdout 2>&1 | \
    tee "$TEMP_FILE" | \
    grep -q "coverage," || true

# Parse the coverage data
grep "lex-parser/src" "$TEMP_FILE" | \
    grep -E "[0-9]+/[0-9]+" | \
    sed 's|lex-parser/src/lex/||g' | \
    python3 "$SCRIPT_DIR/parse_coverage.py"
