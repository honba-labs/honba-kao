# HANDOFF — `honba-kao-app` Screener Phase 3 (Multi-Asset Universes) — COMMITTED

## 1. Phase / Slice Status — Phase 3 COMPLETE & COMMITTED (gates green)
Spec: `docs/screener-honba-leptos.md` (§8 Phase 3 "Multi-Asset Zero-Floats Expansion"; §4.3 record shapes).
Doc-challenge history: Phase 1 `d9bb81a` → Phase 2 `de2fdc4` → **Phase 3 `2ce864c`** (+2316/−299, exactly 7 paths).

Gates (verified from `honba-kao/` crate root on the committed content):
- `cargo check -p honba-kao-app --all-targets` ✅
- `cargo fmt -p honba-kao-app -- --check` ✅
- `cargo test -p honba-kao-app` ✅ — market_feed 6/6, page_mount 9/9, routing_invariants 10/10, screener_heatmap 7/7, **screener_multi_asset 11/11 (new)**, screener_plumbing 9/9 (seed contract intact)
- `cargo clippy -p honba-kao-app --no-deps --all-targets -- -D warnings` ✅ **0 screener hits**; 12 pre-existing committed lints remain (`shared/contexts/mod.rs` ×9, `router/chrome/app_chrome.rs` ×2, `features/risk_ops/ops_page.rs` ×1)
- `cargo check -p honba-kao-app --target wasm32-unknown-unknown` ✅

## 2. Commit scope hygiene
- `2ce864c` contains ONLY: `types.rs`, `fixtures.rs`, `filters.rs`, `components/screener_table.rs`, `components/detail_drawer.rs`, `screener_page.rs`, `tests/screener_multi_asset.rs` (new).
- Other lanes' worktree state untouched/uncommitted as before: `Cargo.toml`, trading/wealth pages, `app_chrome`, `shared/mod.rs`, ui-core molecules/organisms, `atoms/`, `honba-kao-assets/`, `theme.json`, `honba-kao-app/assets`, `docs/screener-honba-leptos.md` (spec doc still untracked — commit it in whichever docs lane convenient).

## 3. Decisions & Discoveries (probe-verified by gates)
- **Seed-contract strategy**: `seed_fixtures()` (12 rows) and the Phase-1 pinned plumbing tests were left byte-untouched; the 30-stock requirement ships as `stock_universe()` = seed + 18 appended rows, so ZERO existing golden tests needed edits. Abstract-stock pins hold by construction (all 18 appended rows: F&O `false`, watchlist `false`, price < 500, PE ∈ (15,25), rvol = 1.00 < 2, RSI < 55, `market_cap_cr` ∈ [113k, 512k] Cr, volumes < 1.8M descending, sector ≠ "Financial Services", cap category "Large Cap", no "bank" substring — none of them perturb any pinned filter/sort assertion).
- `ScreenerRecord` union is a **plain inline enum** (`#[allow(clippy::large_enum_variant)]`): no `Box`, because the pipeline clones per filter/sort pass and boxing the equity arm would allocate per row. Justified in a comment at the enum.
- `filter_universe` is a **predicate shim**: Stock arm delegates to `FilterCriteria::matches` (full stock semantics preserved byte-for-byte); non-stocks = search-only (lowercase contains over `symbol()` + `name()`). `FilterCriteria` struct itself is UNCHANGED — non-stock criteria fields simply don't apply yet (candidate slice for later phases).
- `sort_universe`: numeric keys through `ScreenerRecord::sort_decimal(key) -> Option<Decimal>`; a union sorts numerically only if EVERY record yields `Some` else unsorted clone; `maturity_date` is the one string key (Bond-only, lexical, non-bonds compare equal); unknown keys → unsorted clone. Header sort arrows are key-deduped per-`'static str` dir signals shared across all four catalogs.
- **Asset switcher**: filter-bar segment strings stay `RwSignal<String>`; the page derives `typed_asset: Signal<ScreenerAssetType>` via `from_bar` (unknown → Stocks). Heatmap is gated to Stocks (its sector math is equity-shaped); non-stock + Heatmap renders the class table plus a one-line hint row. Row-click/drawer works for all four classes (ETF symbol = ETF ticker, Bond = ISIN, MF = scheme code; drawer change-tone from `change_pct()`, `0.00%` for bonds/MFs).
- **Column modal remains stock-only** (Phase-5 scope): visibility gating applies only when `_asset == Stocks`; non-stock classes render their full catalog.
- Fixture derivations: ETF `ltp = nav + nav·premium_discount_pct/100` (exact; assert golden `dec!(262.87232)`-style values), bond maturity as plain ISO strings (deterministic, lexical-sortable), MF scheme_code as `String` (e.g. `"120465"`).
- Pre-carried from Phase 2 (still true): rust_decimal 1.43 `.round()` is banker's; heat anchors/midpoints pinned in `screener_heatmap.rs` tests; `when(Signal<bool>)` not `when(bool)`; subagents have no shell — orchestrator runs every gate.

## 4. Immediate Next Step — Phase 4 (Contextual Flyouts & AI Bar) per doc §8
1. `components/filter_bar.rs`: build popover flyouts for Market region / Index (11 benchmarks Broad+Sectoral hint) / Price-range / P/E-range pills; doc §5 wording serves price/PE min-max through the existing Filters-modal criteria — decide single-source (recommend: extend `FilterCriteria` with `#[serde(default)]` fields, wire `filter_universe` stock arm, TDD-first in `tests/screener_multi_asset.rs`).
2. `✦ AI` bar stub: currently logs via `tracing`; keep `honba-kao-app` free of `honba-nou` linkage.
3. UI probe (Playwright, port 8790 per doc §7) if a browser harness becomes available.
4. Queued by other lane: ui-core `primitive → atoms` rename commit (HANS-off note: committed `filter_bar.rs` imports `atoms::segmented::Segment`, which resolves only against the lane's uncommötted worktree).

## 5. Operating Invariants
- Rust 2024; Leptos 0.8 CSR; `rust_decimal` only in financial paths (f32 allowed only at px() layout boundaries).
- Tests live in `tests/` parallel to `src/`; NO `#[test]` inline in `src/`.
- `seed_fixtures()` 12-row order contract & Phase-1 pinned assertions: DO NOT touch (add universes to the side, like `stock_universe`).
- Subagent discipline: ≤2 waves of concurrent agents, zero-recon self-contained prompts, resumes via `sessionID`; orchestrator runs ALL cargo gates and is the only committer.
- Commit hygiene on the shared tree: `git add` exactly this batch's paths; other lanes' files stay worktree-local; code commit first, HANDOFF amendment as its own `docs(kao)` commit referencing the code hash.
- Verify from `honba-kao/` crate root; wasm gate is its own invocation (`--target wasm32-unknown-unknown`).
- Do not regress: `Hf*`/`SortDirection` contracts, formatter golden strings (`format_price/change_pct/volume_compact/mcap_rs_cr`, `record_cell`), `seed_fixtures()` order, §5 pinned plumbing, and the heat-anchor RGB math.

## 6. Other-lane batch log — honba-kao-assets (committed separately)
- **`6da42ef feat(kao-assets): centralized assets crate** — fonts (Exo2/Inter/JetBrains Mono/IBM Plex/RobotoMono + OFLs + manifest), branding emblems, OMS audio WAVs, canonical/bloomberg/monokai themes, orderbook & replay fixtures; compile-time `include_bytes`/`asset!` NSH registry; optional rodio behind the non-default `audio` feature (default build stays wasm-clean). 214 files.
- Closure: root `Cargo.toml` (member + `honba-kao-assets` dep + `nsh-core`/`nsh-base` workspace entries) **and `crates/honba-kao-app/theme.json`** — required because `assets/tests/theme_canonical_test.rs` `include_str!`s it (byte-equal web-flow copy; drift pin). Without it the committed crate would not self-build.
- Gates: fmt ✅, clippy `-D warnings` ✅, `cargo test -p honba-kao-assets` 19/19 (2 doctests ignored), wasm32 check ✅.
- Still in-flight for the same lane (preserved uncommitted): app old fonts/CSS deletion set + new `honba-kao-app/assets` dir. No committed consumer exists for the crate yet (`honba-kao-app-nsh` is outside the workspace members).
