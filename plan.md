# Master Plan: 本場・顔 honba-kao

> **The face of the real thing.**  
> High-performance Leptos (0.7+ CSR) web trading terminal for Indian equities (`honba-ya` backend, `honba-kokoro` types, `honba-me` indicators), modeled on the professional UI architecture, styling language, and component design of [Bitfinex Honey Framework Core (`bfx-hf-ui-core`)](https://github.com/bitfinexcom/bfx-hf-ui-core), powered by an extensible **256-color named design token palette** for limitless custom theming.

---

## 1. Architectural Vision & Principles

`honba-kao` is a single-page WebAssembly application (CSR) built with **Leptos 0.7**, compiled via **Trunk**, communicating with `honba-ya` over typed HTTP REST and WebSocket streaming.

### Key Tenets
1. **Trading-Terminal First**: Dense, high-contrast, information-rich layouts designed for intraday algo-trading, position monitoring, and decision audits.
2. **Visual Fidelity to `bfx-hf-ui-core`**: Strict adherence to the Bitfinex Honey Framework design system—including dark/light palettes, variable-based shades, 5px corner radii, elevation shading, and tabular numeric typography.
3. **256-Color Named Palette System**: A rigorous, extensible 256-color named palette architecture (`--c-000` to `--c-255`) acting as canonical design tokens for current Bitfinex themes and any future custom themes (e.g., Monokai, Nord, Solarized, Bloomberg Amber, Cyberpunk Neon).
4. **Pure Rust Reactivity**: Signals, memos, and resources in Leptos 0.7 without virtual DOM overhead, delivering 60fps tick updates and zero-jank UI reflows.
5. **Native WASM Compute**: Seamless integration with `honba-kao-api-wasm` to run screener filters, OHLC aggregations, and technical indicators locally in the browser thread.

---

## 2. Design System & 256-Color Named Palette Engine

The styling infrastructure is divided into three distinct layers:
1. **Canonical Palette Layer (Tokens `000` to `255`)**: 256 named, calibrated color tokens.
2. **Semantic Theme Alias Layer**: Maps Honey Framework variables (`--background-color`, `--primary-*`, `--shade-*`) to canonical palette tokens.
3. **Runtime Custom Theme Engine**: Allows dynamic JSON-based theme loading, runtime CSS injection, and Canvas/SVG chart color synchronization in Rust.

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
│  HfPanel, HfButton, HfPLNumber, PositionsTable, LivePnlSparkline       │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.1 The 256-Color Canonical Named Token Architecture

Every token in the palette has an index (`--c-000` .. `--c-255`) and an unambiguous, descriptive semantic name (`--c-<name>`):

#### 1. System & Terminal Core (Tokens `000`–`015` — 16 Colors)
Standard terminal color primitives adapted for high-contrast trading terminals:
- `000` (`--c-term-black`): `#0a1017`
- `001` (`--c-term-red`): `#d9383e`
- `002` (`--c-term-green`): `#00b88d`
- `003` (`--c-term-yellow`): `#d9b300`
- `004` (`--c-term-blue`): `#0073d9`
- `005` (`--c-term-magenta`): `#b340b3`
- `006` (`--c-term-cyan`): `#00a3a3`
- `007` (`--c-term-white`): `#d0d7de`
- `008` (`--c-term-bright-black` / gray): `#4f5a62`
- `009` (`--c-term-bright-red`): `#f05359`
- `010` (`--c-term-bright-green`): `#00e0ad`
- `011` (`--c-term-bright-yellow`): `#fcd206`
- `012` (`--c-term-bright-blue`): `#0089ff`
- `013` (`--c-term-bright-magenta`): `#e05ce0`
- `014` (`--c-term-bright-cyan`): `#00d9d9`
- `015` (`--c-term-bright-white`): `#ffffff`

#### 2. Chromatic Spectrum Ramps (Tokens `016`–`231` — 216 Colors = 18 Hues × 12 Luminance Steps)
Each hue contains 12 perceptual luminance steps (Step `01` = deepest shade, Step `06` = pure hue, Step `12` = pastel tint/highlight):
1. **`slate-01` .. `slate-12`** (`016`–`027`): Cool Slate neutrals, card borders, passive text.
2. **`marine-01` .. `marine-12`** (`028`–`039`): Bitfinex dark terminal navy surfaces (`#102331`, `#172d3e`, `#1e384d`).
3. **`azure-01` .. `azure-12`** (`040`–`051`): Vibrant electric blue accents, focus rings, link text.
4. **`cyan-01` .. `cyan-12`** (`052`–`063`): Aqua ticker highlights, code indicators, order book depth.
5. **`teal-01` .. `teal-12`** (`064`–`075`): Bitfinex signature mint & execution indicators (`#00e0ad` ramp).
6. **`emerald-01` .. `emerald-12`** (`076`–`087`): Positive PnL gains, buy wall markers, confirmed fills.
7. **`lime-01` .. `lime-12`** (`088`–`099`): High-intensity neon green for burst volume anomalies.
8. **`chartreuse-01` .. `chartreuse-12`** (`100`–`111`): Oscillators, RSI oversold/overbought markers.
9. **`amber-01` .. `amber-12`** (`112`–`123`): Warnings, risk throttle limits, connecting statuses.
10. **`gold-01` .. `gold-12`** (`124`–`135`): Precious metals, benchmark index comparisons, premium badges.
11. **`orange-01` .. `orange-12`** (`136`–`147`): Pending in-flight orders, margin utilization warnings.
12. **`coral-01` .. `coral-12`** (`148`–`159`): Bitfinex coral red sell scale (`#f05359` ramp).
13. **`crimson-01` .. `crimson-12`** (`160`–`171`): Negative PnL losses, stop loss triggers, kill-switch active.
14. **`rose-01` .. `rose-12`** (`172`–`183`): Subtle drawdown shading, bid/ask spread highlight.
15. **`magenta-01` .. `magenta-12`** (`184`–`195`): Algo state markers, strategy execution signals.
16. **`purple-01` .. `purple-12`** (`196`–`207`): Historical backtest equity curves, simulation benchmark.
17. **`violet-01` .. `violet-12`** (`208`–`219`): Bracket and OCO order boundary indicators.
18. **`indigo-01` .. `indigo-22`** (`220`–`231`): Liquidity heatmaps and cumulative volume profiling.

#### 3. Monochrome Luminance Ramp (Tokens `232`–`255` — 24 Steps)
A calibrated 24-step grayscale ramp from absolute terminal void (`mono-00`) to pure white (`mono-23`):
- `232` (`--c-mono-00`): `#080c10` (deep void / canvas base)
- `233` (`--c-mono-01`): `#0f151c`
- `234` (`--c-mono-02`): `#161e27`
- `...`
- `244` (`--c-mono-12`): `#7a8894` (mid-gray border / inactive icon)
- `...`
- `254` (`--c-mono-22`): `#f0f3f6`
- `255` (`--c-mono-23`): `#ffffff` (pure white text highlight)

---

### 2.2 Semantic Theme Alias Mapping

Component stylesheets reference semantic aliases that resolve to the 256-color palette. Switching themes merely remaps these aliases:

| Semantic Alias | Default Dark (BFX Honey) | Default Light Mode | High-Contrast / Custom Theme |
| :--- | :--- | :--- | :--- |
| `--background-color` | `var(--c-marine-02)` (`#172d3e`) | `var(--c-mono-21)` (`#efefef`) | Custom base background |
| `--darker-background-color` | `var(--c-marine-01)` (`#102331`) | `var(--c-mono-23)` (`#ffffff`) | Custom card/panel surface |
| `--lighter-background-color`| `var(--c-marine-03)` (`#1c364a`) | `var(--c-mono-20)` (`#e5e8ec`) | Elevated hover containers |
| `--grayish-color` | `var(--c-slate-04)` (`#4f5a62`) | `var(--c-slate-02)` (`#d0d7de`) | Structural dividers & borders |
| `--primary-0` (tint) | `var(--c-teal-02)` | `var(--c-teal-01)` | Row hover tint / active pill |
| `--primary-8` (text) | `var(--c-teal-10)` | `var(--c-teal-08)` | Highlighted green text |
| `--primary-9` (pure) | `var(--c-teal-07)` (`#00e0ad`) | `var(--c-teal-07)` (`#00e0ad`) | Primary brand accent |
| `--secondary-0` .. `--9` | `var(--c-azure-02)` .. `10` | `var(--c-azure-01)` .. `08` | Blue interactive accents |
| `--shade-0` .. `--shade-9` | `var(--c-marine-03)` .. `mono-22` | `var(--c-mono-20)` .. `mono-03` | Depth elevation & text scale |
| `--pnl-gain` / `hfui-green` | `var(--c-teal-08)` | `var(--c-emerald-07)` | Positive profit/gain |
| `--pnl-loss` / `hfui-red` | `var(--c-coral-08)` | `var(--c-crimson-07)` | Negative loss |
| `--pnl-neutral` | `var(--c-mono-14)` | `var(--c-mono-10)` | Break-even PnL |
| `--order-buy` | `var(--c-emerald-07)` | `var(--c-emerald-07)` | Buy side indicator |
| `--order-sell` | `var(--c-crimson-07)` | `var(--c-crimson-07)` | Sell side indicator |
| `--status-warning` | `var(--c-amber-08)` (`#fcd206`)| `var(--c-amber-08)` | Connecting / Throttling |
| `--text-color` | `var(--c-mono-18)` | `var(--c-mono-04)` | Body copy |

---

### 2.3 Physical Styling Properties (Corners, Shading & Elevation)

- **Corners & Border Radii**:
  - Global container radius: `$border-radius: 5px;` applied to all panels, buttons, cards, dropdowns, and modal dialogs.
  - Counter & Status Badges: Pill shape with `border-radius: 100px;`.
  - Status Indicators & Side Stick Markers: Circular (`border-radius: 50%;`) or vertical stick (`width: 2px; height: 12px; border-radius: 0;`).
- **Shading & Elevation**:
  - Panel Elevation: Deep visual contrast between workspace base (`--background-color`) and inner body (`--darker-background-color`).
  - Elevated Popovers & Modals: `box-shadow: 5px -4px 15px 0px rgba(0, 0, 0, 0.45);`.
  - Row Hover: Subtle highlighting via `var(--primary-0)` background with `var(--primary-8)` text transition.
- **Typography & Formatting**:
  - Font family: `"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`.
  - Tabular Numbers: `font-feature-settings: "tnum"` enabled across all tables, prices, quantities, and PnL fields.
  - Trailing decimals: `opacity: 0.6;` styling for trailing fractional zeros.
- **Custom Scrollbars**:
  - Track: transparent.
  - Thumb: `var(--shade-1)`, hovering to `var(--shade-3)` with smooth transitions.

---

### 2.4 Extensible Custom Theme Engine Architecture

To support any custom theme (Monokai, Solarized, Nord, Bloomberg Terminal, Cyberpunk Neon, OLED Black):

1. **Rust Palette Representation (`honba-kao-app/src/theme/palette.rs`)**:
   ```rust
   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub struct Palette256 {
       pub colors: [String; 256], // 256 Hex / OKLCH strings
   }

   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub struct CustomThemeDefinition {
       pub id: String,
       pub name: String,
       pub is_dark: bool,
       pub palette: Palette256,
       pub semantic_overrides: HashMap<String, String>,
   }
   ```
2. **Dynamic In-Browser CSS Injection**:
   - Leptos reactive effect injects or updates a `<style id="honba-custom-theme">` tag on the fly.
   - Allows users to drag-and-drop or paste a `.json` theme file and preview it instantly with zero page reloads.
3. **Canvas & SVG Chart Color Synchronization**:
   - Canvas/SVG sparklines (`LivePnlSparkline`) retrieve RGB values directly from the active `Palette256` struct in Rust memory, guaranteeing exact color match between CSS panels and rendered charts.
4. **Theme Studio & Palette Inspector (`ThemeStudioModal`)**:
   - An in-terminal palette inspector displaying all 256 color swatches, WCAG contrast verification against `--background-color`, and interactive color picker for tweaking and exporting custom themes.

---

## 3. Leptos Component Matrix & Design Specifications

Every component is modeled on its corresponding pattern in `bfx-hf-ui-core/src/ui` and `src/components`:

```
bfx-hf-ui-core Pattern           honba-kao Leptos Component        Module Path
────────────────────────────────────────────────────────────────────────────────────────────
ui/Panel                        HfPanel, HfPanelHeader            crates/honba-kao-app/src/ui/panel.rs
ui/Button                       HfButton, HfIconButton            crates/honba-kao-app/src/ui/button.rs
ui/PLNumber                     HfPLNumber                        crates/honba-kao-app/src/ui/pl_number.rs
ui/Dropdown, SimpleDropdown     HfDropdown, HfSelect              crates/honba-kao-app/src/ui/dropdown.rs
ui/Modal                        HfModal                           crates/honba-kao-app/src/ui/modal.rs
ui/AttentionBar                 HfAttentionBar                    crates/honba-kao-app/src/ui/attention_bar.rs
ui/ProgressBar                  HfProgressBar                     crates/honba-kao-app/src/ui/progress_bar.rs
ui/Tabs                         HfTabs, HfTab                     crates/honba-kao-app/src/ui/tabs.rs
components/Navbar               Navbar                            crates/honba-kao-app/src/components/navbar.rs
components/StatusBar            StatusBar                         crates/honba-kao-app/src/components/status_bar.rs
components/PositionsTable       PositionsTablePanel               crates/honba-kao-app/src/components/positions.rs
components/AtomicOrdersTable    OrdersTablePanel                  crates/honba-kao-app/src/components/orders.rs
components/TradesTablePanel     TradesTablePanel                  crates/honba-kao-app/src/components/trades.rs
components/TradingStatePanel    TradersOverviewPanel              crates/honba-kao-app/src/components/traders.rs
(New / Honba-Specific)          DecisionLogPanel                  crates/honba-kao-app/src/components/decision_log.rs
(New / Honba-Specific)          RiskLimitsModal                   crates/honba-kao-app/src/components/risk_modal.rs
(New / Honba-Specific)          LivePnlSparkline                  crates/honba-kao-app/src/components/sparkline.rs
(New / Custom Themes)           ThemeStudioModal                  crates/honba-kao-app/src/components/theme_studio.rs
```

### 3.1 Component Specifications

#### 1. `HfPanel` (`.hfui-panel`)
- **Structure**:
  - `.hfui-panel`: Flex container, `background: var(--darker-background-color)`, `border-radius: 5px`, `overflow: hidden`.
  - `.hfui-panel__header`: `background: var(--background-color)`, title label (`.hfui-panel__label`), badge counter (`.hfui-panel__counter`), optional tab switcher (`.hfui-panel__header-tabs`), header action buttons (`HfIconButton`).
  - `.hfui-panel__secondaryheader__wrapper`: Sub-bar for filters, search inputs, symbol selector, or timeframe dropdowns.
  - `.hfui-panel__content`: Flexible body with optional collapsible sidebar drawer (`.hfui_panel__sidebar-container`).
- **Features**: Collapsible, maximizable to full screen, tab switching with signal persistence, loading overlay spinner.

#### 2. `HfButton` (`.hfui-button`)
- **Variants**: `Default` (`--lighter-background-color`), `Green` (`$green-button-color`), `Red` (`$red-button-color`), `Blue` (`$blue-color`), `Ghost`.
- **States**: Hover opacity (`0.8`), Active compression effect, Disabled (`opacity: 0.5`, `cursor: not-allowed`).
- **Props**: `on_click`, `variant`, `disabled`, `loading`, `icon`, `children`.

#### 3. `HfPLNumber` (`.hfui-plnumber`)
- **Visuals**: Displays profit/loss formatted as Indian Rupees (`₹`) or percentage.
- **Directional Indicators**: Prefixed with `▲` (green `var(--pnl-gain)`) for positive/gain, `▼` (red `var(--pnl-loss)`) for loss.
- **Change Animation**: Brief flash on tick updates when value increases or decreases compared to previous signal value.

#### 4. `HfDropdown` / `HfSelect`
- **Design**: Themed select trigger matching `.hfui-select`, custom popup menu rendered in a portal with `var(--background-color)` surface, 5px radius, `--shade-0` item hover.

#### 5. `Navbar` (`.hfui-navbar__wrapper`)
- **Brand**: `本場・顔` logo with status badge.
- **Navigation Routes**:
  - `/traders` — Active strategy traders and supervisor control.
  - `/positions` — Real-time position ledger and live PnL.
  - `/orders` — Atomic orders, algo execution, and fill logs.
  - `/decisions` — Decision journal, risk rejection audit.
  - `/backtest` — Interactive backtest visualizer powered by `honba-kao-api-wasm`.
- **Controls**:
  - Trading Mode Toggle: `SANDBOX` (warning yellow pill) vs `LIVE NSE` (green pill).
  - Theme Selector: Preset selector (Dark BFX, Light BFX, Monokai, Nord, Custom) and Theme Studio opener.
  - Notifications Bell: Sliding side drawer for system alerts and fill notifications.
  - Session Controls: Auth status, token expiration indicator, disconnect button.

#### 6. `StatusBar` (`.hfui-statusbar__wrapper`)
- **Fixed Position**: Pinned to the bottom viewport edge.
- **Indicators**:
  - API HTTP Health: `.hfui-statusbar__statuscircle` (Green = Connected, Yellow = Connecting, Red = Offline).
  - WebSocket Stream: `.hfui-statusbar__statuscircle` with real-time latency readout (e.g., `WS Connected (14ms)`).
  - Market Session Clock: NSE market hours indicator (`Pre-open`, `Regular Trading`, `Closing`, `Closed`).
  - Active Theme: Badge showing current theme name.
  - Build Version: Release git commit and build timestamp.

#### 7. `PositionsTablePanel`
- **Columns**: Symbol, Exchange (`NSE`/`BSE`), Side (`LONG` / `SHORT` with `.row-marker`), Qty, Average Price, LTP / Mark Price, Unrealized PnL (`HfPLNumber`), Realized PnL, Risk Utilization Bar, Actions (`Square Off`, `Edit SL/TP`).
- **Interactivity**: Row click navigates to trader detail; hover state highlights with `var(--primary-0)`.

#### 8. `OrdersTablePanel`
- **Sections**: Tabbed between `Active Orders`, `Algo Orders`, and `Execution History`.
- **Features**: Side color dot (`.row-marker.green` / `.row-marker.red`), order type badge, cumulative fill percentage bar (`HfProgressBar`), instant cancel trigger.

#### 9. `DecisionLogPanel`
- **Auditing**: Live stream of trading decisions emitted by `honba-nou` and `honba-ya`.
- **Visual Filters**: Rejection badges showing the exact risk gate that halted execution (e.g., `CIRCUIT_FILTER`, `MAX_DRAWDOWN_BREACH`, `INSUFFICIENT_MARGIN`, `COOLDOWN_ACTIVE`).

#### 10. `LivePnlSparkline`
- **Implementation**: Lightweight inline SVG / Canvas micro-chart displaying real-time equity curves, color-tinted to green/red querying active `Palette256` tokens.

#### 11. `ThemeStudioModal`
- **Capabilities**: Visual inspector of all 256 colors (`--c-000` to `--c-255`), live alias mapper, JSON import/export, and real-time contrast ratio auditing.

---

## 4. State Management & Reactive Data Flow

```
                               ┌────────────────────────────────────────────────┐
                               │             App (Root Component)               │
                               │  - ThemeContext (256-Palette + CSS Injection)  │
                               │  - AuthContext (JWT + SessionStorage)          │
                               │  - ApiClientContext (Typed REST Client)        │
                               │  - WsContext (Reconnecting Event Stream)       │
                               └───────────────────────┬────────────────────────┘
                                                       │
                           ┌───────────────────────────┴───────────────────────────┐
                           ▼                                                       ▼
        ┌────────────────────────────────────┐                  ┌────────────────────────────────────┐
        │        Typed REST Client           │                  │     WebSocket Event Consumer       │
        │    (honba-kao-client crate)        │                  │      (honba-ya-ws::Event)          │
        │  - /health                         │                  │  - Tick / LTP Stream               │
        │  - /api/traders                    │                  │  - Order Fill Notifications        │
        │  - /api/positions                  │                  │  - Risk Limit Alert Events         │
        │  - /api/decisions                  │                  │  - Trader State Change Broadcasts  │
        └─────────────────┬──────────────────┘                  └──────────────────┬─────────────────┘
                          │                                                        │
                          └───────────────────────────┬────────────────────────────┘
                                                      ▼
                                   ┌──────────────────────────────────────┐
                                   │       Reactive View State Layer      │
                                   │  - Traders Signal / Memo             │
                                   │  - Positions & Realtime PnL Signal   │
                                   │  - Orders & Fill Progress Signal     │
                                   │  - Decision Log FIFO Buffer Signal   │
                                   └──────────────────┬───────────────────┘
                                                      ▼
                                   ┌──────────────────────────────────────┐
                                   │       WASM Computation Worker        │
                                   │      (honba-kao-api-wasm crate)      │
                                   │  - Local Screener Filtering          │
                                   │  - Technical Indicators (EMA, RSI)   │
                                   │  - Local Backtest Replay Engine      │
                                   └──────────────────────────────────────┘
```

---

## 5. Phased Implementation Milestones

### Phase M0: Foundations, 256-Color Palette & UI Primitives (Week 1)
- [ ] **M0.1 Pin Leptos 0.7 & Dependencies**: Upgrade/verify Leptos 0.7 CSR setup in `honba-kao-app`, `leptos_router`, `wasm-bindgen`, and `gloo-net`.
- [ ] **M0.2 256-Color Named Token SCSS & Engine**:
  - Implement `style/palette_256.scss` declaring canonical `--c-000` through `--c-255` and semantic names (`--c-marine-*`, `--c-teal-*`, `--c-coral-*`, etc.).
  - Port `variables.scss` from `bfx-hf-ui-core` into `honba-kao-app/style/variables.scss`, mapping BFX variables to canonical 256-tokens.
  - Implement `body.Dark` and `body.Light` default alias maps.
- [ ] **M0.3 Rust ThemeEngine & Palette256 Struct**:
  - Implement `Palette256` and `CustomThemeDefinition` in `honba-kao-app::theme`.
  - Build runtime CSS injector `<style id="honba-theme-custom">` for on-the-fly theme switching without reloading.
  - Persist chosen theme (preset ID or custom JSON) in `localStorage`.
- [ ] **M0.4 Base UI Primitives (`crates/honba-kao-app/src/ui/`)**:
  - `HfPanel` & `HfPanelHeader` with label, counter pill, action icons, and 5px radius styling.
  - `HfButton` supporting `Green`, `Red`, `Blue`, `Default`, and `Ghost` variants.
  - `HfPLNumber` supporting currency (`₹`), percentage, `▲`/`▼` glyphs, and dynamic palette coloring.
  - `HfDropdown` / `HfSelect` with themed options menu.
  - `HfModal` dialog with backdrop overlay.

### Phase M1: Shell, Navigation & System Status (Week 2)
- [ ] **M1.1 Global Layout Grid**: Flexible multi-panel layout matching Honey Framework terminal design.
- [ ] **M1.2 Top Navbar**:
  - Logo (`本場・顔`), navigation route links, mode toggle (`Sandbox` / `Live NSE`).
  - Layout switch dropdown, notification drawer trigger button, theme dropdown with quick switcher.
- [ ] **M1.3 Fixed Bottom StatusBar**:
  - API HTTP connection indicator (pinging `/health` with green/yellow/red pill).
  - WebSocket connection indicator and latency timer.
  - NSE market phase indicator (Pre-open, Regular, Closed).
  - Active theme badge and build version.
- [ ] **M1.4 Notification Drawer**: Flyout sidebar displaying actionable notifications and toasts.

### Phase M2: Trader Views & Supervisory Controls (Weeks 3–4)
- [ ] **M2.1 Typed Client Expansion (`honba-kao-client`)**:
  - Endpoints for `GET /api/traders`, `POST /api/traders/{id}/start`, `POST /api/traders/{id}/pause`, `POST /api/traders/{id}/stop`.
  - Models for trader profiles, strategy configs, and run statuses.
- [ ] **M2.2 Traders Overview Panel**:
  - Card/grid view of active strategy instances (trader name, universe, state badge, net PnL).
  - Instant action buttons: Pause, Resume, Stop, and Emergency Square-Off.
- [ ] **M2.3 Trader Detail Panel**:
  - Strategy parameters breakdown, target universe list, and active position holdings.
  - Inline PnL sparkline (`LivePnlSparkline`) tracking session equity curve with palette-synced Canvas/SVG.
- [ ] **M2.4 Risk Limits Inspector & Editor**:
  - Modal dialog (`RiskLimitsModal`) to inspect and update max drawdown, max daily loss, and position limits against `honba-kokoro-risk`.
- [ ] **M2.5 Theme Studio & 256-Color Inspector Modal**:
  - `ThemeStudioModal` to preview all 256 colors, test contrast ratios, and export custom theme JSON.

### Phase M3: Live Execution, Positions & Orders (Weeks 5–6)
- [ ] **M3.1 WebSocket Event Consumer (`honba-ya-ws`)**:
  - Resilient WebSocket connection to `honba-ya-ws`.
  - Stream decoder mapping server events (`OrderPlaced`, `OrderFilled`, `OrderCancelled`, `PositionUpdated`, `TickUpdate`) to reactive signals.
- [ ] **M3.2 Positions Table Panel**:
  - Interactive table displaying open positions, symbol, exchange, side (with `.row-marker`), qty, LTP, unrealized PnL via `HfPLNumber`.
  - Quick "Square-Off" button invoking REST endpoint with confirmation modal.
- [ ] **M3.3 Atomic & Algo Orders Table Panel**:
  - Tabbed order viewer (Active, Algo/TWAP/Iceberg, History).
  - Progress fill indicator (`HfProgressBar`), status dots, cancellation action.
- [ ] **M3.4 Trades Execution History Panel**:
  - Timestamped execution list, slippage calculation, brokerage fee estimate.

### Phase M4: Decision Journal & Audit Log (Week 7)
- [ ] **M4.1 Decision Stream Ingestion**:
  - Fetch and stream decision audit records from `honba-ya-api-rest` (`/api/decisions`).
- [ ] **M4.2 Decision Log Panel**:
  - High-density table displaying timestamp, strategy ID, symbol, action proposed, and execution verdict (`APPROVED` / `REJECTED`).
  - Prominent rejection reason badges (e.g. `CIRCUIT_FILTER`, `MAX_DRAWDOWN`, `SPREAD_TOO_WIDE`).
- [ ] **M4.3 Decision Inspection Flyout**:
  - Deep-dive drawer showing raw JSON decision payload, market snapshot at decision time, and evaluated risk matrices.

### Phase M5: WASM Analytics, Screener & Local Backtesting (Week 8)
- [ ] **M5.1 In-Browser Screener Integration**:
  - Embed `honba-kao-api-wasm::screener` into a dedicated Screener Panel.
  - Multi-metric filtering (volume spikes, momentum, RSI, ATR) executed client-side without roundtrip latency.
- [ ] **M5.2 Local Backtest Replay Visualizer**:
  - Hook into `honba-kao-api-wasm::replay` to run historical bar replay in the browser.
  - Interactive scrubbing timeline with bar-by-bar order execution visualization.

### Phase M6: Authentication, Hardening & Static Deployment (Week 9)
- [ ] **M6.1 Auth & Session Management**:
  - JWT storage in `sessionStorage` with automatic header injection.
  - Token expiration warning and silent refresh or re-login modal.
- [ ] **M6.2 Responsive & Mobile Polish**:
  - Touch-friendly tap targets, collapsible panels, and responsive breakpoints for tablets/mobile monitors.
- [ ] **M6.3 Quality Assurance & Conformance**:
  - Unit tests via `wasm-bindgen-test` for client serialization and component state.
  - Theme contrast validation tests verifying WCAG AA/AAA compliance on all 256 tokens.
  - End-to-end testing with Playwright running against simulated `honba-ya` backend.
- [ ] **M6.4 Static Asset Packaging & Production Distribution**:
  - All builds explicitly resort to `dev` (fast iteration, opt-level = 1) or `prod` (distribution, opt-level = 3, lto = true, codegen-units = 1, panic = "abort") profiles instead of debug.
  - Trunk production build (`make build-prod` / `trunk build --release`) with wasm optimization.
  - Deployable Dockerfile and Nginx configuration with compression and cache headers.
  - Direct integration into `tower-http::services::ServeDir` for unified single-binary hosting.

---

## 6. Directory Structure Target

```
honba-kao/
├── Cargo.toml
├── Makefile
├── README.md
├── plan.md
├── index.html
├── Trunk.toml
└── crates/
    ├── honba-kao-api-wasm/              # High-speed in-browser WASM compute
    │   ├── Cargo.toml
    │   └── src/
    │       ├── indicators.rs            # Browser indicator calculations
    │       ├── replay.rs                # Local backtest replay
    │       └── screener.rs              # Client-side stock screener
    ├── honba-kao-client/                # Typed REST client
    │   ├── Cargo.toml
    │   └── src/
    │       ├── auth.rs                  # Session & token handlers
    │       ├── error.rs                 # Typed client errors
    │       ├── lib.rs                   # ApiClient root
    │       └── models.rs                # Wire DTOs shared with honba-ya
    └── honba-kao-app/                   # Main Leptos CSR Terminal App
        ├── Cargo.toml
        ├── style/
        │   ├── palette_256.scss         # 256 named canonical color tokens
        │   ├── variables.scss           # BFX HF color palette, shades & vars
        │   ├── theme.scss               # body.Dark, body.Light & custom alias maps
        │   ├── layout.scss              # Terminal grid, navbar & statusbar
        │   └── components/
        │       ├── panel.scss           # hfui-panel styling & sub-headers
        │       ├── button.scss          # hfui-button variants
        │       ├── table.scss           # Tabular formatting & row markers
        │       └── modal.scss           # Dialogs & backdrop overlays
        └── src/
            ├── lib.rs                   # SPA entrypoint & wasm_bindgen(start)
            ├── theme/                   # 256-Color Theme System
            │   ├── mod.rs               # ThemeEngine & ThemeContext
            │   ├── palette.rs           # Palette256 struct & color conversions
            │   ├── presets.rs           # Built-in presets (Dark, Light, Monokai, Nord)
            │   └── injector.rs          # Runtime CSS <style> injector
            ├── context/
            │   ├── auth.rs              # AuthContext
            │   └── ws.rs                # WebSocket event stream context
            ├── ui/                      # BFX HF UI Primitives
            │   ├── attention_bar.rs     # HfAttentionBar
            │   ├── button.rs            # HfButton & HfIconButton
            │   ├── dropdown.rs          # HfDropdown & HfSelect
            │   ├── modal.rs             # HfModal dialog
            │   ├── panel.rs             # HfPanel & HfPanelHeader
            │   ├── pl_number.rs         # HfPLNumber (colored with ▲/▼)
            │   ├── progress_bar.rs      # HfProgressBar
            │   └── tabs.rs              # HfTabs & HfTab
            └── components/              # Domain-Specific Terminal Panels
                ├── navbar.rs            # Top navigation bar & tools
                ├── status_bar.rs        # Bottom connectivity & clock bar
                ├── traders.rs           # Trader grid & supervisory control
                ├── positions.rs         # PositionsTablePanel
                ├── orders.rs            # OrdersTablePanel (Atomic & Algo)
                ├── trades.rs            # TradesTablePanel
                ├── decision_log.rs      # DecisionLogPanel with rejection badges
                ├── risk_modal.rs        # RiskLimitsModal editor
                ├── sparkline.rs         # LivePnlSparkline canvas/SVG
                └── theme_studio.rs      # ThemeStudioModal & 256-swatch inspector
```

---

## 7. Definition of Done & Success Criteria

1. **256 Named Tokens**: The application defines and names all 256 tokens (`--c-000` to `--c-255`), organized into 16 terminal basics, 18 chromatic 12-step ramps, and a 24-step monochrome scale.
2. **Dynamic Custom Theme Loading**: Custom themes can be imported as JSON or chosen from presets without recompiling, immediately updating all CSS variables and Canvas/SVG charts.
3. **Aesthetic Consistency**: The UI looks and feels like a native Bitfinex Honey Framework terminal, strictly respecting `--background-color`, `--darker-background-color`, `--shade-0..9`, and 5px rounded corners.
4. **Zero Inlined Styles**: All layouts and colors adhere to the SCSS variable hierarchy without ad-hoc hardcoded color codes.
5. **Reactive Real-Time Performance**: Handles bursts of 100+ tick updates/sec from WebSocket without stuttering or dropping frames.
6. **Resilient Connectivity**: Gracefully handles network loss with explicit visual cues in `StatusBar`, automatic exponential backoff reconnection, and state synchronization.
7. **Full Audit Traceability**: Any rejected order or decision visibly explains the exact constraint breached (e.g. risk limit, circuit band, margin shortfall) in `DecisionLogPanel`.
