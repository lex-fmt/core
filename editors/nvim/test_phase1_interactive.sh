#!/bin/bash
# Interactive manual test script for Phase 1 LSP features
# This script guides you through testing each feature interactively
#
# Phase 1 features tested:
#   1. Syntax Highlighting (semantic tokens)
#   2. Document Symbols (outline/navigation)
#   3. Hover Information (show content on references)
#   4. Folding Ranges (collapse/expand sections)
#
# Usage:
#   ./editors/nvim/test_phase1_interactive.sh
#
# The script will open Neovim multiple times, each demonstrating a different feature.
# Follow the on-screen instructions to verify each feature works correctly.
#
# See docs/dev/nvim-fasttrack.lex for architecture overview

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_FILE="$PROJECT_ROOT/specs/v1/benchmark/010-kitchensink.lex"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Lex Neovim Plugin - Phase 1 Manual Testing ===${NC}\n"

# Check if lex-lsp is running
echo -e "${YELLOW}Checking lex-lsp binary...${NC}"
if ! command -v lex-lsp &> /dev/null; then
    echo -e "${RED}Error: lex-lsp not found in PATH${NC}"
    echo "Please build and install it first:"
    echo "  cd lex-lsp && cargo build --release"
    echo "  # Add target/release to your PATH or install it"
    exit 1
fi
echo -e "${GREEN}✓ lex-lsp found${NC}\n"

# Check test file exists
if [ ! -f "$TEST_FILE" ]; then
    echo -e "${RED}Error: Test file not found: $TEST_FILE${NC}"
    exit 1
fi

echo -e "${BLUE}Test file: $TEST_FILE${NC}\n"

# Function to wait for user
wait_for_user() {
    echo -e "\n${YELLOW}Press ENTER to continue...${NC}"
    read
}

echo -e "${BLUE}=== Feature 1: Syntax Highlighting (Semantic Tokens) ===${NC}\n"
echo "We will open the test file and you should verify:"
echo "  1. Session headers (1., 1.1., etc.) are highlighted"
echo "  2. Inline formatting (*bold*, _italic_, \`code\`, \$math\$) is styled"
echo "  3. References ([42], [@cite], [TK-ref]) are highlighted"
echo "  4. Annotations (@ note) are highlighted"
echo "  5. Verbatim blocks have distinct highlighting"
echo ""
echo "The file will open in Neovim. Look for colored/styled text."
echo "Close Neovim (:q) when you've verified highlighting."
wait_for_user

# Open with a brief help message
nvim -c "echo 'Look for: sessions in bold, *bold*, _italic_, [refs] highlighted. Press :q to exit'" \
     -c "sleep 2" \
     "$TEST_FILE"

echo -e "\n${GREEN}Did you see syntax highlighting? (colors/styles on different elements)${NC}"
echo "1) Yes - everything looks good"
echo "2) No - no colors at all"
echo "3) Partial - some elements highlighted, others not"
read -p "Your answer (1/2/3): " answer

case $answer in
    1)
        echo -e "${GREEN}✓ Syntax highlighting working!${NC}\n"
        ;;
    2)
        echo -e "${RED}✗ Syntax highlighting NOT working${NC}"
        echo "Troubleshooting steps:"
        echo "  1. Check :LspInfo in nvim to see if lex-lsp is attached"
        echo "  2. Check :messages for errors"
        echo "  3. Try :lua vim.lsp.buf.semantic_tokens_full()"
        echo ""
        ;;
    3)
        echo -e "${YELLOW}⚠ Partial highlighting - some elements missing${NC}"
        echo "Note which elements are not highlighted and check the LSP implementation"
        echo ""
        ;;
esac

wait_for_user

echo -e "${BLUE}=== Feature 2: Document Symbols (Outline) ===${NC}\n"
echo "We will test if the LSP returns document symbols."
echo "You should see a hierarchical structure with:"
echo "  - Sessions (1., 1.1., etc.)"
echo "  - Definitions"
echo "  - Annotations"
echo "  - Lists"
echo ""
echo "Instructions:"
echo "  1. When the file opens, run this command:"
echo "     :lua print(vim.inspect(vim.lsp.buf_request_sync(0, 'textDocument/documentSymbol', {textDocument = vim.lsp.util.make_text_document_params()}, 2000)))"
echo "  2. You should see nested tables with 'name', 'kind', 'range', 'children'"
echo "  3. If using Telescope, try: :Telescope lsp_document_symbols"
echo "  4. Close with :q when done"
wait_for_user

nvim -c "echo 'Run the documentSymbol LSP request (see terminal for command). Press :q to exit'" \
     "$TEST_FILE"

echo -e "\n${GREEN}Did you see a hierarchical list of symbols?${NC}"
echo "1) Yes - saw sessions, definitions, etc. in a tree"
echo "2) No - no symbols shown"
echo "3) Error - got an error message"
read -p "Your answer (1/2/3): " answer

case $answer in
    1)
        echo -e "${GREEN}✓ Document symbols working!${NC}\n"
        ;;
    2)
        echo -e "${RED}✗ Document symbols NOT working${NC}"
        echo "Troubleshooting:"
        echo "  1. Check if server supports it: :lua print(vim.lsp.get_active_clients()[1].server_capabilities.documentSymbolProvider)"
        echo ""
        ;;
    3)
        echo -e "${RED}✗ Error with document symbols${NC}"
        echo "Check :messages and :LspLog for details"
        echo ""
        ;;
esac

wait_for_user

echo -e "${BLUE}=== Feature 3: Hover Information ===${NC}\n"
echo "We will test hover on references and annotations."
echo ""
echo -e "${YELLOW}CRITICAL: Position cursor INSIDE the reference, not on brackets${NC}"
echo "  For [@spec2025] - cursor must be ON '@spec2025', NOT on '[' or ']'"
echo "  For [42] - cursor must be ON '42', NOT on the brackets"
echo "  For @ note - cursor must be ON 'note'"
echo ""
echo "Test locations in the file:"
echo "  Line 3: [@spec2025, pp. 45-46] - citation"
echo "  Line 12: [42] - footnote reference"
echo "  Line 24: :: warning severity=high :: - annotation"
echo ""
echo "Instructions:"
echo "  1. Find a reference (e.g., search for 'spec2025' with /spec2025)"
echo "  2. Position cursor ON the reference text (not brackets!)"
echo "  3. Press K (or :lua vim.lsp.buf.hover())"
echo "  4. A popup should show citation/footnote/annotation info"
echo "  5. Try different references"
echo "  6. Close with :q when done"
wait_for_user

nvim -c "echo 'Position cursor ON reference text (inside brackets), then press K. Press :q to exit'" \
     -c "normal 3G" \
     -c "call search('spec2025')" \
     "$TEST_FILE"

echo -e "\n${GREEN}Did hover show information when you pressed K on references/annotations?${NC}"
echo "1) Yes - saw popup with content preview"
echo "2) No - nothing happened"
echo "3) Only for some elements"
read -p "Your answer (1/2/3): " answer

case $answer in
    1)
        echo -e "${GREEN}✓ Hover information working!${NC}\n"
        ;;
    2)
        echo -e "${RED}✗ Hover NOT working${NC}"
        echo "Troubleshooting:"
        echo "  1. Check if LSP is attached: :LspInfo"
        echo "  2. Check for errors: :messages"
        echo ""
        ;;
    3)
        echo -e "${YELLOW}⚠ Partial hover support${NC}"
        echo "Note which elements don't show hover and check implementation"
        echo ""
        ;;
esac

wait_for_user

echo -e "${BLUE}=== Feature 4: Folding Ranges ===${NC}\n"
echo "We will test LSP-provided folding ranges."
echo ""
echo -e "${YELLOW}IMPORTANT: LSP folding must be configured first${NC}"
echo ""
echo "The file will open with these settings:"
echo "  :set foldmethod=expr"
echo "  :set foldexpr=v:lua.vim.lsp.foldexpr()"
echo ""
echo "You should be able to fold:"
echo "  1. Sessions (entire section collapses)"
echo "  2. Lists with children"
echo "  3. Annotations with content"
echo "  4. Verbatim blocks"
echo ""
echo "Instructions:"
echo "  1. Wait a moment for LSP to attach and provide fold ranges"
echo "  2. Navigate to session '1. Primary Session' (line 17)"
echo "  3. Press zc to close/fold"
echo "  4. The entire session should collapse to one line"
echo "  5. Press zo to open/unfold"
echo "  6. Try: zM (close all), zR (open all)"
echo "  7. Close with :q when done"
wait_for_user

nvim -c "set foldmethod=expr" \
     -c "set foldexpr=v:lua.vim.lsp.foldexpr()" \
     -c "sleep 1" \
     -c "normal 17G" \
     -c "echo 'Folding ready. Try: zc=close, zo=open, zM=close all, zR=open all. Press :q to exit'" \
     "$TEST_FILE"

echo -e "\n${GREEN}Did folding work?${NC}"
echo "1) Yes - could fold/unfold sessions, lists, etc."
echo "2) No - nothing happened when pressing zc"
echo "3) Partial - some elements fold, others don't"
read -p "Your answer (1/2/3): " answer

case $answer in
    1)
        echo -e "${GREEN}✓ Folding ranges working!${NC}\n"
        ;;
    2)
        echo -e "${RED}✗ Folding NOT working${NC}"
        echo "Troubleshooting:"
        echo "  1. Make sure foldmethod is set: :set foldmethod?"
        echo "  2. Try manually: :set foldmethod=expr foldexpr=v:lua.vim.lsp.foldexpr()"
        echo "  3. Check if server supports it"
        echo ""
        ;;
    3)
        echo -e "${YELLOW}⚠ Partial folding support${NC}"
        echo "Note which elements don't fold and check implementation"
        echo ""
        ;;
esac

echo -e "\n${BLUE}=== Summary ===${NC}\n"
echo "Phase 1 features tested:"
echo "  1. Syntax Highlighting (Semantic Tokens)"
echo "  2. Document Symbols (Outline)"
echo "  3. Hover Information"
echo "  4. Folding Ranges"
echo ""
echo -e "${YELLOW}For detailed testing instructions, see: editors/nvim/MANUAL_TESTING.md${NC}"
echo ""
echo -e "${GREEN}Testing complete!${NC}"
