# Plan: 本場・顔 honba-kao

## M0 — Wire up (W1)
- [x] Scaffold app + client crates
- [x] Trunk proxy, health endpoint check
- [ ] Pin to current Leptos release

## M1 — Trader views (W2–3)
- [ ] Trader list (name, universe, PnL)
- [ ] Trader detail (positions, recent decisions)
- [ ] Decision log with rejection reasons

## M2 — Live feed (W4–5)
- [ ] WebSocket consumer for `honba-ya-ws::Event`
- [ ] Live PnL sparkline (canvas or SVG)
- [ ] Fill notifications

## M3 — Controls (W6–7)
- [ ] Auth (JWT stored in sessionStorage, refresh flow)
- [ ] Start / pause / stop traders
- [ ] Risk limits editor

## M4 — Polish (W8)
- [ ] Dark theme, mobile layout
- [ ] Deploy as static assets (nginx / CDN / tower-http ServeDir)
- [ ] Playwright e2e test
