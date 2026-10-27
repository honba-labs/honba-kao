# Plan 1: OpenTerminalUI Mission Control Landing Page Port (`/home`)

> **Objective**: Replace the current legacy post-login routing (which redirects to `/portfolio` broker account aggregation) with the full-fidelity **OpenTerminalUI Mission Control Dashboard (`/home`)**, faithfully porting its layout, UI components, keyboard navigation, and live telemetry into Leptos 0.7 CSR.

---

## 1. Context & Motivation

In OpenTerminalUI (`frontend/src/pages/HomePage.tsx`), authenticated users immediately land on the **Mission Control Dashboard** (`/home`). It acts as an institutional cockpit providing immediate situational awareness:
1. **Header Ribbon & Clocks**: Multi-market live clocks (IST, EST, UTC), Desk / Market / Preset / Currency pills, and quick action buttons (`Portfolio HQ`, `Launchpad`, `Intel Wire`).
2. **Market Heat Strip**: Real-time index and commodity price strip (`NIFTY 50`, `SENSEX`, `BANKNIFTY`, `NASDAQ`, `S&P 500`, `GOLD`, `CRUDE OIL`) with flash-up/down animations and selection state.
3. **AI Market Outlook (`AiInsightCard`)**: Macro regime and collection briefing.
4. **Ideas Radar**: Live algorithmic trade signals (volume shockers, 52-week breakouts, momentum).
5. **Portfolio HQ & Telemetry Row**:
   - `MetricCard` cluster: Net Liquidation, Net P&L (color coded), Holdings count, Watchlists count, F&O PCR & posture signal.
   - `PortfolioMiniChart`: Portfolio equity curve vs benchmark (Nifty 50) overlay.
   - `SystemHealthBar`: Multi-pill feed health, WebSocket status, active session, and latency.
   - `Intel Wire`: Streaming headlines with sentiment tags (`Bullish`, `Bearish`, `Neutral`).
6. **Dashboard Intelligence**:
   - `ResultsSummaryCards`: Quarterly corporate earnings scorecards.
   - `ExposureHeatmap`: Sector / asset concentration heatmap.
   - `IntelligenceTimeline`: Corporate events, filing summaries.
7. **Launch Matrix (`QuickNavGrid`)**: 4-column keyboard-navigable grid of workspace shortcuts with function key badges (`[M1] Equity`, `[FO] F&O`, `[CR] Crypto`, `[SH] Security Hub`, `[F2] Screener`, `[F9] Backtest`, `[WS] Workstation`, `[LP] Launchpad`).

Currently, `honba-kao` lacks `HomePage` and routes authenticated users to `/portfolio` (a bare broker OAuth connection list). This plan specifies the full architecture and implementation tasks to achieve complete OpenTerminalUI parity.

---

## 2. Architecture & File Structure

```text
honba-kao/
├── crates/
│   ├── honba-kao-ui-core/src/
│   │   ├── home/
│   │   │   ├── mod.rs                  # Export all home primitives
│   │   │   ├── clock_strip.rs          # LiveClockStrip (IST, EST, UTC with live tick)
│   │   │   ├── market_heat_strip.rs    # MarketHeatStrip with flash states & click selection
│   │   │   ├── metric_card.rs          # MetricCard with delta pill, tone & sparkline
│   │   │   ├── sparkline_cell.rs       # SparklineCell SVG chart with gradient & tooltip
│   │   │   ├── system_health_bar.rs    # SystemHealthBar with dot indicators & tone
│   │   │   └── quick_nav_grid.rs       # QuickNavGrid with 4-column roving tabindex
│   │   └── lib.rs                      # Expose pub mod home;
│   │
│   └── honba-kao-app/src/
│       ├── features/
│       │   └── home/
│       │       ├── mod.rs              # Re-export HomePage
│       │       ├── home_page.rs        # Mission Control Dashboard container
│       │       └── components/
│       │           ├── mod.rs
│       │           ├── ai_insight_card.rs      # AI Market Outlook panel
│       │           ├── ideas_radar.rs          # Algorithmic ideas radar
│       │           ├── intel_wire.rs           # Intel news wire with sentiment chips
│       │           ├── portfolio_mini_chart.rs # Equity curve vs benchmark mini chart
│       │           └── results_summary.rs      # Earnings scorecards & timeline
│       │
│       └── router/
│           ├── paths.rs                # ROUTES::HOME = "/home"
│           ├── routes.rs               # Gated route /home + RootRoute redirect
│           └── chrome/
│               └── nav_rail.rs         # HM (Home) as top rail destination
│
└── style/
    └── home.css                        # .ot-home-widget-* styles ported from OpenTerminalUI
```

---

## 3. Detailed Work Breakdown & Milestones

### Milestone 1: Core Home UI Primitives (`honba-kao-ui-core`)
- [x] **1.1 `LiveClockStrip`**:
  - Model after `LiveClockStrip.tsx`.
  - Configurable zones (default: `IST` [Asia/Kolkata], `EST` [America/New_York], `UTC`).
  - Interval timer updating time signal every 1000ms.
  - Accessible semantics: `role="list"`, `role="listitem"`, `<time dateTime="...">`.
- [x] **1.2 `MarketHeatStrip`**:
  - Model after `MarketHeatStrip.tsx`.
  - Items list with `label`, `value`, `change_pct`, `change_label`, and `flash` ("up" | "down").
  - Click-to-focus callback with `data-selected` and `data-tone` (`up`, `down`, `neutral`).
- [x] **1.3 `MetricCard`**:
  - Model after `MetricCard.tsx`.
  - Properties: `label`, `value`, `delta` (`label`, `tone`), `tone` (`accent`, `up`, `down`, `neutral`), `details` (`label`, `value`, `tone`).
  - Embedded sparkline slot / SVG cell with tooltip.
- [x] **1.4 `SystemHealthBar`**:
  - Model after `SystemHealthBar.tsx`.
  - Status items with `tone` (`ok`, `warning`, `critical`, `offline`, `info`, `neutral`).
  - Pulsing status dots with accessible text.
- [x] **1.5 `QuickNavGrid`**:
  - Model after `QuickNavGrid.tsx`.
  - 4-column responsive grid of function-key navigation tiles.
  - Keyboard navigation: Arrow Left/Right/Up/Down, Home, End, Enter/Space activation, roving `tabindex`.

### Milestone 2: Composite Intelligence Panels (`honba-kao-app/src/features/home/components`)
- [x] **2.1 `AiInsightCard`**:
  - Macro assessment card displaying regime classification and institutional flows summary.
- [x] **2.2 `IdeasRadar`**:
  - Visual cards for algorithmic signals (Volume Shockers, 52W Breakouts, Mean Reversion).
  - Badge tag indicating trigger criteria (e.g. `+3.8x vol`, `+4.2% momentum`).
- [x] **2.3 `PortfolioMiniChart`**:
  - Dual-line SVG mini chart plotting portfolio equity curve against benchmark (NIFTY 50).
  - Compact date labels and crosshair hover inspection.
- [x] **2.4 `IntelWire`**:
  - Top 5-10 breaking headlines with source, timestamp, and sentiment chips (`Bullish`, `Bearish`, `Neutral` with confidence %).
- [x] **2.5 `ResultsSummaryCards` & `IntelligenceTimeline`**:
  - Corporate quarterly earnings scorecards (QoQ / YoY revenue & PAT growth) and corporate filings feed.

### Milestone 3: Mission Control Dashboard Page (`HomePage`)
- [ ] **3.1 Header Ribbon**:
  - "MISSION CONTROL" kicker and title.
  - Metadata badges: Desk user name, Market pill (`IN` / `US`), Preset indicator (`General` / `Equities` / `F&O`), Currency (`INR`), and Refresh cadence.
  - Action buttons: `Portfolio HQ` (`/portfolio`), `Launchpad` (`/equity/launchpad`), `Intel Wire` (`/equity/news`), `Workstation` (`/equity/chart-workstation`).
  - Mount `LiveClockStrip` in top right.
- [ ] **3.2 Market Heat Strip Integration**:
  - Mount `MarketHeatStrip` beneath header ribbon with indices (`NIFTY 50`, `SENSEX`, `BANKNIFTY`, `NASDAQ`, `S&P 500`, `GOLD`, `CRUDE OIL`).
- [ ] **3.3 Grid Layout**:
  - 2-column responsive layout:
    - Left (1.85fr): Net Liquidation MetricCard + PortfolioMiniChart.
    - Right (1fr): SystemHealthBar + Intel Wire feed.
  - Bottom section: Launch Matrix `QuickNavGrid` organized by domain (Markets, Derivatives, Research, Labs, Portfolio, Intel, Workspace).

### Milestone 4: Routing, Navigation & Session Policy
- [ ] **4.1 Router Constants (`paths.rs`)**:
  - Define `pub const HOME: &str = "/home";`.
- [ ] **4.2 Route Table (`routes.rs`)**:
  - Add `<Route path=path!("/home") view=HomePage />` inside the authenticated `AppShell` parent route.
  - Update `RootRoute` (`/`):
    - Authenticated -> redirect to `ROUTES::HOME`.
    - Guest -> show public marketing `LandingPage`.
- [ ] **4.3 Auth Redirect (`login_page.rs`)**:
  - Update `LoginPage` on submit success: `navigate(ROUTES::HOME, Default::default())`.
- [ ] **4.4 Navigation Rail (`nav_rail.rs`)**:
  - Add `HM` (Home / Mission Control) as the top anchor item (`outline::home()` glyph) pointing to `ROUTES::HOME`.

### Milestone 5: Stylesheet & Theme Token Integration
- [ ] **5.1 Port CSS Tokens**:
  - Create `honba-kao/style/home.css` with `.ot-home-widget-*` selectors from OpenTerminalUI.
  - Bind colors to semantic theme variables (`--background-color`, `--card-background-color`, `--text-color`, `--border-color`, `--accent-color`, `--color-up`, `--color-down`).
  - Import `home.css` in `main.css`.

### Milestone 6: Verification & Quality Assurance
- [ ] **6.1 Parallel Tests in `tests/`**:
  - Test `LiveClockStrip` rendering and time formatting.
  - Test `MarketHeatStrip` item rendering and click events.
  - Test `QuickNavGrid` roving focus behavior.
  - Test routing invariants: login transitions to `/home`, root redirects to `/home` for authenticated sessions.
- [ ] **6.2 CI Verification**:
  - Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --workspace`.
- [ ] **6.3 Inter-Agent Handoff**:
  - Update `.agents/HANDOFF.md` with milestone status and wire contracts.
