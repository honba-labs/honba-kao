# HANDOFF — `honba-kao-app` Screener Phase 4 (Contextual Flyouts & ✦ AI Bar) — COMMITTED

## 1. Phase / Slice Status — Phase 4 COMPLETE & COMMITTED (all five gates green)
Spec: `docs/screener-honba-leptos.md` (§8 Phase 4 "Contextual Flyouts & AI Bar"; §6.1 row C; §7 keys unpinned).
Doc-challenge history: Phase 1 `d9bb81a` · Phase 2 `de2fdc4` · Phase 3 `2ce864c` · **Phase 4 `c99feed`** (+1280/−18, exactly 7 paths: `components/filter_bar.rs`, `types.rs`, `screener_page.rs`, `mod.rs`, `indices.rs` (new), `prompt_ast.rs` (new), `tests/screener_flyouts.rs` (new)).

Gates (verified from `honba-kao/` crate root on the committed content):
- `cargo check -p honba-kao-app --all-targets` ✅
- `cargo fmt -p honba-kao-app -- --check` ✅ (formatted scoped to the 7 batch paths; other lanes' worktree state untouched)
- `cargo test -p honba-kao-app` ✅ — 68 passed: market_feed 6, page_mount 9, routing_invariants 10, screener_heatmap 7, screener_multi_asset 11, screener_plumbing 9 (seed contract intact), **screener_flyouts 16 (new)**
- `cargo clippy -p honba-kao-app --no-deps --all-targets -- -D warnings` ✅ **0 screener hits**; the documented 12 pre-existing lint baseline is retained unchanged (`shared/contexts/mod.rs` ×9, `router/chrome/app_chrome.rs` ×2, `features/risk_ops/ops_page.rs` ×1)
- `cargo check -p honba-kao-app --target wasm32-unknown-unknown` ✅

## 2. Decisions & Discoveries (probe-verified)
- **Price/P-E flyouts are single-source UI**: the panels bind the SAME `min_price/max_price/min_pe/max_pe` RwSignals the Filters modal uses (dec!-0 clears, `HfNumberInput` grid, hint "Shared with the Filters modal"). No duplicate criteria fields were added — the prior lane's open question is closed as "shared-signal variant" instead of extending FilterCriteria.
- **Only `FilterCriteria.market_index: String` was added** (`#[serde(default)]`; `""`/`"ALL"`/UNKNOWN key ⇒ no restriction via new private helper `index_is_active`, used consistently in both `matches()` and `active_count()`). `filters.rs` is UNTOUCHED — index membership lives in `criteria.matches`, so the heatmap path (`filter_records`) narrows identically. Unknown keys degrade to unset (pinned by `market_index_filter_via_matches` test).
- **Index universe = side-table, fixtures untouched**: `indices.rs::INDEX_CATALOG`, 11 benchmarks = 7 Broad + 4 Sectoral, every member a valid `stock_universe()` symbol (test-pinned). NIFTY 50/Next 50/100 are cap-desc prefixes of the golden ordering (14/9/23), NIFTY 200/500 cover the full 30-row demo universe, Midcap 150 = cap ranks 16–25, Smallcap 250 = ranks 26–30; Sectoral: Bank (HDFCBANK/ICICIBANK/SBIN), IT (TCS/INFY/WIPRO/HCLTECH/TECHM), Auto (TATAMOTORS/MOTHERSON/ASHOKLEY), Pharma (SUNPHARMA). `benchmark_for`/`index_contains`/`member_count` are the only accessors — `member_count` feeds the flyout row "(n)" badges.
- **`prompt_ast.rs` is a deterministic local parser (no `honba_nou` linkage)**: clauses split on `,`/`;`; per clause bool → index-alias → numeric. Greedy rule: a complete `field rel num` / `field a-b` / `between a and b` / `a to b` assignment consumes the clause — trailing junk ("and moon") is silently dropped; the whole clause becomes residual only when the first tokens already fail. Residual becomes `search_query` ONLY when zero terms parsed (otherwise discarded). `<=`/`>=` resolve via one-character lookahead (the naive replace-then-space approach shatters `<=` into `< =` — caught by `parse_upper_bounds_and_macros`). `RvolMax` does not exist in the frozen enum → an upper rvol bound is residual by contract. Amount parsing: `Decimal::from_str_exact` after stripping `₹ `, `,`, `%` and a trailing `x`; `<`/`>`/`words` = min/max.
- **AI application is signal-mapped, not criteria-replaced**: page-level `on_ai_apply` walks `PromptAst.terms` and sets the SAME row-C signals (price/PE bounds, rvol, RSI band, `selected_index`, `fno_only`, `watchlist_only`); `ai_label = prompt_label(text)` renders a dismissable chip (click ≡ indication-clear only; `on_reset` clears every AI/index signal).
- **Flyout mount contract**: one shared `RwSignal<Option<FlyoutKey>>` (single-open; Market/Index/Price/Pe), panels mount with `.then(|| view! {…})` off `flyout_open.get() == Some(key)`; local `FlyoutPill` component with `Signal<String>` label; Market panel owns the former exchange-select options (ALL/NSE/BSE/NASDAQ/NYSE). Index pill label resolves through `benchmark_for`. ds note: `.get()` ALWAYS read inside the mount closure (reactive, not `Reactive::Static`).
- Leptos gotcha reproduced again: brace hygiene in `then(|| { view! {…} })` — the view!-close `}` and the closure-body `}` are separate; the guarded eager form `then(|| view! {…})` is preferred with no let-bindings.

## 3. Subagent Log (discipline: ≤2 alive; one wave — single coder + resumed coder; orchestrator ran every gate)
- Wave 1 (parallel, disjoint files): coder `ses_e5e56ad4cffeqa3qaf0Bh5ldeE` (mod.rs + indices.rs + prompt_ast.rs + tests; first dispatch died on the FREE-model usage limit, resumed via sessionID — wrote everything on resume; zero deviations, but self-declared recon of filters.rs/fixtures.rs to confirm `universe_for`/`filter_universe` signatures) and coder `ses_e5e1357d4ffeEsSvLDx1WNteIC` (types.rs + screener_page.rs + filter_bar.rs; hit the step cap AFTER all writes — gate fallout left to the orchestrator by design).
- Orchestrator gate-round fixes: Market-panel missing view!-close `}` (only panel lacking let-bindings, so only it drifted); `<=` lookahead rewire in `prompt_stream`; unknown-key semantics via `index_is_active`; clippy char-comparison (`split([',',';'])`); scoped `cargo fmt` on the 7 batch paths.

## 4. Immediate Next Step — Phase 5 (12 Category Tabs & Contextual Column Headers) per doc §8
1. TDD-first in a new `tests/` file: per-tab column sets lookup (tab key → column catalog slice) and formatted-cell golden strings via `format_number2/format_percent2/record_cell` (already public).
2. Wire `active_tab` → `visible_columns` mapping in `screener_page.rs` (Overview tab keeps TODAY'S default 9-stock behavior byte-for-byte; the other 11 tabs derive 4–9 metric columns from `COLUMN_CATALOG` slices).
3. Column header dropdown (`components/column_header_menu.rs` new): Sort Asc/Desc, Move Left/Right, Hide, numeric min/max Apply — Sort/Filter state comes from the criteria/sort signals; hide is `column_flags`.
4. Keep the `✦ AI` bar + flyouts signal surface frozen (AI bar tests/deps already pin prompt_ast/indices keys).
5. UI probe (Playwright port 8790 per doc §7) remains optional/blocked on a browser harness.

## 5. Operating Invariants (carried forward)
- Rust 2024, Leptos 0.8 CSR; zero floats (`rust_decimal::Decimal` only; `f32` at `px()` layout boundaries only).
- Tests LIVE in `tests/` parallel to `src/`; no `#[test]`/`#[cfg(test)]` inline in src/ (16 flyout tests live in `tests/screener_flyouts.rs`).
- `seed_fixtures()` 12-row order contract and Phase-1 pinned assertions: DO NOT touch (extend universes side-table style, as Phase 3/4 did).
- `filter_instruments`-era signatures: `filter_records`, `filter_universe`, `sort_universe`, `record_cell`, formatter golden strings stay byte-compatible; do not fork price/PE signals (they are the single source shared by modal + flyouts + AI bar).
- honba-kao-app stays free of `honba_nou` linkage — AI parsing is local + deterministic.
- Commit hygiene on the shared tree: `git add` exactly the batch paths; other lanes' files (assets-restructure, ui-core, trading/wealth pages…) stay worktree-local; code commit first, HANDOFF amendment as its own `docs(kao)` commit referencing the code hash.
- Verify from `honba-kao/` crate root; the wasm gate is its own invocation (`--target wasm32-unknown-unknown`); clippy baseline of 12 pre-existing lints is accepted and NOT part of this batch.
- Subagent discipline: ≤2 waves alive, zero-recon self-contained prompts (direct prompt omission = stop-and-question), resumes via sessionID; the ORCHESTRATOR runs every gate and is the only committer.
- Do not regress: `Hf*`/`SortDirection` atom contracts, `ScreenerViewMode`/asset switcher semantics, heat-anchor RGB math, `dynamic(move||…)`/`when(Signal)` reactivity rules.
