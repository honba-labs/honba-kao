# Honba Simulator UI/UX Alignment Plan: `honba-kao` to `honba-fe`

## 1. Executive Summary & Visual Audit

A side-by-side visual and architectural inspection was conducted across the live Chrome instances:
- **`honba-kao` (Port 8080)**: Rust/Leptos CSR frontend (`http://localhost:8080/simulator`)
- **`honba-fe` (Port 5175)**: React 19 / TypeScript reference frontend (`http://localhost:5175/simulator`)

Screenshots were captured via Chrome DevTools Protocol (CDP) and stored in the artifact repository.

```
+----------------------------------------------------------------------------------------------------+
|                                    CURRENT STATE COMPARISON                                        |
+----------------------------------------------------+-----------------------------------------------+
| honba-fe (Target Architecture)                     | honba-kao (Current Implementation)            |
+----------------------------------------------------+-----------------------------------------------+
| • Edge-to-edge full-bleed financial workstation     | • Page-level scroll inside .console wrapper   |
| • Consolidated 56px Command Bar with 3 zones:      | • <HfPageHeader> with title/surplus text      |
|     - Left: Strategy, Symbol, Timeframe, Range     | • Primitive .sim-toolbar with basic buttons   |
|     - Center: Integrated Replay Cluster            | • Replay controls in separate .sim-replay bar |
|     - Right: Live P&L, Confluence, Drawer Toggle   | • No Parameters drawer in top bar             |
| • Collapsible right-hand Parameters Drawer (360px) | • Bottom dock has 8 fragmented text tabs      |
| • Overview Tab: 4 KPI cards + split grid with      | • Overview Tab: Simple ribbon + equity card;  |
|   Equity Curve and Monthly Heatmap side-by-side    |   Monthly Heatmap in separate tab             |
| • Rich tab navigation with Lucide icons & counters | • Raw text buttons with no count badges       |
+----------------------------------------------------+-----------------------------------------------+
```

*Note: Implementation in progress per design document phases P0-P9.*

---

## 2. Gap Analysis by UI Zone

### Zone 1: Top Replay Command Bar (`.sim-command-bar`)
In `honba-fe`, the command bar is a fixed 56px sticky bar divided into three functional zones:

1. **Left Zone (`.sim-header-left`)**:
   - **Strategy Selector**: Dropdown grouped into `Active Workspace (From Designer)`, `Global Library (Institutional & OpenAlgo)`, and `User Library (My Saved)`.
   - **Symbol Badge Button (`.sim-symbol-badge-btn`)**: Monospace ticker button with search icon, e.g. `TCS.NSE`, `(Change Symbol)` badge, and dropdown caret. Opens symbol search modal.
   - **Timeframe Selector (`.sim-timeframe-wrap`)**: Dropdown button with clock icon opening a categorized popover:
     - `MINUTES`: `1m`, `2m`, `3m`, `5m`, `10m`, `15m`, `30m`, `45m`
     - `HOURS`: `1h`, `2h`, `3h`, `4h`
     - `DAYS, WEEKS & MONTHS`: `1d`, `1w`, `1mo`
   - **Date Range Selector (`.sim-date-range-wrap`)**: Button with calendar icon, preset pill (`1Y`, `YTD`, `3M`, etc.), date span (`2024-01-01 → 2025-01-01`), and popover with quick preset chips and ISO date inputs.

2. **Center Zone (`.sim-replay-cluster`)**:
   - Integrated directly into the top bar (not floating below the chart):
     - **Reset (`RotateCcw`)**: Resets simulation engine and portfolio.
     - **Play/Pause (`Play` / `Pause`)**: Pulsing action button with state-aware icon.
     - **Step (`SkipForward`)**: Step forward 1 candle.
     - **Instant (`Zap`)**: Fast-forward simulation to completion.
     - **Speed Pills**: `1x`, `5x`, `10x`, `25x`, `50x`.
     - **Scrub Timeline**: Slider range + metadata (`Bar 250 / 250 (100%)`) + tick timestamp (`15:30:00 IST`).

3. **Right Zone (`.sim-header-right`)**:
   - **Live P&L Badge (`.sim-live-pnl-chip`)**: Realized + Unrealized PnL in currency formatting with dynamic gain/loss coloring: `+₹45,230.55 (+45.23%)`.
   - **Confluence Pill (`.sim-confluence-pill`)**: `▲ 2 / ▼ 0` showing active bullish/bearish indicators.
   - **Parameters Drawer Toggle**: `Sliders` icon button toggling the right sidebar.
   - **Strategy Studio Link**: `ExternalLink` icon button.

*Current `honba-kao` state*: `honba-kao` has `<HfPageHeader>` and a primitive `.sim-toolbar` with just text and basic buttons. The replay controls are located in a separate `.sim-replay` container below the chart.

---

### Zone 2: Main Workspace & Collapsible Parameters Drawer
`honba-fe` uses a two-column workspace body:
- **Primary Chart Arena (`.sim-chart-arena`)**:
  - Contains `SimulatorChart` with full width/height responsive scaling.
- **Collapsible Parameters Drawer (`.sim-params-drawer`)**:
  - Right sidebar (360px wide) toggled by the "Parameters" button.
  - Header: Sliders icon, "Simulation Parameters" title, and close (X) button.
  - **Account Selector**: Select between sandbox accounts (e.g. `acc-sandbox-primary`).
  - **Execution Parameters**:
    - Initial capital (numeric input with rupee symbol)
    - Fill Model (Next Open, Close, Bid/Ask)
    - Commission & Slippage inputs
    - Benchmark index selector (NIFTY 50, SENSEX, etc.)
  - **Primary Run Action Button**: "Run Backtest" CTA with loading spinners and status indicators.

*Current `honba-kao` state*: `honba-kao` has no parameters drawer. Parameters cannot be edited in the UI without navigating elsewhere.

---

### Zone 3: Institutional Bottom Dock (`.sim-bottom-dock`)
`honba-fe` organizes the bottom area into 6 unified tabs with icon headers, dynamic count badges, and contextual dock action buttons:

1. **Overview & Performance**:
   - **4 Institutional KPI Cards Grid**:
     1. *Net Simulated P&L*: Big currency figure, % total return.
     2. *Profit Factor & Win Rate*: `1.87 (64.3%)`, `9 wins / 5 losses`.
     3. *Max Drawdown*: `-6.21%`, current equity `₹145,230`.
     4. *Sharpe / Risk-Adjusted Alpha*: `1.87`, annualized metric.
   - **Side-by-Side Split Grid (`.sim-split-grid`)**:
     - *Left*: Cumulative Strategy Equity Curve (smooth SVG area curve with gradient, benchmark line, and peak line).
     - *Right*: Monthly Return Heatmap (Year rows, Month columns, color-coded cells).
2. **Trade Journal (`trips.length`)**:
   - Data table with Status pills (`CLOSED`, `OPEN`), Side pills (`LONG`, `SHORT`), Instrument, Entry Time/Price, Exit Time/Price, Realized PnL, Return %, Bars Held.
   - Header action: **Export CSV** button.
3. **Sandboxed Portfolio (`positions.length`)**:
   - Portfolio overview banner, Open Positions table, Closed Positions table.
   - Header action: **Reset Sandbox P&L** button.
4. **Signal Confluence (`confluence.length`)**:
   - Active signals table with rule names, trigger status pills (`TRIGGERED`, `INACTIVE`), indication pills (`BULLISH`, `BEARISH`), weights, and observed values.
5. **Execution Logs (`logs.length`)**:
   - Monospace log terminal with timestamped events and search filtering.
6. **Server Backtest Report**:
   - Consolidated validation badges, data confidence, assumption checklist ("not modelled: market impact, wire latency..."), and raw request/response JSON viewer.

*Current `honba-kao` state*: `honba-kao` has 8 flat text tabs (`Overview`, `Journal`, `Signals`, `Portfolio`, `Heatmap`, `Validation`, `Logs`, `Payload`). `Heatmap` is separated from `Overview`, while `Validation` and `Payload` are split instead of merged into a backtest report.

---

## 3. Step-by-Step Implementation Roadmap

### Phase 1: CSS & Design System Alignment
- Port `honba-fe/apps/web/src/features/simulator/SimulatorView.css` classes into `honba-kao/style/simulator.css`.
- Ensure CSS variables map cleanly to `honba-kao`'s theme tokens:
  - `--bg-page`: `#0f1117`
  - `--bg-card`: `#161922`
  - `--bg-surface`: `#1e222d`
  - `--border`: `#2a2e39`
  - `--accent-primary`: `#2962ff`
  - `--color-bullish`: `#22c55e`
  - `--color-bearish`: `#ef4444`
  - Font families: `Inter` for UI, `IBM Plex Mono` for tabular figures.

### Phase 2: Page Shell & Layout Modernization
- In `honba-kao/crates/honba-kao-app/src/pages/simulator_page.rs`:
  - Replace `<div class="console">` and `<HfPageHeader>` with `<div class="simulator-view-root">`.
  - Configure root to be `height: 100%`, `display: flex`, `flex-direction: column`, `overflow: hidden`.

### Phase 3: Consolidated Replay Command Bar
- Consolidate `.sim-toolbar` and `ReplayControls` into a single `<header class="sim-command-bar">`:
  - **Left**: Add Strategy selector `<select>`, Symbol badge button `<button class="sim-symbol-badge-btn">`, Timeframe popover selector `<div class="sim-timeframe-wrap">`, and Date Range popover `<div class="sim-date-range-wrap">`.
  - **Center**: Move the transport buttons (`RotateCcw`, `Play`/`Pause`, `SkipForward`, `Zap`, speed pills `1x-50x`, and scrub slider) from `replay_controls.rs` directly into `.sim-replay-cluster`.
  - **Right**: Add live P&L chip (`.sim-live-pnl-chip`), confluence pill (`.sim-confluence-pill`), and "Parameters" toggle button (`Sliders` icon).

### Phase 4: Collapsible Parameters Drawer
- Implement `<aside class="sim-params-drawer">` in `simulator_page.rs` controlled by `is_params_open: RwSignal<bool>`.
- Wire up inputs for:
  - Account selection (Sandbox primary)
  - Initial capital
  - Fill model (Next Open, Close, Bid/Ask)
  - Slippage and commission
  - Benchmark index selection
  - "Run Backtest" action button triggering `store.execute(...)`.

### Phase 5: Bottom Dock Tab Consolidation & KPI Cards
- Consolidate `SimTab` enum into 6 tabs:
  1. `Overview` (with sub-grid: 4 KPI Cards + Equity Card + Monthly Heatmap)
  2. `Trades` (Trade Journal with Export CSV)
  3. `Portfolio` (Sandboxed Portfolio with Reset Sandbox P&L)
  4. `Signals` (Signal Confluence)
  5. `Logs` (Execution Logs)
  6. `Report` (Server Backtest Report: Badges + Assumptions + Raw Payload)
- Add procedural SVG icons from `honba-kao-ui-core::icons::outline` to the tab buttons.
- Display dynamic item count badges on tabs (e.g. `Trade Journal ({count})`).

### Phase 6: Chart Arena Refinement & Integration
- Anchor `<SimulatorChart>` inside `<div class="sim-chart-arena">`.
- Verify responsive layout when the right parameters drawer opens and closes.
- Ensure candlestick, volume, and trade marker snapping match `honba-fe`.

---

## 4. Verification Checklist

1. [ ] Command Bar renders at 56px with 3 balanced zones (left selectors, center replay, right telemetry).
2. [ ] Parameters drawer smoothly opens and closes via the top-right button.
3. [ ] Replay controls operate seamlessly in the command bar (play, pause, scrub, speed change).
4. [ ] Overview tab shows the 4 KPI cards alongside the equity curve and monthly heatmap.
5. [ ] Trade Journal and Portfolio tabs display item counts and toolbar action buttons (Export CSV, Reset Sandbox).
6. [ ] Full visual parity achieved between `http://localhost:8080/simulator` and `http://localhost:5175/simulator`.
