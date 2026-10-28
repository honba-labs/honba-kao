# honba-kao Screener Layout & Nesting Design

## 1. Executive Summary & Vision

This document details the architectural layout, nesting hierarchy, component structure, and design tokens for aligning `honba-kao`'s screener with the OpenTerminalUI institutional Equity Screener workspace (`/home/pmallapp/tmp/OpenTerminalUI/assets/readme/screener.jpg` and `frontend/src/pages/equity/ScreenerPage.tsx`).

The goal is to elevate `honba-kao` from a legacy static table/heatmap switch to a high-density, real-time institutional screening terminal featuring:
1. **Hero Deck & Preset Shelf**: One-click guru and thematic screens (Buffett Quality, High Growth Tech, Dividend Kings, Graham Net-Net, Momentum Breakout), free-text screen search, and multi-universe selection (`nse_500`, `all_india`, `us_large_cap`).
2. **Screen Setup Deck**:
   - 5 dedicated workspaces: `Library` (Presets), `Custom` (Visual Query Builder), `Custom Formula` (Freeform arithmetic engine with 25+ fundamental/technical metrics), `Saved` (User screens), and `Public` (Community library).
   - Dynamic Active Filter Strip with count badge, pill chips, and expansion toggles (`Edit rules`, `Multi-market`, `AI brief`).
   - Integrated `MultiMarketScanPanel` for cross-exchange scans across India, US, UK, and European markets.
3. **High-Density Results DataGrid & Alternative Visualizations**:
   - 6 switchable view modes: `Table`, `Split`, `Charts`, `Map` (Treemap), `Scatter` (2D Factor Scatter), `Scores` (Radar/Scorecard).
   - Rich DataGrid Columns: Symbol & Company badge, Composite Factor Score (0-100 gradient badge), Factor Driver Chips (`VALUE`, `MOM`, `QUALITY`, `LOW-VOL`), Market Metrics (Price, 3M Return % with directional arrow, 52W Range position bar, 1-Year SVG Sparkline), Fundamental Ratios (P/E, P/B, ROE, ROCE, D/E, Market Cap in ₹Cr / $B), and "Why Ranked" algorithmic rationale.
4. **Docked 360px Company Detail Drawer**:
   - Deep inspection sidebar: Financial metrics 2x2 grid, 1-Year interactive SVG Sparkline, signal layer factor breakdown bars, and 6 quick action triggers (`Open Chart`, `Security Hub`, `Backtest`, `Compare`, `Add Watchlist`, `Create Alert`).
   - Mobile responsive bottom float-bar and slide-over drawer.

---

## 2. Layout & Nesting Architecture

### 2.1 Visual & DOM Hierarchy

```text
<ScreenerPage> (src/features/screener/screener_page.rs)
└── <ScreenerProvider> (Context/Store provider managing Leptos signals)
    └── <div class="sc-screener-workspace"> (max-w-[1680px] responsive shell)
        │
        ├── <section class="sc-hero-deck"> [Hero Header & Quick Presets]
        │   ├── <div class="sc-hero-deck__header">
        │   │   ├── <h1 class="sc-hero-deck__title"> "Equity screener"
        │   │   └── <div class="sc-hero-deck__meta"> "{count} matches · {activeLabel} · {universe} · {execMs} ms"
        │   ├── <div class="sc-hero-deck__controls">
        │   │   ├── <div class="sc-search-input"> [Search screens input with magnifying glass icon]
        │   │   ├── <div class="sc-universe-select"> [Universe dropdown: NSE 500, Large Cap, US, etc.]
        │   │   ├── <SavedViewsControl> [Save / Restore named layout views]
        │   │   └── <TerminalButton variant="accent"> [Run screen action with Play icon]
        │   └── <div class="sc-preset-shelf"> [Horizontal overflow scroll list]
        │       └── For each preset in presets:
        │           └── <button class="sc-preset-card" data-active="...">
        │               ├── <span class="sc-preset-card__name"> (e.g. "Buffett Quality")
        │               └── <span class="sc-preset-card__category"> (e.g. "Guru", "Value")
        │
        ├── <section class="sc-workspace-grid" data-detail-open="..."> [Split Grid: 1fr + 360px]
        │   │
        │   ├── <div class="sc-workspace-primary"> [Main Screen & Results Stream]
        │   │   │
        │   │   ├── <TerminalPanel title="Screen Setup"> [Setup & Filter Deck]
        │   │   │   ├── Header Action: <div class="sc-tab-bar">
        │   │   │   │   ├── <TerminalButton> Library [Sparkles]
        │   │   │   │   ├── <TerminalButton> Custom [SlidersHorizontal]
        │   │   │   │   ├── <TerminalButton> Custom Formula [Columns3]
        │   │   │   │   ├── <TerminalButton> Saved [Bookmark]
        │   │   │   │   └── <TerminalButton> Public [Share2]
        │   │   │   │
        │   │   │   ├── <div class="sc-filter-summary">
        │   │   │   │   ├── <div class="sc-filter-chips">
        │   │   │   │   │   ├── <span class="sc-filter-count"> "{N} active filters"
        │   │   │   │   │   ├── For each chip in activeFilters:
        │   │   │   │   │   │   └── <span class="sc-chip"> "{field} {op} {value}"
        │   │   │   │   │   └── <span class="sc-chip-overflow"> "+N more"
        │   │   │   │   └── <div class="sc-filter-actions">
        │   │   │   │       ├── <TerminalButton> "Edit rules" [ChevronDown]
        │   │   │   │       ├── <TerminalButton> "Multi-market" [ChevronDown]
        │   │   │   │       └── <TerminalButton> "AI brief" [ChevronDown]
        │   │   │   │
        │   │   │   ├── [Expanded Drawer 1] <Show when=query_open>
        │   │   │   │   └── <div class="sc-query-container">
        │   │   │   │       ├── <QueryBar> [Raw query editor textarea + badges + quick run]
        │   │   │   │       └── <Show when=tab == Custom>
        │   │   │   │           ├── <QueryBuilder> [GUI row filter builder: Field | Op | Value | And/Or]
        │   │   │   │           ├── <AlternateDataFilter> [ESG, Institutional flows, Promoter pledge]
        │   │   │   │           ├── <AdvancedArithmetic> [Cross-field arithmetic formulas]
        │   │   │   │           └── <SaveScreenDialog> [Modal/Card to save custom screen]
        │   │   │   │
        │   │   │   ├── [Expanded Drawer 2] <Show when=scan_open>
        │   │   │   │   └── <MultiMarketScanPanel> [Multi-market selector, criteria filters, cross-border scan]
        │   │   │   │
        │   │   │   ├── [Tab Content 1] <Show when=tab == Formula>
        │   │   │   │   └── <CustomFormulaScreener> [Expression input, 25+ metrics cheat sheet, formula grid]
        │   │   │   ├── [Tab Content 2] <Show when=tab == Saved>
        │   │   │   │   └── <SavedScreens> [List of user-saved screens with run/edit/delete]
        │   │   │   ├── [Tab Content 3] <Show when=tab == Public>
        │   │   │   │   └── <PublicScreens> [Community shared screens library]
        │   │   │   └── [Expanded Drawer 3] <Show when=ai_open>
        │   │   │       └── <AiInsightCard> [AI briefing on top matched stocks]
        │   │   │
        │   │   ├── <TerminalPanel title="Results"> [Results Data & Visualizations Deck]
        │   │   │   ├── Header Action: <div class="sc-view-switcher">
        │   │   │   │   ├── <TerminalButton> Table [Table2]
        │   │   │   │   ├── <TerminalButton> Split [LayoutDashboard]
        │   │   │   │   ├── <TerminalButton> Charts [LineChart]
        │   │   │   │   ├── <TerminalButton> Map [Trees]
        │   │   │   │   ├── <TerminalButton> Scatter [BarChart3]
        │   │   │   │   └── <TerminalButton> Scores [Sparkles]
        │   │   │   │
        │   │   │   ├── <Show when=view == Table || view == Split>
        │   │   │   │   └── <ResultsTable>
        │   │   │   │       └── <DataGrid> [Ranked stocks table]
        │   │   │   │           ├── Header: [Ticker, Score, Signals, Price/Move/Trend, P/E, P/B, ROE, ROCE, D/E, Cap, Why, Actions]
        │   │   │   │           └── Rows:
        │   │   │   │               ├── <TickerCell> (symbol, name, market pill)
        │   │   │   │               ├── <ScoreBadge> (composite score 0-100 gradient)
        │   │   │   │               ├── <FactorChips> (VALUE, MOM, QUALITY, LOW-VOL)
        │   │   │   │               ├── <MarketMetricsCell> (Price, Return%, 52W Range Bar, SparklineCell SVG)
        │   │   │   │               ├── <FundamentalCells> (P/E, P/B, ROE%, ROCE%, D/E, Mkt Cap)
        │   │   │   │               ├── <WhyRankedCell> (algorithmic explanation summary)
        │   │   │   │               └── <RowActions> (Chart, Hub, Backtest, Watchlist, Alert icons)
        │   │   │   │
        │   │   │   └── <Show when=view != Table>
        │   │   │       └── <ScreenVizLoader> [Interactive SVG/Canvas visualizers]
        │   │   │           ├── SectorTreemap (hierarchical market cap map)
        │   │   │           ├── ScatterPlot (2D factor correlation scatter)
        │   │   │           ├── RadarScorecard (5-factor pentagon radar)
        │   │   │           ├── CandlestickGallery (compact multi-stock mini charts)
        │   │   │           └── DistributionHistogram (metric distribution curves)
        │   │   │
        │   │   └── <StatusBar> [Execution metrics, record count, universe, pagination]
        │   │
        │   └── <aside class="sc-workspace-secondary"> [Docked 360px Detail Drawer]
        │       └── <Show when=selected_row.is_some()>
        │           └── <CompanyDetailDrawer>
        │               ├── Header: [Company Name, Ticker badge, Sector, Exchange, Close button]
        │               ├── Metric Tiles (2x2): [Sector, Market Cap, P/E, ROE]
        │               ├── Action Grid (2x3): [Open Chart, Security Hub, Backtest, Compare, Add Watchlist, Create Alert]
        │               ├── Trend Panel: <SparklineCell> [1-Year SVG price curve]
        │               ├── Signal Layer: [Factor Chips + Factor Score Rating Bars]
        │               ├── Deep Financials: [Valuation & Growth key ratios]
        │               └── Why Ranked: [Natural language explainability card]
        │
        └── <div class="sc-mobile-drawer"> [Mobile Fixed Bottom Bar & Slide-over Sheet]
```

---

## 3. Component Specifications

### 3.1 Hero Deck & Preset Scroller (`sc-hero-deck`)
- **Metadata Header**: Renders screen title, total matches formatted in Indian / US numbering, active preset label, universe tag, and query execution duration in milliseconds.
- **Controls Row**:
  - `Search screens` input with inline magnifying glass icon and debounced filter across preset names, categories, and descriptions.
  - `Universe` dropdown: selects target equity universe (`NSE 500`, `NIFTY 50`, `BSE AllCap`, `US LargeCap`).
  - `SavedViewsControl`: preserves filter settings, active tabs, view mode, and selected ticker.
  - `Run screen` button with `<Play />` icon, accent tone, and loading spinner state.
- **Preset Shelf**: Horizontally scrolling card rack (`overflow-x-auto pb-1 flex gap-2`) with preset name, category badge (`Guru`, `Value`, `Quality`, `Momentum`), and active selection state.

### 3.2 Screen Setup Deck (`sc-screen-setup`)
- **Tab Bar**: 5 primary workflow modes:
  1. `Library`: Default view displaying active preset rules.
  2. `Custom`: Interactive visual query builder.
  3. `Custom Formula`: Freeform mathematical expression evaluator.
  4. `Saved`: Saved user screens with edit/delete actions.
  5. `Public`: Community curated screens.
- **Active Filter Summary Bar**:
  - Filter count: `X active filters`.
  - Filter chips: Individual clauses (e.g. `Market Cap > 500`, `ROE > 15%`) with overflow indicator (`+N more`).
  - Quick action toggles: `Edit rules` (ChevronDown), `Multi-market` (ChevronDown), `AI brief` (ChevronDown).
- **Expandable Drawers**:
  - `QueryBar`: Syntax editor textarea with status badges and quick run action.
  - `QueryBuilder`: Visual GUI row editor with Field, Operator (`>`, `>=`, `<`, `<=`, `=`, `!=`), Value, and AND/OR join combinator.
  - `CustomFormulaScreener`: Expression input with 25+ metric tokens cheat sheet (`pe`, `pb`, `roe`, `roce`, `fcf`, `ebitda_margin`, `debt_equity`, `rsi`, `beta`) and validation.
  - `MultiMarketScanPanel`: Multi-region screening panel supporting cross-border scans (India, US, UK, EU, Asia).

### 3.3 Results Panel & Visualizations (`sc-results-deck`)
- **View Switcher**: 6 switchable display modes:
  - `Table`: Standard high-density DataGrid.
  - `Split`: Side-by-side DataGrid and visualization view.
  - `Charts`: Candlestick mini-chart gallery.
  - `Map`: Sector and market cap hierarchical Treemap.
  - `Scatter`: 2D Factor correlation scatter plot (e.g. Valuation vs Momentum).
  - `Scores`: 5-factor pentagon Radar scorecard grid.
- **Results DataGrid Columns**:
  1. `Ticker`: Symbol, company name, exchange/market badge (`India`, `US`).
  2. `Score`: Composite factor score rendered with `ScoreBadge` (0–100 tonal color scale).
  3. `Signals`: Factor driver badges (`VALUE`, `MOM`, `QUALITY`, `LOW-VOL`).
  4. `Market Metrics`: Last price, 3M return % with trend arrow, 52W Range horizontal bar (`RangeBarCell`), and 1-Year SVG sparkline (`SparklineCell`).
  5. `Fundamentals`: P/E, P/B, ROE %, ROCE %, Debt/Equity, Market Cap formatted in Indian Lakhs/Crores (`₹Cr`, `₹L Cr`) or US Dollars (`$B`, `$M`).
  6. `Why Ranked`: Natural language explanation of rank drivers.
  7. `Row Actions`: Quick action buttons (`Open Chart`, `Security Hub`, `Backtest`, `Watchlist`, `Alert`).

### 3.4 Company Detail Drawer (`sc-detail-drawer`)
- **Docked 360px Sidebar**: Automatically docks on the right when a row is clicked in the DataGrid.
- **Header**: Selected company title, ticker badge, sector badge, exchange badge, and close button.
- **Metric Tiles (2x2)**: Sector, Market Cap, P/E, ROE.
- **Action Grid (2x3)**:
  - `Open Chart`: navigates to `/equity/chart-workstation?symbol={ticker}`.
  - `Security Hub`: navigates to `/equity/security/{ticker}`.
  - `Backtest`: navigates to `/backtesting?symbol={ticker}`.
  - `Compare`: navigates to `/equity/chart-workstation?compare=true`.
  - `Add Watchlist`: triggers direct watchlist API call with confirmation feedback.
  - `Create Alert`: navigates to `/equity/alerts?symbol={ticker}`.
- **Trend Panel**: 1-Year interactive SVG Sparkline (width: 240px, height: 60px).
- **Signal Layer**: Active factor chips + individual factor score rating bars (Value, Momentum, Quality, Low-Vol).
- **Why Ranked Rationale**: Explanatory breakdown card justifying the asset's inclusion and ranking in the screen.

---

## 4. Design Tokens & Styling Contract

All styles are scoped under the `honba-kao` 256-token contract and `.sc-*` class namespace:

| Category | Token / Class | Description |
| :--- | :--- | :--- |
| **Surfaces** | `var(--canvas)`, `var(--surface-1)`, `var(--surface-2)` | Main background, panels, and elevated cards |
| **Borders** | `var(--border-color)`, `var(--border-subtle)` | Subtle structural divider borders |
| **Typography** | `var(--text-color-primary)`, `var(--text-color-muted)` | High-contrast and subdued data labels |
| **Accents** | `var(--primary-8)`, `var(--primary-6)` | Interactive highlights and active tab indicators |
| **Tones** | `var(--pnl-gain)` (green), `var(--pnl-loss)` (red), `var(--warning)` (amber) | Price moves, score badge grades, and signals |

### ScoreBadge Grades
- **High ($\ge 75$)**: `.badge-high` with `rgba(0, 230, 118, 0.15)` background and `var(--pnl-gain)` text.
- **Mid ($50 - 74$)**: `.badge-mid` with `rgba(255, 215, 64, 0.15)` background and `var(--warning)` text.
- **Low ($< 50$)**: `.badge-low` with `rgba(255, 82, 82, 0.15)` background and `var(--pnl-loss)` text.

---

## 5. File Structure in `honba-kao`

```text
honba-kao/
├── crates/
│   ├── honba-kao-ui-core/src/screener/
│   │   ├── mod.rs
│   │   ├── score_badge.rs          # 0-100 composite score badge with 3-tier tone
│   │   ├── sparkline_cell.rs       # Reusable SVG sparkline chart with gradient
│   │   ├── range_bar_cell.rs       # 52-week price position bar
│   │   └── factor_chips.rs         # Factor driver pill badges
│   │
│   └── honba-kao-app/src/features/screener/
│       ├── screener_page.rs        # Page container & split grid
│       ├── stores/
│       │   └── screener_store.rs   # Expanded Leptos reactive signals & API actions
│       └── components/
│           ├── hero_deck.rs        # Header ribbon & PresetShelf scroller
│           ├── screen_setup.rs     # Tab bar (Library/Custom/Formula/Saved/Public)
│           ├── query_builder.rs    # Visual GUI filter builder & QueryBar
│           ├── custom_formula.rs   # Custom formula engine & token sheet
│           ├── multi_market_scan.rs# Multi-exchange cross-border scan
│           ├── results_table.rs    # High-density DataGrid
│           ├── viz_loader.rs       # SectorTreemap, ScatterPlot, RadarScorecard
│           ├── detail_drawer.rs    # Docked 360px company drawer
│           └── status_bar.rs       # Bottom execution metrics bar
│
└── style/
    └── screener.css                # 256-token CSS contract under .sc-* prefix
```

---

## 6. GitHub Milestones & Epic Tracking

This architectural design is tracked under GitHub Epic **#11** in `honba-labs/honba-kao`:

- **[Epic #11: Align Screener Layout, Components, and Interaction with OpenTerminalUI Screener](https://github.com/honba-labs/honba-kao/issues/11)**
  - **[Issue #7: [UI-Core & Styles] Port Screener Primitives & 256-Token Tokens (ScoreBadge, SparklineCell, RangeBarCell, CSS)](https://github.com/honba-labs/honba-kao/issues/7)**
  - **[Issue #8: [Store & API] Expand ScreenerStore for Dynamic Presets, Universes, Query State & Run Execution](https://github.com/honba-labs/honba-kao/issues/8)**
  - **[Issue #9: [App] Implement Hero Deck, Preset Shelf, QueryBuilder & Custom Formula Screener](https://github.com/honba-labs/honba-kao/issues/9)**
  - **[Issue #10: [App] Implement Results DataGrid, Visualization Loader, and Docked Company Detail Drawer](https://github.com/honba-labs/honba-kao/issues/10)**
