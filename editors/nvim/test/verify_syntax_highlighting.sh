#!/usr/bin/env bash
# Verify that syntax highlighting is working in the test config
# Usage: ./verify_syntax_highlighting.sh [file]
# Example: ./verify_syntax_highlighting.sh AGENTS.md

set -e

FILE="${1:-AGENTS.md}"

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Make file path absolute if relative
if [[ ! "$FILE" = /* ]]; then
    FILE="$PROJECT_ROOT/$FILE"
fi

if [ ! -f "$FILE" ]; then
    echo "Error: File not found: $FILE"
    exit 1
fi

echo "Testing syntax highlighting for: $FILE"
echo "================================================"
echo ""

# Create a temporary Lua script for verification
TEMP_SCRIPT=$(mktemp /tmp/nvim_syntax_verify_XXXXXX.lua)
trap "rm -f $TEMP_SCRIPT" EXIT

cat > "$TEMP_SCRIPT" <<EOF
-- Open the file
vim.cmd("edit $FILE")

-- Wait a moment for syntax to load
vim.wait(100)

-- Print diagnostics
print("File: " .. vim.fn.expand("%:p"))
print("")
print("Filetype: " .. vim.bo.filetype)
print("Syntax: " .. vim.bo.syntax)
print("")
vim.cmd("verbose filetype")
print("")

-- Capture syntax list
local syntax_output = vim.fn.execute("syntax list")
local lines = vim.split(syntax_output, "\\n")
print("Syntax groups defined: " .. #lines .. " lines of syntax definitions")
print("")

if #lines > 0 then
  print("First 10 syntax items:")
  for i = 1, math.min(10, #lines) do
    print(lines[i])
  end
else
  print("WARNING: No syntax groups found!")
end

print("")
print("Syntax highlighting status:")
print("  syntax=" .. vim.inspect(vim.o.syntax))
print("  termguicolors=" .. tostring(vim.o.termguicolors))

vim.cmd("qall!")
EOF

# Test in headless mode
echo "=== Headless Mode Test ==="
NVIM_APPNAME=lex-test nvim -u "$SCRIPT_DIR/minimal_init.lua" --headless -l "$TEMP_SCRIPT" 2>&1

echo ""
echo "=== Interactive Mode Test (5 second timeout) ==="
echo "This will open the file in normal nvim for 5 seconds so you can visually verify highlighting."
echo "Press Ctrl+C to skip this test."
echo ""

# Give user a chance to cancel
sleep 2

# Run in normal mode with a timeout
NVIM_APPNAME=lex-test timeout 5s nvim -u "$SCRIPT_DIR/minimal_init.lua" "$FILE" || true

echo ""
echo "=== Verification Complete ==="
echo ""
echo "If syntax highlighting is working:"
echo "  - Filetype should be detected (not empty)"
echo "  - Syntax should match filetype"
echo "  - Syntax groups should show multiple lines of definitions"
echo "  - Interactive mode should show colored text (not plain white/gray)"
