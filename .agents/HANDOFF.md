# HANDOFF — honba-kao Screener & Design System Components Alignment Batch

**Status: COMPLETE — Screener aligned with `honba-fe` & `honba-kao-ui-core` / `components.html`; Components showcase route added; All CI gates green.**
Date: 2026-11-04 · Crate: `honba-kao/crates/honba-kao-app`, `honba-kao/crates/honba-kao-ui-core`, `honba-kao/crates/honba-kao-assets`

## Phase / Slice Status
| Slice | Milestone | State |
| :---: | :--- | :---: |
| 1 | **NavRail Navigation Fix**: Replaced `<A>` wrapper with native `<a>` with active path comparison so Leptos 0.8 keeps CSS classes; `/` and empty route correctly highlight Screener. Added NavRail and footer links to Components Showcase (`/components`). | ✅ |
| 2 | **Design System Tokens (`components.html`)**: Extracted all CSS tokens and atom/molecule classes from `components.html` into `crates/honba-kao-assets/assets/style/components.css`; imported into `main.css` and linked via Trunk in `index.html`. Added `data-theme="dark"` / `data-mode="dark"` attributes. | ✅ |
| 3 | **Router Entry for `components.html`**: Added `pub const ROUTE_COMPONENTS: &str = "/components";` in `router/paths.rs`. Implemented `ComponentsPage` in `features/system/components_page.rs` embedding `components.html`. Registered `<Route path=path!("components") view=ComponentsPage />` (and aliases `/design-system`, `/showcase`) in `router/routes.rs`. | ✅ |
| 4 | **Screener Popular Screens Bubbles (`honba-fe` parity)**: Integrated `builtin_screens()` as reactive `HfChip` bubble filters in Row A of `ScreenerFilterBar`. Clicking a screen chip dynamically applies the preset's criteria, active tab, and sorting, updating `active_screen_id`. | ✅ |
| 5 | **AI Search & Signature Filter Pills (`honba-fe` parity)**: Added signature `✦ AI` toggle button with expandable natural language prompt input. Preserved and styled Market, Index, Price band, and P/E band flyout popovers (`.hf-menu-popover`, `.hf-menu-item`, `.hf-filter-pill`). | ✅ |
| 6 | **Results Table & Drawer (`core-ui` atoms)**: Color-coded price changes and return metrics (`text-emerald-400` / `text-rose-400`). Wrapped analyst and technical ratings in `HfBadge` (`BadgeVariant::Success` / `Danger` / `Warning`). Polished 52W Range bar with tokenized indicators. Fixed `HfDrawer` in `detail_drawer.rs` to slide out from the right (`right=true`). | ✅ |

## Final Gate Report (all from `honba-kao/` crate root)
```
1  cargo check  -p honba-kao-app --all-targets                     PASS
2  cargo fmt    -p honba-kao-app -- --check                       PASS
3  cargo test   -p honba-kao-app                                  PASS (99 tests, 0 failed across 14 test suites)
4  cargo check  -p honba-kao-app --target wasm32-unknown-unknown   PASS
```

## Decisions & Discoveries
- **Router & Link Classes in Leptos 0.8**: In Leptos 0.8 CSR, custom attribute forwarding on router `<A>` can drop dynamic CSS classes. Using native `<a href=... class=class>` with reactive location derivation guarantees that active styling (`border-l-2 border-emerald-600 bg-surface-elevated`) is rendered reliably.
- **Trunk Copied File Assets**: Trunk copies `components.html` directly to the web dist root. Embedding via `<iframe src="/components.html">` inside `ComponentsPage` allows users to explore all 54 core UI atoms/molecules directly within the running workstation application shell without navigating away.
- **Drawer Placement Invariant**: `HfDrawer` defaults to left slide-over unless explicitly passed `right=true`. Detail inspection drawers for market depth and screener symbols must pass `right=true` to preserve institutional split-workspace convention.
- **Zero Floats Rule**: All screener numbers (prices, P/E ratios, market caps, RSI bounds, decimal bands) strictly use `rust_decimal::Decimal`. No floats leaked across domain models or components.
