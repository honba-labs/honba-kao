# Master Plan: 本場・顔 honba-kao

> **The face of the real thing.**
> High-performance Leptos (0.7+ CSR) web trading terminal for Indian equities (`honba-ya` backend, `honba-kokoro` types, `honba-me` indicators), modeled on the professional UI architecture, styling language, and component design of [Bitfinex Honey Framework Core (`bfx-hf-ui-core`)](https://github.com/bitfinexcom/bfx-hf-ui-core), powered by an extensible **256-color named design token palette** for limitless custom theming.

---

## 0. Executive Summary & Critical Assessment

### Current State (as of 2026-10-19)
- **Working prototype** with 256-color palette, 8 themes, widget-tier chart integration
- **Critical gaps** preventing professional-grade delivery:
  - ❌ No design system crate — 10 primitives only, missing 25+ Radix/Shadcn components
  - ❌ Domain components use raw HTML + custom CSS (violates `hints.md`)
  - ❌ Monolithic stores (1659-line `workbench.rs`), no query layer, no form handling
  - ❌ No route loaders, error boundaries, code splitting, a11y patterns
  - ❌ No component tests, no e2e, no CI enforcement

### Target State: Professional Leptos Architecture
- ✅ `honba-kao-ui-core` crate with 35+ accessible primitives (Radix parity)
- ✅ Feature-sliced architecture (`features/`, `shared/`, `widgets/`, `pages/`)
- ✅ `leptos-query` for server state, optimistic mutations, caching
- ✅ Form library with schema validation
- ✅ Route loaders, auth guards, error boundaries
- ✅ Full a11y (ARIA, focus management, keyboard nav)
- ✅ Component tests + Playwright e2e
- ✅ CI enforcement for design system usage

---

## 1. Architectural Vision & Principles

`honba-kao` is a single-page WebAssembly application (CSR) built with **Leptos 0.7**, compiled via **Trunk**, communicating with `honba-ya` over typed HTTP REST and WebSocket streaming.

### Key Tenets
1. **Trading-Terminal First**: Dense, high-contrast, information-rich layouts for intraday algo-trading, position monitoring, and decision audits.
2. **Visual Fidelity to `bfx-hf-ui-core`**: Strict adherence to Bitfinex Honey Framework design system—dark/light palettes, variable-based shades, 5px corner radii, elevation shading, tabular numeric typography.
3. **256-Color Named Palette System**: Rigorous, extensible 256-color named palette (`--c-000` to `--c-255`) as canonical design tokens for current Bitfinex themes and any future custom themes.
4. **Pure Rust Reactivity**: Signals, memos, resources in Leptos 0.7 without virtual DOM overhead, delivering 60fps tick updates and zero-jank UI reflows.
5. **Native WASM Compute**: Seamless integration with `honba-kao-api-wasm` to run screener filters, OHLC aggregations, and technical indicators locally.
6. **Design System Discipline**: Every component composes from `honba-kao-ui-core` primitives. Zero raw HTML in domain components (enforced by CI).

---

## 2. Design System & 256-Color Named Palette Engine

The styling infrastructure is divided into three distinct layers:
1. **Canonical Palette Layer (Tokens `000` to `255`)**: 256 named, calibrated color tokens.
2. **Semantic Theme Alias Layer**: Maps Honey Framework variables (`--background-color`, `--primary-*`, `--shade-*`) to canonical palette tokens.
3. **Runtime Custom Theme Engine**: Dynamic JSON-based theme loading, runtime CSS injection, Canvas/SVG chart color synchronization in Rust.

```
┌────────────────────────────────────────────────────────────────────────┐
│               Layer 0: Canonical 256-Color Named Palette               │
│  [000-015] Core System/Terminal (16)  - ANSI Standards & Brights       │
│  [016-231] Chromatic Spectrum (216)   - 18 Hues × 12 Luminance Steps   │
│  [232-255] Monochrome Ramp (24)       - Calibrated Gray Luminance      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│               Layer 1: Semantic Theme Aliases (CSS Vars)               │
│  --background-color        ──> var(--c-marine-02) / c026              │
│  --darker-background-color ──> var(--c-marine-01) / c025              │
│  --pnl-gain                ──> var(--c-teal-08)   / c068              │
│  --pnl-loss                ──> var(--c-coral-08)  / c152              │
│  --shade-0 .. --shade-9    ──> var(--c-marine-03..mono-20)            │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│             Layer 2: Leptos UI Components & Canvas Renderers           │
│  HfPanel, HfButton, HfPLNumber, HfSelect, HfDataTable, ChartWidget    │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.1 The 256-Color Canonical Named Token Architecture

**Every token has an index (`--c-000` .. `--c-255`) and an unambiguous semantic name (`--c-<name>`):**

#### 1. System & Terminal Core (Tokens `000`–`015` — 16 Colors)
| Index | Name | Hex | Purpose |
|-------|------|-----|---------|
| 000 | `--c-term-black` | `#0a1017` | Deep void / canvas base |
| 001 | `--c-term-red` | `#d9383e` | Error / loss |
| 002 | `--c-term-green` | `#00b88d` | Success / gain |
| 003 | `--c-term-yellow` | `#d9b300` | Warning |
| 004 | `--c-term-blue` | `#0073d9` | Info / links |
| 005 | `--c-term-magenta` | `#b340b3` | Algo signals |
| 006 | `--c-term-cyan` | `#00a3a3` | Ticker highlights |
| 007 | `--c-term-white` | `#d0d7de` | Primary text |
| 008 | `--c-term-bright-black` | `#4f5a62` | Muted text |
| 009 | `--c-term-bright-red` | `#f05359` | Bright loss |
| 010 | `--c-term-bright-green` | `#00e0ad` | Bright gain |
| 011 | `--c-term-bright-yellow` | `#fcd206` | Bright warning |
| 012 | `--c-term-bright-blue` | `#0089ff` | Bright info |
| 013 | `--c-term-bright-magenta` | `#e05ce0` | Bright algo |
| 014 | `--c-term-bright-cyan` | `#00d9d9` | Bright ticker |
| 015 | `--c-term-bright-white` | `#ffffff` | Max contrast |

#### 2. Chromatic Spectrum Ramps (Tokens `016`–`231` — 216 Colors = 18 Hues × 12 Steps)
Each hue: 12 perceptual luminance steps (01 = deepest, 06 = pure hue, 12 = pastel).

| Hue | Tokens | Use Case |
|-----|--------|----------|
| `slate-01..12` | `016–027` | Cool neutrals, card borders, passive text |
| `marine-01..12` | `028–039` | **Bitfinex dark terminal navy** (`#102331`, `#172d3e`, `#1e384d`) |
| `azure-01..12` | `040–051` | Electric blue accents, focus rings, links |
| `cyan-01..12` | `052–063` | Aqua ticker highlights, code indicators |
| `teal-01..12` | `064–075` | **Signature mint** (`#00e0ad` ramp) — buy/gain |
| `emerald-01..12` | `076–087` | Positive PnL, buy walls, confirmed fills |
| `lime-01..12` | `088–099` | Neon green for burst volume anomalies |
| `chartreuse-01..12` | `100–111` | Oscillators, RSI oversold/overbought |
| `amber-01..12` | `112–123` | Warnings, risk throttle, connecting |
| `gold-01..12` | `124–135` | Precious metals, benchmarks, premium |
| `orange-01..12` | `136–147` | Pending orders, margin warnings |
| `coral-01..12` | `148–159` | **Sell scale** (`#f05359` ramp) |
| `crimson-01..12` | `160–171` | Negative PnL, stop loss, kill-switch |
| `rose-01..12` | `172–183` | Drawdown shading, spread highlight |
| `magenta-01..12` | `184–195` | Algo state markers, strategy signals |
| `purple-01..12` | `196–207` | Backtest equity curves, benchmarks |
| `violet-01..12` | `208–219` | Bracket/OCO boundaries |
| `indigo-01..12` | `220–231` | Liquidity heatmaps, cumulative volume |

#### 3. Monochrome Luminance Ramp (Tokens `232`–`255` — 24 Steps)
Calibrated grayscale from absolute void (`mono-00`) to pure white (`mono-23`):
- `232` (`--c-mono-00`): `#080c10` (deep void / canvas base)
- `244` (`--c-mono-12`): `#7a8894` (mid-gray border / inactive icon)
- `254` (`--c-mono-22`): `#f0f3f6`
- `255` (`--c-mono-23`): `#ffffff` (pure white text highlight)

---

## 3. Professional Architecture Redesign

### 3.1 Crate Structure (New)

```
honba-kao/
├── crates/
│   ├── honba-kao-ui-core/        # NEW: Design system primitives (35+ components)
│   ├── honba-kao-app/            # REFACTORED: Feature-sliced architecture
│   ├── honba-kao-client/         # Typed REST client (keep)
│   ├── honba-kao-api-wasm/       # WASM compute (keep)
│   └── honba-kao-test-utils/     # NEW: Testing helpers
├── docs/
│   ├── widget-integration.md     # Widget tier isolation
│   ├── architecture.md           # This document
│   ├── component-guide.md        # Component authoring guide
│   └── contributing.md           # Development workflow
├── scripts/
│   ├── check-widget-integration.sh
│   ├── check-ui-core-usage.sh    # Enforces ui-core usage
│   └── check-a11y.sh             # Accessibility audit
└── style/                        # Organized SCSS modules
```

### 3.2 `honba-kao-ui-core` — Design System Primitives (Mirroring `bfx-hf-ui-core`)

**Structure mirrors `bfx-hf-ui-core/src/ui` and `src/components` with `Hf` prefix:**

```
honba-kao-ui-core/src/
├── primitive/              # Unstyled, accessible primitives (Radix parity)
│   ├── icons.rs           # HfIcon — procedural outline icons (stroke="currentColor", fill="none", 16/20/24px)
│   ├── button.rs          # HfButton, HfIconButton — variants: Default/Green/Red/Blue/Ghost
│   ├── input.rs           # HfInput, HfTextarea
│   ├── select.rs          # HfSelect, HfSelectTrigger, HfSelectContent, HfSelectItem, HfSelectSeparator
│   ├── checkbox.rs        # HfCheckbox
│   ├── radio.rs           # HfRadioGroup, HfRadioGroupItem
│   ├── switch.rs          # HfSwitch
│   ├── label.rs           # HfLabel
│   ├── separator.rs       # HfSeparator
│   ├── slider.rs          # HfSlider
│   ├── tabs.rs            # HfTabs, HfTabsList, HfTabsTrigger, HfTabsContent
│   ├── tooltip.rs         # HfTooltip, HfTooltipTrigger, HfTooltipContent
│   ├── popover.rs         # HfPopover, HfPopoverTrigger, HfPopoverContent
│   ├── menu.rs            # HfMenu, HfMenuTrigger, HfMenuContent, HfMenuItem, HfMenuSeparator
│   ├── dialog.rs          # HfDialog, HfDialogTrigger, HfDialogContent, HfDialogHeader, HfDialogFooter
│   ├── drawer.rs          # HfDrawer, HfDrawerTrigger, HfDrawerContent, HfDrawerHeader, HfDrawerFooter
│   ├── sheet.rs           # HfSheet (mobile drawer)
│   ├── toast.rs           # HfToast, HfToastProvider, HfToastViewport, HfToastTitle, HfToastDescription, HfToastAction
│   ├── alert.rs           # HfAlert, HfAlertTitle, HfAlertDescription
│   ├── avatar.rs          # HfAvatar, HfAvatarImage, HfAvatarFallback
│   ├── badge.rs           # HfBadge
│   ├── skeleton.rs        # HfSkeleton
│   ├── scroll_area.rs     # HfScrollArea
│   ├── table.rs           # HfTable, HfTableHeader, HfTableBody, HfTableRow, HfTableCell, HfTableCaption, HfTableHeaderCell
│   ├── pagination.rs      # HfPagination, HfPaginationItem, HfPaginationLink
│   ├── breadcrumb.rs      # HfBreadcrumb, HfBreadcrumbItem, HfBreadcrumbSeparator, HfBreadcrumbLink, HfBreadcrumbPage, HfBreadcrumbEllipsis
│   ├── accordion.rs       # HfAccordion, HfAccordionItem, HfAccordionTrigger, HfAccordionContent
│   ├── collapsible.rs     # HfCollapsible, HfCollapsibleTrigger, HfCollapsibleContent
│   ├── hover_card.rs      # HfHoverCard, HfHoverCardTrigger, HfHoverCardContent
│   ├── context_menu.rs    # HfContextMenu, HfContextMenuTrigger, HfContextMenuContent, HfContextMenuItem, HfContextMenuSeparator
│   ├── command.rs         # HfCommand, HfCommandInput, HfCommandList, HfCommandItem, HfCommandSeparator, HfCommandShortcut
│   ├── navigation_menu.rs # HfNavigationMenu, HfNavigationMenuList, HfNavigationMenuItem, HfNavigationMenuLink, HfNavigationMenuIndicator
│   ├── progress.rs        # HfProgress
│   ├── toggle.rs          # HfToggle, HfToggleGroup, HfToggleGroupItem
│   ├── form.rs            # HfForm, HfField, HfFormItem, HfFormLabel, HfFormControl, HfFormMessage, HfFormSubmit
│   ├── aspect_ratio.rs    # HfAspectRatio
│   ├── resizable.rs       # HfResizablePanelGroup, HfResizablePanel, HfResizableHandle
│   └── scrollbar.rs       # HfScrollbar (custom scrollbar styling)
├── trading/                # Trading-specific primitives (bfx-hf-ui-core `components/` parity)
│   ├── pl_number.rs       # HfPLNumber — P&L with ▲/▼, ₹/%, color-coded (teal/coral)
│   ├── status_circle.rs   # HfStatusCircle — Green/Yellow/Red with latency text
│   ├── kpi.rs             # HfKPI — Key metric card with label, value, change, sparkline
│   ├── panel.rs           # HfPanel, HfPanelHeader, HfPanelContent, HfPanelSidebar
│   ├── attention_bar.rs   # HfAttentionBar — top-bar warnings (throttle, circuit, margin)
│   ├── progress_bar.rs    # HfProgressBar — fill percentage with color coding
│   ├── side_marker.rs     # HfSideMarker — row marker dot (green/red) for buy/sell
│   ├── order_badge.rs     # HfOrderBadge — order type (MKT/LMT/SL/SL-M), status, side
│   ├── risk_badge.rs      # HfRiskBadge — rejection reason (CIRCUIT_FILTER, MAX_DRAWDOWN, etc.)
│   ├── market_phase.rs    # HfMarketPhase — Pre-open/Regular/Closing/Closed indicator
│   ├── trading_mode.rs    # HfTradingMode — SANDBOX (yellow) vs LIVE NSE (green)
│   ├── quantity_input.rs  # HfQuantityInput — stepper with quick chips (25/50/100/250)
│   ├── price_input.rs     # HfPriceInput — ₹ formatting, tick-size validation
│   └── symbol_chip.rs     # HfSymbolChip — symbol with exchange badge, clickable
├── composed/               # Opinionated, styled patterns (feature-level)
│   ├── data_table.rs      # HfDataTable — sortable, filterable, paginated, virtualized
│   ├── filter_bar.rs      # HfFilterBar — segmented pills, HfSelect, HfSwitch, search
│   ├── header_bar.rs      # HfHeaderBar — logo, nav, mode toggle, theme, notifications
│   ├── sidebar.rs         # HfSidebar — icon rail navigation (HfSidebarItem)
│   ├── chart_toolbar.rs   # HfChartToolbar — symbol, timeframe, chart type, indicators, snapshot
│   ├── order_ticket.rs    # HfOrderTicket — symbol, side, type, qty, price, SL/TP, submit
│   ├── trading_drawer.rs  # HfTradingDrawer — tabs: Holdings/Positions/Orders/Trades/Analytics/Depth
│   ├── modal.rs           # HfModal — confirm, alert, prompt (HfConfirmDialog, HfAlertDialog)
│   ├── positions_table.rs # HfPositionsTable — Symbol, Side, Qty, Avg, LTP, UPL, RPL, actions
│   ├── orders_table.rs    # HfOrdersTable — tabs: Active/Algo/History, fill progress, cancel
│   ├── trades_table.rs    # HfTradesTable — timestamp, slippage, fees
│   ├── decision_log.rs    # HfDecisionLog — timestamp, strategy, symbol, verdict, rejection badges
│   ├── risk_modal.rs      # HfRiskModal — max drawdown, daily loss, position limits editor
│   ├── sparkline.rs       # HfSparkline — Canvas/SVG micro-chart, palette-synced colors
│   └── theme_studio.rs    # HfThemeStudio — 256-swatch inspector, contrast audit, JSON import/export
├── hooks/
│   ├── use_click_outside.rs
│   ├── use_escape_key.rs
│   ├── use_focus_trap.rs
│   ├── use_media_query.rs
│   ├── use_local_storage.rs
│   ├── use_debounce.rs
│   ├── use_controllable_state.rs
│   ├── use_tick_stream.rs      # WebSocket tick subscription
│   ├── use_market_phase.rs     # NSE phase computation
│   └── use_trading_state.rs    # HALTED/REDUCING/ACTIVE sync
├── icons/
│   ├── outline.rs         # Procedural outline icons (stroke only, fill="none")
│   │   # All icons: fn name() -> View { view! { <svg viewBox="0 0 24 24" stroke="currentColor" fill="none" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">...</svg> } }
│   │   # Trading: chart, candlestick, trending_up, trending_down, buy, sell, order, position, portfolio, wallet
│   │   # Navigation: home, settings, search, menu, chevron_left, chevron_right, chevron_up, chevron_down
│   │   # Actions: plus, minus, x, check, edit, delete, copy, download, upload, refresh, filter, sort
│   │   # Status: check_circle, alert_circle, alert_triangle, info, loader, wifi, wifi_off, server
│   │   # Chart: timeframe_1m, timeframe_1h, timeframe_1d, chart_type_candle, chart_type_line, chart_type_area
│   │   # Drawing: trend_line, horizontal_line, vertical_line, ray, rectangle, ellipse, text, measure
│   │   # UI: bell, bell_off, eye, eye_off, lock, unlock, key, user, log_out, log_in
│   └── mod.rs             # Re-exports: pub use outline::*;
└── theme/
    ├── tokens.rs          # Semantic token access (--shade-*, --primary-*, --pnl-*)
    ├── css_vars.rs        # CSS variable helpers
    └── palette_256.rs     # Palette256 struct, contrast audit, JSON import/export
```

**bfx-hf-ui-core → honba-kao-ui-core Mapping:**

| bfx-hf-ui-core | honba-kao-ui-core | Category |
|---|---|---|
| `ui/Panel` | `trading/panel.rs` → `HfPanel`, `HfPanelHeader` | Trading primitive |
| `ui/Button` | `primitive/button.rs` → `HfButton`, `HfIconButton` | Primitive |
| `ui/Icons` | `primitive/icons.rs` + `icons/outline.rs` → `HfIcon` | Primitive |
| `ui/PLNumber` | `trading/pl_number.rs` → `HfPLNumber` | Trading primitive |
| `ui/Dropdown`, `SimpleDropdown` | `primitive/select.rs` → `HfSelect` | Primitive |
| `ui/Modal` | `primitive/dialog.rs` → `HfDialog` | Primitive |
| `ui/AttentionBar` | `trading/attention_bar.rs` → `HfAttentionBar` | Trading primitive |
| `ui/ProgressBar` | `trading/progress_bar.rs` → `HfProgressBar` | Trading primitive |
| `ui/Tabs` | `primitive/tabs.rs` → `HfTabs` | Primitive |
| `ui/StatusBar` | `trading/status_circle.rs` → `HfStatusCircle` + composed | Trading |
| `components/Navbar` | `composed/header_bar.rs` → `HfHeaderBar` | Composed |
| `components/StatusBar` | `composed/trading_drawer.rs` + `trading/status_circle.rs` | Composed |
| `components/PositionsTable` | `composed/positions_table.rs` → `HfPositionsTable` | Composed |
| `components/AtomicOrdersTable` | `composed/orders_table.rs` → `HfOrdersTable` | Composed |
| `components/TradesTablePanel` | `composed/trades_table.rs` → `HfTradesTable` | Composed |
| `components/TradingStatePanel` | `composed/trading_drawer.rs` + `trading/status_circle.rs` | Composed |
| *(New)* | `trading/decision_log.rs` → `HfDecisionLog` | Trading |
| *(New)* | `trading/risk_badge.rs` → `HfRiskBadge` | Trading |
| *(New)* | `composed/sparkline.rs` → `HfSparkline` | Composed |
| *(New)* | `composed/theme_studio.rs` → `HfThemeStudio` | Composed |

**Compliance Rule (CI-enforced):**
```bash
# Domain components MUST compose from honba-kao-ui-core
# Forbidden in domain components: raw <button>, <input>, <select>, <dialog>, <table>
# Required: import from honba-kao-ui-core::{primitive, trading, composed}

# ICON RULE: Outline SVG only, generated on-the-fly
# - No static .svg files in the repo
# - All icons generated via `honba_kao_ui_core::icons::outline::*`
# - Each icon is a Rust function returning `View` with stroke="currentColor" fill="none"
# - Sizes: 16px (default), 20px (lg), 24px (xl) via CSS variables
# - CI check: `grep -r "\.svg" crates/honba-kao-app/src --include="*.rs" && exit 1`
```

**Compliance Rule (CI-enforced):**
```bash
# No raw <button>, <input>, <select> in domain components
# Must import from honba-kao-ui-core
```

### 3.3 Feature-Sliced Architecture (`honba-kao-app`)

```
crates/honba-kao-app/src/
├── app.rs                    # Root: providers + router + error boundary
├── main.rs                   # WASM entry point
├── error_boundary.rs         # Global error boundary
├── features/                 # Business logic slices
│   ├── auth/
│   │   ├── api.rs            # Auth API calls
│   │   ├── model.rs          # User, Session types
│   │   ├── store.rs          # AuthStore (signals + actions)
│   │   ├── components/       # LoginForm, RegisterForm
│   │   └── hooks.rs          # use_auth, use_login
│   ├── portfolio/
│   ├── screener/
│   ├── workbench/
│   ├── strategy_studio/
│   ├── simulator/
│   ├── tradebook/
│   ├── funds/
│   └── settings/
├── shared/                   # Cross-feature kernel
│   ├── api/                  # Base client, interceptors, query client
│   ├── hooks/                # use_query, use_mutation, use_debounce
│   ├── components/           # Layout, ErrorBoundary, Loading
│   ├── types/                # Shared domain types
│   └── utils/
├── widgets/                  # Composite UI blocks (reusable)
│   ├── chart/                # ChartWidget, ChartToolbar, DrawingRail
│   ├── order_ticket/
│   └── trading_drawer/
├── pages/                    # Thin page composition
│   ├── portfolio_page.rs
│   ├── workbench_page.rs
│   └── ...
├── router/
│   ├── routes.rs             # Route definitions with loaders
│   ├── guards.rs             # AuthGuard, RoleGuard
│   └── layout.rs             # AppChrome, PublicLayout
└── providers/
    ├── theme_provider.rs
    ├── auth_provider.rs
    ├── query_provider.rs     # leptos-query provider
    └── toast_provider.rs
```

### 3.4 State Management Overhaul

**Replace monolithic stores with:**
- **`leptos-query`** (TanStack Query port) for server state
- **`leptos_router::use_query_map`** for URL state
- **Local signals** for UI state only
- **`leptos_store` pattern** for complex client state (workbench)

```rust
// features/workbench/store.rs - Refactored
use leptos::prelude::*;
use leptos_query::{use_query, use_mutation, QueryClient};

#[derive(Clone)]
pub struct WorkbenchStore {
    query_client: QueryClient,
    active_pane: RwSignal<usize>,
    layout: RwSignal<WorkbenchLayout>,
    drawer_open: RwSignal<bool>,
    ticket_open: RwSignal<bool>,
}

impl WorkbenchStore {
    // Server state via queries
    pub fn positions(&self) -> Query<Vec<Position>> {
        use_query(["positions"], || api::get_positions())
    }
    
    pub fn orders(&self) -> Query<Vec<Order>> {
        use_query(["orders"], || api::get_orders())
    }
    
    // Mutations with optimistic updates
    pub fn place_order(&self) -> Mutation<OrderRequest, Order> {
        use_mutation(
            |req| api::post_order(req),
            OnMutate::new(|req| {
                self.orders.update(|o| o.insert(0, req.into()));
            }),
            OnError::new(|_, _, _| self.orders.invalidate()),
            OnSettled::new(|_| {
                self.orders.invalidate();
                self.positions.invalidate();
            })
        )
    }
}
```

### 3.5 Data Fetching Layer

```rust
// shared/api/query_client.rs
use leptos_query::{QueryClient, QueryClientProvider};

pub fn create_query_client() -> QueryClient {
    QueryClient::new()
        .set_default_stale_time(Duration::from_secs(30))
        .set_default_cache_time(Duration::from_secs(300))
        .set_retry(3)
        .set_retry_delay(|attempt| Duration::from_millis(1000 * attempt as u64))
}

// Usage in components
let positions = use_query(
    QueryKey::new(&["positions", portfolio_id]),
    || api::get_portfolio_positions(portfolio_id)
);

let place_order = use_mutation(
    |req: OrderRequest| api::post_order(req),
    MutationOptions {
        on_mutate: Some(|req| { /* optimistic update */ }),
        on_error: Some(|err, _, _| toast.error(err.to_string())),
        on_success: Some(|_| {
            query_client.invalidate_queries(&["positions"]);
            query_client.invalidate_queries(&["orders"]);
        }),
    }
);
```

### 3.6 Form Handling

```rust
// ui-core/src/form.rs
pub fn use_form<T: Default + Clone + PartialEq>(
    initial: T,
    validate: impl Fn(&T) -> FieldErrors<T> + 'static,
    on_submit: impl Fn(T) + 'static,
) -> FormHandle<T> { /* ... */ }

// Usage
#[component]
fn OrderTicketForm() -> impl IntoView {
    let form = use_form(
        OrderTicketDraft::default(),
        |draft| validate_order_ticket(draft),
        |draft| store.place_order(draft),
    );
    
    view! {
        <Form on_submit=form.handle_submit>
            <FormField name="symbol">
                <FormLabel>"Symbol"</FormLabel>
                <FormInput bind:value=form.field("symbol") />
                <FormError field="symbol" />
            </FormField>
            <FormSubmit disabled=form.is_submitting>"Place Order"</FormSubmit>
        </Form>
    }
}
```

### 3.7 Routing with Loaders

```rust
// router/routes.rs
#[component]
pub fn AppRoutes() -> impl IntoView {
    let query_client = create_query_client();
    
    view! {
        <QueryClientProvider client=query_client>
            <Router>
                <Routes fallback=|| view! { <Redirect path="/portfolio" /> }>
                    <Route path="/" view=RootRoute />
                    <Route path="/login" view=LoginPage />
                    
                    <Route 
                        path="/portfolio" 
                        view=move || view! {
                            <AppChrome current_page=Page::Portfolio>
                                <PortfolioPage />
                            </AppChrome>
                        }
                        loader=|| async move {
                            let _ = api::get_portfolio_summary().await;
                            let _ = api::get_positions().await;
                        }
                    />
                    <Route 
                        path="/workbench" 
                        view=WorkbenchPage 
                        loader=|| async move {
                            let _ = api::get_trading_state().await;
                        }
                    />
                </Routes>
            </Router>
        </QueryClientProvider>
    }
}
```

---

### 3.8 Workbench Terminal Chrome — single account entry + trading-mode indicator (done)

Investigated the honba-fe workbench chrome (reference) and aligned honba-kao:

| Surface | honba-fe reference | honba-kao (now) |
|---|---|---|
| User entry | **One** initials avatar, bottom of the 68px rail | **One** `AccountCluster`: initials avatar bottom-left of `NavRail`; popover = identity header, Portfolio, Palette studio, Settings, Funds, sign out; closes on outside click (`use_click_outside`) |
| Header right | Trading-mode segmented indicator (PaperTrading amber / Live pulsing green) + funds + bell | `HfTradingModeToggle` (ui-core) — togglable `PAPER | LIVE` pill, amber/green dots, live pulses; wired to `TradingModeStore`; header's old `UserMenu` removed (guests keep the sign-in CTA) |
| Trading-mode state | Local `equityStore.tradingMode` | `stores/trading_mode.rs`: `TradingMode {Paper, Live}`, localStorage-persisted (`honba.trading_mode`), provided app-wide, Settings checkbox shares it |
| Palette studio | n/a | Hosted by `AppChrome`; opened from the rail's account popover (`palette_open` signal threaded through `NavRail`) |
| Icons | lucide-react | All chrome icons from `honba_kao_ui_core::icons::outline` (`briefcase`, `search_check`, `terminal`, `sliders`, `play_circle`, `book`, `settings`, `wallet`, `user`, `log_out`, `layout_grid`) — made `const fn` so they compose into statics; garbled paths fixed (`user`, `settings`, `wallet`, `alert_circle`, `wifi`, `wifi_off`); no inline SVG in the chrome |

The widget's own drawing rail stays enabled (`rail: true` in `honba-bridge.js` — ~100
grouped tools); the compliance script (`check-widget-integration.sh`) was synced to this
decision: only `topbar: true` is forbidden and only the `.oac-topbar` hide rule is required.
All scripts green.


## 4. Updated Phased Implementation Milestones

### Phase 0: Foundation & Design System Extraction (Week 1-2)
- [ ] **0.1** Create `honba-kao-ui-core` crate with workspace config
- [ ] **0.2** Implement 15 core primitives: `Button`, `Input`, `Select`, `Checkbox`, `Radio`, `Switch`, `Label`, `Dialog`, `Drawer`, `Toast`, `Tooltip`, `Tabs`, `Table`, `Badge`, `Avatar`
- [ ] **0.3** Implement 10 composed patterns: `DataTable`, `FilterBar`, `HeaderBar`, `ChartToolbar`, `OrderTicket`, `TradingDrawer`, `Modal`, `Sidebar`, `Pagination`, `Breadcrumb`
- [ ] **0.4** Add CI enforcement: `check-ui-core-usage.sh` (fails on raw `<button>` in domain components)
- [ ] **0.5** Migrate existing `hints.md` violations to ui-core (filter_bar, screener_table, header_bar)

### Phase 1: Architecture Migration (Week 3-4)
- [ ] **1.1** Restructure `honba-kao-app` to feature-sliced architecture
- [ ] **1.2** Add `leptos-query` + `QueryClientProvider` at root
- [ ] **1.3** Refactor `WorkbenchStore` → query + mutation pattern
- [ ] **1.4** Refactor `ScreenerStore`, `PortfolioStore` similarly
- [ ] **1.5** Add `ErrorBoundary` at route level

### Phase 2: Forms, Routing, Auth (Week 5-6)
- [ ] **2.1** Implement form library with `use_form` hook + schema validation
- [ ] **2.2** Migrate `OrderTicket`, `LoginForm`, `SettingsForm` to new forms
- [ ] **2.3** Add route loaders for `/portfolio`, `/workbench`, `/screener`
- [ ] **2.4** Add `AuthGuard` + `RoleGuard` for protected routes
- [ ] **2.5** Implement JWT refresh + expiration handling

### Phase 3: Workbench & Chart Parity (Week 7-8)
- [ ] **3.1** Complete widget-tier chart integration (already done)
- [ ] **3.2** Build `ChartWidget` + `ChartToolbar` + `DrawingRail` as reusable widgets
- [ ] **3.3** Implement 502-indicator picker with categories/favorites/recents
- [ ] **3.4** Add `SymbolSearchModal` with 4000+ symbols, expressions
- [ ] **3.5** Add snapshot (download/copy), fullscreen, objects panel

### Phase 4: Screener, Strategy Studio, Simulator (Week 9-10)
- [ ] **4.1** Screener: `HfDataTable` with virtualized rows, column visibility, presets
- [ ] **4.2** Strategy Studio (Leptos CSR Visual Canvas & Auto-Researcher)
      — see `honba-docs/docs/architecture/strategy_studio.md` (HBD-2026-DES-006)
  - [x] Task 1: 16px grid-snap node positioning on drop (COMPLETED)
  - [x] Task 2: Edge creation between typed input/output ports with validation (COMPLETED)
  - [x] Task 3: Debounced (200ms) StudioStore persistence to localStorage (COMPLETED)
  - [x] Task 4: CSS category color variable mapping (--studio-cat-*) (COMPLETED)
  - [ ] Task 5: AI Co-Pilot chat drawer, candidate diff cards, and apply/discard actions
  - [ ] Task 6: Autonomous Researcher browser mode: up to 1000-iteration closed-loop mutation
  - [ ] Task 7: StockSharp 100+ Indicator Palette (Visual canvas nodes for all Trend, Momentum, Volatility, Volume, Depth, and Pattern indicators)
  - [ ] Task 8: StockSharp Odysseus Genetic Generator Workbench (Visual rule tree inspector, generation-by-generation evolution monitor, interactive Pareto frontier chart)
- [ ] **4.3** Simulator Workbench (Leptos 0.7 CSR Port)
      — see `honba-docs/docs/architecture/kao_simulator_port.md` (HBD-2026-DES-005)
  - [ ] Strategy hand-off from Strategy Studio via sessionStorage / query parameter
  - [ ] Run configuration controls (symbol, date range, capital, fill model, slippage)
  - [ ] Candle chart + strategy indicator overlay via openalgo-charts bridge
  - [ ] Metrics ribbon: Total Return, Sharpe, Sortino, Max Drawdown, Calmar, Win Rate
  - [ ] Equity and drawdown curve with NIFTY 50 benchmark overlay
  - [ ] Filterable trade journal table with RFC-4180 CSV export and monthly PnL heatmap
  - [ ] Statistical validation trust badges (DSR, PBO, Walk-Forward Efficiency)
- [ ] **4.4** Tradebook: `HfDataTable` with filters, export

### Phase 5: Quality, Testing, Polish (Week 11-12)
- [ ] **5.1** Add `leptos_testing_library` component tests (ui-core + features)
- [ ] **5.2** Add Playwright e2e: auth flow, workbench, order placement
- [ ] **5.3** Accessibility audit: `check-a11y.sh` (WCAG AA)
- [ ] **5.4** Bundle splitting by route (`leptos_router::Lazy`)
- [ ] **5.5** Performance: virtualized lists, memoization, bundle analysis
- [ ] **5.6** Documentation: `architecture.md`, `component-guide.md`, `contributing.md`

---

## 5. Directory Structure Target (Updated)

```
honba-kao/
├── Cargo.toml
├── Makefile
├── README.md
├── plan.md
├── index.html
├── Trunk.toml
├── rust-toolchain.toml
└── crates/
    ├── honba-kao-ui-core/              # NEW: Design system (35+ primitives)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── primitive/              # 25 Radix primitives + icons
    │       │   ├── icons.rs           # HfIcon — procedural outline icons
    │       │   ├── button.rs
    │       │   ├── input.rs
    │       │   ├── select.rs
    │       │   ├── checkbox.rs
    │       │   ├── radio.rs
    │       │   ├── switch.rs
    │       │   ├── label.rs
    │       │   ├── separator.rs
    │       │   ├── slider.rs
    │       │   ├── tabs.rs
    │       │   ├── tooltip.rs
    │       │   ├── popover.rs
    │       │   ├── menu.rs
    │       │   ├── dialog.rs
    │       │   ├── drawer.rs
    │       │   ├── sheet.rs
    │       │   ├── toast.rs
    │       │   ├── alert.rs
    │       │   ├── avatar.rs
    │       │   ├── badge.rs
    │       │   ├── skeleton.rs
    │       │   ├── scroll_area.rs
    │       │   ├── table.rs
    │       │   ├── pagination.rs
    │       │   ├── breadcrumb.rs
    │       │   ├── accordion.rs
    │       │   ├── collapsible.rs
    │       │   ├── hover_card.rs
    │       │   ├── context_menu.rs
    │       │   ├── command.rs
    │       │   ├── navigation_menu.rs
    │       │   ├── progress.rs
    │       │   ├── toggle.rs
    │       │   ├── form.rs
    │       │   ├── aspect_ratio.rs
    │       │   ├── resizable.rs
    │       │   └── scrollbar.rs
    │       ├── composed/               # 10 opinionated patterns
    │       ├── hooks/                  # 7 reusable hooks
    │       └── theme/                  # Token access
    ├── honba-kao-app/                  # REFACTORED: Feature-sliced
    │   ├── Cargo.toml
    │   ├── style/
    │   │   ├── palette_256.scss        # 256 canonical tokens
    │   │   ├── variables.scss          # BFX HF semantic aliases
    │   │   ├── themes.scss             # 8 theme definitions
    │   │   ├── layout.scss             # Terminal grid
    │   │   └── components/             # Component-specific
    │   └── src/
    │       ├── app.rs                  # Root: providers + router + error boundary
    │       ├── main.rs                 # WASM entry
    │       ├── error_boundary.rs
    │       ├── features/               # 8 feature slices
    │       ├── shared/                 # Cross-feature kernel
    │       ├── widgets/                # 3 composite widgets
    │       ├── pages/                  # 8 thin pages
    │       ├── router/                 # Routes + guards + loaders
    │       └── providers/              # 4 context providers
    ├── honba-kao-client/               # Typed REST client
    ├── honba-kao-api-wasm/             # WASM compute
    └── honba-kao-test-utils/           # NEW: Testing helpers
```

---

## 6. Definition of Done & Success Criteria

### Design System
1. **256 Named Tokens**: All `--c-000` to `--c-255` defined with semantic names
2. **Dynamic Custom Themes**: JSON import/export, instant CSS injection, Canvas/SVG sync
3. **35+ Primitives**: Full Radix/Shadcn parity, all accessible (ARIA, focus, keyboard)
4. **Zero Inlined Styles**: All layouts/colors via SCSS variable hierarchy

### Architecture
5. **Feature-Sliced**: Clear `features/`/`shared/`/`widgets/`/`pages/` separation
6. **Server State**: `leptos-query` with caching, deduping, retries, optimistic updates
7. **Forms**: Schema validation, field-level errors, accessible messages
8. **Routing**: Loaders, guards, error boundaries, code splitting

### Quality
9. **Tests**: Component tests (ui-core + features) + Playwright e2e (critical paths)
10. **A11y**: WCAG AA compliant, focus management, keyboard nav, ARIA
11. **CI Enforcement**: `check-ui-core-usage.sh`, `check-widget-integration.sh`, `check-a11y.sh`
12. **Performance**: <100ms TTI, 60fps ticks, virtualized tables, bundle splitting

### Trading Terminal
13. **Workbench Parity**: Full openalgo-charts widget tier (502 indicators, drawing rail, objects, settings, snapshot)
14. **Real-time**: 100+ ticks/sec via WebSocket, zero-frame-drop
15. **Audit**: Every rejected order/decision shows exact constraint breached

---

## 7. Immediate Next Steps (This Week)

| Task | Owner | Status |
|------|-------|--------|
| Create `honba-kao-ui-core` crate | — | 🔴 Not started |
| Implement `Button`, `Input`, `Select`, `Dialog`, `Toast` | — | 🔴 Not started |
| Add `check-ui-core-usage.sh` to CI | — | 🔴 Not started |
| Migrate `filter_bar.rs` → `HfFilterBar` | — | 🔴 Not started |
| Migrate `header_bar.rs` → `HfHeaderBar` | — | 🔴 Not started |
| Add `leptos-query` to workspace | — | 🔴 Not started |
| Refactor `WorkbenchStore` to query/mutation | — | 🔴 Not started |

---

**This plan supersedes the previous phased milestones (M0-M6).** The new architecture addresses all critical gaps while preserving the original vision: a Bitfinex-grade trading terminal with 256-color theming, native WASM compute, and professional Reactivity.