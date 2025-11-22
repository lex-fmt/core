#!/usr/bin/env bash
# Headless test runner for Lex Neovim plugin
# This script runs tests without opening a GUI

set -e

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="$PLUGIN_DIR/config/init.lua"

echo "===================================="
echo "Lex Neovim Plugin - Headless Tests"
echo "===================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_file="$2"
    local config="${3:-$CONFIG_FILE}"

    TESTS_RUN=$((TESTS_RUN + 1))
    echo -n "Running: $test_name ... "

    if NVIM_APPNAME=lex-test nvim --headless -u "$config" -l "$test_file" 2>&1 | grep -q "TEST_PASSED"; then
        echo -e "${GREEN}PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}FAILED${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Run tests
echo "Running plugin tests..."
echo ""

run_test "Plugin loads successfully" "$SCRIPT_DIR/test_plugin_loads.lua"
run_test "Filetype detection for .lex files" "$SCRIPT_DIR/test_filetype.lua"
run_test "LSP hover functionality" "$SCRIPT_DIR/test_lsp_hover.lua" "$SCRIPT_DIR/minimal_init.lua"
run_test "LSP semantic tokens functionality" "$SCRIPT_DIR/test_lsp_semantic_tokens.lua" "$SCRIPT_DIR/minimal_init.lua"

# Summary
echo ""
echo "===================================="
echo "Test Summary"
echo "===================================="
echo "Total:  $TESTS_RUN"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
