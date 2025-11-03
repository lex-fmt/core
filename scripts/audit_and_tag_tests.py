#!/usr/bin/env python3
"""
Scan all Rust test files and detect three types of violations:
1. no_source: AST nodes/tokens created without proper source/location data
2. manual_construction: Tokens/tags created manually instead of using factories
3. hardcoded_source: Tests using ad hoc hardcoded txxt source strings
"""

import re
import os
from pathlib import Path
from collections import defaultdict
from typing import List, Set, Tuple

# Patterns for each violation type
PATTERNS = {
    'no_source': [
        r'\.new\([^)]*vec!\[\][^)]*\)',  # BlankLineGroup::new(..., vec![])
        r'from_string\([^,]+,\s*None\)',  # TextContent::from_string(..., None)
        r'Position::new\(0,\s*0\)',  # Default position (0,0)
        r'default_location\(\)',  # Explicit default location call
    ],
    'manual_construction': [
        r'Token::\w+\([^)]*\.to_string\(\)[^)]*\)',  # Token::Text("...".to_string())
        r'vec!\[[\s\n]*Token::\w+',  # vec![ Token::...
        r'Token::(?:Text|Number|Dash|Newline|Whitespace|Indent|Dedent|Period|Comma|Colon|Semicolon|OpenBrace|CloseBrace|OpenBracket|CloseBracket|OpenParen|CloseParen|Equal|Hash)',  # Direct Token enum usage
    ],
    'hardcoded_source': [
        r'let\s+source\s*=\s*"[^"]*\\n',  # let source = "...\n...
        r'"[^"]*-\s+[^"]*\\n',  # List marker: "- Item\n
        r'"[^"]*:\s*\\n\s+',  # Definition: "Label:\n    Content
        r'"[^"]*::\s*\w+[^"]*::"',  # Annotation: ":: note ... ::"
        r'"[^"]*\\n\\n',  # Blank lines in string
        r'format!\([^)]*"[^"]*\\n',  # format!("...\n...)
    ],
}

def find_test_files(root_dir: Path) -> List[Path]:
    """Find all Rust files that contain tests."""
    test_files = []

    # Find files in tests/ directory
    tests_dir = root_dir / 'tests'
    if tests_dir.exists():
        test_files.extend(tests_dir.rglob('*.rs'))

    # Find files with #[cfg(test)] modules in src/
    src_dir = root_dir / 'src'
    if src_dir.exists():
        for rust_file in src_dir.rglob('*.rs'):
            with open(rust_file, 'r', encoding='utf-8') as f:
                content = f.read()
                if '#[test]' in content or '#[cfg(test)]' in content:
                    test_files.append(rust_file)

    return sorted(set(test_files))

def extract_test_functions(content: str) -> List[Tuple[str, int, int]]:
    """Extract test function names with their line ranges."""
    tests = []
    lines = content.split('\n')

    i = 0
    while i < len(lines):
        line = lines[i]

        # Look for #[test] attribute
        if '#[test]' in line:
            # Find the function definition (may be on next few lines)
            for j in range(i, min(i + 5, len(lines))):
                fn_match = re.search(r'fn\s+(\w+)\s*\(', lines[j])
                if fn_match:
                    fn_name = fn_match.group(1)
                    start_line = i + 1  # +1 for 1-based indexing

                    # Find the end of the function (naive: look for closing brace at indent level 0)
                    brace_count = 0
                    end_line = j + 1
                    for k in range(j, len(lines)):
                        brace_count += lines[k].count('{') - lines[k].count('}')
                        if brace_count == 0 and '{' in lines[k]:
                            end_line = k + 1
                            break

                    tests.append((fn_name, start_line, end_line))
                    break

        i += 1

    return tests

def detect_violations(content: str, start_line: int, end_line: int) -> Set[str]:
    """Detect violations in a specific section of code."""
    violations = set()

    # Get the relevant section
    lines = content.split('\n')[start_line-1:end_line]
    section = '\n'.join(lines)

    for violation_type, patterns in PATTERNS.items():
        for pattern in patterns:
            if re.search(pattern, section):
                violations.add(violation_type)
                break  # One match per violation type is enough

    return violations

def has_audit_tag(content: str, test_start_line: int) -> bool:
    """Check if test already has an @audit tag."""
    lines = content.split('\n')
    # Look a few lines before the test for existing @audit tag
    for i in range(max(0, test_start_line - 5), test_start_line):
        if '@audit' in lines[i]:
            return True
    return False

def insert_audit_tag(content: str, test_start_line: int, violations: Set[str]) -> str:
    """Insert @audit comment before a test function."""
    lines = content.split('\n')

    # Check if already has tag
    if has_audit_tag(content, test_start_line):
        return content

    # Find the #[test] line
    test_attr_line = test_start_line - 1  # Assuming #[test] is right before
    while test_attr_line > 0 and '#[test]' not in lines[test_attr_line - 1]:
        test_attr_line -= 1
        if test_attr_line == 0:
            test_attr_line = test_start_line - 1
            break

    # Determine indentation
    indent = ''
    if test_attr_line > 0:
        match = re.match(r'^(\s*)', lines[test_attr_line - 1])
        if match:
            indent = match.group(1)

    # Create the audit comment
    violation_tags = ', '.join(sorted(violations))
    audit_line = f'{indent}// @audit: {violation_tags}'

    # Insert the comment
    lines.insert(test_attr_line - 1, audit_line)

    return '\n'.join(lines)

def main():
    root_dir = Path(__file__).parent.parent

    print("=== Test Audit Report ===\n")

    test_files = find_test_files(root_dir)
    print(f"Found {len(test_files)} test files\n")

    stats = defaultdict(int)
    stats['total_tests'] = 0
    stats['tests_with_violations'] = 0

    violation_counts = defaultdict(int)
    files_modified = []

    for test_file in test_files:
        with open(test_file, 'r', encoding='utf-8') as f:
            content = f.read()

        tests = extract_test_functions(content)
        if not tests:
            continue

        print(f"\n{test_file.relative_to(root_dir)} ({len(tests)} tests)")

        modified_content = content
        offset = 0  # Track line offset as we insert comments

        for test_name, start_line, end_line in tests:
            stats['total_tests'] += 1

            violations = detect_violations(content, start_line, end_line)

            if violations:
                stats['tests_with_violations'] += 1
                for v in violations:
                    violation_counts[v] += 1

                violation_str = ', '.join(sorted(violations))
                print(f"  - {test_name}: {violation_str}")

                # Insert audit tag if not already present
                if not has_audit_tag(modified_content, start_line + offset):
                    new_content = insert_audit_tag(modified_content, start_line + offset, violations)
                    offset += (new_content.count('\n') - modified_content.count('\n'))
                    modified_content = new_content

        # Write back if modified
        if modified_content != content:
            with open(test_file, 'w', encoding='utf-8') as f:
                f.write(modified_content)
            files_modified.append(test_file)

    print("\n" + "="*50)
    print(f"\n=== Summary ===")
    print(f"Total tests scanned: {stats['total_tests']}")
    print(f"Tests with violations: {stats['tests_with_violations']}")
    print(f"Clean tests: {stats['total_tests'] - stats['tests_with_violations']}")

    print(f"\n=== Violations by Type ===")
    for violation_type in ['no_source', 'manual_construction', 'hardcoded_source']:
        count = violation_counts[violation_type]
        percentage = (count / stats['total_tests'] * 100) if stats['total_tests'] > 0 else 0
        print(f"{violation_type:20s}: {count:3d} ({percentage:5.1f}%)")

    print(f"\n=== Files Modified ===")
    print(f"Tagged {len(files_modified)} files with @audit comments")

    print("\n=== Next Steps ===")
    print("1. Review the @audit tags added to test files")
    print("2. To find all violations: rg '@audit' --type rust")
    print("3. To find specific type: rg '@audit:.*no_source' --type rust")
    print("4. To count by type: rg '@audit:.*PATTERN' -c")

if __name__ == '__main__':
    main()
