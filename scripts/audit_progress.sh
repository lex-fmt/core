#!/bin/bash
# Track progress on test audit remediation

echo "=== Test Audit Progress ==="
echo ""

total_violations=$(rg '@audit' --type rust | wc -l | tr -d ' ')
echo "Total violations remaining: $total_violations"
echo ""

echo "By type:"
no_source=$(rg '@audit:.*no_source' --type rust | wc -l | tr -d ' ')
manual=$(rg '@audit:.*manual_construction' --type rust | wc -l | tr -d ' ')
hardcoded=$(rg '@audit:.*hardcoded_source' --type rust | wc -l | tr -d ' ')

printf "  no_source:            %3d\n" "$no_source"
printf "  manual_construction:  %3d\n" "$manual"
printf "  hardcoded_source:     %3d\n" "$hardcoded"
echo ""

echo "By file:"
rg '@audit' --type rust -c | sort -t: -k2 -rn | head -10
echo ""

# Calculate progress (started with 40 violations)
INITIAL_TOTAL=40
fixed=$((INITIAL_TOTAL - total_violations))
percentage=$((fixed * 100 / INITIAL_TOTAL))

echo "Progress: $fixed/$INITIAL_TOTAL fixed ($percentage%)"

# Show progress bar
bar_length=50
filled=$((percentage * bar_length / 100))
printf "["
printf "%${filled}s" | tr ' ' '='
printf "%$((bar_length - filled))s" | tr ' ' '-'
printf "]\n"
