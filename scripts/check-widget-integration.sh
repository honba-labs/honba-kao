#!/usr/bin/env bash
# Widget Integration Compliance Check
# Run in CI to enforce honba-kao ↔ openalgo-charts widget isolation rules

set -euo pipefail

ROOT="/home/pmallapp/Devel/Finance/honba-workspace/honba-kao"
ERRORS=0

echo "🔍 Checking widget integration compliance..."

# 1. No .oac-* selectors in honba-kao CSS/RS (except display:none hide rules)
echo "  Checking for forbidden .oac-* selectors..."
if grep -r "\.oac-" "$ROOT/crates/honba-kao-app/src" --include="*.css" --include="*.rs" 2>/dev/null | \
   grep -v "display: none" | grep -v "hide" | grep -v "isolation: isolate" | grep -v "widget-integration.md"; then
    echo "❌ FAIL: Found .oac-* selectors not in hide rules"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No forbidden .oac-* selectors"
fi

# 2. No --oac-* tokens in honba-kao CSS/RS
echo "  Checking for forbidden --oac-* tokens..."
if grep -r "\-\-oac-" "$ROOT/crates/honba-kao-app/src" --include="*.css" --include="*.rs" 2>/dev/null | \
   grep -v "widget-integration.md"; then
    echo "❌ FAIL: Found --oac-* token usage"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No forbidden --oac-* tokens"
fi

# 3. Widget mounts use topbar:false (the widget's *drawing rail* stays on —
#    it exposes ~100 grouped tools vs. nine in honba-kao's old custom rail;
#    see vendor/openalgo-charts/honba-bridge.js createWidget options).
echo "  Checking widget mount configuration..."
if grep -r "topbar:\s*true" "$ROOT/vendor/openalgo-charts/honba-bridge.js" 2>/dev/null; then
    echo "❌ FAIL: Widget topbar enabled (topbar:true) — honba-kao owns the topbar"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ Widget topbar properly disabled (widget rail kept as designed)"
fi

# 4. Bridge uses mount_chart_with_options (not raw js_mount for actual mounting)
echo "  Checking bridge uses widget-tier mount..."
# Check that mount_chart_with_options is used in PaneChart (actual mount point)
if ! grep -q "mount_chart_with_options" "$ROOT/crates/honba-kao-app/src/features/workbench/components/pane_chart.rs" 2>/dev/null; then
    echo "❌ FAIL: mount_chart_with_options() not used in PaneChart"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ Widget-tier mount used in PaneChart"
fi

# Also verify js_mount is only in extern block (import) not called directly in effect
if grep -A5 -B5 "match js_mount(" "$ROOT/crates/honba-kao-app/src/shared/components/charts/openalgo_chart.rs" 2>/dev/null | grep -v "mount_chart_with_options"; then
    echo "❌ FAIL: Raw js_mount() called in effect instead of mount_chart_with_options()"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No raw js_mount() calls in effects"
fi

# 5. Required hide rules present in workbench.css
echo "  Checking required CSS hide rules..."
REQUIRED_HIDES=(
    "\.oac-widget \.oac-topbar"
)
for pattern in "${REQUIRED_HIDES[@]}"; do
    if ! grep -q "$pattern" "$ROOT/style/workbench.css" 2>/dev/null; then
        echo "❌ FAIL: Missing hide rule for $pattern"
        ERRORS=$((ERRORS + 1))
    fi
done
if [ $ERRORS -eq 0 ]; then
    echo "✅ All required hide rules present"
fi

# 6. Theme bridge maps all required tokens
echo "  Checking theme bridge token mapping..."
REQUIRED_TOKENS=(
    "background:"
    "grid:"
    "upColor:"
    "downColor:"
    "buy:"
    "sell:"
)
for token in "${REQUIRED_TOKENS[@]}"; do
    if ! grep -q "$token" "$ROOT/vendor/openalgo-charts/honba-bridge.js" 2>/dev/null; then
        echo "❌ FAIL: Theme bridge missing mapping for $token"
        ERRORS=$((ERRORS + 1))
    fi
done
if [ $ERRORS -eq 0 ]; then
    echo "✅ Theme bridge has all required mappings"
fi

# 7. Components use .wb-* namespace
echo "  Checking component namespace usage..."
if grep -r "class=\"wb-" "$ROOT/crates/honba-kao-app/src/features/workbench/components/pane_chart.rs" 2>/dev/null | head -5; then
    echo "✅ Components use .wb-* namespace"
else
    echo "⚠️  Warning: Could not verify .wb-* namespace (may be false negative)"
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "✅ All widget integration checks passed!"
    exit 0
else
    echo "❌ $ERRORS compliance error(s) found"
    exit 1
fi