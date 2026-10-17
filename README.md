# 本場・顔 honba-kao

> *The face of the real thing.*

Leptos SPA for the honba Indian-equities platform. Talks to `honba-ya` over
HTTP; shares wire types via `honba-kokoro-types`. Modeled on the Bitfinex Honey Framework
UI design system (`bfx-hf-ui-core`) with a 256-color named design token palette.

## Crates
- `honba-kao-app`      — Leptos CSR app (`cdylib`, mounted to `<body>`)
- `honba-kao-client`   — typed HTTP client (`/health`, `/traders`, `/orders`, `/pnl`)
- `honba-kao-api-wasm` — in-browser WASM compute (screener, indicators, replay)

## Prerequisites
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk

## Profiles & Build Targets

All builds explicitly resort to either `dev` or `prod` profiles (instead of unoptimized debug), defaulting to **`dev`**:

- **Dev Profile (`dev`) — Default**:
  - `make build` / `make build-dev` (via Trunk) or `cargo check --profile dev --workspace`
  - Optimized for fast iterative compilation and interactive debugging with `opt-level = 1`.
  - `make serve` / `trunk serve` runs on `http://127.0.0.1:8080` with `/api/*` proxied to `:3000`.

- **Production Profile (`prod`)**:
  - `make build-prod` (via Trunk) or `cargo check --profile prod --workspace`
  - Max optimization (`opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`).
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
- change the base URL in `honba-kao-app/src/lib.rs`.

License: AGPL-3.0.
