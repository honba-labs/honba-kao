# 本場・顔 honba-kao

> *The face of the real thing.*

Leptos SPA for the honba Indian-equities platform. Talks to `honba-ya` over
HTTP; shares wire types via `honba-kokoro-types`. The app directory mirrors the
`web/` frontend of [`NoFxAiOS/nofx`](https://github.com/NoFxAiOS/nofx/tree/dev/web)
(Vite + React + TS), translated to Rust/Leptos. Visual design follows the Bitfinex
Honey Framework UI design system (`bfx-hf-ui-core`) with a 256-color named design
token palette.

## Crates
- `honba-kao-app`      — Leptos CSR app (`cdylib`, mounted to `<body>`)
- `honba-kao-client`   — typed HTTP client (`/health`, `/traders`, `/orders`, `/pnl`)
- `honba-kao-api-wasm` — in-browser WASM compute (screener, indicators, replay)

## Frontend structure (`crates/honba-kao-app/src`)

Mirrors nofx `web/src/`, layer for layer. Leptos has no virtual DOM and compiles
to WASM, so the same layering (route table → pages → domain components →
primitives, over contexts/stores/api) is kept while the runtime gets faster.

| nofx `web/src/`                  | honba-kao `crates/honba-kao-app/src/`               | Notes |
| -------------------------------- | --------------------------------------------------- | ----- |
| `App.tsx`                        | `app.rs`                                            | provider stack (contexts + stores) |
| `main.tsx`                       | `lib.rs`                                            | `mount_to_body(App)` |
| `router/paths.ts`                | `router/paths.rs`                                   | `ROUTES` constants + `Page` enum |
| `router/AppRoutes.tsx`           | `router/app_routes.rs`, `router/app_chrome.rs`      | route table + `AppChrome` layout |
| `pages/`                         | `pages/`                                            | top-level screens |
| `components/<domain>/`           | `components/<domain>/`                              | `auth`, `terminal`, `trader`, `charts`, `strategy`, `planning`, `modals`, `faq`, `landing` |
| `components/ui/`                 | `components/ui/`                                    | primitives (bfx-hf-ui-core design language) |
| `contexts/`                      | `contexts/`                                         | `provide_context` / `expect_context` |
| `stores/`                        | `stores/`                                           | reactive signals (≈ Zustand) |
| `lib/api/*`                      | `api/`                                              | namespaced client + transport |
| `lib/*`                          | `utils/`                                            | `format`, `indicators`, `text` |
| `hooks/`                         | `hooks/`                                            | reusable reactive helpers |
| `types/`                         | `types/`                                            | domain types |
| `i18n/`, `constants/`, `data/`   | same names                                          | translations, branding, sample data |
| `src/styles/`                    | `style/` (repo root, served by Trunk)               | `app.css` (chrome), `brand.css` (landing) |

Route map (same as nofx): `/`, `/login`, `/register`, `/reset-password`,
`/setup`, `/welcome`, `/faq`, `/data`, `/settings`, `/competition`, `/traders`,
`/dashboard`, `/strategy`, `/strategy-market`, `/frontier` (alias `/planning`).
Authenticated screens are gated inline, exactly like the reference app's
`isLoggedIn ? <Screen/> : <Fallback/>`.

### Performance notes
- **Leptos 0.7 CSR + WASM** — compiled output, no virtual DOM, fine-grained
  reactivity instead of tree diffing.
- **Polling instead of SWR** — `hooks::use_interval` is the equivalent of SWR's
  `refreshInterval`; screens read the tick to refresh positions/account/decisions.
- **Charts are Rust-native SVG** — no vendored TradingView/charting bundle.
- **Production profile** — `trunk build --release` selects cargo's `release`
  profile, so `[profile.release]` carries `opt-level = 3`, `lto = true`,
  `codegen-units = 1`, `panic = "abort"`; `profile.prod` inherits from it.
- **API layer is real, data is sample** — `api::http` is wired to the typed
  `honba-kao-client`; each namespace currently returns `data::demo_data` so the
  UI is fully explorable before the backend routes exist.

## Prerequisites
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk

## Profiles & Build Targets

All builds explicitly resort to either `dev` or `prod` profiles (instead of unoptimized debug), defaulting to **`dev`**:

- **Dev Profile (`dev`) — Default**:
  - `make build` / `make build-dev` (via Trunk) or `cargo check --profile dev --workspace`
  - Optimized for fast iterative compilation and interactive debugging with `opt-level = 1`.
  - `make serve` / `trunk serve` runs on `http://127.0.0.1:8080` with `/api/*` proxied to `:3000`.

- **Production build (`release`, mirrored by `prod`)**:
  - `make build-prod` (`trunk build --release`) or `cargo check --profile prod --workspace`
  - Max optimization (`opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`).
    Trunk's `--release` selects cargo's `release` profile, so the tuning lives in
    `[profile.release]`; `profile.prod` inherits from it.
  - Generates distribution bundle in `dist/`.

## Dev Workflow
    # Terminal 1 — backend
    cd honba-ya && cargo run -p honba-ya-cli -- serve

    # Terminal 2 — frontend
    make serve            # http://127.0.0.1:8080 (dev profile)
                          # /api/* proxied to :3000

## Release Workflow
    make build-prod       # → dist/ (prod profile)
    # serve dist/ from nginx, a CDN, or a tower-http::ServeDir layer

## API URL

Dev uses the relative path `/api` (proxied). For prod, either:
- serve `dist/` and the API from the same origin (simplest), or
- change `API_BASE_URL` in `crates/honba-kao-app/src/constants/branding.rs`.

License: AGPL-3.0.
