plan-restructure.md v2.0: M1,2,3,4 done+committed (features/ reorg landed). Next: M5 (simulator UI/UX redesign per docs/simulator-design-20-Oct.md), then M6 (A11y/HfDataGrid).

Bg agent a6887e3 doing plan.md Phase1&2. No Terminal* components — port real OpenTerminalUI components into Leptos using existing shadcn ui-core primitives.

## Phase 1 & 2 (plan.md roadmap)

Status at stop (Phase 1 partly done, Phase 2 not started).

Commits (honba-kao):
- fd12e76 fix(kao): resolve moved is_open closure in HfAccordionContent (ui-core compile fix)
- 3c9d0a6 fix(kao): decode response envelope without T: Default bound (client compile fix)
- 0a890ae feat(kao): implement CommandBar Ctrl+G go bar (Phase 1.2)
- 4822841 feat(kao): implement CommandPalette Ctrl+K modal runner (Phase 1.3)
- 37b72ef feat(kao): add SymbolLinkContext and use_link_group for link groups A-D (Phase 1.4)
- 05dbad6 feat(kao): add typed live stream client with provenance feed (Phase 1.5)

Sub-items:
- 1.1 Persistent shell: done. AppChrome (HeaderBar, TickerTape, NavRail, `<Outlet/>`). TickerTape lives in `router/chrome/ticker_tape.rs`, mounted under HeaderBar, reading `MarketFeed` over `MOCK_SYMBOLS` (`.ticker-tape` CSS in `style/app.css`).
- 1.2 CommandBar: done. Ported from OpenTerminalUI commanding.ts / CommandBar.tsx. Grammar, fuzzy score, ticker hints, history key `ot:gobar:history:v1`. Preview uses MOCK symbols.
- 1.3 CommandPalette: done. NAV/ASSET/FUNC/CMD rows, Ctrl+K. Uses HfModal.
- 1.4 Link groups: done in-process. Colour groups (red/blue/green/yellow, OpenTerminalUI model) labelled A-D. Pop-out BroadcastChannel sync NOT ported.
- 1.5 Live stream: typed frames + parser in honba-kao-client (stream.rs); ui-core `use_market_socket`; app `shared/stream/feed.rs`. Endpoint `/api/ws/live` is unconfirmed; falls back to labelled Synthetic quotes.
- 2.1-2.6 Security Hub, Financials, Value Chain, Filings RAG, Research Dossiers, Screener factors: NOT STARTED. No routes or pages added.

Verification:
- The real workspace does NOT compile in this checkout (pre-existing, sibling repos): honba-kao/Cargo.toml points at ../honba-ya/crates/honba-ya-engine (absent); honba-kao-api-wasm needs honba_entities / honba_messages / honba_indicators::screener::group_metric_keys (absent everywhere).
- New code was verified in a scratch workspace outside the repo (scratchpad), with stubs for the missing engine/screener and copied api-wasm. 35 scratch tests pass: command_parse 17, command_palette 5, symbol_link 7, market_feed 6, plus stream_frames 6 in the client. ui-core and client compile.
- NOT verified: crates/honba-kao-app/src/router/app_chrome.rs (reviewed by eye only), the real app build, and in-browser behaviour.

Immediate next step: 2.1 Security Hub at `/equity/security/:ticker` (route `?tab=`; commanding.rs already emits these URLs). Register the route in router/routes.rs and paths.rs, and add `Page` variants. Then 2.2+.

Operating invariants: no `Terminal*` naming (coordinator instruction). Compose from existing ui-core primitives. Tests go in tests/. Do not commit plan.md or the other agent's HANDOFF lines.
