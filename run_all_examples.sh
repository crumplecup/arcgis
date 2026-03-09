#!/bin/bash
# Script to run all examples and track results

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get list of all examples
EXAMPLES=$(ls examples/*.rs | sed 's|examples/||' | sed 's|\.rs$||' | sort)

# Arrays to track results
PASSED=()
FAILED=()
TOTAL=0

echo "========================================="
echo "Running All Examples"
echo "========================================="
echo ""

for example in $EXAMPLES; do
    TOTAL=$((TOTAL + 1))
    echo -e "${YELLOW}[$TOTAL] Running: $example${NC}"

    # Create temp file for output
    TMPFILE=$(mktemp)

    if cargo run --example "$example" > "$TMPFILE" 2>&1; then
        echo -e "${GREEN}✅ PASSED: $example${NC}"
        PASSED+=("$example")
    else
        echo -e "${RED}❌ FAILED: $example${NC}"
        FAILED+=("$example")
        # Show last few lines of error
        echo "  Last 5 lines of output:"
        tail -5 "$TMPFILE" | sed 's/^/  /'
    fi

    rm -f "$TMPFILE"
    echo ""
done

# Print summary
echo "========================================="
echo "Summary"
echo "========================================="
echo "Total examples: $TOTAL"
echo -e "${GREEN}Passed: ${#PASSED[@]}${NC}"
echo -e "${RED}Failed: ${#FAILED[@]}${NC}"
echo ""

if [ ${#FAILED[@]} -gt 0 ]; then
    echo "Failed examples:"
    for example in "${FAILED[@]}"; do
        echo -e "  ${RED}❌ $example${NC}"
    done
    echo ""
    exit 1
fi

echo -e "${GREEN}All examples passed!${NC}"
exit 0
