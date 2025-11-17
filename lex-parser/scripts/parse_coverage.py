#!/usr/bin/env python3
"""Parse tarpaulin coverage output and group by directory."""

import sys
from collections import defaultdict

coverage = defaultdict(lambda: {'covered': 0, 'total': 0})

for line in sys.stdin:
    if ':' not in line:
        continue
    parts = line.strip().split(':')
    if len(parts) < 2:
        continue

    filepath = parts[0].strip()
    # Get directory (everything before last /)
    if '/' in filepath:
        directory = '/'.join(filepath.split('/')[:-1])
    else:
        directory = '(root)'

    # Extract coverage numbers
    nums_part = parts[1].strip().split()[0]
    if '/' in nums_part:
        covered, total = map(int, nums_part.split('/'))
        coverage[directory]['covered'] += covered
        coverage[directory]['total'] += total

# Calculate total
total_covered = sum(d['covered'] for d in coverage.values())
total_lines = sum(d['total'] for d in coverage.values())
total_pct = (total_covered / total_lines * 100) if total_lines > 0 else 0

# Sort and print
print(f"{'Directory':<50} {'Covered':>8} {'Total':>8} {'Coverage':>10}")
print('=' * 80)

# Group by coverage level for better readability
high_cov = []
med_cov = []
low_cov = []

for directory in sorted(coverage.keys()):
    c = coverage[directory]['covered']
    t = coverage[directory]['total']
    pct = (c / t * 100) if t > 0 else 0

    entry = (directory, c, t, pct)
    if pct >= 80:
        high_cov.append(entry)
    elif pct >= 50:
        med_cov.append(entry)
    else:
        low_cov.append(entry)

if high_cov:
    print(f"\n{'HIGH COVERAGE (≥80%)':<50}")
    print('-' * 80)
    for directory, c, t, pct in sorted(high_cov, key=lambda x: x[3], reverse=True):
        print(f"{directory:<50} {c:>8} {t:>8} {pct:>9.1f}%")

if med_cov:
    print(f"\n{'MEDIUM COVERAGE (50-79%)':<50}")
    print('-' * 80)
    for directory, c, t, pct in sorted(med_cov, key=lambda x: x[3], reverse=True):
        print(f"{directory:<50} {c:>8} {t:>8} {pct:>9.1f}%")

if low_cov:
    print(f"\n{'LOW COVERAGE (<50%)':<50}")
    print('-' * 80)
    for directory, c, t, pct in sorted(low_cov, key=lambda x: x[3], reverse=True):
        print(f"{directory:<50} {c:>8} {t:>8} {pct:>9.1f}%")

print('\n' + '=' * 80)
print(f"{'OVERALL':<50} {total_covered:>8} {total_lines:>8} {total_pct:>9.1f}%")
