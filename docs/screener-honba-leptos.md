# Institutional Screener & Technical Scanner: `honba-kao-app` (Leptos CSR)

**Target Crate**: `crates/honba-kao-app`  
**Target Route**: `/screener` (`crates/honba-kao-app/src/features/screener/`)  
**Design Inspirations**:  
- `OpenTerminalUI` (`$HOME/tmp/OpenTerminalUI`): Terminal ergonomics, explainable scanner detectors, progressive depth, multi-market scope.  
- `Secured-Finance-App` (`$HOME/tmp/secured-finance-app`): Atomic design (54 atoms in `honba-kao-ui-core`), institutional tabular density, exact decimal financial precision.  
- `OpenAlgo` (`$HOME/tmp/openalgo`): High-throughput real-time scanner architecture, deterministic indicator pack, live tick streaming.  
**Runtime**: Leptos 0.8 CSR (WASM32), Rust 2024 edition, Zero Floats (`rust_decimal::Decimal`).  
**Status**: Specification & Architectural Blueprint.

---

## 1. Executive Summary & Core Philosophy

The `honba-kao-app` screener is an institutional-grade security discovery and technical scanner workstation. Designed for quantitative researchers, algorithmic traders, and portfolio managers, it bridges macro scanning with micro execution.

### Architectural Invariants:
1. **Zero Floats Policy**: Every monetary value, index point, valuation ratio, percentage change, yield, and indicator calculation uses `rust_decimal::Decimal`. Floating-point IEEE-754 approximations (`f64`/`f32`) are strictly forbidden in financial calculations and display layers.
2. **Atomic Design System**: All UI elements inherit directly from `crates/honba-kao-ui-core` (Atoms, Molecules, Organisms, Templates) preserving strict design consistency with the rest of the Honba workstation suite.
3. **Progressive Depth**: Surface immediate, critical decisions first (Asset class, Quick Presets, 12 domain tabs, Sector/Cap breakdown), and expose granular multi-predicate filtering and technical indicator math on demand.
4. **Scan Before Configure & Explainability**: Traders must understand *why* a security matched without leaving the screen. Every scanner detector emits an `explain_steps` breakdown.
5. **Fast Next Action**: Seamless routing from any screener row into `/cockpit`, `/strategy-studio`, `/simulator`, `/derivatives`, or `/oms`.

---

## 2. Tri-Pillar Inspirations & Synthesized Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    HONBA SCREENER SYNTHESIS MATRIX                                      │
├──────────────────────────┬───────────────────────────────┬──────────────────────────────────────────────┤
│ Source Codebase          │ Core Capabilities & Patterns  │ Direct Synthesis into `honba-kao-app`        │
├──────────────────────────┼───────────────────────────────┼──────────────────────────────────────────────┤
│ OpenTerminalUI           │ • Research terminal UX        │ • Explainability panel (`explain_steps`)     │
│ ($HOME/tmp/              │ • Scanner detectors & alerts  │ • Quick Presets ("Breakout", "NR7", "Value") │
│  OpenTerminalUI)         │ • Natural Language / AI Bar   │ • `✦ AI` natural language filter bar         │
│                          │ • Multi-asset / Multi-market  │ • NSE/BSE + Global equities, ETF, Bonds, MF  │
│                          │ • Table ↔ Heatmap dual view   │ • Cap-weighted / Sector-grouped treemap      │
├──────────────────────────┼───────────────────────────────┼──────────────────────────────────────────────┤
│ Secured-Finance-App      │ • 54 Atomic UI components     │ • Uses `honba-kao-ui-core::atoms::*`         │
│ ($HOME/tmp/              │ • Institutional Dark Theme    │ • Table cells with sort arrows, chips, badges│
│  secured-finance-app)    │ • Exact BigInt/Decimal tokens │ • Day & 52W range progress bars              │
│                          │ • Density & tabular layout    │ • Monospace figures, zero visual clutter     │
├──────────────────────────┼───────────────────────────────┼──────────────────────────────────────────────┤
│ OpenAlgo                 │ • Real-time scanner service   │ • Live tick-flashing (green/red background)  │
│ ($HOME/tmp/              │ • Deterministic indicators    │ • RSI14, EMA 9/21/50/200, Supertrend, RVOL   │
│  openalgo)               │ • Zero rate-limit tick stream │ • WebSocket tick subscription integration    │
└──────────────────────────┴───────────────────────────────┴──────────────────────────────────────────────┘
```

---

## 3. Workstation Layout & Visual Wireframe

The screener operates as a high-density, full-bleed workstation layout divided into three interactive zones:

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ ZONE 1: TOP FILTER COMMAND BAR                                                                                   │
│ [Stocks | ETF | Bonds | MF]  |  [All Presets ▼] [Breakouts] [52W High] [NR7 Squeeze] [Value] [High Div] [✦ AI]   │
│ [Overview] [Performance] [Technicals] [Valuation] [Dividends] [Profitability] [Financials] [Per Share] [Forecasts]│
│ [🔍 Search Ticker/Name] [🇮🇳 Market: NSE/BSE ▼] [Index: All ▼] [Sector: All ▼] [Cap: Large+Mid ▼] [Filters (3)]   │
│ View: [▦ Grid | ☷ Heatmap] | Matches: 30 / 30 | [Reset] [Columns] [Export CSV]                                   │
├────────────────────────────────────────────────────────────────────────┬─────────────────────────────────────────┤
│ ZONE 2: PRIMARY WORKSPACE (~70% Resizable)                             │ ZONE 3: DETAIL & EXPLAIN DRAWER (~30%)  │
│                                                                        │                                         │
│ ┌─ Mode A: HIGH-DENSITY VIRTUALIZED TABLE GRID ──────────────────────┐ │ [★] RELIANCE-EQ  NSE 🇮🇳 [X]            │
│ │ [★] Ticker ▲  Price      Chg%    Vol    Cap       P/E  52W Range   │ │ Reliance Industries Limited             │
│ │ ────────────────────────────────────────────────────────────────── │ │                                         │
│ │ [★] RELIANCE  ₹2,842.50  +1.87%  42L    ₹19.2L Cr 24.2 [══•══] 📈  │ │ ₹2,842.50  ▲ +₹52.30 (+1.87%) Live     │
│ │ [★] TCS       ₹3,920.10  -0.40%  18L    ₹14.1L Cr 28.5 [═•═══] 📉  │ │ [Cockpit] [Hub] [F&O] [Studio] [Order]  │
│ │ [★] HDFCBANK  ₹1,680.00  +0.75%  65L    ₹12.8L Cr 19.8 [══•══] 📈  │ │ ─────────────────────────────────────── │
│ │ [★] INFY      ₹1,745.20  +2.30%  35L    ₹7.2L Cr  26.1 [════•] 🚀  │ │ Chart: [1D | 1W | 1M | 1Y] [Candles|Line]│
│ │ ...                                                                │ │ [openalgo-charts mini interactive chart]│
│ └────────────────────────────────────────────────────────────────────┘ │ ─────────────────────────────────────── │
│                                                                        │ Technical Consensus: [Strong Buy (14/2)]│
│ ┌─ Mode B: MARKET HEATMAP (Sector Grouped / Cap Weighted) ───────────┐ │ Day Range:  ₹2,790.00 [══•═══] ₹2,860.00 │
│ │ ┌─ Energy (₹19.2L Cr) ─────────┐ ┌─ IT Services (₹21.3L Cr) ─────┐ │ │ 52W Range:  ₹2,220.00 [════•═] ₹3,020.00 │
│ │ │ [RELIANCE +1.87% (Large)]    │ │ [TCS -0.40%] [INFY +2.30%]    │ │ │ ─────────────────────────────────────── │
│ │ └──────────────────────────────┘ └───────────────────────────────┘ │ │ Explainability Breakdown:               │
│ └────────────────────────────────────────────────────────────────────┘ │ • Price > SMA 200 (₹2,842.50 > ₹2,610)   │
│                                                                        │ • BB Squeeze: Bandwidth 4.2% < 5.0%     │
│                                                                        │ • RVOL: 2.1x (42L vs 20D Avg 20L)       │
│                                                                        │ [⚡ Create Scanner Alert]                │
└────────────────────────────────────────────────────────────────────────┴─────────────────────────────────────────┘
```

---

## 4. Data Models & Zero-Floats Contract

All models reside in `crates/honba-kao-app/src/features/screener/types.rs`. Pure `Decimal` eliminates rounding artifacts across currency and ratio conversions.

### 4.1 Asset Types & View Enumerations

```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScreenerAssetType {
    #[default]
    Stocks,
    Etf,
    Bonds,
    MutualFunds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScreenerViewMode {
    #[default]
    Table,
    Heatmap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScreenerViewTab {
    #[default]
    Overview,
    Performance,
    Technicals,
    Valuation,
    Dividends,
    Profitability,
    Financials,
    IncomeStatement,
    BalanceSheet,
    CashFlow,
    PerShare,
    Forecasts,
}
```

### 4.2 Equity Record Specification (50+ Decimal Metrics)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerEquityRecord {
    // Identity
    pub symbol: String,
    pub name: String,
    pub exchange: String,      // "NSE", "BSE", "NASDAQ", "NYSE"
    pub sector: String,
    pub industry: String,
    pub market_cap_category: String, // "Large Cap", "Mid Cap", "Small Cap"
    pub is_fno: bool,
    pub is_watchlist: bool,

    // Real-Time Price & Volume
    pub price: Decimal,
    pub change: Decimal,
    pub change_pct: Decimal,
    pub volume: u64,
    pub avg_volume_20d: u64,
    pub rvol: Decimal,         // Relative Volume (Current / 20D Avg)
    pub turnover_cr: Decimal,
    pub day_low: Decimal,
    pub day_high: Decimal,
    pub high_52w: Decimal,
    pub low_52w: Decimal,

    // Performance Returns
    pub return_1d: Decimal,
    pub return_1w: Decimal,
    pub return_1m: Decimal,
    pub return_3m: Decimal,
    pub return_6m: Decimal,
    pub return_ytd: Decimal,
    pub return_1y: Decimal,
    pub return_3y: Decimal,

    // Technical Indicators
    pub rsi_14: Decimal,
    pub macd_line: Decimal,
    pub macd_signal: Decimal,
    pub macd_hist: Decimal,
    pub sma_20: Decimal,
    pub sma_50: Decimal,
    pub sma_200: Decimal,
    pub ema_9: Decimal,
    pub ema_21: Decimal,
    pub atr_14: Decimal,
    pub atr_pct: Decimal,
    pub bb_upper: Decimal,
    pub bb_lower: Decimal,
    pub bb_bandwidth: Decimal,
    pub supertrend_direction: i8, // +1 Buy, -1 Sell

    // Valuation & Fundamentals
    pub market_cap_cr: Decimal,
    pub pe_ratio: Decimal,
    pub forward_pe: Decimal,
    pub pb_ratio: Decimal,
    pub ps_ratio: Decimal,
    pub ev_ebitda: Decimal,
    pub peg_ratio: Decimal,

    // Dividends & Yield
    pub dividend_yield: Decimal,
    pub dividend_per_share: Decimal,
    pub payout_ratio: Decimal,

    // Profitability & Financial Health
    pub roe: Decimal,
    pub roce: Decimal,
    pub operating_margin: Decimal,
    pub net_margin: Decimal,
    pub debt_to_equity: Decimal,
    pub current_ratio: Decimal,
    pub interest_coverage: Decimal,

    // Growth & Per Share (Cr)
    pub revenue_cr: Decimal,
    pub revenue_growth_yoy: Decimal,
    pub net_profit_cr: Decimal,
    pub eps: Decimal,
    pub fcf_cr: Decimal,
    pub book_value_per_share: Decimal,
}
```

### 4.3 Multi-Asset Records: ETFs, Bonds, and Mutual Funds

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerEtfRecord {
    pub symbol: String,
    pub name: String,
    pub underlying_index: String,
    pub nav: Decimal,
    pub ltp: Decimal,
    pub premium_discount_pct: Decimal,
    pub aum_cr: Decimal,
    pub expense_ratio: Decimal,
    pub tracking_error: Decimal,
    pub return_1y: Decimal,
    pub volume: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerBondRecord {
    pub isin: String,
    pub issuer_name: String,
    pub bond_type: String,       // "Sovereign G-Sec", "Corporate AAA", "SDL"
    pub coupon_rate: Decimal,
    pub yield_to_maturity: Decimal,
    pub face_value: Decimal,
    pub clean_price: Decimal,
    pub maturity_date: String,
    pub modified_duration: Decimal,
    pub credit_rating: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerMfRecord {
    pub scheme_code: String,
    pub scheme_name: String,
    pub category: String,        // "Large Cap Flexi", "Small Cap", "Arbitrage"
    pub nav: Decimal,
    pub aum_cr: Decimal,
    pub expense_ratio: Decimal,
    pub cagr_3y: Decimal,
    pub cagr_5y: Decimal,
    pub risk_grade: String,
}
```

### 4.4 Technical Scanner Detectors & Explainability

Inspired by `OpenTerminalUI/backend/scanner_engine/detectors.py` and `openalgo/docs/scanner-architecture.md`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainStep {
    pub condition: String,       // e.g. "RSI(14) > 55.00"
    pub actual_value: Decimal,   // e.g. 62.40
    pub threshold: Decimal,      // e.g. 55.00
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerResult {
    pub symbol: String,
    pub setup_type: String,      // "BB_SQUEEZE_BREAKOUT", "NR7_EXPANSION", "TREND_RETEST"
    pub score: Decimal,          // Stable 0.00..100.00 rank score
    pub trigger_level: Decimal,  // Entry pivot level
    pub invalidation_level: Decimal, // Stop / risk level
    pub explain_steps: Vec<ExplainStep>,
}
```

---

## 5. Leptos 0.8 CSR Reactive Architecture

### 5.1 Signal Graph & State Containers

In `crates/honba-kao-app/src/features/screener/screener_page.rs`:

```rust
use leptos::prelude::*;
use crate::features::screener::types::*;

#[component]
pub fn ScreenerPage() -> impl IntoView {
    // 1. Core Selection Signals
    let (asset_type, set_asset_type) = signal(ScreenerAssetType::Stocks);
    let (view_mode, set_view_mode) = signal(ScreenerViewMode::Table);
    let (active_tab, set_active_tab) = signal(ScreenerViewTab::Overview);
    let (active_symbol, set_active_symbol) = signal(Some("RELIANCE".to_string()));
    let (drawer_open, set_drawer_open) = signal(true);
    let (filter_modal_open, set_filter_modal_open) = signal(false);
    let (catalog_modal_open, set_catalog_modal_open) = signal(false);

    // 2. Filter Criteria State
    let (search_query, set_search_query) = signal(String::new());
    let (selected_exchange, set_selected_exchange) = signal("ALL".to_string());
    let (selected_sector, set_selected_sector) = signal("ALL".to_string());
    let (selected_market_cap, set_selected_market_cap) = signal("ALL".to_string());
    let (watchlist_only, set_watchlist_only) = signal(false);
    let (ai_prompt, set_ai_prompt) = signal(String::new());
    let (ai_bar_visible, set_ai_bar_visible) = signal(false);

    // 3. Sorting & Pagination
    let (sort_col, set_sort_col) = signal("market_cap_cr".to_string());
    let (sort_ascending, set_sort_ascending) = signal(false);

    // 4. Reactive Filter & Sort Pipeline Memo
    let filtered_records = Memo::new(move |_| {
        // Deterministic filtering across search, sector, cap, and criteria
        // Followed by stable decimal sort
    });

    view! {
        <div class="screener-workstation-container flex flex-col h-full w-full bg-[var(--hf-bg-base)] text-[var(--hf-text-primary)]">
            <ScreenerFilterBar ... />
            <div class="screener-body flex flex-1 overflow-hidden">
                <div class="screener-main-workspace flex-1 overflow-auto">
                    {move || match view_mode.get() {
                        ScreenerViewMode::Table => view! { <ScreenerTable ... /> }.into_any(),
                        ScreenerViewMode::Heatmap => view! { <ScreenerHeatmap ... /> }.into_any(),
                    }}
                </div>
                {move || drawer_open.get().then(|| view! {
                    <ScreenerDetailDrawer ... />
                })}
            </div>
            {move || filter_modal_open.get().then(|| view! { <ScreenerFilterModal ... /> })}
            {move || catalog_modal_open.get().then(|| view! { <ScreenCatalogModal ... /> })}
        </div>
    }
}
```

---

## 6. UI Component Decomposition & Atomic Wiring

All screener components utilize atoms from `honba-kao-ui-core::atoms::*`:

| Screener Sub-Component | Path | Key UI-Core Atoms Used |
| :--- | :--- | :--- |
| **Filter Command Bar** | `components/filter_bar.rs` | `HfButton`, `HfChip`, `HfInput`, `HfSelect`, `HfKbdBadge`, `HfSegmentedControl` |
| **Grid Table** | `components/screener_table.rs` | `HfTable`, `HfTableHeader`, `HfTableRow`, `HfTableCell`, `HfSortArrows`, `HfStatusChip` |
| **Market Heatmap** | `components/screener_heatmap.rs` | `HfCard`, `HfAssetBadge`, `HfColorBar` |
| **Detail Drawer** | `components/detail_drawer.rs` | `HfDrawer`, `HfButton`, `HfCloseButton`, `HfPercentSlider`, `HfStatusLed`, `HfBadge` |
| **Preset Catalog Modal**| `components/screen_catalog_modal.rs`| `HfModal`, `HfDialog`, `HfInput`, `HfButton`, `HfKbdBadge` |
| **Column Manager Modal**| `components/column_modal.rs` | `HfModal`, `HfCheckbox`, `HfButton`, `HfSortArrows` |

### 6.1 The 12 Category Tabs Specification

To achieve full parity with institutional terminals (`honba-fe`), the table supports 12 functional views:

1. **Overview**: Ticker, Price, Change%, Volume, RVOL, Market Cap, P/E, 52W Range, Trend Sparkline.
2. **Performance**: Returns across 1D, 1W, 1M, 3M, 6M, YTD, 1Y, 3Y, Volatility (ATR%).
3. **Technicals**: RSI(14), MACD, Supertrend, SMA(20/50/200), EMA(9/21), Bollinger Bandwidth.
4. **Valuation**: P/E, Forward P/E, P/B, P/S, EV/EBITDA, PEG, Free Cash Flow Yield.
5. **Dividends**: Dividend Yield, Payout Ratio, DPS, 5-Year DPS Growth Rate.
6. **Profitability**: ROE, ROCE, Operating Margin, Net Profit Margin.
7. **Financials**: Revenue, YoY Revenue Growth, EBITDA, PAT, EPS.
8. **Income Statement**: Gross Profit, Operating Expenses, EBIT, Net Interest, Tax, Net Income.
9. **Balance Sheet**: Total Debt, Debt/Equity, Current Ratio, Cash & Equivalents, Net Worth.
10. **Cash Flow**: Operating Cash Flow, Capex, Free Cash Flow, Financing Cash Flow.
11. **Per Share**: Cash/Share, Sales/Share, Book Value/Share, FCF/Share.
12. **Forecasts**: Analyst Consensus (Buy/Hold/Sell count), Target Price, Upside/Downside%.

### 6.2 Heatmap Mode: Sector Grouping & Market Cap Sizing

When toggled to `Heatmap` mode:
- Stocks are grouped by `Sector` (e.g. Financials, Energy, Technology, Auto, Pharma).
- Each sector card displays cumulative sector market cap and average change.
- Individual stock tiles are sized proportionally to `market_cap_cr`.
- Tile background colors transition dynamically based on `change_pct`:
  - `> +3.0%`: Deep Emerald Green (`#059669`)
  - `+1.0% to +3.0%`: Mild Emerald (`#10b981`)
  - `0.0% to +1.0%`: Subtle Green (`#064e3b`)
  - `0.0%`: Slate Base (`#1e293b`)
  - `-1.0% to 0.0%`: Subtle Red (`#450a0a`)
  - `-3.0% to -1.0%`: Mild Rose (`#f43f5e`)
  - `< -3.0%`: Deep Crimson (`#e11d48`)
- Clicking any tile selects `active_symbol` and pops the detail drawer.

### 6.3 Explainability Panel & Scanner Alerts

Selecting any matched row displays the exact math that qualified the stock:
```
┌─ RELIANCE-EQ: Breakout Setup ───────────────────────────────────┐
│ Status: QUALIFIED (Score: 88.5)                                 │
│ ─────────────────────────────────────────────────────────────── │
│ [✓] 20-Day High Breakout: ₹2,842.50 > 20D High ₹2,820.00        │
│ [✓] Relative Volume (RVOL): 2.10x > 1.50x Gate (42L vs 20L avg) │
│ [✓] RSI Healthy Momentum: 62.40 (between 55.00 and 70.00)       │
│ [✓] EMA Alignment: Close > EMA 9 > EMA 21 > EMA 50              │
│ ─────────────────────────────────────────────────────────────── │
│ Trigger Level:      ₹2,845.00  (+0.08% away)                    │
│ Invalidation Level: ₹2,810.00  (-1.14% risk)                    │
│ [⚡ Arm Real-Time Scanner Alert]  [📈 Send to Strategy Studio]   │
└─────────────────────────────────────────────────────────────────┘
```

### 6.4 Browser-Native Fast CSV Stream Download

Exporting table data in WASM without server roundtrips:

```rust
#[cfg(target_arch = "wasm32")]
pub fn export_screener_csv(filename: &str, headers: &[&str], rows: &[Vec<String>]) {
    use web_sys::{Blob, BlobPropertyBag, Url};
    
    let mut csv = String::new();
    csv.push_str(&headers.join(","));
    csv.push('\n');
    for row in rows {
        csv.push_str(&row.join(","));
        csv.push('\n');
    }

    let array = js_sys::Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(&csv));

    let mut bag = BlobPropertyBag::new();
    bag.type_("text/csv;charset=utf-8;");
    if let Ok(blob) = Blob::new_with_str_sequence_and_options(&array, &bag) {
        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Ok(elem) = doc.create_element("a") {
                        let _ = elem.set_attribute("href", &url);
                        let _ = elem.set_attribute("download", filename);
                        if let Some(html_elem) = elem.dyn_ref::<web_sys::HtmlElement>() {
                            html_elem.click();
                        }
                    }
                }
            }
            let _ = Url::revoke_object_url(&url);
        }
    }
}
```

---

## 7. Keyboard Navigation & Terminal Shortcuts

Aligned with `OpenTerminalUI` workstation ergonomics:

| Key Binding | Target Scope | Action |
| :--- | :--- | :--- |
| `.` | Screener Workstation | Open Screen Catalog / Preset Selector Dialog |
| `Shift + N` | Screener Workstation | Save active filters and columns as Custom Screen |
| `j` / `Down` | Table Grid | Select next instrument row (updates Detail Drawer) |
| `k` / `Up` | Table Grid | Select previous instrument row |
| `Space` | Table Grid | Toggle Instrument Star / Watchlist flag |
| `Enter` | Selected Row | Route directly to `/cockpit` with active symbol |
| `f` | Table Grid | Open Granular Multi-Predicate Filter Modal |
| `c` | Table Grid | Open Column Configuration Modal |
| `h` | Primary Workspace | Toggle between Table Grid and Market Heatmap |
| `Esc` | Modal / Drawer | Dismiss open modal or collapse detail drawer |
| `Ctrl + K` / `Cmd + K` | Universal | Open Honba Global Command Bar / Quick Find |

---

## 8. Implementation Roadmap & Phasing

```
┌───────────────────────────────────────────────────────────────────────────────────┐
│                               PHASE IMPLEMENTATION TIMELINE                       │
├─────────┬─────────────────────────────┬───────────────────────────────────────────┤
│ Phase 1 │ Foundation & Reactivity Fix │ Fix detail drawer toggle & modal signals  │
│ Phase 2 │ Market Heatmap View         │ Sector treemap tiles with cap sizing      │
│ Phase 3 │ Multi-Asset Universes       │ Pure Decimal datasets for ETF, Bonds, MF  │
│ Phase 4 │ Flyouts & Index Drawer      │ NIFTY 50/500, Sectoral indices, AI Bar    │
│ Phase 5 │ 12 Tabs & Column Controls   │ 50+ financial metrics, sort & filter menu │
│ Phase 6 │ Technical Scanner & Alerts  │ Explainability engine, CSV export, keys   │
└─────────┴─────────────────────────────┴───────────────────────────────────────────┘
```

- **Phase 1: Reactive Plumbing & Modal Triggers**
  - Verify detail drawer opens when clicking a table row.
  - Wire `ScreenerFilterModal` and `ScreenerColumnModal` to signal triggers.
- **Phase 2: Sector-Grouped Heatmap Mode**
  - Implement `screener_heatmap.rs` with responsive flex grid.
  - Implement dynamic RGB color interpolation based on `change_pct`.
- **Phase 3: Multi-Asset Zero-Floats Expansion**
  - Populate deterministic fixtures for 30 Stocks, 15 ETFs, 10 Bonds, 10 Mutual Funds.
  - Implement asset class switcher in top filter bar.
- **Phase 4: Contextual Flyouts & AI Bar**
  - Build popover flyouts for Market, Index, Price range, P/E range.
  - Build `✦ AI` natural language bar connecting to parser AST.
- **Phase 5: 12 Category Tabs & Contextual Column Headers**
  - Wire all 12 view tabs with formatted `Decimal` cells.
  - Add column header dropdown: Sort Asc/Desc, Move Left/Right, Hide, Filter.
- **Phase 6: Technical Scanner Engine & Browser CSV Export**
  - Add explainability drawer card (`explain_steps`).
  - Wire WASM CSV file download helper.
  - Wire keyboard shortcut listener (`.`, `Shift+N`, `j`, `k`, `Space`, `Enter`).

---

## 9. Verification & Gate Standards

Every increment must pass the five canonical Honba quality gates:

```bash
# 1. Native host compilation
cargo check -p honba-kao-app --all-targets

# 2. Browser WASM compilation gate
cargo check -p honba-kao-app --target wasm32-unknown-unknown

# 3. Strict Clippy check
cargo clippy -p honba-kao-app --all-targets -- -D warnings

# 4. Feature unit & regression tests
cargo test -p honba-kao-app

# 5. Code formatting
cargo fmt -p honba-kao-app -- --check
```
