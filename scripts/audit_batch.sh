#!/bin/bash
# Show audit violations for a specific batch

usage() {
    echo "Usage: $0 [TYPE|FILE]"
    echo ""
    echo "Show violations by type or file:"
    echo "  $0 no_source           - Show all no_source violations"
    echo "  $0 manual_construction - Show all manual_construction violations"
    echo "  $0 hardcoded_source    - Show all hardcoded_source violations"
    echo "  $0 FILE_PATH           - Show all violations in a specific file"
    echo ""
    echo "Examples:"
    echo "  $0 no_source"
    echo "  $0 tests/lexer_proptest.rs"
    exit 1
}

if [ $# -eq 0 ]; then
    usage
fi

TYPE_OR_FILE="$1"

# Check if it's a file path
if [ -f "$TYPE_OR_FILE" ]; then
    echo "=== Violations in $TYPE_OR_FILE ==="
    echo ""
    rg '@audit' --type rust -n -B 1 -A 5 "$TYPE_OR_FILE"
    echo ""
    count=$(rg '@audit' --type rust "$TYPE_OR_FILE" | wc -l | tr -d ' ')
    echo "Total: $count violations"
else
    # Assume it's a violation type
    echo "=== All '$TYPE_OR_FILE' violations ==="
    echo ""
    rg "@audit:.*$TYPE_OR_FILE" --type rust -n -B 1 -A 5
    echo ""
    count=$(rg "@audit:.*$TYPE_OR_FILE" --type rust | wc -l | tr -d ' ')
    echo "Total: $count violations"
fi
