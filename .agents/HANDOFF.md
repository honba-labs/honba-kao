# HANDOFF — `honba-kao-app` Screener Phase 2 (Sector-Grouped Heatmap) — COMMITTED

## 1. Phase / Slice Status — Phase 2 COMPLETE & COMMITTED (gates green)
Spec: `docs/screener-honba-leptos.md` (Phase 1: drawer row-click open + modal signal wiring; Phase 2: sector treemap + dynamic RGB heat interpolation).
- **Phase 1** committed: `d9bb81a feat(kao-app): screener Phase 1 — reactive plumbing & modal triggers`.
- **Phase 2** committed: `de2fdc4 feat(kao-app): screener Phase 2 — sector-grouped market heatmap` (+406/−5, exactly 6 paths; fulfils the Phase-2 restoration list recorded at `34ba352`).
Gates (verified on the committed content):
- `cargo check -p honba-kao-app --all-targets` ✅
- `cargo check -p honba-kao-app --target wasm32-unknown-unknown` ✅
- `cargo test -p honba-kao-app` ✅ (screener_heatmap **7/7**, screener_plumbing 9/9, page_mount/market_feed/routing green)
- `cargo fmt -p honba-kao-app -- --check` ✅
- clippy: **0 new warnings from this batch**. The `-p` gate is blocked by pre-existing committed debt only: `honba-kao-client` (20 errors, 0 uncommitted changes) aborts before the app crate; app lib has 12 pre-existing lints (`shared/contexts/mod.rs` ×9, `router/chrome/app_chrome.rs` ×2, `features/risk_ops/ops_page.rs` ×1). Zero screener hits (`--no-deps` confirmed).

## 2. Commit scope hygiene
- `de2fdc4` contains ONLY the 6 Phase-2 paths (3 new: `heatmap.rs`, `components/screener_heatmap.rs`, `tests/screener_heatmap.rs`; 3 modified: `screener/mod.rs`, `components/mod.rs`, `screener_page.rs`).
- Sibling live work preserved uncommitted: `Cargo.toml`, trading pages, `sip_planner_page`, `app_chrome`, `shared/mod.rs`, ui-core molecules, `honba-kao-assets/`, `atoms/`, `theme.json`, `honba-kao-app/assets`.

## 3. Decisions & Discoveries (probe-verified)
- **rust_decimal 1.43 `.round()` is banker's rounding** (MidpointNearestEven): 49.5→50, 25.5→26, but 34.5→**34**. For pinned half-away-from-zero midpoints use `round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)` (`rust_decimal::prelude`). Heat lerp contract: `t = ((p−lo)/(hi−lo)).round_dp(4)`, channel = `(from + (to−from)·t)` with that strategy, `to_u8()` via `num_traits::ToPrimitive`.
- Heat anchors (positions/pinned RGB): ≤−3 (225,29,72) `#e11d48`, −2 (244,63,94) `#f43f5e`, −0.5 (69,10,10) `#450a0a`, 0 (30,41,59) `#1e293b`, +0.5 (6,78,59) `#064e3b`, +2 (16,185,129) `#10b981`, ≥+4 (5,150,105) `#059669`; exact midpoints pinned in tests: 1.00→(9,114,82), −0.25→(50,26,35).
- Decimal `PartialEq/Eq` compares numerically across scales (`dec!(1)` == `dec!(1.00)`).
- `ScreenerHeatmap(records: Signal<Vec<_>>, active_symbol: RwSignal<String>, drawer_open: RwSignal<bool>)` re-derives groups in a `Memo`; only sector cards are wrapped in `flex-1 min-w-[280px] max-w-[460px]`, tiles `style="background-color:{heat_hex}; flex-grow:{share}; flex-basis:0"` + `min-w-[72px]`.
- ⚠️ **Pre-existing `main` link breakage (ui-core lane blocker, NOT this batch)**: `filter_bar.rs` (committed in d9bb81a) imports `honba_kao_ui_core::atoms::segmented::Segment`, but `HEAD:crates/honba-kao-ui-core/src` exposes only `primitive`; the `primitive → atoms` rename lives uncommitted in the ui-core lane's worktree (`git status` shows `?? atoms/` + `M lib.rs` + molecules `M`). Until that lane commits the rename, `git show HEAD:`-style builds of `honba-kao-app` fail at that one import. My Phase-2 files add **no new** ui-core paths (only `crate::*` + `leptos`).
- Subagent sessions have **no shell tool** — write code in subagents; the orchestrator runs every cargo gate itself.

## 4. Immediate Next Step — Phase 3 (Multi-Asset Zero-Floats Expansion)
Per spec §8: `ScreenerEtfRecord`/`ScreenerBondRecord`/`ScreenerMfRecord` + deterministic fixtures (15 ETFs, 10 bonds, 10 MFs, optionally expand stocks 12→30) and the asset-class switcher wiring (`asset_type: RwSignal<String>` exists — consider migrating to the `ScreenerAssetType` enum in `types.rs`). TDD: write `tests/screener_records_test.rs` (or per-class suites) FIRST. Do not break `seed_fixtures()` order contract or the Phase-1 `screener_plumbing` pinned assertions when touching fixtures.
Also queued by other lanes: ui-core `primitive → atoms` commit (fixes the pre-existing link breakage above); repo-wide clippy debt (20 client + 12 app pre-existing).

## 5. Operating Invariants
- Rust 2024; Leptos 0.8 CSR; `rust_decimal` only for financial values (f32/f64 forbidden in financial paths).
- Tests live in `tests/` parallel to `src/`; NO `#[test]` inline in `src/`.
- Subagent discipline: ≤2 concurrent, zero-recon prompts, resumes via sessionID; orchestrator runs all gates and all commits.
- Commit hygiene on the shared tree: index must contain ONLY your batch's paths before `git commit` (other lanes' work stays worktree-local); code commit first, HANDOFF amendment as its own `docs(kao)` commit referencing the hash.
- Verify from `honba-kao/` crate root; wasm gate is its own invocation (`--target wasm32-unknown-unknown`).
- Do not regress: `Hf*, SortDirection`, formatter contracts (`format_price/change_pct/volume_compact/mcap_rs_cr`), `seed_fixtures()` order, §2 pinned Phase-1 plumbing.
