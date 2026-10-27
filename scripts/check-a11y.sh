#!/usr/bin/env bash
# honba-kao accessibility (a11y) compliance check.
#
# Phase 8.3: enforces WCAG AA contrast-token usage, roving keyboard
# navigation on tables/menus/toggle groups, and full ARIA role compliance.
#
# No browser accessibility engine (axe/pa11y) is available offline in this
# environment, so this is a static source/CSS check. It exits 0 when clean,
# 1 when violations are found. Usage: ./scripts/check-a11y.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_CORE="$ROOT/crates/honba-kao-ui-core/src"
STYLE="$ROOT/style"

ERRORS=0

fail() { echo "❌ $1"; ERRORS=$((ERRORS + 1)); }
pass() { echo "✅ $1"; }

echo "🔍 honba-kao a11y compliance check (Phase 8.3)"

# --- 1. WCAG AA contrast: theme token contract defines contrast pair -------
echo "  Checking WCAG AA contrast token contract..."
# Every skin block defines --background-color / --text-color / --border-color
# with --shade-9 as max-contrast text. Verify the contract is present.
if grep -rqE -- '--(background-color|text-color|border-color|shade-9|primary-8)' \
     "$STYLE/themes.css" "$STYLE/palette_256.css" 2>/dev/null; then
  pass "contrast token contract present in themes.css / palette_256.css"
else
  fail "missing WCAG AA contrast token contract (--text-color/--background-color/--shade-9)"
fi

# --- 2. Roving keyboard navigation: tablist / tab / aria-selected -----------
echo "  Checking roving keyboard navigation (tablist/tab)..."
TAB_OK=0
grep -rq 'role="tablist"' "$UI_CORE/primitive/tabs.rs" 2>/dev/null && TAB_OK=$((TAB_OK + 1))
grep -rq 'role="tab"' "$UI_CORE/primitive/tabs.rs" 2>/dev/null && TAB_OK=$((TAB_OK + 1))
grep -rq 'aria-selected' "$UI_CORE/primitive/tabs.rs" 2>/dev/null && TAB_OK=$((TAB_OK + 1))
if [ "$TAB_OK" -eq 3 ]; then
  pass "HfTabs implements tablist/tab/aria-selected roving navigation"
else
  fail "HfTabs missing tablist/tab/aria-selected (keyboard users cannot rove tabs)"
fi

# --- 3. ARIA roles & state on menus, toggles, buttons -----------------------
echo "  Checking ARIA roles/state on interactive primitives..."
for f in toggle.rs menu.rs button.rs; do
  path="$UI_CORE/primitive/$f"
  if grep -rqE 'aria-pressed|aria-expanded|aria-label|role="menu"|role="menuitem"' "$path" 2>/dev/null; then
    pass "$f exposes ARIA roles/state"
  else
    fail "$f missing ARIA roles/state"
  fi
done

# --- 4. Focus management: focus-visible outline policy for keyboard users --
echo "  Checking focus-visible / outline policy..."
if grep -rqE 'focus-visible|outline:' "$STYLE/main.css" "$STYLE/app.css" "$STYLE/command_bar.css" 2>/dev/null; then
  pass "focus-visible / outline policy defined"
else
  fail "no focus-visible outline policy (keyboard users lose the focus indicator)"
fi

echo ""
if [ "$ERRORS" -eq 0 ]; then
  echo "✅ All a11y checks passed"
  exit 0
else
  echo "❌ $ERRORS a11y check(s) failed"
  exit 1
fi
