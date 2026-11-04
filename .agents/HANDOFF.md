# HANDOFF — `honba-kao-app` Screener Phase 6 (Scanner Engine & Browser CSV Export) — COMMITTED

## 1. Phase / Slice Status — Phase 6 COMPLETE & COMMITTED
Spec: `docs/screener-honba-leptos.md` (§8 Phase 6; §4.4 scanner contracts; §6.3 explain card; §6.4 CSV stream download; §7 keyboard map).
Batch history: Phase 1 `d9bb81a` · Phase 2 `de2fdc4` · Phase 3 `2ce864c` · Phase 4 `c99feed` · Phase 5 `884fa48` · **Phase 6 `47a8b03`** (+1706/−2, exactly 13 paths: `scanner.rs` (new), `csv_export.rs` (new), `keyboard.rs` (new), `screen_catalog.rs` (new), `components/screen_catalog_modal.rs` (new), `components/filter_bar.rs`, `components/detail_drawer.rs`, `components/mod.rs`, `feature mod.rs`, `screener_page.rs`, `tests/screener_scanner.rs` (new), `tests/screener_catalog.rs` (new), `tests/screener_csv.rs` (new)).

Gates (verified from `honba-kao/` crate root on the committed content):
- `cargo check -p honba-kao-app --all-targets` ✅
- `cargo fmt -p honba-kao-app -- --check` ✅ (package-scoped; workspace fmt drags in other lanes per standing invariant)
- `cargo test -p honba-kao-app` ✅ — **99 passed across 14 suites, 0 failed** incl. `screener_scanner` 8 (new), `screener_catalog` 9 (new), `screener_csv` 10 (new); pre-existing suites unchanged (plumbing 9, columns 10, column_menu 2, flyouts 16, heatmap 7 (rnd-up), multi_asset 11, market_feed 6, page_mount 9, routing 10, …)
- `cargo clippy -p honba-kao-app --all-targets -- -D warnings` ⚠️ blocked ONLY by pre-existing/de-other-lane debt — **batch paths contribute ZERO errors** (verified via `--no-deps` run: every remaining error sits in `honba-kao-api-wasm` (8), `honba-kao-client` (~18, `websocket/connection.rs` & `stream/*`), `features/risk_ops/ops_page.rs:78`, `shared/contexts/mod.rs` ×9 (incl. `HonbaClientContext` Default), `router/chrome/app_chrome.rs` ×2) — deps are at HEAD (`d1ff550`/`f442b41`) so this trunk debt predates the batch; `app_chrome.rs` is other-lane dirty, untouched
- `cargo check -p honba-kao-app --target wasm32-unknown-unknown` ✅ **zero warnings** (after `BlobPropertyBag::set_type` fix)

## 2. Decisions & Discoveries (probe-verified)
- **Scanner detector table pinned** (order matters; weights): Bollinger squeeze w20 (evaluable iff `bb_upper != 0 && bb_bandwidth != 0`; passed = bandwidth < 5.00) → trend w25 (evaluable iff `sma_200 != 0`; passed = price > sma_200) → RVOL w20 (always evaluable; ≥ 1.50) → RSI band w20 (55..70, threshold field = 55.00) → 52W proximity w15 (evaluable iff `high_52w != 0`; passed = price ≥ 0.95·high_52w). Zero-valued pivots mean "not available" (fixtures zero all technicals except RSI) and their steps are SKIPPED, not failed. `score = round2(passed_weight*100 / evaluable_weight)`; RELIANCE calibration = 81.25 (`100·65/80`).
- **Setup routing precedence**: `BB_SQUEEZE_BREAKOUT` (squeeze passed AND price > bb_upper) → `TREND_RETEST` (trend AND RVOL) → `BREAKOUT_WATCH` (52W) → `UNQUALIFIED`. Trigger/invalidation derive from the route's own pivots (BB: upper/lower; trend: sma_200 / smallest-nonzero of {day_low, sma_50} w/ low_52w then price fallbacks; 52W: high/low; unqualified: price/day_low).
- **`sort_universe`-grade determinism**: `scan_universe` stable-sorts with `sort_by_key(|r| Reverse(score))` — ties keep input order.
- **CSV contract**: `csv_headers` drops `star`; `csv_rows` special-cases `"symbol"`/`"name"` (record_cell returns `""` for identity keys — this was caught in review and fixed); `csv_escape` quotes cells containing `,` `"` `\n` `\r` with doubled quotes; `csv_document` joins with `\n`; `export_screener_csv` = WASM-only Blob → object URL → invisible anchor `click()` → revoke, `BlobPropertyBag::set_type("text/csv;charset=utf-8;")` (NOT deprecated `type_`). Native path: `tracing::info!` only (no clipboard in this crate).
- **Screen catalog**: 12 built-ins FIXED (ids/labels in `screen_catalog.rs`) grouped Popular 5 / Value 2 / Momentum 2 / Universe 3; criteria expressed ONLY through the existing flat `FilterCriteria` (0 = off) — no criteria-shape additions (serde-safe for older persisted JSON). Custom screens: `CustomScreen{id,name,description,tab,sort_col:String,sort_ascending,criteria,visible_columns}` stored under localStorage key `screener.screens` (wasm `window().local_storage()`; deserialize failures degrade to empty, `Effect` auto-persists on mutation; native = tracing).
- **Keyboard map implemented (keyboard.rs)**: j/ArrowDown, k/ArrowUp (row select with wrap, updates active_symbol + opens drawer), Space (star toggle), Enter (`navigate("/trading/cockpit")`), `.` (catalog modal), Shift+N (catalog modal with save sheet open), f (filters modal), c (columns modal), h (Table↔Heatmap). Guards: ctrl/meta/alt, IME composing, INPUT/TEXTAREA/SELECT targets; `ev.prevent_default()` before dispatch. Escape intentionally NOT mapped here (ui-core `use_escape_key` handles modal/drawer close). Note: KeyboardEvent.key() already uppercases with Shift, so the SaveScreen arm matches `"N" | "n" when shift_key()`.
- **HfInput contract (leptos `optional` prop trap)**: `placeholder` is `#[prop(into, optional)]` — pass the BARE value (`placeholder="Screen name"`); wrapping in `Some(...)` attempts `String: From<Option<String>>` → E0277. Bind with BOTH `value=signal` and `on_input=Callback::new(move |v| signal.set(v))`.
- **Drawer card**: scanner card sited after 52W range, before nav buttons, in the Stock arm only (ETF/Bond/MF untouched); `pct_vs(price, level)` guards `price == 0` → `"—"` BEFORE division (rust_decimal panics on div-by-zero).
- **Web-sys 0.3 wasm facts confirmed**: anchor perceived through `HtmlElement` (no `HtmlAnchorElement` feature enabled in this crate); `set_type` over deprecated `type_`; storage/Blob/Url/KeyboardEvent features already in Cargo.toml.

## 3. Subagent Log (discipline: ≤2 alive; zero-recon prompts; orchestrator runs every gate, only committer)
- Wave 1 (parallel, pure logic): coder `ses_e5dc31931ffeUtZ8Qy0ykOgnpl` — `scanner.rs` + tests (арithmetic spec errors caught by their tests: 85.94 → 81.25 corrected per the stated formula) · coder `ses_e5dc3192fffenTagiXLhDDcgCY` — `screen_catalog.rs` + tests.
- Wave 2 (parallel, disjoint scopes): coder — drawer scanner card (full, signature untouched; zero-price hardening added by orchestrator) · wiring agent maxed out on recon (zero writes; facts re-inlined); its CANCELLED background resume wrote `csv_export.rs`/`tests/screener_csv.rs`/filter-bar button/page export wiring before cancellation (naming: `csv_export` not `export_csv`).
- Wave 3 (fresh lean agent by user request): coder `ses_e5c1d97a4ffeAp3Q81eexCKJHn` — csv_rows symbol/name fix + tests, `keyboard.rs`, `screen_catalog_modal.rs`, mod registrations, page wiring (7/7 applied).
- Orchestrator gate-round fixes: `HfInput` placeholder E0277; wasm-only import cfg-split (host unused-imports); my-lint fixes (`sort_by_key`, `needless_borrow`); fmt pass; wasm `set_type` deprecation.

## 4. Immediate Next Step (optional, none blocking)
1. Scanner v2: NR7 expansion needs a 20-day-high field (`ScreenerEquityRecord` has none — either add `high_20d` or derive from `bb_upper` when ≈flat); BB fixture data is zeroed — a technicals-populated variant of `seed_fixtures()` would light up squeeze/trend steps in the drawer demo.
2. Sidebar/UX polish carried from Phase 5: none regression observed; AI bar & flightouts unaffected.
3. api-wasm/client trunk clippy debt (+12 app baseline) — separate lane; NOT this batch's surface.

## 5. Operating Invariants (carried forward)
- Rust 2024, Leptos 0.8 CSR; zero floats (`rust_decimal::Decimal` only; `f32` at `px()` layout boundaries only).
- Tests LIVE in `tests/` parallel to `src/`; no `#[test]`/`#[cfg(test)]` inline in `src/` (99 suite-wide screener/host tests across 14 files now; scanner/catalog/csv contracts pinned in their own files — do NOT weaken weight/routing pin tests).
- `seed_fixtures()` 12-row deterministic order + Phase-1 pinned assertions: DO NOT touch.
- Criteria signals are the SINGLE source of filter truth (filter bar + modal + flyouts + AI bar + header menus + Screen apply all bind them); sort `sort_col: RwSignal<&'static str>` only accepts the 7 known keys (see `sort_key_for`).
- Scanner weight/evaluability semantics: zero-numbered pivots are "unavailable" — never silently treat 0 as a real threshold.
- Kbd actions must keep imitating the input-target guard; no Escape mapping in `keyboard.rs`.
- honba-kao-app stays free of `honba_nou` linkage.
- Commit hygiene on the shared tree: batch-paths-only `git add`; other lanes' worktree-local files stay uncommitted; code commit first, HANDOFF amendment as its own `docs(kao)` commit referencing the code hash.
- Verify from `honba-kao/` crate root; wasm gate is its own invocation; fmt gate package-scoped; clippy: batch must add ZERO errors (scope-verify with `--no-deps` attribution).
- Subagent discipline: ≤2 alive; zero-recon self-contained prompts (inline every API shape; recon budget = 0); resume before restart when context is substantial, but prefer a FRESH lean agent once facts are fully catalogued (per orchestrator's corrected policy); orchestrator runs all gates and commits.
