#!/usr/bin/env bash
# CI-enforced check: no raw <button>/<input>/<select> in domain components
# Domain components are anything under crates/honba-kao-app/src/ EXCEPT ui-core
# Usage: ./scripts/check-ui-core-usage.sh
# Exits with 0 if clean, 1 if violations found

set -euo pipefail

# Find all .rs files in the app crate that are NOT in ui-core
VIOLATIONS=0

# Check for raw HTML button/input/select tags in domain components
# Domain components = under honba-kao-app/src but not in ui-core patterns
for file in $(find crates/honba-kao-app/src -name '*.rs' -not -path '*/ui-core/*' -not -path '*/.git/*'); do
    # Check for raw HTML tags that should go through ui-core
    if grep -qP '<button[ >]' "$file" || grep -qP '<input[ >]' "$file" || grep -qP '<select[ >]' "$file"; then
        echo "VIOLATION: $file contains raw HTML tags - should use ui-core primitives"
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
done

if [ "$VIOLATIONS" -gt 0 ]; then
    echo "Found $VIOLATIONS violations of ui-core usage policy"
    exit 1
else
    echo "No violations: all domain components use ui-core primitives"
    exit 0
fi
