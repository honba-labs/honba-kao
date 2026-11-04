# `honba-kao-assets`

Canonical static asset library for the Honba Kao UI (`honba-kao`): fonts,
branding, OMS audio cues, themes, and offline market-simulation fixtures in
one place, compiled into the binary and registered with the NSH engine.

Static resources were previously fragmented between `honba-kao-showcase`
(fonts, logo) and `honba-kao-app` (legacy CSS + `.woff` faces unusable by
native NSH targets), with no institutional trading audio cues and no unified
theme token file. This crate is the single source of truth:

1. **Typography & TrueType Fonts** — `Exo 2` and `Roboto Mono` as native NSH
   `FontAsset` family handles.
2. **Branding & Graphic Media** — high-DPI logos, app icon, favicon, and
   exchange/broker emblem badges.
3. **Institutional OMS Audio Cues** — low-latency PCM/WAV bites for order
   placement, fills, rejections, and margin alerts.
4. **Theme Palettes & Tokens** — canonical `theme.json` in the shadcn
   `cssVars` shape parsed by `nsh_base::Theme::from_json`.
5. **Offline Market Simulation Fixtures** — a replay tick stream and an L2
   order book snapshot, zero I/O at runtime.

## Tree

```
crates/honba-kao-assets/
├── Cargo.toml
├── README.md
├── assets/                             <-- shipped via asset!()/assets!()
│   ├── fonts/                          <-- Exo 2 (400/600/700/800),
│   │                                       Roboto Mono (Regular/Bold), OFL.txt
│   ├── branding/                       <-- logo@2x, logo-monochrome@2x,
│   │                                       app-icon.png, favicon.ico
│   ├── emblems/                        <-- exchanges/{nse,bse,mcx}.png,
│   │                                       brokers/{zerodha,interactive_brokers,openalgo}.png
│   ├── audio/                          <-- 7 OMS cues (order_submitted, order_filled,
│   │                                       order_cancelled, order_rejected,
│   │                                       margin_warning, price_alert, ping_connected)
│   ├── themes/                         <-- theme.json, theme-monokai.json,
│   │                                       theme-bloomberg.json (shadcn cssVars)
│   └── fixtures/                       <-- orderbook_l2_snapshot.json,
│                                           replay_sample_ticks.bin
├── src/
│   ├── lib.rs                          <-- root module + registry()
│   ├── fonts.rs                        <-- FontAsset family handles
│   ├── branding.rs                     <-- ImageAsset constants + emblem modules
│   ├── audio.rs                        <-- SoundCue enum + play()
│   ├── themes.rs                       <-- theme JSON constants + parser helpers
│   └── fixtures.rs                     <-- raw fixture accessors + replay header parse
└── tests/
    └── assets_test.rs                  <-- magic bytes, sizes, family pins, themes
```

## Consumer wiring

```toml
# crates/honba-kao-ui-core/Cargo.toml
[dependencies]
honba-kao-assets.workspace = true
```

In `crates/honba-kao-ui-core/src/theme/tokens.rs`:

```rust
pub use honba_kao_assets::fonts::{EXO2, MONO, FAMILY_EXO2, FAMILY_MONO};
pub use honba_kao_assets::themes::default_theme;
```

Native desktop runners register the whole tree once:

```rust
platform::set_ui_sets(vec![
    nsh_base::registry(),
    honba_kao_assets::registry(),
    honba_kao_ui_core::registry(),
]);
```

`fonts::{EXO2, MONO}` are **family handles** (a `FontAsset` carries the
family, not a face): all static faces ship via `assets!()` and NSH maps the
requested `FontWeight` onto the nearest registered face. `FAMILY_EXO2`
(`"Exo 2"`) and `FAMILY_MONO` (`"Roboto Mono"`) are the string constants for
Tailwind `nsh.json` / CSS authoring, pinned by `tests/assets_test.rs` to the
`.family` field of each handle.

## Notes

- **Generated stand-ins**: the audio cues, branding derivatives
  (logo-monochrome, app-icon), and exchange/broker emblem badges are
  generated stand-ins pending final art and sound design. Layouts and
  constants are final; only the bytes will be swapped.
- **Web font layer**: the self-hosted `woff2`/`woff` web faces (Inter, IBM
  Plex Mono, JetBrains Mono, Roboto Mono subsets + `fonts.css` @font-face
  layer, spec §4.6) live under `assets/fonts/web/` and land in a follow-up
  slice. Nothing in this crate references those files yet; native desktop
  keeps loading the `EXO2` / `MONO` TTFs via `asset!`.
- **Themes**: all three themes ship in the shadcn **cssVars** format
  (`{"name", "cssVars": {"theme", "light", "dark"}}`) that
  `nsh_base::Theme::from_json` parses — see the spec §5 mapping table from
  honba design tokens (`surface`, `bullish`, `bearish`, …) onto cssVars keys.
  Light and dark blocks share one key set in every file.
- **Audio playback**: `SoundCue::play()` is a no-op unless the `audio`
  cargo feature is enabled (native desktop; `rodio` output thread). WAV
  bytes are always compiled in via `wav_bytes()` regardless of feature.
- **Zero-float invariant**: price/quantity values in fixtures cross the wire
  as decimal **strings** or scaled integers (`*_scaled_e4`), never floats.
