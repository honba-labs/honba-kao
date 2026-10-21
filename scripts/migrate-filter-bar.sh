#!/usr/bin/env bash
# Migration script: Replace raw HTML in filter_bar.rs with ui-core primitives
# Usage: ./scripts/migrate-filter-bar.sh
set -euo pipefail

FILTER_BAR="crates/honba-kao-app/src/features/screener/components/filter_bar.rs"

echo "=== Checking $FILTER_BAR ==="

# Check if HfButton is already used
if grep -q "HfButton" "$FILTER_BAR"; then
    echo "HfButton already used in filter_bar.rs"
else
    echo "WARNING: HfButton not found in filter_bar.rs"
fi

# Check for raw <button> tags
if grep -qP '<button[ >]' "$FILTER_BAR"; then
    echo "Raw <button> tags found - need migration"
    # Count them
    COUNT=$(grep -cP '<button[ >]' "$FILTER_BAR")
    echo "Found $COUNT raw <button> tags"
else
    echo "No raw <button> tags found"
fi

# Check for raw <input> tags
if grep -qP '<input[ >]' "$FILTER_BAR"; then
    echo "Raw <input> tags found - need migration"
    COUNT=$(grep -cP '<input[ >]' "$FILTER_BAR")
    echo "Found $COUNT raw <input> tags"
else
    echo "No raw <input> tags found"
fi

echo ""
echo "Migration suggestions:"
echo "1. Replace raw <button> with HfButton (already partially done - HfButton is imported and used)"
echo "2. Replace raw <input> with HfInput from ui-core"
echo "3. Add HfSegmentedControl for asset-class pills/view toggle"
echo "4. Add HfToolbar for filter bar layout"
