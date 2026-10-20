#!/usr/bin/env bash
# check-ui-core-usage.sh
# CI enforcement: Ensure honba-kao-app uses honba-kao-ui-core primitives

set -euo pipefail

echo "🔍 Checking honba-kao-ui-core usage compliance..."

APP_SRC="crates/honba-kao-app/src"

# 1. Check for raw HTML elements that should use ui-core primitives
echo "  Checking for raw HTML elements that should use ui-core primitives..."
# Check for <button> not using HfButton (WARN - migration in progress)
RAW_BUTTONS=$(grep -rn "<button" "$APP_SRC" --include="*.rs" | grep -v "HfButton\|HfIconButton" | grep -v "// " | grep -v "test" || true)
if [ -n "$RAW_BUTTONS" ]; then
    echo "⚠️  Warning: Found raw <button> elements (should migrate to HfButton/HfIconButton):"
    echo "$RAW_BUTTONS" | wc -l | xargs echo "    Total:"
    # Don't exit - migration in progress
fi

# Check for <input> not using HfInput (WARN - migration in progress)
RAW_INPUTS=$(grep -rn "<input" "$APP_SRC" --include="*.rs" | grep -v "HfInput" | grep -v "// " | grep -v "test" || true)
if [ -n "$RAW_INPUTS" ]; then
    echo "⚠️  Warning: Found raw <input> elements (should migrate to HfInput):"
    echo "$RAW_INPUTS" | wc -l | xargs echo "    Total:"
fi

# Check for <select> not using HfSelect (WARN - migration in progress)
RAW_SELECTS=$(grep -rn "<select" "$APP_SRC" --include="*.rs" | grep -v "HfSelect" | grep -v "// " | grep -v "test" || true)
if [ -n "$RAW_SELECTS" ]; then
    echo "⚠️  Warning: Found raw <select> elements (should migrate to HfSelect):"
    echo "$RAW_SELECTS" | wc -l | xargs echo "    Total:"
fi

# Check for <textarea> not using HfTextarea (WARN - migration in progress)
RAW_TEXTAREAS=$(grep -rn "<textarea" "$APP_SRC" --include="*.rs" | grep -v "HfTextarea" | grep -v "// " | grep -v "test" || true)
if [ -n "$RAW_TEXTAREAS" ]; then
    echo "⚠️  Warning: Found raw <textarea> elements (should migrate to HfTextarea):"
    echo "$RAW_TEXTAREAS" | wc -l | xargs echo "    Total:"
fi

# 2. Check for .svg file references (icon rule)
echo "  Checking for .svg file references..."
SVG_REFS=$(grep -rn "\.svg" "$APP_SRC" --include="*.rs" | grep -v "// " | grep -v "test" || true)
if [ -n "$SVG_REFS" ]; then
    echo "❌ Found .svg file references (icons must be procedural via icons::outline):"
    echo "$SVG_REFS"
    exit 1
fi
echo "  ✅ No .svg file references"

# 3. Check for direct --oac-* CSS variable usage
echo "  Checking for direct --oac-* CSS variable usage..."
OAC_VARS=$(grep -rn -- "--oac-" "$APP_SRC" --include="*.rs" | grep -v "// " | grep -v "test" || true)
if [ -n "$OAC_VARS" ]; then
    echo "❌ Found direct --oac-* CSS variable usage (should use honba-kao-ui-core tokens):"
    echo "$OAC_VARS"
    exit 1
fi
echo "  ✅ No direct --oac-* CSS variable usage"

# 4. Check for .oac-* CSS class usage
echo "  Checking for .oac-* CSS class usage..."
OAC_CLASSES=$(grep -rn "\.oac-" "$APP_SRC" --include="*.rs" | grep -v "// " | grep -v "test" || true)
if [ -n "$OAC_CLASSES" ]; then
    echo "❌ Found .oac-* CSS class usage (should use honba-kao-ui-core classes):"
    echo "$OAC_CLASSES"
    exit 1
fi
echo "  ✅ No .oac-* CSS class usage"

# 5. Check that ui-core is imported in components using it
echo "  Checking ui-core imports in component files..."
COMPONENT_FILES=$(find "$APP_SRC/components" -name "*.rs" -type f)
for file in $COMPONENT_FILES; do
    # Check if file uses ui-core types but doesn't import them
    if grep -q "HfButton\|HfInput\|HfSelect\|HfCheckbox\|HfRadioGroup\|HfSwitch\|HfTabs\|HfIconButton\|HfIcon\|HfBadge\|HfSideMarker\|HfPLNumber\|HfStatusCircle\|HfOrderBadge\|HfRiskBadge\|HfMarketPhase\|HfTradingMode\|HfFilterBar\|HfHeaderBar\|HfSidebar\|HfChartToolbar\|HfOrderTicket\|HfTradingDrawer\|HfModal\|HfPositionsTable" "$file"; then
        if ! grep -q "use.*honba_kao_ui_core\|use.*crate::.*ui_core\|use.*ui_core" "$file" && ! grep -q "honba_kao_ui_core::" "$file"; then
            echo "⚠️  Warning: $file uses ui-core components but may not import them explicitly"
        fi
    fi
done
echo "  ✅ Component imports look OK"

# 6. Check that all primitive categories are used
echo "  Checking primitive component coverage..."
PRIMITIVES=("HfButton" "HfInput" "HfSelect" "HfCheckbox" "HfRadioGroup" "HfSwitch" "HfTabs" "HfIconButton" "HfIcon" "HfLabel" "HfSeparator")
for p in "${PRIMITIVES[@]}"; do
    if ! grep -rn "$p" "$APP_SRC" --include="*.rs" > /dev/null; then
        echo "⚠️  Warning: $p is not used anywhere in app components"
    fi
done
echo "  ✅ Primitive coverage checked"

echo "✅ All honba-kao-ui-core usage checks passed!"