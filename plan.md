# Master Plan: 本場・顔 honba-kao

> **The face of the real thing.**
> High-performance, self-hosted Leptos (0.7+ CSR) web trading and research terminal for Indian equities and derivatives (`honba-ya` backend, `honba-core` types, `honba-mon` risk/OMS engine, `honba-me` indicators, `honba-nou` AI brain, and `honba-tamashii` MCP/catalog), modeled on the professional UI architecture, deep market workflows, and modular component design of **[OpenTerminalUI](https://github.com/Hitheshkaranth/OpenTerminalUI/)**, powered by a modern **unbounded semantic color engine (OKLCH / Display P3)** for dense terminal-noir, classic Bloomberg, midnight navy, and custom theming without legacy 256-color ANSI restrictions.

---

## 0. Executive Summary & Vision Revamp

### 0.1 The Shift: From Narrow Crypto UI to Bloomberg/OpenTerminalUI Grade
The original `honba-kao` blueprint took architectural inspiration from Bitfinex Honey Framework Core (`bfx-hf-ui-core`). While Honey Framework provided good initial patterns for simple trading panels, it suffered from severe functional limitations:
- **Narrow scope**: Restricted to basic crypto order tickets, generic tables, and simple panels.
- **Missing institutional workflows**: No equity fundamental research, no SEC/NSE Filings Intelligence (RAG with verbatim citations), no company research dossiers, no derivatives option chains or multi-leg strategy payoff curves, no quantitative model labs, and no operational risk cockpit.
- **Limited shell interactions**: Lacked a Bloomberg-style `Ctrl+G` GO bar / CommandBar, `Ctrl+K` command palette, `Ctrl+J` AI agent slide-over, panel symbol link groups (`A`, `B`, `C`, `D`), multi-pane chart workstations, or DOM depth ladders.

### 0.2 The Target: OpenTerminalUI Architecture Ported to Pure Rust / Leptos
We are revamping `honba-kao` to directly adopt the battle-tested pages, components, and workspace patterns from **OpenTerminalUI** (`frontend/src`):
1. **Institutional-Grade Shell (`Ctrl+G`, `Ctrl+K`, `Ctrl+J`)**:
   - `CommandBar` (Bloomberg-style GO bar): Ticker jumper, asset disambiguation, live sparkline preview, command hints (`TICKER DES`, `TICKER FA`, `TICKER FNO`, `TICKER GP`), and execution shortcuts.
   - `CommandPalette` (Ctrl+K): Global fuzzy search for navigation, chart actions, functions, and tools.
   - `AgentConsole` (Ctrl+J): Slide-over AI research drawer powered by `honba-nou` & `honba-tamashii` MCP, featuring multi-agent debate (analyst team → bull vs bear → PM decision), Strategy Lab, Screener membership, interactive action proposal cards with 1-click confirmation, verbatim quote citations, and research memory.
   - `IconRail` & `NavRail`: 8 primary domains (Markets, Research, Charts, Derivatives, Portfolio, Quant, Risk & Ops, System).
   - `Launchpad` & Multi-Pane Workspaces: Drag-and-drop panel grids, pop-out windows (`/equity/launchpad/popout`), saved views, and color-coded Symbol Link Groups (`A`–`D`) for synchronized crosshair and symbol dispatch.
2. **Comprehensive Market Coverage & Deep Equity Research**:
   - **Security Hub (`/equity/security`)**: 11-tab single-page equity deep dive (Overview DES snapshot, Financials transposed matrices, Chart, News & Sentiment, Ownership, Estimates, Peers & Value Chain, ESG, Tape, Insider, and Filings).
   - **Filings Intelligence & Company Research Dossiers**: Ingestion of NSE/BSE & SEC filings, 21 scored drivers (growth engines vs headwinds) with verbatim verified citations, "Ask the Filings" QA, management guidance tracker, and shareable frozen-data research dossiers.
   - **Tijori-Style Business Pack**: Operating KPIs over time, customer → company → supplier value chains with 10-K citations, and reverse DCF implied growth engine.
3. **Derivatives & F&O Suite (`/fno`)**:
   - Full Option Chain with real-time Greeks, spot price, Max Pain strike, ATM highlight, PCR (OI/Volume), and strike range filters.
   - Multi-leg Options Strategy Builder (Bull/Bear Spreads, Straddles, Strangles, Iron Condors, Butterflies, Calendars, Ratio Spreads) with interactive payoff curves at expiry and T+0, POP, and breakeven calculators.
   - Greeks Heatmap, Futures Term Structure (basis curve & rollover stats), OI Build-Up matrix, and Options Flow (unusual activity & block deals).
4. **Quantitative Research, Model Lab & Backtesting**:
   - Backtesting Workstation with 16+ strategy templates, realistic execution slippage, Monte Carlo bootstrap (1,000+ paths), Walk-Forward Optimization (WFO) timeline, 2D/3D parameter surfaces, 3D drawdown terrain, trade journals, and statistical robustness checks (DSR, PBO).
   - Model Lab (`/backtesting/model-lab`): Experiment registry, versioned run reports, model comparison, and leaderboards.
   - Model Governance: Model lifecycle state machines (draft → candidate → paper → live), approval gates, and risk constraints.
   - Portfolio Lab & Optimizer: Multi-model blending, mean-variance / Black-Litterman / risk-parity optimization, and rebalance schedules.
   - Factor Dashboard, Alpha Zoo (100+ alpha factors with IC/turnover), Statistical Lab, and Pair-Trading Lab.
5. **Pre-Trade Risk, OMS Compliance & Operations Cockpit**:
   - Direct integration with `honba-mon`: Pre-trade risk validation, position sizing gates, order throttle rate (OTR), circuit filter checks, restricted symbols list, and audit trail.
   - Ops Dashboard: Multi-desk tabs (Trader, Quant, PM, Risk, System), emergency Kill Switches (Panic Sell All, Cancel All Pending, Halt Trading Engine, Enable Safe Mode), feed health, and data quality scoring.
6. **Pure Rust Reactivity & Unbounded Semantic Design System**:
   - Native Leptos 0.7 CSR (Signals, Memos, Resources) compiling to WebAssembly.
   - Zero virtual DOM overhead; sub-millisecond tick updates and 60fps chart rendering.
   - Modern OKLCH & Display P3 semantic tokens supporting `terminal-noir`, `classic-bloomberg`, `midnight-navy`, `light-desk`, and dynamic runtime themes with continuous alpha channels (no legacy 256-color ANSI restrictions).

---

## 1. High-Level Architecture & Ecosystem Integration

`honba-kao` sits at the apex of the Honba trading platform, acting as the unified visual shell for all backend microservices, core calculation engines, and AI agents.

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   honba-kao (Browser CSR WASM)                                   │
│                                                                                                  │
│   ┌─────────────────────────────────── Terminal Shell ────────────────────────────────────────┐  │
│   │ TopBar (TickerTape · MarketStatus · TradingMode · LinkGroup · Notifications · Account)    │  │
│   │ CommandBar (Ctrl+G GO Bar)  │  CommandPalette (Ctrl+K)  │  AgentConsole (Ctrl+J Slide-Over) │  │
│   │ NavRail (Markets · Research · Charts · Derivatives · Portfolio · Quant · Risk · System)   │  │
│   └─────────────────────────────────────────────┬─────────────────────────────────────────────┘  │
│                                                 │                                                │
│   ┌──────────────────────────────────── Core Workspaces ──────────────────────────────────────┐  │
│   │  Markets Hub   │  Security Hub   │  F&O Terminal  │  Chart Workstation │  Quant/Backtest  │  │
│   │  Watchlists    │  11-Tab Deep    │  Option Chain  │  Multi-Pane (MTA)  │  Model Lab (Runs)│  │
│   │  Heatmap/News  │  Filings / RAG  │  Payoff Engine │  OpenScript IDE    │  Portfolio Lab   │  │
│   │  Macro/Forex   │  Value Chain    │  Greeks/Futures│  DOM Depth Ladder  │  Strategy Studio │  │
│   │  Commodities   │  Dossiers       │  OI Build-Up   │  Time & Sales      │  Simulator Rep.  │  │
│   └─────────────────────────────────────────────┬─────────────────────────────────────────────┘  │
│                                                 │                                                │
│   ┌───────────────────────────────────── Design System ───────────────────────────────────────┐  │
│   │ honba-kao-ui-core: Panel · DataTable · Button · Tabs · Badge (shadcn-parity primitives)    │  │
│   │ Semantic Color Engine (OKLCH/P3): terminal-noir · classic-bloomberg · midnight-navy · light │  │
│   └─────────────────────────────────────────────┬─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┼────────────────────────────────────────────────┘
                                                  │ Typed REST & WebSocket Streaming (JSON / CBOR)
                                                  ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                       Honba Platform Backend                                     │
│                                                                                                  │
│   ┌───────────────────────┐  ┌───────────────────────┐  ┌────────────────────────────────────┐   │
│   │       honba-ya        │  │       honba-mon       │  │             honba-nou              │   │
│   │ (Axum / Market Data / │  │  (Risk Engine / OTR / │  │ (AI Brain / Multi-Agent Debate /   │   │
│   │  Execution / Orders / │  │  Limits / Sizing Gate │  │  Research Loop / Genetic Engine /  │   │
│   │  Positions / Feeds)   │  │  Refusal / Audit Log) │  │  Tool Calling / Prompt Gateway)    │   │
│   └───────────┬───────────┘  └───────────┬───────────┘  └─────────────────┬──────────────────┘   │
│               │                          │                                │                      │
│   ┌───────────┴───────────┐  ┌───────────┴───────────┐  ┌─────────────────┴──────────────────┐   │
│   │      honba-core       │  │   honba-strategies    │  │           honba-tamashii           │   │
│   │ (Types, Sizing Models,│  │(Step Manifests, Alpha │  │(OpenAPI Schema, Catalog, Golden    │   │
│   │ Screener, Instruments)│  │ Catalog, Options Str.)│  │ Vectors, MCP Server Protocol)      │   │
│   └───────────────────────┘  └───────────────────────┘  └────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Terminal Shell & Global Interaction Model

Modeled directly on OpenTerminalUI (`src/components/layout/`), the terminal shell provides an uncompromising keyboard-first, high-density workflow.

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────┐
│ [TOPBAR] [TICKER TAPE: NIFTY 24,850 ▲0.4%  BANKNIFTY 52,100 ▲0.2%  RELIANCE 2,980 ▼0.3%]        │
│          [LIVE NSE ●] [PAPER TRADING ▾] [LINK: GROUP A ▾] [LAYOUT ▾] [🔔 3] [INITIALS AVATAR]   │
├─────────┬─────────────────────────────────────────────────────────────────────────────────────────┤
│ [RAIL]  │ [COMMAND BAR: Ctrl+G] 🔍 Search symbol, type 'TICKER DES', or '?' for AI ... [Ctrl+K]  │
│ ┌─────┐ ├─────────────────────────────────────────────────────────────────────────────────────────┤
│ │ HM  │ │                                                                                         │
│ │ MK  │ │                                                                                         │
│ │ SH  │ │                                   ACTIVE WORKSPACE                                      │
│ │ WS  │ │                       (Security Hub / Workstation / F&O / Lab)                          │
│ │ OC  │ │                                                                                         │
│ │ PF  │ │                                                                                         │
│ │ BT  │ │                                                                                         │
│ │ RK  │ │                                                                                         │
│ ├─────┤ ├─────────────────────────────────────────────────────────────────────────────────────────┤
│ │ ⚙️  │ │ [STATUS BAR] [FEED: OK 12ms] [OMS: NORMAL] [RISK: GREEN] [WSS: 142 ticks/s] [PROVENANCE: LIVE]
│ └─────┘ └─────────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Navigation Structure (The 8 Primary Domains)
Derived from OpenTerminalUI's `navConfig.ts`:

| Section | Label | Glyphs & Hotkeys | Key Destinations |
|---|---|---|---|
| **Markets** | Markets | `HM` (Home), `MK` (Market), `MO` (Monitoring), `WL` (Watchlist), `HOT` (Hotlists), `HM2` (Heatmap), `NW` (News) | `/home`, `/equity/stocks`, `/equity/monitoring`, `/equity/watchlist`, `/equity/hotlists`, `/equity/heatmap`, `/equity/news`, `/equity/economics`, `/equity/commodities`, `/equity/forex`, `/equity/crypto`, `/equity/bonds`, `/equity/yield-curve`, `/equity/etf-analytics`, `/equity/mutual-funds` |
| **Research** | Research | `SH` (Security Hub), `SC` (Screener), `FA` (Factors), `IN` (Insider), `DV` (Dividends), `ER` (Earnings), `RD` (Dossiers) | `/equity/security`, `/equity/screener`, `/equity/factors`, `/equity/insider`, `/equity/dividends`, `/equity/earnings`, `/equity/intelligence-timeline`, `/equity/research-dossiers`, `/equity/research`, `/equity/sector-rotation`, `/equity/rs` |
| **Charts** | Charts | `WS` (Workstation), `LP` (Launchpad), `CP` (Compare), `MT` (Multi-TF), `DM` (DOM), `TP` (Tape) | `/equity/chart-workstation`, `/equity/launchpad`, `/equity/launchpad/popout`, `/equity/compare`, `/equity/mta`, `/equity/dom`, `/equity/tape` |
| **Derivatives** | Derivatives | `OC` (Option Chain), `GK` (Greeks), `FU` (Futures), `OI` (Open Interest), `ST` (Strategy), `FL` (Flow) | `/fno`, `/fno/greeks`, `/fno/futures`, `/fno/oi`, `/fno/strategy`, `/fno/pcr`, `/fno/flow`, `/fno/heatmap`, `/fno/expiry`, `/equity/option-greeks` |
| **Portfolio** | Portfolio | `PF` (Portfolio), `PL` (Portfolio Lab), `PO` (Optimizer), `PT` (Paper), `JR` (Journal), `PS` (Sizer), `SA` (Shadow) | `/equity/portfolio`, `/equity/portfolio/lab`, `/backtesting/portfolio-optimizer`, `/equity/paper`, `/equity/journal`, `/equity/position-sizer`, `/equity/shadow-account`, `/equity/alerts` |
| **Quant** | Quant | `BT` (Backtest), `ML` (Model Lab), `MG` (Governance), `AF` (Algo Framework), `SL` (Stat Lab), `PR` (Pairs), `AZ` (Alpha Zoo) | `/backtesting`, `/backtesting/model-lab`, `/backtesting/model-governance`, `/backtesting/algorithm-framework`, `/equity/stat-lab`, `/equity/pair-trading`, `/equity/alpha-zoo`, `/equity/research-autopilot`, `/equity/strategy-export`, `/strategy`, `/simulator` |
| **Risk & Ops** | Risk & Ops | `RK` (Risk), `CO` (Correlation), `CK` (Cockpit), `OM` (OMS), `OP` (Ops), `DQ` (Data Quality), `RP` (Reports) | `/equity/risk`, `/equity/correlation`, `/equity/cockpit`, `/equity/oms`, `/equity/ops`, `/equity/data-quality`, `/equity/reports` |
| **System** | System | `PG` (Plugins), `SV` (Saved Views), `SG` (Settings), `AC` (Account), `AB` (About) | `/equity/plugins`, `/equity/saved-views`, `/equity/settings`, `/account`, `/about` |

### 2.2 Global Interaction Surfaces
1. **CommandBar (`Ctrl+G` GO bar / `/` trigger)**:
   - Persistent quick input at the top of the shell.
   - Live autocomplete with fuzzy scoring across symbols, instruments, and quick command verbs.
   - Command verbs:
     - `RELIANCE DES` → Jump directly to Security Hub Overview.
     - `INFY FA` → Jump to Financials transposed matrix.
     - `TCS FNO` → Jump to F&O Option Chain for TCS.
     - `NIFTY GP` → Jump to full-screen price chart.
     - `? <query>` → Direct route to AI Agent inquiry.
   - Live hover preview card: current LTP, change%, sparkline, asset class badge, and exchange.
2. **CommandPalette (`Ctrl+K`)**:
   - Modal palette searching actions, open tabs, chart indicators, and navigation shortcuts with badges: `[CMD]`, `[CHART]`, `[FUNC]`, `[ASSET]`, `[NAV]`.
3. **AgentConsole (`Ctrl+J`)**:
   - Slide-over research drawer with SSE streaming.
   - Selectable modes: **Multi-Agent Debate**, **Strategy Lab**, **Screener Membership**, and **Ensemble Consensus**.
   - Generates action proposal cards with user confirmation (e.g. Place Paper Order, Add to Watchlist, Create Alert).
4. **Symbol Link Groups (`A`, `B`, `C`, `D`)**:
   - Colored link tags on panel headers allowing independent synchronization clusters.
   - Updating the symbol in a Group A panel simultaneously syncs all other Group A panels (chart, DOM, option chain, filings) while leaving Group B panels intact.
5. **Panel Pop-out System**:
   - Every `Panel` can pop out into an independent browser window (`/equity/launchpad/popout?panel=...`), communicating bidirectionally with the main terminal via BroadcastChannel.

### 2.3 NAV Landing Architecture & Layout Shell Hierarchy

Honba terminal organizes navigation into **4 persistent Layout Shells** and **8 dedicated Navigation Landing Pages**. When a user clicks any primary section on the `NavRail` or enters via a shortcut, they are directed to the designated primary landing page for that domain, wrapped within its specialized workspace chrome.

#### 1. Root Landing Policy (`/` Redirect)
* **Unauthenticated Guests**: Full-page redirect out of the SPA shell to `/landing/` (the static marketing and product overview site).
* **Authenticated Users**: Redirected directly into the application cockpit at `/home` (`HomePage`).
* **Session Restoration**: Renders a low-contrast terminal splash ("Restoring workspace...") while hydrating auth tokens and localStorage layout state.

#### 2. The 4 Layout Shells
Each workspace domain uses a dedicated layout shell wrapping Leptos `<Outlet/>`, providing persistent context without unmounting or reloading child components:

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    AppShell (Root Layout)                                       │
│  [TopBar] Ticker Tape · Live Clock · Trading Mode · Link Group · Notifications · Account        │
│  [NavRail] Primary 8 Domain Icons (HM, SH, WS, OC, PF, BT, RK, SG)                              │
├──────────────────────────────────────────────────────────────────┬──────────────────────────────┤
│                     ACTIVE DOMAIN LAYOUT SHELL                   │     CONTEXTUAL RIGHT RAIL    │
│                                                                  │                              │
│ 1. EquityLayout (/equity/*):                                     │ • EquityRightRail:           │
│    Hosts Markets, Security Hub, Screener, Charts, Portfolio,     │   - Dynamic Watchlist        │
│    and Risk/Ops. Injects active symbol context.                  │   - Active Symbol News       │
│                                                                  │   - Quick Alerts             │
│ 2. FnoLayout (/fno/*):                                           │                              │
│    Hosts Option Chain, Greeks, Futures, and Strategy Builder.    │ • FnoRightRail:              │
│    Injects FnoHeaderBar (NIFTY/BANKNIFTY presets, expiry drop-   │   - Active Contract Summary  │
│    down) + sub-nav hotkeys ([F1] Chain .. [F8] Expiry).          │   - Spot / ATM Strike / IV   │
│                                                                  │   - PCR (OI) & Max Pain      │
│ 3. BacktestingLayout (/backtesting/*):                           │                              │
│    Hosts Backtest Runner, Model Lab, Governance, and Framework.  │ • BacktestingRightRail:      │
│    Injects strategy preset bar + model status pills.             │   - Active Strategy / Preset │
│                                                                  │   - Parameter Snapshot       │
│ 4. AccountLayout (/account/*):                                   │                              │
│    Hosts Profile, API Keys, Sessions, and Preferences.           │ • AccountSubNav              │
└──────────────────────────────────────────────────────────────────┴──────────────────────────────┘
```

#### 3. The 8 Primary Navigation Landing Pages
Every section of the terminal has an opinionated, high-value primary landing page loaded by default when clicking its icon in the `NavRail`:

| Nav Section | Glyph | Default Route & Component | Landing Page Architecture & Key Visual Components |
|---|---|---|---|
| **1. Markets** | `HM` / `MK` | `/home`<br/>`HomePage` | • **LiveClockStrip**: Multi-market clocks (NSE IST, NYSE EST, LSE GMT, Tokyo JST)<br/>• **MarketHeatStrip**: Index strip (NIFTY 50, SENSEX, BANKNIFTY, S&P 500, CRUDE)<br/>• **MetricCard Row**: Equity Value, Net PnL, Holdings count, Watchlist items, F&O PCR<br/>• **PortfolioMiniChart**: Portfolio equity curve vs benchmark overlay<br/>• **IdeasRadar**: Algorithmic ideas (breakouts, order wins, volume shockers)<br/>• **ExposureHeatmap**: Interactive sector concentration map<br/>• **IntelligenceTimeline**: Real-time corporate events and concall summaries<br/>• **ResultsSummaryCards**: Latest corporate earnings scorecards (QoQ / YoY)<br/>• **SystemHealthBar**: Feed health, provider latency (ms), and websocket status |
| **2. Research** | `SH` | `/equity/security`<br/>`SecurityHubPage` | • **DES Header**: LTP, 52W Range Bar, Valuation Ratios (P/E, P/B, EV/EBITDA)<br/>• **Reverse DCF Implied Growth**: Valuation gauge against consensus<br/>• **11-Tab Workstation**: Overview, Financials transposed matrices, Chart, News & Sentiment, Ownership, Estimates, Peers & Value Chain, ESG, Tape, Insider, Filings Intelligence |
| **3. Charts** | `WS` | `/equity/chart-workstation`<br/>`ChartWorkstationPage` | • **Multi-Pane Mosaic**: Grid selector (`1x1`, `1x2`, `2x2`, `1+2`, `3x1`, `1x3`)<br/>• **CrosshairSyncProvider**: Time and price cursor synchronization across panes<br/>• **OpenScript Editor & Library**: Custom client-side indicator scripting and live canvas compilation<br/>• **Market Replay Controls**: Historical step-by-step bar playback (`1x`–`50x`)<br/>• **Indicator Catalog**: Categorized library of 500+ technical indicators |
| **4. Derivatives** | `OC` | `/fno`<br/>`OptionChainPage` | • **Inside `FnoLayout`**: Popular underlying pills (NIFTY, BANKNIFTY, FINNIFTY) + Expiry picker<br/>• **StrikeSummaryBar**: Spot price, PCR (OI & Volume), Max Pain strike, ATM IV<br/>• **OptionChainTable**: Calls (Left) vs Puts (Right), LTP, Change, Greeks (Delta, Gamma, Theta, Vega), OI & OI Change, ATM highlight, strike ranges (10/15/20/All)<br/>• **Sub-nav pills**: [F1] Chain, [F2] Greeks, [F3] Futures, [F4] OI, [F5] Strategy, [F6] PCR, [F7] Flow, [F8] Expiry |
| **5. Portfolio** | `PF` | `/equity/portfolio`<br/>`PortfolioPage` | • **Portfolio Summary Ribbon**: Total Value, Invested Capital, Total Unrealized PnL, Realized PnL<br/>• **Interactive Tabs**: Holdings, Positions, Open Orders, Executed Trades, Analytics, Dividends<br/>• **PortfolioImportDrawer**: Broker sync via Zerodha Kite API + CSV statement parser (Groww, Zerodha, generic)<br/>• **Asset Allocation & Factor Attribution**: Sector distribution, style drift, tax lot accounting |
| **6. Quant** | `BT` | `/backtesting`<br/>`BacktestingPage` | • **Inside `BacktestingLayout`**: Strategy templates (Trend, Mean Reversion, Stat-Arb, Options Theta)<br/>• **Backtest Runner Deck**: Execution parameters, transaction cost & slippage models<br/>• **Multi-Panel Analytics**: Equity curve vs benchmark, underwater drawdown curve, monthly PnL calendar matrix, trade returns distribution histogram<br/>• **Monte Carlo & WFO**: 1,000+ simulation bootstrap and walk-forward timeline<br/>• **Statistical Robustness Badges**: Deflated Sharpe Ratio (DSR), Probability of Backtest Overfitting (PBO) |
| **7. Risk & Ops** | `CK` / `RK` | `/equity/cockpit`<br/>`CockpitDashboard` | • **Executive Cockpit Dashboard**: Unifies Portfolio Snapshot, Risk Heatmap, Market Pulse, Sentiment Gauge, Results Tracker, and Intelligence Timeline<br/>• **Alternative Landing (`/equity/risk`)**: Deep Risk Dashboard (VaR 95/99, CVaR, EWMA vol, PCA factor attribution, crisis stress testing scenarios)<br/>• **Operations Console (`/equity/ops`)**: Trader, Quant, PM, Risk, System desks + Emergency Kill Switches (Panic Sell All, Cancel Pending, Halt Engine) |
| **8. System** | `SG` | `/equity/settings`<br/>`SettingsPage` | • **Appearance**: Semantic theme palette selector (`terminal-noir`, `classic-bloomberg`, `midnight-navy`, `light-desk`, custom OKLCH), terminal density toggle<br/>• **Trading Configuration**: Paper Trading vs Live NSE toggle, default lot multiplier, slippage model<br/>• **Data Providers**: Broker credentials, FRED/EDGAR/Kite API keys, fallback waterfall configuration<br/>• **System Health**: WebSocket feed inspector, latency diagnostics, database status |

---

## 3. Detailed Page & Feature Architecture

### 3.1 Security Hub (`/equity/security`) — Single-Page Company Intelligence
The crown jewel of equity research, unifying quantitative metrics, financial statement matrices, and verified filings NLP into 11 tabbed workstations:

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ RELIANCE IND. (NSE: RELIANCE)  |  LTP: ₹2,982.50 ▲1.4%  |  MCAP: ₹20.17T  |  P/E: 28.4  |  BETA: 0.95    │
│ 52W RANGE: [₹2,220.00 ──────────●────── ₹3,024.90]  |  REVERSE DCF IMPLIED GROWTH: 11.2% (Fair Value)    │
├──────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ [Overview] [Financials] [Chart] [News] [Ownership] [Estimates] [Peers] [ESG] [Tape] [Insider] [Filings]  | 
├──────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Overview / DES ───────────────┐ ┌─ Filings Intelligence (RAG) ──────────────────────────────────────┐ │
│ │ • Sector: Oil & Gas / Retail / │ │ Growth Engines (Scored Drivers):                                  │ │
│ │   Telecom                      │ │  [+85] Jio ARPU expansion to ₹195 ("ARPU increased to ₹195        │ │
│ │ • ROE: 9.8%  ROCE: 10.4%       │ │        driven by tariff hike" - Q1 FY26 Report, p.14)             │ │
│ │ • Debt/Equity: 0.38            │ │  [+72] Retail store footprint +12% YoY (Annual Report, p.42)      │ │
│ │ • Operating Margin: 17.2%      │ │ Headwinds:                                                        │ │
│ │ • Reverse DCF Implied: 11.2%   │ │  [-45] O2C refining margin squeeze ("Singapore GRM softened       │ │
│ │ • Catalyst Conviction: High    │ │        to $4.2/bbl" - Investor Presentation, p.8)                 │ │
│ └────────────────────────────────┘ └───────────────────────────────────────────────────────────────────┘ │
│ ┌─ Value Chain (Tijori-style) ─────────────────────────────────────────────────────────────────────────┐ │
│ │ Key Suppliers: [ONGC (Crude), McDermott (EPC), Samsung (Network Equip)]                              │ │
│ │ Key Customers: [Indian Retail Consumers, Telecom Subscribers, Petrochem Converters]                  │ │
│ │ Raw Material Sensitivity: Crude Oil (+$10/bbl impact on operating margin: -1.2%)                     │ │
│ └──────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

#### The 11 Workstation Tabs:
1. **Overview (DES)**: Snapshot, valuation ratios (P/E, P/B, EV/EBITDA), quality scores, 52W range bar, reverse DCF growth implied, catalyst conviction score.
2. **Financials**: Transposed multi-period financial tables (Income Statement, Balance Sheet, Cash Flow) for annual and quarterly reports with growth percentages and common-size analysis.
3. **Chart**: High-resolution candlestick chart with volume, technical overlays, and comparison tickers.
4. **News & Sentiment**: Curated news feed, real-time sentiment scoring (Bullish/Bearish/Neutral), and news impact badges.
5. **Ownership**: Shareholding pattern trends (Promoter, FII, DII, Public), promoter pledging changes, and top institutional holders table.
6. **Estimates**: Consensus analyst estimates, EPS surprises, revenue targets, and forward P/E forecasts.
7. **Peers & Value Chain**: Relative valuation matrix against sector peers; supplier → company → customer network graph with citations; raw material sensitivity analysis.
8. **ESG**: Environmental, social, and governance rating scores with underlying disclosures.
9. **Tape**: Real-time DOM ladder (Depth of Market) and Time & Sales print stream with aggressive buy/sell trade filtering.
10. **Insider**: Form 4 / SAST disclosures, promoter bulk/block deals, and insider transaction trends.
11. **Filings (Filings Intelligence)**: RAG pipeline over NSE/SEC filings:
    - **21 Scored Drivers**: Growth engines vs headwinds with verbatim citations, document name, and page number.
    - **"Ask the Filings" QA**: Natural language semantic query with exact source quoting.
    - **Guidance Tracker**: Historical management promises vs actual reported delivery quarter-by-quarter.
    - **Concall Summaries**: Structured key takeaways, capex plans, and margin outlook.

### 3.2 Company Research Dossiers (`/equity/research-dossiers`)
A dedicated research workbench for long-term investment theses and hypothesis validation:
- **Thesis Construction**: Core investment thesis, fundamental assumptions, forward catalysts, and explicit invalidation criteria.
- **Evidence Management**: Linked claims tagged as `Supporting`, `Opposing`, or `Needs Review`. Every claim carries a verbatim quote, publication date, source URL/filing, and associated financial metric.
- **Revision History**: Immutable revision logs with side-by-side diffing between revisions.
- **Frozen-Data Experiments**: Direct attachment of backtest and simulation runs frozen against specific historical data versions.
- **Shareable Snapshots**: Generate revocable, read-only frozen snapshots for peer review or compliance export.

### 3.3 Derivatives & F&O Terminal (`/fno`)
Comprehensive options and futures workstation for intraday traders and quantitative derivatives strategists:
- **Option Chain (`/fno`)**: Full strike matrix with Calls on the left and Puts on the right; real-time Greeks (Delta, Gamma, Theta, Vega, Rho), Implied Volatility (IV), OI, Change in OI, and Volume. Strike range filters (10, 15, 20, or All strikes around ATM).
- **Strike Summary Bar**: Real-time spot price, Put-Call Ratio by Open Interest (PCR OI), PCR by Volume, Max Pain strike, and ATM IV.
- **OI Analysis (`/fno/oi`)**: Strike-by-strike Call vs Put OI comparison bar charts and multi-expiry open interest accumulation.
- **Multi-Leg Strategy Builder (`/fno/strategy`)**:
  - Interactive leg configuration (Calls/Puts, Long/Short, strike selection, lot sizing, expiry).
  - Presets: Bull Call Spread, Bear Put Spread, Long Straddle, Strangle, Iron Condor, Iron Butterfly, Jade Lizard, Ratio Spread, Calendar Spread.
  - Payoff Graph: Expiry payoff curve + T+0 Greeks-adjusted dynamic curve.
  - Strategic Metrics: Maximum Profit, Maximum Loss, Breakeven Points, Risk/Reward Ratio, Net Debit/Credit, Probability of Profit (POP).
- **Greeks Heatmap (`/fno/greeks`)**: 2D heatmaps of Greeks across all strikes and forward expiries.
- **Futures Term Structure (`/fno/futures`)**: Multi-month futures curve, annualized basis, cost of carry, and historical rollover percentages.
- **Options Flow (`/fno/flow`)**: Live algorithmic feed of unusual options volume, block trades, and multi-leg sweeps.
- **Option Greeks Calculator (`/equity/option-greeks`)**: Black-Scholes-Merton pricing engine with IV solving and scenario stress testing.

### 3.4 Chart Workstation, Multi-Timeframe & Execution Tape
- **Chart Workstation (`/equity/chart-workstation`)**:
  - Multi-pane slot arrangements: `1x1`, `1x2`, `2x1`, `2x2`, `1+2`, `3x1`, `1x3`.
  - Independent or synchronized Crosshair and Time scrubbers.
  - **OpenScript Editor & Library**: Sandboxed client-side script editor for custom indicator authoring, compiling, and real-time canvas rendering.
  - Market Replay mode: Step-by-step bar progression at configurable speeds (`1x`–`50x`) for historical trade practice.
- **Multi-Timeframe Analysis (`/equity/mta`)**:
  - 4 synchronized chart panels aligned to trader psychology presets:
    - *Position*: Monthly (Macro) → Weekly (Trend) → Daily (Key Levels) → 4H (Trigger).
    - *Swing*: Daily (Trend) → 4H (Structure) → 1H (Setup) → 15m (Trigger).
    - *Day-Trade*: 1H (Context) → 15m (Key Levels) → 5m (Pattern) → 1m (Execution).
    - *Scalp*: 15m (Context) → 5m (Trend) → 1m (Trigger) → Tick/Tape (Order Flow).
- **DOM Depth Ladder (`/equity/dom`)**:
  - Vertical ladder with 10-level bid/ask depth, resting limit order volume profiles, and single-click limit/market order placement.
- **Time & Sales (`/equity/tape`)**:
  - Sub-millisecond print tape with color-coded trade aggressor tags (green = buyer-initiated at ask, red = seller-initiated at bid) and minimum size filtering.

### 3.5 Quant Suite, Model Lab & Backtesting
Integrated with `honba-core`, `honba-mon`, and `honba-strategies`:
- **Backtesting Deck (`/backtesting`)**:
  - 16+ canonical strategy templates (Trend Following, Mean Reversion, Statistical Arbitrage, Market Making, Options Theta Decay).
  - Realistic transaction cost models (brokerage, STT, exchange turnover fees, GST, SEBI charges, and liquidity-based slippage).
  - Advanced Visualizations: Equity curve vs benchmark, underwater drawdown curve, monthly PnL calendar heatmap, return distribution histogram, rolling Sharpe/Beta.
  - Statistical Robustness: Monte Carlo bootstrap (1,000+ paths), Walk-Forward Optimization (WFO) in-sample vs out-of-sample timeline, Deflated Sharpe Ratio (DSR), Probability of Backtest Overfitting (PBO).
  - 3D Interactive Surfaces: 3D Parameter Sensitivity, 3D Drawdown Terrain, and 3D Implied Volatility Surface.
- **Model Lab (`/backtesting/model-lab`)**:
  - Experiment versioning registry: Track every backtest iteration as an immutable run report (`/runs/:runId`).
  - Model comparison view (`/compare`): Side-by-side metric overlay for up to 4 models.
  - Quantitative Leaderboard: Ranked by Sharpe ratio, CAGR, max drawdown, turnover, and governance score.
- **Model Governance (`/backtesting/model-governance`)**:
  - Lifecycle state machine: `Draft` → `Candidate` → `Paper Approved` → `Live Production` → `Decommissioned`.
  - Enforced approval checks, risk limits, and model drift monitors.
- **Portfolio Lab (`/equity/portfolio/lab`) & Portfolio Optimizer**:
  - Blending multiple strategies into cohesive portfolios.
  - Mean-Variance Efficient Frontier, Black-Litterman subjective views, Risk Parity, and Hierarchical Risk Parity (HRP) allocation algorithms.
- **Statistical Lab (`/equity/stat-lab`) & Pair-Trading**:
  - Cointegration testing (Engle-Granger two-step, Johansen test), Augmented Dickey-Fuller (ADF) unit root test, Ornstein-Uhlenbeck mean-reversion half-life, and z-score spread tracking.
- **Alpha Zoo (`/equity/alpha-zoo`)**:
  - 100+ vetted alpha factor catalog with Information Coefficient (IC), t-statistics, turnover, and decay curves.
- **Research Autopilot & Strategy Export**:
  - Autonomous evolutionary search for winning strategy rules.
  - 1-click export to Pine Script, MQL5, Python, or Honba Step Manifest.

### 3.6 Risk, Compliance & Operations Cockpit
Deep integration with `honba-mon` for real-time risk auditing and trading safety:
- **Risk Dashboard (`/equity/risk`)**:
  - Real-time Value at Risk (VaR 95% / 99%) and Conditional VaR (Expected Shortfall).
  - Historical EWMA volatility and PCA factor risk attribution.
  - Stress Testing Engine: Historical crisis replays (2008 GFC, 2020 COVID Crash, 2024 Election Volatility, Interest Rate Shock, Flash Crash).
  - Sector and asset concentration heatmaps.
- **OMS & Pre-Trade Compliance (`/equity/oms`)**:
  - Pre-trade order screening: Maximum order notional, maximum daily turnover, fat-finger protection, order throttle rate (OTR), circuit filter distance, and restricted symbol checking.
  - Complete immutable audit log of all order requests, approvals, and refusals.
- **Operations Dashboard (`/equity/ops`)**:
  - Multi-desk operational views: Trader, Quant, PM, Risk, and System.
  - **Emergency Kill Switches**:
    - `Panic Sell All`: Liquidate all active market positions immediately.
    - `Cancel All Pending`: Cancel all unexecuted limit/stop orders.
    - `Halt Trading Engine`: Freeze all algo trading loops and reject new incoming signals.
    - `Safe Mode Toggle`: Restrict operations to risk-reducing orders only.
  - Feed health monitors, provider latency histograms, and data quality anomaly flags.
- **Cockpit Dashboard (`/equity/cockpit`)**:
  - Executive overview bringing together portfolio PnL, risk alerts, market pulse, news sentiment, results tracker, and intelligence timeline.

### 3.7 Portfolio, Execution & Shadow Accounting
- **Portfolio Management (`/equity/portfolio`)**:
  - Multi-portfolio tracking (Holdings, Positions, Orders, Trades, PnL).
  - Broker synchronization: Direct Zerodha Kite API sync + CSV import for Groww, Zerodha, and generic broker statements.
  - Tax lot accounting (FIFO / LIFO / Specific Identification) and dividend tracker.
- **Paper Trading Desk (`/equity/paper`)**:
  - Real-time simulated execution with realistic order matching against live market depth and configurable fill models.
- **Trade Journal (`/equity/journal`)**:
  - Execution logging with psychological emotion tagging (`confident`, `fearful`, `greedy`, `neutral`).
  - Mistake categorisation (chasing, early exit, oversized, revenge trade).
  - Setup tagging (breakout, pullback, mean reversion, gap & go).
  - Interactive daily PnL calendar and equity curve.
- **Dynamic Position Sizer (`/equity/position-sizer`)**:
  - Direct UI implementation of `honba-core::sizing`: Fixed Fractional, Kelly Criterion (Full, Half, Quarter), ATR-adjusted volatility sizing, and volatility targeting.
- **Shadow Account Reconciler (`/equity/shadow-account`)**:
  - Automated reconciliation between internal OMS trade records and broker end-of-day statements.
  - Immediate highlighting of discrepancies: phantom executions, unmatched positions, corporate action drift, and commission mismatches.

---

## 4. Design System Architecture (`honba-kao-ui-core`)

All domain pages and composite widgets are strictly constructed from `honba-kao-ui-core` primitives. Direct usage of raw HTML tags (`<button>`, `<input>`, `<select>`, `<table>`) in application feature modules is prohibited and enforced by CI guardrails.

> **Naming & Porting Principle (supersedes any earlier "Terminal*"-branded component scheme)**:
> `honba-kao-ui-core` already has a mature, shadcn/Radix-parity primitive set (`button.rs`, `card.rs`, `dialog.rs`, `tabs.rs`, `table.rs`, `command.rs`, `sheet.rs`, `drawer.rs`, etc. — see §4.1). We do **not** introduce a parallel, generically "Terminal"-prefixed component family (no `TerminalButton`, `TerminalPanel`, `TerminalShell`, …). Instead:
> 1. **Port, don't reinvent**: For every OpenTerminalUI (`frontend/src/components/...`) component being adopted, port its actual structure, props, and behavior into a Leptos component built from the existing primitives above — reimplemented idiomatically in Rust, not duplicated verbatim.
> 2. **Plain, descriptive names**: Name ported/composed components after what they are (`CommandBar`, `SecurityHubPage`, `OptionChainTable`, `OrderTicket`, `AppShell`), mirroring OpenTerminalUI's own naming where sensible, never a generic brand prefix.
> 3. **Theme names are unaffected**: Palette/theme identifiers such as `terminal-noir` remain — this rule is about component naming, not theme naming.

### 4.1 Component Crate Hierarchy

```
crates/honba-kao-ui-core/src/
├── lib.rs
├── primitive/                     # Existing shadcn/Radix-parity primitives (already implemented)
│   ├── button.rs, input.rs, select.rs, checkbox.rs, radio.rs, switch.rs, slider.rs
│   ├── table.rs, tabs.rs, segmented.rs, toggle.rs, pagination.rs, toolbar.rs
│   ├── panel.rs, card.rs, accordion.rs, collapsible.rs, separator.rs, resizable.rs
│   ├── dialog.rs, drawer.rs, sheet.rs, popover.rs, hover_card.rs, context_menu.rs, menu.rs
│   ├── command.rs, navigation_menu.rs, breadcrumb.rs, page_header.rs, form.rs, label.rs
│   ├── toast.rs, tooltip.rs, alert.rs, badge.rs, avatar.rs, kpi.rs, progress.rs
│   ├── skeleton.rs, placeholder.rs, scroll_area.rs, scrollbar.rs, aspect_ratio.rs
│   └── icons.rs                  # Procedural outline SVG icons (no external font or static files)
│   New primitives are added here only for genuinely missing building blocks (e.g. a
│   virtualized `DataTable` on top of `table.rs`, or a `PanelChrome` link-group/popout
│   header on top of `panel.rs`) — not to re-skin what already exists.
├── trading/                       # Domain-specific financial visualization primitives
│   ├── pl_number.rs              # PlNumber (existing): PnL display with ▲/▼, ₹/$, color-coded, auto-rounding
│   ├── mini_sparkline.rs         # MiniSparkline: inline SVG sparklines
│   ├── mini_range_bar.rs         # MiniRangeBar: 52-week & day-range visual bar with current price pip
│   ├── sentiment_badge.rs        # SentimentBadge: Bullish / Bearish / Neutral score pill
│   ├── sentiment_chart.rs        # SentimentChart: sentiment distribution gauge & trend line
│   ├── emotion_indicator.rs      # EmotionIndicator: trading psychology icons & labels
│   ├── ai_insight_card.rs        # AiInsightCard: card for AI summaries, key drivers, bullet takeaways
│   ├── status_bar.rs             # MarketStatusBar, FeedHealthIndicator, LatencyPill
│   ├── order_ticket.rs           # OrderTicket: Symbol, Side, Type, Qty Stepper, Limits, SL/TP brackets
│   ├── dom_ladder.rs             # DomLadder: interactive market depth ladder
│   ├── time_and_sales.rs         # TimeAndSales: real-time trade print tape
│   └── hotkey_panel.rs           # HotkeyPanel: fast keyboard-driven execution deck
├── composed/                      # Composite widgets ported from OpenTerminalUI
│   ├── modal.rs                  # (existing) higher-order modal composition over dialog.rs
│   ├── command_bar.rs            # CommandBar (Ctrl+G GO Bar with live preview & command hints)
│   ├── command_palette.rs        # CommandPalette (Ctrl+K global fuzzy command runner, over command.rs)
│   ├── agent_console.rs          # AgentConsole (Ctrl+J slide-over AI research drawer, over sheet.rs/drawer.rs)
│   ├── top_bar.rs                # TopBar (ticker tape, market status, mode switcher, link groups)
│   ├── icon_rail.rs              # IconRail & NavRail (domain icon navigation)
│   ├── launchpad_grid.rs         # LaunchpadGrid (resizable mosaic workspace grid, over resizable.rs)
│   ├── symbol_context_rail.rs    # SymbolContextRail (linked symbol/timeframe context bar)
│   ├── filings_workspace.rs      # FilingsWorkspace (21 driver cards, verbatim quotes, QA)
│   ├── dossier_view.rs           # ResearchDossierView (thesis, claims, revision history)
│   ├── options_chain_view.rs     # OptionChainView (strike matrix, summary bar, OI charts)
│   ├── strategy_builder_view.rs  # OptionsStrategyView (multi-leg builder, payoff curve)
│   ├── backtesting_deck.rs       # BacktestingDeck (quant metrics ribbon, equity curve, Monte Carlo)
│   └── theme_studio.rs           # ThemeStudio (interactive OKLCH palette studio, contrast audit, JSON export)
└── hooks/                        # Reactive hooks for shell/panel coordination
    ├── use_link_group.rs         # Panel link group subscription (A, B, C, D)
    ├── use_crosshair_sync.rs     # Crosshair & time synchronization across charts
    ├── use_recent_securities.rs  # LocalStorage-backed recent symbol history
    ├── use_keyboard_shortcuts.rs # Global hotkey binding dispatcher
    ├── use_quotes_stream.rs      # WebSocket real-time quote streaming hook
    ├── use_virtual_rows.rs       # Table row virtualization for 5,000+ item lists
    ├── use_click_outside.rs      # Outside-click detection for modals/popovers
    └── use_escape_key.rs         # Escape key dismissal handler
```

### 4.2 Modern Semantic Color Engine (Unbounded OKLCH & Display P3)

> **Architectural Liberation from ANSI Constraints**:
> The 256-color palette in earlier drafts and legacy TUI implementations was an artificial artifact inherited from terminal emulators (`xterm-256color` ANSI escape indices `000`–`255`). Because `honba-kao` is a pure WebAssembly / Leptos 0.7 browser application, it has **zero 256-color index restrictions**.
>
> `honba-kao` directly embraces the modern CSS Color Module Level 4/5 stack:
> 1. **Perceptually Uniform OKLCH Color Space**: Independent control of Lightness ($L$), Chroma ($C$), and Hue ($H$) ensures predictable contrast across light and dark themes without perceived brightness shifts.
> 2. **Wide-Gamut Display P3 Support**: High-saturation financial greens (gains/buy walls), vibrant reds (losses/kill-switches), and vivid electric cyans that cannot be rendered in sRGB.
> 3. **Continuous Alpha Channels**: Native CSS `oklch(L C H / alpha)` and `color-mix()` for glassmorphism, subtle panel tints, focus rings, and elevation glows without indexed color lookup tables.
> 4. **Semantic Token Hierarchy**: Clear separation between primitive color scales and functional UI roles.

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           Layer 0: Perceptual OKLCH Scales                                      │
│  Neutral / Zinc        Emerald / Mint (Gain)       Crimson / Coral (Loss)     Amber (Warning)   │
│  oklch(0.12..0.98 ...) oklch(0.20..0.92 0.18 152)  oklch(0.22..0.88 0.22 25)  oklch(...)        │
└────────────────────────────────────────────────┬────────────────────────────────────────────────┘
                                                 │
                                                 ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           Layer 1: Semantic Design Tokens                                       │
│  Surface:       --surface-canvas · --surface-panel · --surface-elevated · --surface-overlay     │
│  Border:        --border-subtle · --border-default · --border-strong · --border-interactive     │
│  Text:          --text-primary · --text-secondary · --text-muted · --text-inverse              │
│  Accent:        --accent-primary · --accent-hover · --accent-muted · --accent-focus             │
│  Market:        --market-up · --market-down · --market-neutral · --candle-up · --candle-down    │
│  Risk & Ops:    --risk-low · --risk-medium · --risk-high · --risk-critical · --ops-running      │
└────────────────────────────────────────────────┬────────────────────────────────────────────────┘
                                                 │
                                                 ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           Layer 2: Leptos UI Components & Canvas                                │
│  Panel · DataTable · Button · DomLadder · TradingChart (Canvas Shaders)                         │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

#### 1. Core Semantic Design Tokens

| Semantic Token | Purpose | Terminal Noir (OKLCH / P3) | Classic Bloomberg (Amber) | Midnight Navy | Light Desk |
|---|---|---|---|---|---|
| `--surface-canvas` | Base app canvas / deep background | `oklch(0.12 0.01 240)` (`#0d1117`) | `oklch(0.08 0 0)` (`#000000`) | `oklch(0.11 0.02 245)` (`#0a1526`) | `oklch(0.98 0.005 240)` (`#f8fafc`) |
| `--surface-panel` | Standard panel background | `oklch(0.16 0.012 240)` (`#161b22`) | `oklch(0.11 0.01 60)` (`#121008`) | `oklch(0.15 0.025 245)` (`#112038`) | `oklch(1.0 0 0)` (`#ffffff`) |
| `--surface-elevated` | Popovers, dropdowns, dialogs | `oklch(0.20 0.015 240)` (`#1a212b`) | `oklch(0.16 0.02 60)` (`#1c180a`) | `oklch(0.19 0.03 245)` (`#182d4d`) | `oklch(0.96 0.005 240)` (`#f1f5f9`) |
| `--border-subtle` | Dividers, subtle grid lines | `oklch(0.24 0.012 240)` (`#212a36`) | `oklch(0.20 0.03 60)` (`#26200a`) | `oklch(0.22 0.02 245)` (`#1d3252`) | `oklch(0.90 0.005 240)` (`#e2e8f0`) |
| `--border-default` | Card borders, table headers | `oklch(0.30 0.015 240)` (`#2d3745`) | `oklch(0.32 0.06 60)` (`#3d3008`) | `oklch(0.28 0.03 245)` (`#29436e`) | `oklch(0.82 0.01 240)` (`#cbd5e1`) |
| `--border-strong` | Active input border, focused cards | `oklch(0.40 0.02 240)` (`#3a4658`) | `oklch(0.55 0.12 60)` (`#806200`) | `oklch(0.38 0.04 245)` (`#3a5d96`) | `oklch(0.60 0.015 240)` (`#64748b`) |
| `--text-primary` | Main titles, numbers, tickers | `oklch(0.92 0.01 240)` (`#d7dde7`) | `oklch(0.85 0.18 65)` (`#ff9900`) | `oklch(0.94 0.01 240)` (`#e2e8f0`) | `oklch(0.18 0.01 240)` (`#0f172a`) |
| `--text-secondary` | Labels, table columns, breadcrumbs | `oklch(0.78 0.015 240)` (`#adb7c6`) | `oklch(0.72 0.12 65)` (`#d48000`) | `oklch(0.76 0.02 240)` (`#94a3b8`) | `oklch(0.42 0.015 240)` (`#475569`) |
| `--text-muted` | Inactive icons, timestamps, units | `oklch(0.60 0.015 240)` (`#7f8b9d`) | `oklch(0.50 0.08 65)` (`#8f5c05`) | `oklch(0.58 0.02 240)` (`#64748b`) | `oklch(0.60 0.01 240)` (`#94a3b8`) |
| `--accent-primary` | Brand orange, primary action CTA | `oklch(0.68 0.22 45)` (`#FF6B00`) | `oklch(0.75 0.20 60)` (`#ff9e00`) | `oklch(0.65 0.22 245)` (`#3b82f6`) | `oklch(0.58 0.22 45)` (`#ea580c`) |
| `--market-up` | Bullish ticker, positive PnL | `oklch(0.72 0.20 152)` (`#00c176`) | `oklch(0.74 0.18 145)` (`#10b981`) | `oklch(0.72 0.20 152)` (`#00c176`) | `oklch(0.62 0.18 152)` (`#059669`) |
| `--market-down` | Bearish ticker, negative PnL | `oklch(0.65 0.24 25)` (`#ff4d4f`) | `oklch(0.64 0.22 25)` (`#ef4444`) | `oklch(0.65 0.24 25)` (`#ff4d4f`) | `oklch(0.56 0.22 25)` (`#dc2626`) |
| `--risk-critical` | Kill-switch, breach refusal | `oklch(0.60 0.26 28)` (`#ff3338`) | `oklch(0.62 0.25 28)` (`#f82c32`) | `oklch(0.60 0.26 28)` (`#ff3338`) | `oklch(0.52 0.24 28)` (`#b91c1c`) |

#### 2. Supported Terminal Themes
1. **Terminal Noir (Default)**: Deep OLED canvas (`#0d1117`), elevated graphite panels (`#161b22`), electric orange primary accent (`#FF6B00`), vibrant emerald green (`#00c176`), and crisp crimson red (`#ff4d4f`).
2. **Classic Bloomberg**: Pure vintage amber/orange typography (`#ff9e00`) against pure void black (`#000000`), with high-contrast amber borders and CRT terminal character aesthetics.
3. **Midnight Navy**: Institutional maritime dark theme with deep navy canvas (`#0a1526`), cool slate borders, cyan/mint ticker highlights, and electric azure links (`#3b82f6`).
4. **Light Desk**: High-contrast, glare-free daylight theme designed for high-ambient-light trading floors with crisp white cards (`#ffffff`), chalk dividers, and deep charcoal text (`#0f172a`).
5. **Runtime Custom Themes**: Dynamic CSS Custom Properties injected at runtime via `ThemeStudio` with live contrast verification (WCAG APCA / AA) and Canvas/WebGL shader color synchronization.

---

## 5. Application Structure & Feature-Sliced Architecture

To eliminate technical debt and avoid monolithic stores, `honba-kao-app` strictly follows a Feature-Sliced Architecture (FSA):

```
crates/honba-kao-app/
├── Cargo.toml
├── src/
│   ├── main.rs                   # WebAssembly entry point
│   ├── app.rs                    # Root component: providers + app shell + router
│   ├── error_boundary.rs         # Global fatal error boundary
│   │
│   ├── features/                 # Self-contained business capability slices
│   │   ├── auth/                 # Login, session, token refresh, protected routes
│   │   ├── markets/              # Market overview, monitoring, hotlists, heatmap, commodities, forex
│   │   ├── security_hub/         # 11-tab company deep dive, financials matrices, value chain
│   │   ├── filings/              # Filings Intelligence RAG, 21 scored drivers, QA, concall summaries
│   │   ├── dossiers/             # Research dossiers, thesis, verbatim evidence, revisions
│   │   ├── fno/                  # Option chain, Greeks heatmap, futures curve, strategy payoff builder
│   │   ├── charts/               # Chart workstation, multi-timeframe analysis (MTA), OpenScript IDE
│   │   ├── execution/            # DOM depth ladder, Time & Sales tape, hotkey order desk
│   │   ├── portfolio/            # Multi-portfolio tracking, Kite/CSV import, tax lots, attribution
│   │   ├── quant/                # Backtesting deck, Monte Carlo, WFO, Model Lab, Alpha Zoo, Pair Trading
│   │   ├── strategy_studio/      # Visual canvas node editor, StockSharp indicators, genetic generator
│   │   ├── simulator/            # Execution simulator, replay clock, trade journal
│   │   ├── risk/                 # VaR/CVaR, stress test scenarios, correlation matrix, PCA factors
│   │   ├── compliance/           # OMS pre-trade risk checks, restricted symbols, audit trails
│   │   ├── ops/                  # Multi-desk ops console, emergency kill switches, data quality
│   │   └── agent/                # AI Agent slide-over console, multi-agent debate, action proposals
│   │
│   ├── shared/                   # Cross-cutting foundational kernel
│   │   ├── api/                  # Base HTTP client, WebSocket client, auth interceptor
│   │   ├── query/                # leptos-query client setup, query keys, cache configuration
│   │   ├── state/                # Global signals: trading mode, selected market, active symbol
│   │   ├── types/                # Re-exported domain models from honba-core
│   │   └── utils/                # Number formatters, date parsers, currency multipliers
│   │
│   ├── router/                   # Routing configuration & route guards
│   │   ├── routes.rs             # Complete route tree with loaders and lazy components
│   │   ├── paths.rs              # Path constants (single source of truth)
│   │   ├── app_chrome.rs         # Persistent shell wrapping routes with Outlet
│   │   └── guards.rs             # AuthGuard, TradingModeGuard, RoleGuard
│   │
│   └── providers/                # Top-level context providers
│       ├── theme_provider.rs     # Semantic color engine & theme runtime
│       ├── query_provider.rs     # leptos-query QueryClientProvider
│       ├── symbol_link_provider.rs # LinkGroup (A-D) context
│       ├── toast_provider.rs     # Alert toast notifications
│       └── agent_provider.rs     # Agent SSE streaming & active thread context
```

### 5.1 Route Hierarchy
All routes are declared in `router/paths.rs` and composed in `router/routes.rs`:

```rust
// router/paths.rs
pub mod routes {
    // Markets & Overview
    pub const HOME: &str = "/";
    pub const MARKET: &str = "/equity/stocks";
    pub const MONITORING: &str = "/equity/monitoring";
    pub const WATCHLIST: &str = "/equity/watchlist";
    pub const HOTLISTS: &str = "/equity/hotlists";
    pub const HEATMAP: &str = "/equity/heatmap";
    pub const NEWS: &str = "/equity/news";
    pub const ECONOMICS: &str = "/equity/economics";
    pub const COMMODITIES: &str = "/equity/commodities";
    pub const FOREX: &str = "/equity/forex";
    pub const CRYPTO: &str = "/equity/crypto";
    pub const BONDS: &str = "/equity/bonds";
    pub const YIELD_CURVE: &str = "/equity/yield-curve";
    pub const ETF: &str = "/equity/etf-analytics";
    pub const MUTUAL_FUNDS: &str = "/equity/mutual-funds";

    // Research
    pub const SECURITY_HUB: &str = "/equity/security";
    pub const SCREENER: &str = "/equity/screener";
    pub const FACTORS: &str = "/equity/factors";
    pub const INSIDER: &str = "/equity/insider";
    pub const DIVIDENDS: &str = "/equity/dividends";
    pub const EARNINGS: &str = "/equity/earnings";
    pub const INTELLIGENCE: &str = "/equity/intelligence-timeline";
    pub const RESEARCH_DOSSIERS: &str = "/equity/research-dossiers";
    pub const RESEARCH_LIBRARY: &str = "/equity/research";
    pub const SECTOR_ROTATION: &str = "/equity/sector-rotation";
    pub const RELATIVE_STRENGTH: &str = "/equity/rs";

    // Charts & Workstations
    pub const CHART_WORKSTATION: &str = "/equity/chart-workstation";
    pub const LAUNCHPAD: &str = "/equity/launchpad";
    pub const LAUNCHPAD_POPOUT: &str = "/equity/launchpad/popout";
    pub const COMPARE: &str = "/equity/compare";
    pub const MULTI_TF: &str = "/equity/mta";
    pub const DOM: &str = "/equity/dom";
    pub const TAPE: &str = "/equity/tape";

    // Derivatives / F&O
    pub const FNO_CHAIN: &str = "/fno";
    pub const FNO_GREEKS: &str = "/fno/greeks";
    pub const FNO_FUTURES: &str = "/fno/futures";
    pub const FNO_OI: &str = "/fno/oi";
    pub const FNO_STRATEGY: &str = "/fno/strategy";
    pub const FNO_PCR: &str = "/fno/pcr";
    pub const FNO_FLOW: &str = "/fno/flow";
    pub const FNO_HEATMAP: &str = "/fno/heatmap";
    pub const FNO_EXPIRY: &str = "/fno/expiry";
    pub const OPTION_GREEKS_CALC: &str = "/equity/option-greeks";

    // Portfolio & Execution
    pub const PORTFOLIO: &str = "/equity/portfolio";
    pub const PORTFOLIO_LAB: &str = "/equity/portfolio/lab";
    pub const PORTFOLIO_OPTIMIZER: &str = "/backtesting/portfolio-optimizer";
    pub const PAPER: &str = "/equity/paper";
    pub const JOURNAL: &str = "/equity/journal";
    pub const POSITION_SIZER: &str = "/equity/position-sizer";
    pub const SHADOW_ACCOUNT: &str = "/equity/shadow-account";
    pub const ALERTS: &str = "/equity/alerts";

    // Quant & Backtesting
    pub const BACKTESTING: &str = "/backtesting";
    pub const MODEL_LAB: &str = "/backtesting/model-lab";
    pub const MODEL_GOVERNANCE: &str = "/backtesting/model-governance";
    pub const ALGO_FRAMEWORK: &str = "/backtesting/algorithm-framework";
    pub const STAT_LAB: &str = "/equity/stat-lab";
    pub const PAIR_TRADING: &str = "/equity/pair-trading";
    pub const ALPHA_ZOO: &str = "/equity/alpha-zoo";
    pub const RESEARCH_AUTOPILOT: &str = "/equity/research-autopilot";
    pub const STRATEGY_EXPORT: &str = "/equity/strategy-export";
    pub const STRATEGY_STUDIO: &str = "/strategy";
    pub const SIMULATOR: &str = "/simulator";

    // Risk, OMS & Operations
    pub const RISK: &str = "/equity/risk";
    pub const CORRELATION: &str = "/equity/correlation";
    pub const COCKPIT: &str = "/equity/cockpit";
    pub const OMS: &str = "/equity/oms";
    pub const OPS: &str = "/equity/ops";
    pub const DATA_QUALITY: &str = "/equity/data-quality";
    pub const REPORTS: &str = "/equity/reports";

    // System & Auth
    pub const PLUGINS: &str = "/equity/plugins";
    pub const SAVED_VIEWS: &str = "/equity/saved-views";
    pub const SETTINGS: &str = "/equity/settings";
    pub const ACCOUNT: &str = "/account";
    pub const ABOUT: &str = "/about";
    pub const LOGIN: &str = "/login";
    pub const REGISTER: &str = "/register";
}
```

---

## 6. Phased Implementation Roadmap

This revised roadmap replaces previous milestones with a sequenced migration to full OpenTerminalUI parity.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Phase 0: honba-kao-ui-core Primitives & High-Density Design System         (Weeks 1-2) │
│ Phase 1: App Shell, Navigation & Real-time Plumbing                        (Weeks 3-4) │
│ Phase 2: Equity Security Hub, Filings Intelligence & Research Dossiers     (Weeks 5-6) │
│ Phase 3: Derivatives & F&O Suite (Option Chain · Payoff Engine · Greeks)   (Weeks 7-8) │
│ Phase 4: Chart Workstation, Multi-Timeframe, DOM & Execution Tape          (Weeks 9-10)│
│ Phase 5: Quant Suite, Backtesting, Model Lab & Portfolio Lab              (Weeks 11-12)│
│ Phase 6: Risk Engine, OMS Compliance, Ops Cockpit & Position Sizing       (Weeks 13-14)│
│ Phase 7: AI Research Agent Console, Multi-Agent Debate & MCP Bridge       (Weeks 15-16)│
│ Phase 8: Quality, Performance Benchmarks, Accessibility & E2E Polish      (Weeks 17-18)│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Phase 0: Design System Primitives (`honba-kao-ui-core`)
- [ ] **0.1 Primitives Expansion**: Extend the *existing* shadcn-parity primitives (`button.rs`, `input.rs`, `select.rs`, `tabs.rs`, `table.rs`, `dialog.rs`, `toast.rs`, `badge.rs`, `tooltip.rs`, etc. — do not reintroduce a `Terminal*`-prefixed set) with the pieces OpenTerminalUI needs that are still missing: a virtualized `DataTable` over `table.rs`, and a `PanelChrome` header (LinkGroup selector + Popout action) over `panel.rs`.
- [ ] **0.2 Financial Primitives**: Implement `HfPLNumber`, `MiniSparkline`, `MiniRangeBar`, `SentimentBadge`, `SentimentChart`, `EmotionIndicator`, `AiInsightCard`, `MarketStatusBar`.
- [ ] **0.3 Table Virtualization**: Solidify `use_virtual_rows` to guarantee smooth 60fps scrolling over 5,000+ stock rows in screener and tape tables with zero jank.
- [ ] **0.4 Procedural Outline Icons**: Verify complete procedural SVG icon set (`stroke="currentColor" fill="none"`) for all 8 domain glyphs, trading side actions, and chart tools.
- [ ] **0.5 CI Guardrails**: Update `make lint-design-system` to enforce zero raw HTML tags in feature crates.

### Phase 1: App Shell, Navigation & Real-Time Plumbing
- [ ] **1.1 Persistent Shell Layout**: Implement `AppShell` wrapping Leptos `<Outlet/>`, keeping `TopBar`, `NavRail`, and ticker tape permanently mounted across navigation.
- [ ] **1.2 CommandBar (`Ctrl+G`)**: Implement quick GO bar with live symbol search, fuzzy history matching (`ot:gobar:history:v1`), ticker hints (`DES`, `FA`, `FNO`, `GP`), and asset preview card with sparkline.
- [ ] **1.3 CommandPalette (`Ctrl+K`)**: Implement modal command runner searching routes, tools, and actions.
- [ ] **1.4 Symbol Link Groups (`A`–`D`)**: Implement `SymbolLinkContext` and `use_link_group` allowing independent panel symbol synchronization.
- [ ] **1.5 Real-Time WebSocket Streaming**: Build typed WebSocket client for streaming market depth, trades, and quotes with provenance indicators (`Live / Delayed / Cached / Synthetic`).

### Phase 2: Equity Security Hub, Filings Intelligence & Research Dossiers
- [ ] **2.1 Security Hub Core**: Implement `/equity/security` with DES snapshot, valuation ratios, 52-week range bars, and 11-tab router.
- [ ] **2.2 Financials Transposed Matrices**: Build transposed balance sheet, income statement, and cash flow views with quarterly/annual toggle.
- [ ] **2.3 Value Chain & Business Pack**: Build supplier → company → customer citation graph, operating KPIs chart, and reverse DCF implied growth engine.
- [ ] **2.4 Filings Intelligence (RAG)**: Port 21 scored growth/headwind drivers with verbatim quote verification, page links, and "Ask the Filings" natural language QA.
- [ ] **2.5 Research Dossiers**: Implement `/equity/research-dossiers` with thesis builder, evidence stance cards (`Supporting`/`Opposing`), versioned diffing, and shareable frozen-data snapshots.
- [ ] **2.6 Screener & Factor Dashboard**: Build guru presets, formula engine, filings-derived metrics, and factor heatmap.

### Phase 3: Derivatives & F&O Suite
- [ ] **3.1 Option Chain**: Implement `/fno` with Calls/Puts bid/ask, LTP, IV, Greeks (Delta, Gamma, Theta, Vega), strike filters (10/15/20/All), and ATM highlight.
- [ ] **3.2 Strike Summary Bar**: Render real-time spot price, PCR (OI & Volume), Max Pain strike, and ATM IV.
- [ ] **3.3 Multi-Leg Strategy Builder**: Build `/fno/strategy` with 10+ standard strategy presets (Bull/Bear spreads, Straddles, Iron Condor), interactive payoff graph at expiry & T+0, POP, and max profit/loss calculations.
- [ ] **3.4 Derivatives Analytics**: Implement `/fno/oi` (OI bar charts), `/fno/greeks` (Greeks heatmap), `/fno/futures` (term structure & basis curve), `/fno/flow` (unusual options activity), and `/equity/option-greeks` (Black-Scholes solver).

### Phase 4: Chart Workstation, Multi-Timeframe, DOM & Execution Tape
- [ ] **4.1 Chart Workstation**: Implement `/equity/chart-workstation` with multi-pane grid layouts (`1x1`, `1x2`, `2x2`, `1+2`), Crosshair sync, and workspace snapshot export/import.
- [ ] **4.2 OpenScript IDE**: Embed client-side custom indicator scripting editor with compilation, error diagnostics, and indicator library.
- [ ] **4.3 Multi-Timeframe Analysis (MTA)**: Implement `/equity/mta` with 4 synchronized charts for Position, Swing, Day-Trade, and Scalp presets.
- [ ] **4.4 DOM Ladder & Time & Sales**: Implement `/equity/dom` (10-level interactive depth ladder with 1-click execution) and `/equity/tape` (sub-millisecond print tape with size filtering).
- [ ] **4.5 Launchpad & Pop-Outs**: Build customizable mosaic workspace grid and independent pop-out window dispatch (`/equity/launchpad/popout`).

### Phase 5: Quant Suite, Backtesting, Model Lab & Portfolio Lab
- [ ] **5.1 Backtesting Deck**: Implement `/backtesting` with 16+ canonical strategy templates, execution slippage models, equity/drawdown curves, monthly PnL heatmaps, and Monte Carlo bootstrap (1,000+ paths).
- [ ] **5.2 Statistical Robustness**: Render Deflated Sharpe Ratio (DSR), Probability of Backtest Overfitting (PBO), and Walk-Forward Optimization (WFO) timelines.
- [ ] **5.3 3D Visualization Panels**: Interactive WebGL/Canvas panels for 3D Parameter Sensitivity, 3D Drawdown Terrain, and 3D Volatility Surfaces.
- [ ] **5.4 Model Lab & Governance**: Implement `/backtesting/model-lab` (versioned run reports and leaderboard) and `/backtesting/model-governance` (approval workflows and risk constraints).
- [ ] **5.5 Portfolio Lab & Optimizer**: Implement strategy blending, Mean-Variance Efficient Frontier, Black-Litterman, and Risk Parity optimization.
- [ ] **5.6 Statistical & Pair Trading Labs**: Cointegration, ADF unit root tests, half-life mean-reversion, and z-score spread tracking.

### Phase 6: Risk Engine, OMS Compliance, Ops Cockpit & Position Sizing
- [ ] **6.1 Risk Dashboard**: Implement `/equity/risk` with VaR/CVaR, EWMA vol, PCA factor exposures, and crisis stress scenarios (2008 GFC, COVID-19, Rate Shock).
- [ ] **6.2 OMS & Pre-Trade Compliance**: Implement `/equity/oms` with pre-trade checks (OTR, circuit filter limits, restricted symbols) and immutable audit log.
- [ ] **6.3 Operations Console**: Implement `/equity/ops` with Trader/Quant/PM/Risk/System desks, emergency kill switches (Panic Sell All, Cancel All Pending, Halt Engine), and feed health monitoring.
- [ ] **6.4 Position Sizer**: Build `/equity/position-sizer` with Fixed Fractional, Kelly Criterion, ATR volatility-adjusted sizing, and R:R calculations directly wired to `honba-core::sizing`.
- [ ] **6.5 Shadow Account Reconciler**: Implement `/equity/shadow-account` for automated broker statement vs internal book reconciliation and violation reporting.

### Phase 7: AI Research Agent Console, Multi-Agent Debate & MCP Bridge
- [ ] **7.1 AgentConsole Drawer (`Ctrl+J`)**: Build slide-over dockable drawer with SSE streaming, message threads, and research memory.
- [ ] **7.2 Multi-Agent Debate**: Orchestrate bull vs bear debate with portfolio manager decision syntheses.
- [ ] **7.3 Action Proposals**: Render executable cards (paper order, watchlist, alert) requiring explicit user confirmation before dispatching.
- [ ] **7.4 Grounding & Citations**: Connect filings RAG quotes with verbatim verification directly into agent message cards.
- [ ] **7.5 MCP Integration**: Expose Honba tool catalog over Model Context Protocol (`honba-tamashii`) for Claude Code, Claude Desktop, and local LLM clients.

### Phase 8: Quality, Performance Benchmarks, Accessibility & E2E Polish
- [ ] **8.1 Component Unit Tests**: Complete unit tests for all `honba-kao-ui-core` primitives using `leptos_testing_library`.
- [ ] **8.2 Playwright E2E Suite**: Comprehensive end-to-end test scenarios:
  - Auth flow & protected route redirects.
  - CommandBar GO navigation & ticker search.
  - Security Hub tab switching & filings citation verification.
  - F&O option chain strike range filtering & strategy payoff calculation.
  - Backtesting parameter scan & Monte Carlo execution.
  - Emergency Kill Switch activation in Ops Dashboard.
- [ ] **8.3 Accessibility (a11y)**: Enforce WCAG AA contrast ratios, roving keyboard navigation on tables and menus, and full ARIA role compliance (`check-a11y.sh`).
- [ ] **8.4 Performance Verification**: Sub-100ms Time-to-Interactive (TTI), 60fps tick ingestion without frame reflow, and WASM binary size optimization via `wasm-opt`.

---

## 7. Definition of Done & Success Criteria

1. **OpenTerminalUI Parity**: Complete visual, structural, and workflow parity with OpenTerminalUI (`frontend/src`), with its components ported — not reinvented under a generic branded prefix — into idiomatic Rust / Leptos 0.7 CSR built on the existing `honba-kao-ui-core` primitives.
2. **Design System Discipline**: Zero raw HTML tag violations in application code; all components compose strictly from `honba-kao-ui-core`.
3. **Information Density**: True terminal-grade information density (compact tables, mini sparklines, range bars, high-contrast monospace typography) without clutter.
4. **Verified Research**: Every growth engine, headwind, or thesis evidence claim in the Filings RAG and Research Dossiers links to a verbatim verified quotation in official documents with page numbers.
5. **Real-Time Performance**: Seamless handling of 100+ WebSocket ticks/sec and sub-second chart updates with zero UI lag.
6. **Safety & Auditability**: Every rejected order, risk threshold breach, or kill-switch execution is logged with exact constraint violation details in the OMS compliance audit trail.
7. **Production Build**: Clean trunk build producing optimized WASM assets with automated CI regression testing.
