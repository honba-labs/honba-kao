# 本場・顔 honba-kao

> *The face of the real thing.*

Leptos SPA for the honba Indian-equities platform. Talks to `honba-ya` over
HTTP; shares wire types via `honba-kokoro-types`.

## Crates
- `honba-kao-app`    — Leptos CSR app (`cdylib`, mounted to `<body>`)
- `honba-kao-client` — typed HTTP client (`/health`, `/traders`, `/orders`, `/pnl`)

## Prerequisites
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk

## Dev
    # Terminal 1 — backend
    cd honba-ya && cargo run -p honba-ya-cli -- serve

    # Terminal 2 — frontend
    trunk serve           # http://127.0.0.1:8080
                          # /api/* proxied to :3000

## Release
    trunk build --release # → dist/
    # serve dist/ from nginx, a CDN, or a tower-http::ServeDir layer

## API URL

Dev uses the relative path `/api` (proxied). For prod, either:
- serve `dist/` and the API from the same origin (simplest), or
- change the base URL in `honba-kao-app/src/lib.rs`.

License: AGPL-3.0.
