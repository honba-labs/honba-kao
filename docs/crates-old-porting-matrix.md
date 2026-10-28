# crates-old Porting & Retirement Matrix

**Milestone 8 (M8)** — `crates-old/` retirement audit.

## Decision (per user direction)

`crates-old/` is **RETAINED**. It is **NOT archived or deleted**. This matrix
documents where each legacy file in `crates-old/honba-kao-app` maps in the
target `crates/` architecture, which legacy directories are retired (logically
consolidated/deleted per `restructure.md` §5.1), and which domain algorithms
remain stranded. The matrix is a tracking aid only; `crates-old/` stays in the
workspace.

Source of truth for the target layout: `restructure.md` §5.1 (Consolidation
Matrix), §5.2 (Compute Eviction), §8.7 (8-Cluster Source Tree).

## Legend

| Tag | Meaning |
|-----|---------|
| `PORTED` | Legacy file has a direct counterpart in `crates/` (same responsibility). |
| `MATH→API-WASM` | Numerical algorithm evicted to `honba-kao-api-wasm` (§5.2). |
| `UI→UI-CORE` | Visual primitive/molecule ported to `honba-kao-ui-core`. |
| `CONSOLIDATED` | Legacy directory merged into a new 8-cluster page (file may not exist 1:1). |
| `RETIRED` | Directory deleted per §5.1; responsibilities absorbed elsewhere or dropped. |
| `STRANDED` | No clear counterpart in `crates/`; remains only in `crates-old/`. |

---

## Cluster 1: Trading Cockpit (`crates/honba-kao-app/src/features/trading/`)

| Legacy file (`crates-old/...`) | Destination | Tag | Notes |
|---|---|---|---|
| `features/workbench/workbench_page.rs` | `features/trading/cockpit_page.rs` | CONSOLIDATED | Cockpit uses `HfThreeColumnTradingLayout`. |
| `features/workbench/components/*` | `features/trading/*` / `ui-core::organisms` | UI→UI-CORE | Dock, order ticket, chart grid → UI core organisms. |
| `features/workbench/stores/*` | `features/trading/` (reactive stores) | PORTED | Workbench store logic moved into cluster stores. |
| `features/chart_workstation/chart_workstation_page.rs` | `features/trading/multi_chart_page.rs` | CONSOLIDATED | Multi-pane chart layout. |
| `features/chart_workstation/openscript_panel.rs` | STRANDED | STRANDED | No OpenScript panel equivalent in target. |
| `features/mta/mta_page.rs` | `features/trading/mta_page.rs` | PORTED | Multi-timeframe analysis. |
| `features/dom/dom_page.rs` | `features/trading/dom_page.rs` | PORTED | Depth-of-market ladder. |
| `features/tape/tape_page.rs` | `features/trading/tape_page.rs` | PORTED | Time & sales print stream. |
| `features/compare/*` | `features/trading/compare_page.rs` | CONSOLIDATED | Normalized multi-symbol overlay. |
| `features/launchpad/*` | `features/trading/launchpad_page.rs` | CONSOLIDATED | Mosaic workspace. |

---

## Cluster 2: Derivatives / F&O (`crates/honba-kao-app/src/features/derivatives/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `features/fno/option_chain_page.rs` | `features/derivatives/option_chain_page.rs` | PORTED | F&O option chain workstation. |
| `features/fno/strategy_builder_page.rs` | `features/derivatives/strategy_builder_page.rs` | PORTED | Strategy builder basket. |
| `features/fno/greeks_heatmap_page.rs` | `features/derivatives/greeks_heatmap_page.rs` | PORTED | Greeks heatmap. |
| `features/fno/oi_page.rs` | `features/derivatives/oi_analysis_page.rs` | PORTED | Open-interest analysis. |
| `features/fno/expiry_page.rs` | STRANDED | STRANDED | Expiry page not in target layout. |
| `features/fno/fno_heatmap_page.rs` | STRANDED | STRANDED | Duplicate/legacy heatmap. |
| `features/fno/futures_page.rs` | STRANDED | STRANDED | Futures page not in target layout. |
| `features/fno/black_scholes.rs` | `api-wasm::pricing::black_scholes` | MATH→API-WASM | Black-Scholes evicted to compute engine. |
| `features/fno/option_greeks_page.rs` | `features/derivatives/greeks_heatmap_page.rs` | CONSOLIDATED | Greeks UI consolidated. |
| `features/fno/pcr_page.rs` | STRANDED | STRANDED | PCR page not in target layout. |

---

## Cluster 3: Portfolio & Margin Hub (`crates/honba-kao-app/src/features/portfolio/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `features/portfolio/portfolio_page.rs` | `features/portfolio/portfolio_page.rs` | PORTED | `HfTwoColumnSplitLayout` hub. |
| `features/tradebook/tradebook_page.rs` | `features/portfolio/tradebook_page.rs` | PORTED | Execution journal & tax log. |
| `features/shadow-account/page.rs` | `features/portfolio/shadow_account_page.rs` | PORTED | Broker reconciler. |
| `features/portfolio_lab/portfolio_lab_page.rs` | `features/portfolio/lab_optimizer_page.rs` | PORTED | Portfolio optimizer. |
| `features/portfolio_extras/*` | RETIRED | RETIRED | Merged into `features/portfolio/rebalance/` per §5.1 (dividend, tax, rebalancing). Directory deleted. |
| `features/risk/page.rs` | `features/risk_ops/risk_page.rs` | CONSOLIDATED | VaR/CVaR/PCA moved to risk_ops. |

---

## Cluster 4: Wealth (NEW) (`crates/honba-kao-app/src/features/wealth/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| *(none — new module)* | `features/wealth/mutual_funds_page.rs` | NEW | Fund discovery & comparison. |
| *(none — new module)* | `features/wealth/sip_planner_page.rs` | NEW | SIP calculator (math → api-wasm::analytics). |
| *(none — new module)* | `features/wealth/bonds_page.rs` | NEW | Fixed income & yield curve. |
| *(none — new module)* | `features/wealth/asset_allocation_page.rs` | NEW | Asset allocation view. |

---

## Cluster 5: Quant (`crates/honba-kao-app/src/features/quant/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `features/simulator/simulator_page.rs` | `features/quant/simulator_page.rs` | CONSOLIDATED | Visual runner; math evicted to api-wasm::replay. |
| `features/simulator/kernel/*` | `api-wasm::replay::{clock,fills,journal}` | MATH→API-WASM | Replay clock/fills/journal evicted (§5.2). |
| `features/simulator/components/*` | `features/quant/*` / `ui-core` | UI→UI-CORE | Simulator UI components. |
| `features/backtesting/backtesting_page.rs` | `features/quant/backtesting_deck_page.rs` | PORTED | Backtest deck. |
| `features/model_lab/model_lab_page.rs` | `features/quant/model_lab_page.rs` | PORTED | Experiment leaderboard. |
| `features/stat_lab/stat_lab_page.rs` | `features/quant/stat_lab_page.rs` | PORTED | Stat lab. |
| `features/pair_trading/pair_trading_page.rs` | `features/quant/pair_trading_page.rs` | PORTED | Cointegration research. |
| `features/strategy_studio/strategy_studio_page.rs` | `features/quant/strategy_studio_page.rs` | PORTED | Visual strategy canvas. |
| `features/strategy_studio/canvas/*` | STRANDED (canvas) | STRANDED | Visual node canvas has no direct target counterpart. |
| `features/strategy_studio/kernel/*` | STRANDED | STRANDED | Strategy codegen/graph engine stranded. |
| `features/quant_extras/*` | RETIRED | RETIRED | Factor calculations merged into `api-wasm::analytics` per §5.1. |

---

## Cluster 6: Screener (`crates/honba-kao-app/src/features/screener/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `features/screener/screener_page.rs` | `features/screener/screener_page.rs` | PORTED | AST predicate filters & presets. |
| `features/screener/components/*` | `features/screener/components/*` | PORTED | Query builder, custom formula, hero deck. |
| `features/screener/stores/*` | `api-wasm::screener::{ast,evaluator,sorting}` | MATH→API-WASM | Screener AST predicate evaluator evicted (§5.2). |
| `features/screener/data/*` | STRANDED | STRANDED | Screener instrument data (indices/bhavcopy). |

---

## Cluster 7: Research (`crates/honba-kao-app/src/features/research/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `features/security_hub/security_hub_page.rs` | `features/research/security_hub_page.rs` | PORTED | Single-asset dossier. |
| `features/security_hub/filings_tab.rs` | STRANDED | STRANDED | Filings RAG driver (backend scoped to other repos). |
| `features/security_hub/financials_tab.rs` | STRANDED | STRANDED | Financials tab. |
| `features/security_hub/value_chain_tab.rs` | STRANDED | STRANDED | Value chain tab. |
| `features/dossiers/dossiers_page.rs` | `features/research/dossiers_page.rs` | PORTED | Institutional intelligence. |
| `features/research/*` (old) | `features/research/research_library_page.rs` | CONSOLIDATED | Research library. |
| `features/markets/*` | `features/research/markets_page.rs` | PORTED | Macro & sector heatmaps. |

---

## Cluster 8: Risk & Ops (`crates/honba-kao-app/src/features/risk_ops/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `features/oms/page.rs` | `features/risk_ops/oms_page.rs` | PORTED | Order routing compliance & circuit filters. |
| `features/ops/mod.rs` | `features/risk_ops/ops_page.rs` | PORTED | Operations console & kill switches. |
| `features/risk/page.rs` | `features/risk_ops/risk_page.rs` | PORTED | VaR/CVaR, PCA, stress scenarios. |
| `features/position-sizer/page.rs` | `features/risk_ops/position_sizer_page.rs` | PORTED | Dynamic position sizing (math → api-wasm::margin). |
| `features/risk_ops_extras/*` | RETIRED | RETIRED | Merged into `features/risk_ops/compliance/` per §5.1. |

---

## System / Auth (`crates/honba-kao-app/src/features/system/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `features/home/home_page.rs` | `features/system/home_page.rs` | PORTED | Mission control dashboard. |
| `features/landing/landing_page.rs` | `features/system/landing_page.rs` | PORTED | Public landing page. |
| `features/auth/*` | `features/system/auth_page.rs` | CONSOLIDATED | Login/registration/2FA. |
| `features/account/*` | `features/system/account_page.rs` | PORTED | API keys & broker settings. |
| `features/system_extras/*` | RETIRED | RETIRED | Merged into `features/system/settings/` per §5.1. |

---

## App shell, router & shared (`crates/honba-kao-app/src/`)

| Legacy file | Destination | Tag | Notes |
|---|---|---|---|
| `src/app.rs` | `src/app.rs` | PORTED | SPA entry + `HonbaClientContext` provider. |
| `src/router/*` | `src/router/*` | PORTED | Routes, paths, chrome shell (AppChrome, NavRail, SiteFooter). |
| `src/shared/api/*` | `honba-kao-client::http/*` | PORTED | REST endpoints moved to headless client crate. |
| `src/shared/stream/*` | `honba-kao-client::stream/*` | PORTED | SSE/WebSocket streams moved to client crate. |
| `src/shared/command/*` | STRANDED | STRANDED | Command palette/parsing logic. |
| `src/shared/filings/*` | STRANDED | STRANDED | Filings documents (fictional samples). |
| `src/shared/theme/*` | `ui-core::theme/*` | UI→UI-CORE | Design tokens & palette. |
| `src/shared/components/charts/*` | `openalgo-charts-rs` | PORTED | Charting moved to openalgo-charts-rs. |
| `src/shared/stores/ops_audit.rs` | `features/risk_ops/` | PORTED | Ops audit trail. |

---

## Compute eviction to `honba-kao-api-wasm` (§5.2)

The following algorithm families were evicted from `honba-kao-app` to
`honba-kao-api-wasm` (all `Decimal`, Zero Floats Policy):

- `features/fno/black_scholes.rs` → `api-wasm::pricing::black_scholes`
- `features/simulator/kernel/*` → `api-wasm::replay::{clock, fills, journal}`
- `features/screener/stores/filter_engine.rs` → `api-wasm::screener::{ast, evaluator, sorting}`
- `features/position-sizer/*` (sizing math) → `api-wasm::margin::{sizing, haircut, requirements}`
- `features/quant_extras/*` (factor calcs) → `api-wasm::analytics`
- Greeks, IV, payoff, term structure, indicators, pairing, drawdown, ratios, VaR/CVaR, cointegration → `api-wasm::{pricing, payoff, indicators, pairing, analytics, margin}`

## Stranded / unported domain algorithms

The following legacy code has **no clear counterpart** in `crates/` and
remains stranded only in `crates-old/`:
- `features/chart_workstation/openscript_panel.rs`
- `features/fno/{expiry,fno_heatmap,futures,pcr}_page.rs`
- `features/screener/data/*`
- `features/security_hub/{filings,financials,value_chain}_tab.rs`
- `features/strategy_studio/canvas/*` and `features/strategy_studio/kernel/*`
- `src/shared/command/*`, `src/shared/filings/*`

These are candidates for either (a) porting to the appropriate cluster or
(b) explicit retirement. No extraction was performed in this M8 pass so that
`crates-old/` can remain intact for reference.

## Conclusion

`crates-old/` is **retained** (not archived) per user direction. This matrix
provides the exhaustive porting/retirement mapping requested by M8. The live
application builds and tests against `crates/` only; `crates-old/` is not part
of the workspace build.
