#!/usr/bin/env bash

set -e

# Default to JUnit if no arguments provided, based on the prompt's phrasing.
# "Make the runner output a junit xml output or a friendly output if passed --format=simple"
FORMATTER="junit"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --format=simple) FORMATTER="pretty" ;;
        --format=junit) FORMATTER="junit" ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_FILE="$SCRIPT_DIR/lex_nvim_plugin.bats"

# Check if bats is installed
if ! command -v bats &> /dev/null; then
    echo "Error: bats is not installed."
    echo "Please install bats-core (e.g., brew install bats-core)"
    exit 1
fi

# Run bats
# Note: older bats versions might not support --formatter junit without a plugin.
# Assuming a modern bats environment as tested (Bats 1.13.0).

exec bats "$TEST_FILE" --formatter "$FORMATTER"




