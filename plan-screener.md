# Plan: Align Screener Layout, Components, and Interaction with OpenTerminalUI

## 1. Executive Summary & Context

The current `honba-kao` screener (`crates/honba-kao-app/src/features/screener/`) implements a legacy/labs market view consisting of a simple filter bar, table/heatmap toggle, and modals. 

In contrast, OpenTerminalUI (`/home/pmallapp/tmp/OpenTerminalUI/assets/readme/screener.jpg` and `frontend/src/pages/equity/ScreenerPage.tsx`) implements a comprehensive **Institutional Equity Screener Workspace** featuring:
1. **Hero Header & Preset Scroller**: Single-click guru and thematic presets (Buffett Quality, High Growth Tech, Dividend Kings, Graham Net-Net, Momentum Breakout), search bar, and screening universe picker (`nse_500`, `all_india`, `us_large_cap`).
2. **Screen Setup Deck**:
   - 5 Tabbed Workspaces: `Library` (Presets), `Custom` (Visual Query Builder), `Custom Formula` (Freeform arithmetic engine with 25+ fundamental/technical metrics), `Saved` (User screens), and `Public` (Shared community screens).
   - Dynamic Filter Summary Bar with active chips and quick expansion toggles (`Edit rules`, `Multi-market`, `AI brief`).
   - Integrated `MultiMarketScanPanel` for cross-exchange screening across India, US, UK, Europe, and Asia.
3. **High-Density Results DataGrid & Visualizations**:
   - 6 View Modes: `Table`, `Split`, `Charts`, `Map` (Treemap), `Scatter` (2D Factor Scatter), `Scores` (Radar/Scorecard).
   - Rich DataGrid Columns: Symbol & Company badge, Composite Factor Score (0-100 gradient badge), Factor Driver Chips (`VALUE`, `MOM`, `QUALITY`, `LOW-VOL`), Market Metrics (Price, 3M Return % with trend pill, 52W Range gauge, 1-Year SVG Sparkline), Valuation & Growth metrics (P/E, P/B, ROE, ROCE, D/E, Market Cap), and "Why Ranked" natural language rationale.
4. **Docked 360px Company Detail Drawer**:
   - Deep inspection sidebar: Financial metrics 2x2 grid, 1-Year interactive SVG Sparkline, signal layer factor breakdown bars, and 6 quick action buttons (`Open Chart`, `Security Hub`, `Backtest`, `Compare`, `Add Watchlist`, `Create Alert`).
   - Mobile responsive bottom float-bar and slide-over drawer.

This plan details the full layout/nesting design and work breakdown to achieve complete fidelity with OpenTerminalUI in `honba-kao`'s Leptos 0.8 CSR architecture.

---

## 2. Layout, Nesting Hierarchy & Component Breakdown

### 2.1 DOM & Component Nesting Tree

```text
<ScreenerPage> (src/features/screener/screener_page.rs)
├── <ScreenerProvider> (Context/Store provider managing signals)
└── <div class="sc-screener-workspace"> (max-w-[1680px] responsive container)
    │
    ├── <section class="sc-hero-deck"> [Hero Header & Quick Presets]
    │   ├── <div class="sc-hero-deck__header">
    │   │   ├── <h1 class="sc-hero-deck__title"> "Equity screener"
    │   │   └── <div class="sc-hero-deck__meta"> "{count} matches · {activeLabel} · {universe} · {execMs} ms"
    │   ├── <div class="sc-hero-deck__controls">
    │   │   ├── <div class="sc-search-input"> [Search screens input with icon]
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

## 3. UI Elements & Tokens Specification

### 3.1 Design Token Contract (256-token compliant)
All styles strictly adhere to the `honba-kao` 256-token contract with `sc-` namespace:
- **Canvas & Surfaces**: `var(--canvas)`, `var(--surface-1)`, `var(--surface-2)`, `var(--surface-3)`
- **Borders & Dividers**: `var(--border-color)`, `var(--border-subtle)`
- **Text & Foreground**: `var(--text-color-primary)`, `var(--text-color-muted)`, `var(--text-color-dim)`
- **Accents & States**: `var(--primary-8)`, `var(--primary-6)`, `var(--accent-hover)`
- **Financial Directional Tones**: `var(--pnl-gain)` (green), `var(--pnl-loss)` (red), `var(--warning)` (amber)

### 3.2 Key UI Primitives
- **`ScoreBadge`**: Renders 0-100 composite score with dynamic tone:
  - $\ge 75$: High (`badge-high`, gain tone)
  - $50 - 74$: Mid (`badge-mid`, amber tone)
  - $< 50$: Low (`badge-low`, loss tone)
- **`SparklineCell`**: Lightweight SVG line chart (width: 80px in table, 240px in drawer) with fill gradient and start-to-end delta calculation.
- **`RangeBarCell`**: 52-week price position bar displaying low, high, and current price marker with trend state label (`near high`, `near low`, `uptrend`, `downtrend`, `range bound`).
- **`FactorChips`**: Pill badges for active factor drivers (`VALUE`, `MOM`, `QUALITY`, `LOW-VOL`).

---

## 4. Work Breakdown & Milestones

### Milestone 1: UI Primitives & Styling (`honba-kao-ui-core` & `style/screener.css`)
- [ ] **1.1 `ScoreBadge`**: Port composite score badge with 0-100 score normalization and high/mid/low tone coloring.
- [ ] **1.2 `SparklineCell`**: SVG sparkline renderer supporting 1Y historical series, fill gradient, and min/max baseline.
- [ ] **1.3 `RangeBarCell`**: 52-week low-to-high position bar with current price pip indicator.
- [ ] **1.4 Stylesheet (`screener.css`)**: Implement `.sc-hero-deck`, `.sc-preset-shelf`, `.sc-workspace-grid`, `.sc-tab-bar`, `.sc-chip`, and table cells under 256-token contract.

### Milestone 2: Reactive State & API Integration (`ScreenerStore` / `ScreenerContext`)
- [ ] **2.1 Signals & Store Expansion**: Add reactive signals for `presets`, `universes`, `selected_preset_id`, `tab`, `view`, `query`, `results`, `selected_row`, `query_open`, `scan_open`, and `ai_open`.
- [ ] **2.2 API Client Bindings**: Integrate endpoints for presets, universes, running screen query (`run_screener_v3`), custom formulas, and multi-market scan.
- [ ] **2.3 Persistence & Saved Views**: LocalStorage backup for custom saved screens and layout view restoration.

### Milestone 3: Hero Header, Preset Shelf & Query Configuration Deck
- [ ] **3.1 `HeroDeck` & `PresetShelf`**: Header metadata bar, universe selector, search input, and horizontally scrolling preset cards with category chips.
- [ ] **3.2 `ScreenSetup` & Tab Navigation**: Tab bar (`Library`, `Custom`, `Formula`, `Saved`, `Public`) and active filter summary chips.
- [ ] **3.3 `QueryBar` & `QueryBuilder`**: Expandable raw query editor and GUI row-by-row filter builder (Field, Operator, Value, Combinator).
- [ ] **3.4 `CustomFormulaScreener`**: Custom mathematical formula input, 25+ metric tokens cheat sheet, and execution engine.
- [ ] **3.5 `MultiMarketScanPanel`**: Multi-region screening panel supporting India, US, UK, and EU markets.

### Milestone 4: Results DataGrid, Visualizations & Detail Drawer
- [ ] **4.1 `ResultsTable`**: High-density DataGrid displaying ticker badges, score badges, market metrics, fundamental ratios, why-ranked explanations, and action triggers.
- [ ] **4.2 Visualizations (`VizPanel` / `ScreenVizLoader`)**: Treemap (Sector concentration), Scatter Plot (Factor distribution), and Radar Scorecard.
- [ ] **4.3 `CompanyDetailDrawer`**: Docked 360px sidebar with 2x2 financial metrics, 2x3 navigation launchpad, 1Y sparkline, and factor score breakdown.
- [ ] **4.4 `StatusBar` & Mobile Sheet**: Bottom execution status indicators and responsive mobile drawer.

### Milestone 5: Verification & Quality Assurance
- [ ] **5.1 Unit & Component Tests**: Test query builder synchronization, formula validation, and store signal invariants.
- [ ] **5.2 CI Conformance**: Verify `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --workspace`.
