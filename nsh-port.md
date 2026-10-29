# honba-kao → NSH Port: Architecture, Mapping, and Migration Plan

Research date: 2026-10-29. Sources: `nsh.dev/llms.txt`, `/docs/web`, `/docs/installation`,
`/docs/jsx-and-components`, `/docs/architecture`, `/elements`, `github.com/longbridge/nsh`,
`crates.io` API, plus a read of the current `honba-kao-ui-core` tree.

---

## 1. NSH architecture summary

**NSH (Native Shell)** is a Rust-core, retained-view UI framework (Longbridge,
`github.com/longbridge/nsh`). Apps are authored in Rust, TSX, or both; both compile
to the *same* native renderer — there is no virtual-DOM diff on either path.

### Verified: does the Web backend target `wasm32-unknown-unknown`?

**Yes, for Rust apps.** From `/docs/web`:

- Setup requires `rustup target add wasm32-unknown-unknown` plus a clang that targets wasm.
- "WebAssembly hosts the shared Rust engine for component state, layout and updates."
- `nsh package --target web` emits `.nsh/out/<Name>/index.html`, `nsh.<digest>.js` (a thin
  DOM runtime shim), and `nsh.<digest>.wasm` — described as "The Rust engine, layout and
  components, plus a Rust app's own crate."
- No SSR. A **TSX** app instead ships `app.<digest>.js` as an ES module run by the browser's
  JS engine (QuickJS is used for native/mobile TSX, not web TSX).
- If a Rust crate needs a browser API, NSH runs `wasm-bindgen` on it and ships its glue
  (`web-sys`/`js-sys`) alongside `nsh.wasm` — the same mechanism honba-kao already uses.

This satisfies the workspace's WASM-targetability requirement for Rust-authored UI. It is a
drop-in replacement for the current `trunk`/`wasm32-unknown-unknown` CSR pipeline, not a
different compilation model.

### Retained tree, signals, layout

- A shadow/retained tree lives in `nsh-core`, on an **engine thread**. A `#[component]`
  function runs **once** — there is no re-render and no virtual-DOM diff.
- On a state change (`count.set(count() + 1)`), the native reactive graph re-runs only the
  bindings that read that signal, pushes one op per changed binding into a queue (no JSON,
  no per-prop messages), and on turn-end the core commits: rebuild changed text, **Taffy**
  layout (flexbox + CSS grid), diff, and emit a single `MountBatch` that the platform thread
  applies in one hop (create/insert/update views).
- Signals, computed values, effects, and context are native; nodes are "QuickJS objects of
  native classes" so Rust's GC can trace references — **QuickJS here is an embedded
  object/GC substrate for the reactive graph, not a scripting requirement for Rust apps.**
  You never write JS to get this; it's an internal implementation detail. Worth flagging
  (see §6) because it means "pure Rust" isn't fully JS-engine-free under the hood, even
  though the authoring surface is 100% Rust.
- Text is pre-shaped into one attributed string per `<Text>` (CoreText on Apple,
  `PrecomputedText`/`StaticLayout` on Android), measured with caching, then laid out once —
  color-only changes skip re-measurement.

### Per-platform backends

| Platform | Backend |
|---|---|
| iOS | UIKit views |
| Android | Android views |
| macOS | AppKit windows (or GPUI) |
| Windows | Win32 windows drawn with Direct2D (or GPUI) |
| Linux | GTK 4 windows (or GPUI) |
| Web | Native Shell components rendered as HTML/CSS, Rust→wasm32 |
| GPUI | Drawn by GPUI itself on macOS/Windows/Linux as an alternative to the native-widget backend |

### Crates (from `/docs/architecture`)

`nsh-core` (shadow tree, styles, Taffy layout, text, **list virtualization**, mount items,
Tailwind/CSS values, native animation graph, Rust component framework) ← `nsh-js` (QuickJS
host: reactive graph, renderer, native modules, HMR) ← `nsh-engine` (drives the above, hands
one batch/frame to the platform) ← platform crates (`nsh-platform-{ios,android,macos,
windows,linux,web,gpui}`). Side crates: `nsh-base` (unstyled components, built on
`nsh-core`), `nsh-package` (signed packages), `nsh-bundler` (rolldown/Oxc/JSX/Tailwind for
the TSX path), `nsh-dev`, `nsh-cli`, `nsh-animation`, `nsh-testing`, `nsh-macros`.

### Base component library (`nsh/base`, Rust crate `nsh_base`)

Behavior/state/focus/accessibility owned by the component; **look** supplied by the app
(i.e. genuinely "unstyled"): `Button`, `Input`, `NumberInput`, `OtpInput`, `Checkbox`,
`Radio`/`RadioGroup`, `Switch`, `Toggle`/`ToggleGroup`, `Slider`, `Select`, `Link`,
`Dialog`, `AlertDialog`, `Sheet`, `Popover`, `Tooltip`, `Menu`/`ContextMenu`/`Submenu`,
`Tabs`, `Accordion`, `Collapsible`, `NavStack`/`NavPage`, `Table`, `DataTable`, `DiffView`,
`Editor` (code editor), `Dock`/`DockArea`, `Resizable`, `Scrollbar`, `Progress`,
`Toast`/`ToastStack`, `Plot` (`Line`, `Area`, `Bar`, `Grid`, `Pie`, `Arc`). Sidebar-only
(undocumented depth): `Calendar`, `DatePicker`, `Combobox`, `ColorPicker`, `TimeField`,
`RichEditor`.

Core primitives (not "base", lower-level): `View`, `Text`, `Image`, `Scroll`, `List`,
`Canvas`, `Shader`, `WebView`, `Video`, `Overlay`, `If`/`Each` control flow. **There is no
SVG primitive** — see §6.

### Styling API (typed Rust builder, Tailwind-equivalent)

```rust
view().p_4().rounded(px(12.0)).bg(white()).border_1().border_color(rgb(0xd0d7de))
view().bg_linear(deg(135.0)).gradient_from(rgb(0x54aeff)).gradient_to(rgb(0x8250df))
view().translate_x(px(20.0)).rotate(deg(-8.0)).scale(0.9)
// build-time conditional:
.when(danger, |v| v.bg(rgb(0xcf222e)))
// runtime conditional:
.style_when(busy, |s| s.opacity(0.4))
// signal-driven, re-sends only that property:
.bg(move || if selected.get() { rgb(0xddf4ff) } else { rgb(0xffffff) })
.active(|s| s.bg(ZINC.shade(100)).scale(0.95))   // == `active:` Tailwind variant
```

Each short method (`.p_4()`, `.bg()`, `.flex_1()`, `.shadow_lg()`) is the typed equivalent
of a Tailwind CSS 4 utility class; the JSX compiler and the Rust builder "make the same
native elements." This maps directly onto honba-kao's existing `theme::tokens::class::*`
Tailwind-ish semantic constants (see §3).

### `#[component]` (pure Rust) vs TSX/`mount` — **decision: Rust only**

NSH supports a `#[component]` attribute on a Rust function taking a `#[derive(Props)]`
struct of `Prop<T>` fields (with `#[prop(default = ...)]`), returning `impl IntoElement`;
it auto-generates a snake-case builder constructor (`badge().label(..)`), and children are
a `Prop<Children>` field filled with `.child(..)`. This is a direct structural analogue of
Leptos's `#[component]` + `view!{}` + `#[prop(...)]` that honba-kao-ui-core already uses —
**the lowest-friction migration path available.**

TSX components are mounted with `mount(App)` / `mount(App, props)`, run on QuickJS (native)
or the browser JS engine (web), and can import/export across the boundary with Rust
(TS types get generated from Rust prop structs, and vice versa).

**This port must use the `#[component]` path exclusively and must not introduce
`mount`/TSX.** honba-kao is already a pure-Rust, zero-floats (`rust_decimal::Decimal`),
WASM-targetable codebase; pulling in TSX/JSX would reintroduce the JS/npm/Tailwind-build
tooling this port exists to escape, split the type system (Decimal values would have to be
re-validated or stringified across a TS boundary), and defeat the "retargetable design"
rule already in force elsewhere in this workspace. Every NSH component in this port is a
`#[component]` Rust function; `mount()` is used once only, as the web entry point, exactly
as `leptos::mount::mount_to_body` is used once today.

---

## 2. Current honba-kao architecture summary

Leptos 0.8 CSR/WASM trading-terminal frontend. `Trunk.toml` + `index.html` drive the build
(`trunk build`/`serve`), `leptos_router` provides URL-based page routing. Workspace
(`honba-kao/Cargo.toml`, 5 members):

| Crate | Role |
|---|---|
| `honba-kao-app` | App shell, router, feature pages (`features/{trading,derivatives,portfolio,quant,research,risk_ops,screener,system}/*_page.rs`) |
| `honba-kao-client` | API client: broker adapters (`angel_one`, `zerodha_kite`, `paper_wallet`), REST (`market_api`, `order_api`, `portfolio_api`, `screener_api`), WebSocket (`depth_feed`, `trade_feed`), SSE streams (`agent_stream`, `orders_stream`, `risk_stream`), DTOs |
| `honba-kao-api-wasm` | wasm-bindgen bindings over pure Rust finance math: Black-Scholes/greeks/IV, margin (haircut/liquidation/sizing/utilization), payoff, FIFO/LIFO tax-lot pairing, screener AST/evaluator, portfolio analytics (VaR, drawdown, ratios, cointegration, SIP) |
| `honba-kao-ui-core` | The component library (atomic-design taxonomy, see below) |
| `openalgo-charts-rs` | Charting: `canvas/{engine,bridge,bars,candles}.rs`, `depth/`, `indicators.rs`, `curves/`, `drawing.rs`, `leptos/chart_view.rs` — a bespoke HTML5-Canvas 2D drawing engine with a thin Leptos wrapper, **not** SVG-based |

`honba-kao-ui-core/src/` taxonomy (confirmed via directory listing):

- `primitive/` (24 atoms): badge, button, card, color_bar, counter, currency, dialog,
  display_box, footer, identicon, input, kbd, link, panel, price_yield, prompt, segmented,
  separator, slider, sort_arrows, sparkline, spinner, table, tab, timer
- `molecules/{core,crypto,fno,quant,wealth}/` (29 molecules across 5 domains)
- `organisms/` (10): collateral_organism, market_tape, market_top_bar,
  mutual_fund_dossier, option_chain, order_book, order_ticket, strategy_builder,
  terminal_shell, trading_dock
- `templates/` (4): page, two_columns, three_columns, mosaic_workspace
- Supporting: `hooks/` (use_click_outside, use_escape_key, use_media_query,
  use_virtual_rows), `icons/outline.rs`, `theme/{palette,tokens}.rs` (Tailwind-ish semantic
  CSS class constants, e.g. `class::BG_SURFACE`, `class::TEXT_POSITIVE`)

Everything financial is `rust_decimal::Decimal` end-to-end (zero-floats invariant);
`sparkline.rs` currently renders via a raw inline `<svg><polyline>`, which is the one
primitive whose rendering primitive (SVG) NSH does not have a direct equivalent for.

---

## 3. Restructured layout for NSH

**Verdict: keep the existing taxonomy as-is** (`primitive/molecules/{domain}/organisms/
templates`). It already matches NSH's own `/elements` split (core primitives vs. base
components vs. app-specific composition) and nothing about NSH's model forces a
reorganization. Two additive changes:

1. **A `base/` shim module inside `primitive/`** — one file per wrapped `nsh_base`
   component (`primitive/base/button.rs` wraps `nsh_base::Button`, etc.) so the rest of the
   tree depends on honba's own `HfButton`/`HfInput`/... wrapper types, never on
   `nsh_base::*` directly. This is the same indirection the Leptos version already has
   (raw `<button>` → `HfButton`); it's what keeps primitives swappable if `nsh_base` makes a
   breaking change (likely, given its age — see §6).
2. **A style-token trait over the builder API**, replacing `theme/tokens.rs`'s string
   constants. Something like `trait HonbaStyle { fn surface_base(self) -> Self; fn
   text_positive(self) -> Self; }` implemented for NSH's style builder, so call sites keep
   writing `view().surface_base().text_positive()` instead of hand-rolling `.bg(rgb(...))`
   everywhere a token constant is used today. `theme/palette.rs`'s color values move
   in directly as `rgb(0x..)` constants.

### Primitive mapping

| Primitive | NSH equivalent | Free `nsh/base` wrap? |
|---|---|---|
| button | `nsh_base::Button` | **Yes** |
| input | `nsh_base::Input` | **Yes** |
| link | `nsh_base::Link` | **Yes** |
| slider | `nsh_base::Slider` | **Yes** |
| dialog | `nsh_base::Dialog` / `AlertDialog` | **Yes** |
| prompt | `nsh_base::AlertDialog` + `Input` | **Yes** (composition) |
| segmented | `nsh_base::ToggleGroup` | **Yes** |
| tab | `nsh_base::Tabs` | **Yes** |
| table | `nsh_base::Table` / `DataTable` | **Yes** |
| spinner | `nsh_base::Progress` (indeterminate) | Likely — depends on indeterminate support |
| badge | `View` + `Text` + builder styles | Bespoke (trivial) |
| card, panel, display_box | `View` composition | Bespoke (trivial) |
| color_bar, separator | styled `View` | Bespoke (trivial) |
| counter, timer | `Text` + signal/effect, optionally `nsh-animation` | Bespoke |
| currency, price_yield | `Text` + formatting | Bespoke |
| kbd | styled `Text` | Bespoke |
| sort_arrows | `Text`/icon toggle | Bespoke |
| identicon | `Canvas` (procedural draw) | Bespoke |
| footer | layout `View` | Bespoke |
| **sparkline** | `Canvas` (no SVG primitive — full rewrite of the polyline draw as imperative canvas path) | **Bespoke, rewrite required** |

### Molecule mapping (by domain)

| Domain | Molecule | NSH equivalent | Free wrap? |
|---|---|---|---|
| core | alert_stack | `nsh_base::Toast` / `ToastStack` | **Yes** |
| core | estimation_fields, price_snap_input, quantity_chip_input | `NumberInput`/`Input`/`ToggleGroup` composition | Partial (base controls, bespoke layout) |
| core | spread_indicator, stats_bar, tape_row | `View`/`Text` composition | Bespoke |
| crypto | wallet_widget | `nsh_base::Popover` or `Sheet` shell | Partial |
| crypto | liquidation_bar | `nsh_base::Progress` (colored) | Partial |
| crypto | collateral_asset_pane/simulator/snapshot/tile, funding_rate, itayose_card | `View` + `Slider`(base) for simulator | Bespoke |
| fno | payoff_chart | `nsh_base::Plot` (`Line`/`Area`) | **Yes, big win** |
| fno | pcr_meter | `nsh_base::Plot` (`Arc`) or `Progress` | Partial |
| fno | greeks_group, max_pain_badge, option_chain_row, strategy_leg_row | `View` composition / dense table row | Bespoke |
| quant | backtest_card (equity curve), cointegration_band, underwater_gauge | `nsh_base::Plot` (`Line`/`Area`) | **Yes, big win** |
| quant | filter_pill | `nsh_base::ToggleGroup` item | **Yes** |
| quant | go_bar | `nsh_base::Progress` | Partial |
| quant | agent_console | `nsh_base::Editor`/scroll log + `List` | Partial |
| quant | artifact_card, filings_card, market_regime_badge | `View`/`Text`/`Badge` | Bespoke |
| wealth | asset_allocation_bar | `nsh_base::Plot` (`Pie`/`Bar`) | **Yes, big win** |
| wealth | risk_o_meter | `nsh_base::Plot` (`Arc`, gauge) | **Yes, big win** |
| wealth | rolling_returns | `nsh_base::Plot` (`Line`/`Bar`) | **Yes, big win** |
| wealth | overlap_pill | `Badge` | Bespoke |
| wealth | sip_calculator | `NumberInput`/`Slider`(base) + computed `Text` | Partial |

### Organism mapping

| Organism | NSH equivalent | Free wrap? |
|---|---|---|
| trading_dock | `nsh_base::Dock`/`DockArea` | **Yes, near-free** |
| mutual_fund_dossier | `nsh_base::NavStack`/`NavPage` for drill-down + `DataTable` | Partial |
| order_ticket | `nsh_base::Sheet`/`Dialog` shell + `Input`/`NumberInput`/`Select`/`RadioGroup` fields | **Mostly free** |
| option_chain | `nsh_base::DataTable` substrate + `ContextMenu` for leg actions + bespoke dense row rendering | Partial |
| terminal_shell | `nsh_base::Dock`/`DockArea` + `Resizable` + `Sheet`; on native platforms this becomes a *real OS window*, not a div | Partial |
| strategy_builder | `nsh_base::Dock`/`Resizable` for panes; node-canvas drag/connect logic is bespoke (`Canvas`) | Partial |
| collateral_organism, market_top_bar | `View` composition over molecules | Bespoke |
| market_tape | `nsh-core::List` (ticker scroll) — benefits from NSH's native list virtualization | Bespoke, but easier than today |
| **order_book** | `List` (native virtualization) + `Canvas` for the depth-bar visualization; this is honba's most latency-sensitive view and should be the first organism profiled under NSH's op-diff model | **Bespoke, highest-value target to validate NSH's performance claim** |

### Template mapping

| Template | NSH equivalent | Free wrap? |
|---|---|---|
| **mosaic_workspace** | `nsh_base::Dock`/`DockArea` | **Yes, this is literally what Dock is for** |
| two_columns, three_columns | `View` flex + `nsh_base::Resizable` for draggable splits | **Mostly free** |
| page | `View` layout, `Scroll` | Bespoke (trivial) |

### Supporting modules

- `hooks/use_click_outside`, `use_escape_key`: likely **deletable** — `nsh_base::Dialog`/
  `Popover`/`Sheet` "own behavior, state, focus and accessibility," which per the docs
  includes outside-click and Escape dismissal natively. Verify per-component before
  deleting.
- `hooks/use_virtual_rows`: likely **deletable** — `nsh-core` lists "list virtualization"
  as a built-in responsibility of the `List` primitive. Verify behavior/API before
  deleting; this is a reduction in code to carry, not a gap.
- `hooks/use_media_query`: **open question** — no NSH-documented viewport/platform signal
  was found in this research pass; needs a dedicated doc check (`/docs/platforms` or
  `/docs/reactivity`) before this hook can be ported or dropped.
- `icons/outline.rs`: port as-is if it's data (path strings) driving `Canvas` draws; if it
  currently emits inline `<svg>`, it has the same gap as `sparkline` (see §6).

---

## 4. Phased migration plan

**Phase 0 — Tooling spike (1-2 days).** Install `nsh` CLI, scaffold a throwaway
`nsh new --target web` app, confirm `rustup target add wasm32-unknown-unknown` + `nsh
package --target web` produces a running page with a `#[component]` Button and a signal
counter. Decide the `base/` shim pattern (§3) here, on a toy component, before touching
real code. Exit criteria: a committed-nowhere scratch app proving the Rust-only path end to
end.

**Phase 1 — Primitives (per §3 table).** Port all 24 in dependency order: trivial
`View`/`Text` wraps first (badge, card, separator, color_bar, footer, kbd, display_box,
panel), then `nsh_base` wraps (button, input, link, slider, dialog, prompt, segmented, tab,
table, spinner), then the two hard cases last: **sparkline** (Canvas rewrite) and

> **2026-10-29 update — Phase 1 done, compiling against the real crate (not just docs).**
> `crates/honba-kao-ui-core` now builds clean (`cargo check`/`fmt`/`clippy`: 0 errors) against
> `nsh-core`/`nsh-base` 0.3.0 as actually published, not the `/docs` summary this file's §1-§3
> were written from. Two systemic gaps this research pass missed, now fixed, that the next
> port phase (molecules/organisms) will hit again if not kept in mind:
> 1. **`#[component]` is NOT a drop-in for Leptos's.** Leptos's inline `#[prop(...)]` on fn
>    params does not exist in nsh; every prop lives on a separate
>    `#[derive(Debug, Clone, PartialEq, Props)] struct FooProps { #[prop(default)] pub x:
>    Prop<T>, ... }`, consumed as `fn Foo(props: FooProps, cx: Scope<FooEvent>)`. Composing
>    one `nsh_base` component inside another is `Inner::element(InnerProps { .., ..
>    Default::default() }).child(..).on_event(move |e: &InnerEvent| cx.emit(..))` — never
>    `Inner(props, Scope::new())` (`Scope::new` doesn't exist). Every prop callback
>    (`Option<Box<dyn Fn(..)>>`) must become a local `#[derive(Event)] enum` instead — a
>    boxed closure can't satisfy `Props`'s `Clone + PartialEq` bound. Reference:
>    `primitive/base/button.rs`.
> 2. **The `/elements` summary overstates several "free wrap" cells in §3's tables.**
>    `nsh_base::Tabs`/`TabsProps` turned out to hold only `children: Prop<Children>` — no
>    selected-value/`onChange` at all (selection is each `Tab`'s own `selected`/`onPress`,
>    composed by the app). `nsh_base::Toast`/`ToastStack` hold only lifecycle/motion props +
>    `children` + a `DismissRequest` event — no `title`/`message` fields; content is a child
>    `Text`. Anywhere §3 says "Yes"/"Mostly free" for a `nsh_base` component, re-verify the
>    real prop struct in the installed crate source
>    (`~/.cargo/registry/src/*/nsh-base-0.3.0/src/`) before assuming it — several components'
>    actual surface is narrower than their doc-page one-liner suggests.
> Also fixed along the way, lower-impact but worth knowing: `px`/`rgb`/`AdaptiveColor`/
> `FontWeight`/`Style` are reachable via `nsh_core::prelude::*` but not `nsh_core::` directly
> (and never via the *private* top-level `nsh_core::style` module, a same-named decoy of the
> real `nsh_core::element::style`); `nsh_core::element::input()` (the `Input` constructor) is
> missing from the prelude's curated list and needs its own explicit `use`; any type inside a
> `Prop<T>` must itself implement `JsValue` + `Default` (enums: `PropEnum` derive with a
> `#[default]` variant; plain structs: `JsObject` derive) — foreign types like
> `rust_decimal::Decimal` or tuples can't, so Decimal-valued props go through `Prop<String>`
> pre-formatted by the caller, keeping the real `Decimal` upstream per the zero-floats policy.


**identicon** (Canvas rewrite). Each primitive keeps its existing public prop signature
where possible to minimize churn in callers. Exit criteria: all 24 compile under `nsh
build --target web` with no TSX.

**Phase 2 — Molecules, per domain (core → fno → quant → wealth → crypto, roughly
ascending complexity).** For each domain, port the `Plot`-eligible chart molecules
first (payoff_chart, backtest_card, cointegration_band, underwater_gauge,
asset_allocation_bar, risk_o_meter, rolling_returns) to validate `nsh_base::Plot`'s real
capability against honba's actual data shapes (Decimal-valued series, log/linear toggles,
drawdown shading) — this is the single highest-leverage validation in the whole port, since
6+ molecules depend on it. If `Plot` can't do what `openalgo-charts-rs`/raw SVG currently
do, fall back to `Canvas` and re-scope. Then the input-composition molecules
(estimation_fields, price_snap_input, quantity_chip_input, sip_calculator), then the
remaining bespoke `View` compositions.

**Phase 3 — Organisms.** Order by risk, highest first: **order_book** (perf validation of
the op-diff/List-virtualization model under tick-rate updates — this is the organism most
likely to expose whether NSH's "one MountBatch per turn" model actually beats Leptos's
fine-grained signals for a depth ladder), then option_chain, order_ticket, strategy_builder,
terminal_shell, trading_dock, mutual_fund_dossier, market_tape, market_top_bar,
collateral_organism.

**Phase 4 — Templates + shell.** mosaic_workspace onto `Dock`/`DockArea` (should be
close to a rewrite-free swap per §3), two_columns/three_columns onto `Resizable`, page
last. Stand up the top-level `App` `#[component]` and decide the routing story (see §6 —
`leptos_router`'s URL-based routing has no confirmed NSH equivalent; this phase blocks on
resolving that open question).

**Phase 5 — App wiring.** Port `honba-kao-app`'s feature pages onto the new primitives/
molecules/organisms, wire `honba-kao-client`'s existing REST/WS/SSE plumbing (untouched —
it's transport-layer Rust, no UI dependency) and `honba-kao-api-wasm`'s pure-Rust finance
math (also untouched; only its wasm-bindgen export surface is web-specific and can stay as
is for the web target).

**Phase 6 — Showcase (§5).** Build last, as the regression net for everything above — every
primitive and molecule gets a page in it, so Phase 6 can start in parallel with Phase 1 and
grow incrementally rather than being a big-bang final step.

---

## 5. Showcase page design (model: NSH's own `/gallery` + `/elements`)

A dedicated `honba-kao-showcase` binary (new workspace member) using `mosaic_workspace` as
its own shell, with a left `NavStack` (or simple sidebar `List`) of sections mirroring
`/elements`'s "Primitives | Base" split, extended with honba's domains:

1. **Primitives** — one row per primitive (24), each showing default + variant props
   side by side with the prop values visible as text (NSH's `/elements` pattern: a table of
   "element → native implementation" per entry). sparkline/identicon get a live-updating
   mock-data variant to stress-test the Canvas rewrite.
2. **Molecules by domain** — 5 tabs (core/crypto/fno/quant/wealth) using `nsh_base::Tabs`,
   each tab a grid of molecule cards fed by static mock structs (reuse the Decimal-typed
   fixture data patterns already implied by `OptionLegQuote`, `BookLevel`, etc. in the
   current organism source).
3. **Live organisms** — a dedicated "Trading Floor" section with four tiles:
   - `market_tape` driven by a `Signal<Vec<Tick>>` fed from a local interval timer
     (mock, no network) ticking through a canned price series.
   - `order_book` with a mock bid/ask ladder that reshuffles every ~200ms to exercise the
     op-diff path under realistic churn.
   - `option_chain` with a static 10-strike mock chain (reuse `OptionStrikePair`/
     `OptionLegQuote` shapes) and working strike-row selection callbacks wired to a
     console/log panel instead of a real order ticket.
   - `strategy_builder` with 2-3 pre-populated legs showing the payoff_chart molecule
     live-recomputing as a mock leg is dragged/edited.
4. **Templates** — a page showing mosaic_workspace/two_columns/three_columns each holding
   placeholder panels, to validate `Dock`/`Resizable` drag behavior in isolation from real
   data.

All mock data lives in one `showcase/fixtures.rs` module (Decimal literals via
`rust_decimal_macros::dec!`), so the showcase never depends on `honba-kao-client` and can be
built/deployed independently as a living visual regression target.

---

## 6. Risks and open questions

1. **Project maturity is the dominant risk.** `crates.io` shows `nsh-core`, `nsh-base`,
   `nsh-cli`, `nsh-engine`, `nsh-js`, and 20+ sibling crates were all **first published
   2026-10-17 to 2026-10-22** (7-12 days before this research) and are all pinned at
   **v0.3.0 in lockstep**, with the last bump 2026-10-26 (3 days prior). The GitHub repo has
   845 stars / 59 forks / 29 open issues / 18 open PRs / 593 commits, and **no tagged
   release** is visible — this is a pre-1.0, rapidly-churning project, not yet a safe
   foundation for a production trading terminal. Expect breaking API changes between any
   two `cargo update`s during the port.
2. **License compatibility is fine.** NSH is **Apache-2.0** (confirmed on the GitHub
   license badge and `/docs/installation`); honba-kao is AGPL-3.0. Apache-2.0 is
   copyleft-compatible as an upstream dependency of AGPL-3.0 code (the FSF treats
   Apache-2.0 as GPLv3-compatible, and AGPL-3.0 inherits GPLv3's compatibility rules in
   this direction) — no relicensing is required, only retaining NSH's `LICENSE-APACHE`/
   `NOTICE` attribution in the vendored-dependency sense. Docs are CC BY 4.0, irrelevant to
   code distribution.
3. **No SVG primitive.** NSH's primitive list is `View, Text, Image, Scroll, List, Canvas,
   Shader, WebView, Video, Overlay, If/Each` — no `Svg`. `sparkline.rs` (and possibly the
   icon set) currently render raw `<svg><polyline>`/path markup; porting means rewriting
   that as imperative `Canvas` draw calls. This is a known, scoped cost (§3, §4 Phase 1),
   not a blocker, but it's real engineering work that Leptos's native SVG support doesn't
   require today.
4. **`openalgo-charts-rs`'s Canvas API compatibility is unverified.** Its `canvas/{engine,
   bridge}.rs` naming strongly suggests a hand-rolled `web_sys::CanvasRenderingContext2d`
   bridge. NSH's `Canvas` primitive exposes *some* 2D drawing API, but this research pass
   did not confirm its exact surface (immediate-mode path/arc/fill calls vs. something
   else) against what `openalgo-charts-rs`'s `drawing.rs`/`indicators.rs` actually call.
   **Action before Phase 2's chart work**: read `nsh-core`'s `Canvas` API docs/source
   directly and diff it against `openalgo-charts-rs/src/canvas/bridge.rs`.
5. **Routing model is unconfirmed.** honba-kao-app uses `leptos_router` for URL-based page
   routing (deep-linking, browser back/forward). NSH's documented navigation primitive is
   `NavStack`/`NavPage` (`nsh_base`), which reads as a mobile-style push/pop stack, not
   necessarily URL-synced on web. Needs a dedicated check of `/docs/platforms` or the web
   backend docs for a router/history API before Phase 4 can be scoped confidently.
6. **The reactive graph is QuickJS-backed internally, even for pure-Rust apps** (per
   `/docs/architecture`: signal nodes are "QuickJS objects of native classes"). This doesn't
   reintroduce JS authoring and doesn't contradict the "Rust-only" decision in §1, but it
   means "no JS anywhere" isn't literally true of the runtime — worth knowing before
   making strong claims about the stack to others.
7. **Native packaging is explicitly unfinished.** `/docs/installation` states "the
   binaries are not signed yet," and there's no prebuilt Intel-Mac or arm64-Linux/Windows
   release (source build required). Irrelevant if this port stays Web-target-only (as
   honba-kao is today), but blocks any near-term move to native macOS/Windows/Linux
   trading-terminal builds.
8. **Tooling swap.** `Trunk.toml`/`trunk serve` → `nsh.json`/`nsh dev` (dev server in
   `nsh-dev`), `nsh package --target web` replaces `trunk build --release`. This is a
   mechanical Makefile/CI change but touches every dev's local workflow and should be
   called out in the PR, per this workspace's CI-matrix rule for Rust repos
   (`cargo fmt`/`clippy`/`test` still apply unchanged to the Rust source itself).
9. **`nsh_base::Plot`'s real feature depth is unverified beyond the `/elements` summary**
   (`Line, Area, Bar, Grid, Pie, Arc`). Six-plus honba molecules (§3 Phase 2) are proposed
   to ride on it; if it can't do log-scale axes, drawdown-shaded areas, or Decimal-precision
   tooltips, those molecules fall back to bespoke `Canvas`, which changes the Phase 2
   estimate materially. Validate early, not late.

---

**Bottom line for whoever reads this next**: the architecture maps cleanly (Dock for
mosaic_workspace/trading_dock, Plot for most of the chart molecules, base controls for most
form-shaped primitives, `#[component]` is a near-1:1 analogue of Leptos's own macro) and the
Web target genuinely compiles Rust to `wasm32-unknown-unknown` with no SSR/JS detour. The
open question that should gate a go/no-go decision isn't the architecture — it's whether a
framework whose entire crate family was published 7-12 days ago at a lockstep `v0.3.0` is an
acceptable foundation to migrate a production trading terminal onto right now, versus
tracking it for another 2-3 release cycles first.
