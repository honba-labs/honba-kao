# Widget Integration Strategy: openalgo-charts Widget Tier ↔ honba-kao

## Problem
The openalgo-charts widget tier (v2.6.6+) ships its own:
- **CSS tokens**: `--oac-*`, `--oac-panel`, `--oac-bg`, `--oac-acc`, etc.
- **Components**: `.oac-widget`, `.oac-topbar`, `.oac-rail`, `.oac-rail-host`, `.oac-chart`, dialogs, toasts
- **Styles**: Injected via `injectWidgetStyles(document, WIDGET_COMPONENT_CSS)`

These conflict with honba-kao's token system (`--bg-*`, `--primary-*`, `--shade-*`, `--pnl-*`, `--border-*`) and custom components (`.wb-chart-topbar`, `.wb-draw-rail`, `.wb-pane`, etc.).

## Solution: Strict Isolation Layer

### 1. Token Mapping Bridge
The widget's theme is built from honba-kao's CSS tokens via `honba-bridge.js:themeFromSkin()`. This is the **only** coupling point — widget tokens derive from honba-kao tokens at mount time.

```javascript
// honba-bridge.js - THE ONLY COUPLING POINT
function themeFromSkin() {
  const base = isDarkMode() ? darkTheme : lightTheme;
  return {
    ...base,
    background: cssVar('--chart-background', base.background),
    grid: cssVar('--chart-grid', base.grid),
    // ... ALL widget colors mapped from honba-kao tokens
    upColor: cssVar('--pnl-gain', base.upColor),
    downColor: cssVar('--pnl-loss', base.downColor),
    buy: cssVar('--pnl-gain', base.buy),
    sell: cssVar('--pnl-loss', base.sell),
  };
}
```

**Rule**: Never add new `--oac-*` token references in honba-kao CSS. All widget styling flows through this bridge.

### 2. Component Substitution (Not Composition)
Disable widget's built-in chrome, use honba-kao equivalents:

| Widget Component | Widget Config | honba-kao Replacement | CSS Selector |
|-----------------|---------------|----------------------|--------------|
| Topbar | `topbar: true` | `topbar: false` + `.wb-chart-topbar` | `.oac-topbar` → `display: none` |
| Drawing Rail | `rail: true` | `rail: false` + `.wb-draw-rail` | `.oac-rail`, `.oac-rail-host` → `display: none` |
| Settings Dialog | Built-in | `widget.openSettings()` | Uses widget dialog (OK) |
| Indicator Picker | Built-in | `widget.openIndicatorPicker()` | Uses widget dialog (OK) |
| Objects Panel | Built-in | `widget.openObjects()` | Uses widget dialog (OK) |

### 3. CSS Isolation Rules

**In `workbench.css` (or any honba-kao CSS):**

```css
/* 1. HIDE WIDGET CHROME WE SUBSTITUTE */
.oac-widget .oac-rail,
.oac-widget .oac-rail-host,
.oac-widget [data-oac-rail],
.oac-widget .oac-topbar {
  display: none !important;
}

/* 2. SCOPE OUR COMPONENTS TO .wb-* NAMESPACE ONLY */
.wb-chart-topbar { /* honba-kao styles */ }
.wb-draw-rail { /* honba-kao styles */ }
.wb-pane { /* honba-kao styles */ }

/* 3. NEVER USE .oac-* SELECTORS IN honba-kao CSS */
/*    (except the hide rules above) */

/* 4. NEVER USE --oac-* TOKENS IN honba-kao CSS */
/*    Use --bg-*, --primary-*, --shade-*, --pnl-*, --border-* instead */
```

**In widget integration code (`honba-bridge.js`):**

```javascript
// ALWAYS disable widget chrome we replace
const widget = createWidget(element, {
  rail: false,      // We provide .wb-draw-rail
  topbar: false,    // We provide .wb-chart-topbar
  statusline: true, // OK to keep (thin, no conflicts)
  // ...
});

// Inject widget CSS (required for dialogs, toasts, pickers)
injectWidgetStyles(document, WIDGET_COMPONENT_CSS);
```

### 4. Wrapper Pattern for New Components

When creating a new component that embeds the widget:

```rust
// components/charts/openalgo_widget.rs (example wrapper)
use leptos::prelude::*;

#[component]
pub fn OpenAlgoWidget(
    symbol: String,
    timeframe: String,
    chart_mode: String,
    // ... props
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();

    // Mount widget with substitution config
    Effect::new(move |_| {
        if let Some(el) = host.get() {
            bridge::mount_chart_with_options(&el, &symbol, &timeframe, &chart_mode, qty);
        }
    });

    // Wrap in isolation container
    view! {
        <div class="oac-widget-wrapper" node_ref=host>
            {/* Our custom topbar renders HERE, above widget */}
            <ChartTopbar ... />
            {/* Widget renders chart + statusline inside */}
            <div class="oac-chart-container" />
            {/* Our custom rail renders HERE, overlapping widget */}
            <DrawingRail ... />
        </div>
    }
}
```

CSS for wrapper:
```css
.oac-widget-wrapper {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  position: relative;
  isolation: isolate; /* Creates stacking context */
}

.oac-widget-wrapper > .oac-widget {
  flex: 1;
  min-height: 0;
  width: 100%;
  height: 100%;
}

.oac-widget-wrapper .wb-chart-topbar {
  position: relative;
  z-index: 10; /* Above widget */
}

.oac-widget-wrapper .wb-draw-rail {
  position: absolute;
  top: 104px; /* Clear Buy/Sell bubble */
  left: 8px;
  z-index: 20; /* Above widget chart */
}
```

### 5. Future Component Checklist

Every new component using the widget MUST:

- [ ] Set `rail: false`, `topbar: false` in `createWidget` options
- [ ] Add hide rules for `.oac-rail`, `.oac-rail-host`, `.oac-topbar` in CSS
- [ ] Use `.wb-*` namespace for all custom styles
- [ ] Use honba-kao tokens (`--bg-*`, `--primary-*`, `--shade-*`, `--pnl-*`, `--border-*`)
- [ ] Never reference `--oac-*` tokens directly
- [ ] Wrap widget mount in `.oac-widget-wrapper` with `isolation: isolate`
- [ ] Place custom topbar/rail as siblings to widget root, not inside
- [ ] Call `bridge::mount_chart_with_options()` not raw `js_mount()`

### 6. Automated Enforcement (TODO)

Add to CI:
```bash
# Check for forbidden patterns in honba-kao CSS
grep -r "\.oac-" crates/honba-kao-app/src/ --include="*.css" --include="*.rs" | \
  grep -v "display: none" | grep -v "hide" && echo "FAIL: .oac-* usage found" && exit 1

grep -r "\-\-oac-" crates/honba-kao-app/src/ --include="*.css" --include="*.rs" && \
  echo "FAIL: --oac-* token usage found" && exit 1

# Check widget mounts use correct config
grep -r "rail: true\|topbar: true" crates/honba-kao-app/src/ --include="*.js" && \
  echo "FAIL: widget chrome enabled" && exit 1
```

### 7. Token Reference

**honba-kao tokens (USE THESE):**
```
--bg-app, --bg-surface, --bg-surface-elevated, --bg-surface-hover
--primary-0..9, --accent-primary, --accent-subtle
--shade-0..9, --shade-10, --shade-15, --shade-20
--pnl-gain, --pnl-loss
--border-subtle, --border-medium, --border-strong
--text-primary, --text-secondary, --text-muted, --on-accent
--warning-color, --color-loss
--font-family-mono, --font-mono
```

**Widget tokens (DERIVED ONLY via bridge, NEVER USE DIRECTLY):**
```
--oac-bg, --oac-panel, --oac-elev, --oac-elev-2
--oac-acc, --oac-acc-2, --oac-on-bg, --oac-on-bd
--oac-bd, --oac-bd-soft, --oac-ring
--oac-tx, --oac-mut, --oac-faint
--oac-danger, --oac-amber
--oac-shadow, --oac-chart-background
--oac-up, --oac-down
```

## Migration Complete Status

- [x] Bridge updated: `rail: false`, `topbar: false`
- [x] Custom `ChartTopbar` component (`.wb-chart-topbar`)
- [x] Custom `DrawingRail` component (`.wb-draw-rail`)
- [x] CSS hide rules for widget chrome
- [x] Token mapping in `themeFromSkin()`
- [x] Wrapper isolation with `isolation: isolate`
- [ ] Automated CI checks (TODO)
- [ ] Document this strategy in README (this file)

## Summary

**The widget is a rendering engine only.** It provides:
- Chart canvas with 13 chart types
- 502 indicators via dialogs
- Drawing controller (headless, we add UI)
- Buy/Sell bubble, volume overlay
- Time axis, crosshair, statusline

**honba-kao owns ALL chrome:** toolbars, rails, menus, modals, drawers, tickets.

This separation ensures zero CSS conflicts now and forever.