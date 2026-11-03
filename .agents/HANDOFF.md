# HANDOFF — `honba-kao-app` Screener (Phase 1 COMMITTED `ec1127e`; Phase 2 in flight)

## 1. Phase / Slice Status
- **Phase 1 (Reactive Plumbing & Modal Triggers): COMMITTED** as `d9bb81a1` —
  `feat(kao-app): screener Phase 1 — reactive plumbing & modal triggers`
  (15 files, +2310/−42; spec `docs/screener-honba-leptos.md` Phase 1).
- Gates green at commit: `cargo check -p honba-kao-app --all-targets` ✅,
  `cargo check --target wasm32-unknown-unknown` ✅, `cargo fmt --check` ✅,
  `tests/screener_plumbing.rs` 9/9 + `tests/page_mount.rs` screener cluster ✅,
  clippy 0 new warnings.
- Pre-existing warnings remain untouched (owned by other in-flight batches):
  `shared/contexts/mod.rs` ×9, `router/chrome/app_chrome.rs` ×2,
  `features/risk_ops/ops_page.rs` ×1, `tests/market_feed.rs` ×2, plus ~42 in
  client/api-wasm/ui-core.

## 2. Phase-1 committed contents (this is the truth at HEAD)
- `src/features/screener/types.rs` — 3 enums, `ScreenerEquityRecord` (~60 Decimal fields),
  `ScreenerColumn` + `COLUMN_CATALOG`/`DEFAULT_VISIBLE_COLUMNS`, `FilterCriteria`
  (`matches`, `active_count`, `Decimal::ZERO` = unset bound).
- `src/features/screener/fixtures.rs` — `equity_fixture()` + 12 deterministic rows.
- `src/features/screener/filters.rs` — `filter_records`, `sort_records` (stable,
  reverse-on-desc), `format_price/format_change_pct/format_volume_compact/format_mcap_rs_cr`.
- `src/features/screener/screener_page.rs` — full signal graph, criteria→filter→sort
  Memo pipeline, row click sets `active_symbol` + `drawer_open.set(true)`,
  **Heatmap arm is a placeholder** ("Market Heatmap arrives in Phase 2 (sector treemap)").
- `src/features/screener/components/{filter_bar,screener_table,detail_drawer,filter_modal,column_modal}.rs`
  + `components/mod.rs` (NO `screener_heatmap` registration — see §3).
- `tests/screener_plumbing.rs` (9) + `tests/page_mount.rs` (5 new component asserts).
- `.agents/HANDOFF.md` (this file).

## 3. Phase-2 (sector heatmap) is IN FLIGHT and uncommitted — restoration list
A concurrent batch built Phase 2; to keep `main` compilable its wiring was
**trimmed out of the shared files at commit time** and left in their working tree.
The Phase-2 batch must re-apply on top of `d9bb81a1`:
1. `src/features/screener/components/mod.rs` — restore `pub mod screener_heatmap;`
   + `pub use screener_heatmap::ScreenerHeatmap;` (+ doc mention).
2. `src/features/screener/mod.rs` — restore `pub mod heatmap;` (alphabetical slot).
3. `src/features/screener/screener_page.rs` — restore the `ScreenerHeatmap` import
   and replace the placeholder Heatmap arm with `<ScreenerHeatmap records=… active_symbol=… drawer_open=… />`.
Exact pre-trim content is preserved at `/tmp/opencode/screener-concurrent-backup/`.
Uncommitted Phase-2 artifacts on disk (UNTRACKED, owned by that batch — do not
stage with anything else):
- `components/screener_heatmap.rs` — `ScreenerHeatmap(records: Signal<Vec<_>>,
  active_symbol: RwSignal<String>, drawer_open: RwSignal<bool>)`; sector cards,
  cap-weighted tiles (`flex-basis:0; style=background-color:{heat}; flex-grow:{share}`),
  ring-2 active highlight, click sets symbol + opens drawer, 7-swatch legend.
- `heatmap.rs` — pure grouping/weights/color helpers.
- `tests/screener_heatmap.rs` — 7 tests (grouping totals/avg/cap order incl.
  equal-cap symbol tiebreak, RGB anchor table + piecewise midpoints
  (1.00→(9,114,82), −0.25→(50,26,35)), hex format, share proportions, degenerate guard).

## 4. Verified API facts (pinned contract)
- `HfSelect` options are **(label, value)** tuples; `HfTabPanel` self-sets its
  `active` RwSignal; `HfDialog/HfModal/HfDrawer` self-close and take
  `open: RwSignal<bool>`; `HfModal.on_confirm` is required.
- `Memo<T>` requires `T: PartialEq + Send + Sync` — `Vec<AnyView>` is NOT;
  use plain `move ||` closures as dynamic children for view fragments.
- `HfSortArrows` cycles its own `RwSignal<SortDirection>` and passes the NEW
  value to `on_cycle`.
- Phase-1 sort defaults: `sort_col = "market_cap_cr"`, descending.
- Sector vocabulary (fixtures): Energy | Technology | Financial Services |
  Automotive | Consumer Goods | Metals.
- `BashShell unavailable in coder/typist subagents — orchestrator runs cargo.

## 5. Immediate Next Step (Phase-2 finisher)
Re-apply §3 wiring, then run the five gates of doc §9 (full `cargo test -p
honba-kao-app` should then be green including `screener_heatmap` 7/7), and
commit as `feat(kao-app): screener Phase 2 — sector-grouped cap-weighted heatmap`.

## 6. Operating Invariants
- Rust 2024, Leptos 0.8 CSR, zero floats (`rust_decimal` only).
- Tests in `tests/` parallel to `src/`; no inline `#[test]` in `src/`.
- Subagent discipline: ≤2 concurrent, zero-recon prompts, resumes via sessionID.
- Commit hygiene: never `git add -A` on this shared tree — shared files are
  contended; trim to your batch boundary and restore siblings afterwards.
