#!/bin/bash
# Audit test violations

echo "=== Test Audit Report ==="
echo ""

echo "1. Tests creating AST nodes/tokens without source:"
rg --type rust '#\[test\]' -A 50 tests/ | \
  rg -i 'Token::new|AstNode::new|\.with_position\(None\)' | \
  wc -l

echo ""
echo "2. Tests creating tokens/tags manually (not using factories):"
rg --type rust '#\[test\]' -A 50 tests/ | \
  rg 'Token \{|Tag \{' | \
  grep -v 'factory\|Factory' | \
  wc -l

echo ""
echo "3. Tests with hardcoded txxt source strings:"
rg --type rust '#\[test\]' -A 30 tests/ | \
  rg 'let.*=.*".*\{.*\}"' | \
  wc -l

echo ""
echo "=== Detailed violations by file ==="
rg --type rust -l '#\[test\]' tests/ | while read file; do
  violations=0
  patterns=""

  if grep -q 'Token::new\|AstNode::new\|\.with_position(None)' "$file" 2>/dev/null; then
    violations=$((violations + 1))
    patterns="${patterns}no_source,"
  fi

  if grep -E 'Token \{|Tag \{' "$file" | grep -v 'factory\|Factory' >/dev/null 2>&1; then
    violations=$((violations + 1))
    patterns="${patterns}manual_construction,"
  fi

  if grep -E 'let.*=.*".*\{.*\}"' "$file" >/dev/null 2>&1; then
    violations=$((violations + 1))
    patterns="${patterns}hardcoded_source,"
  fi

  if [ $violations -gt 0 ]; then
    echo "$file: ${patterns%,}"
  fi
done
