//! Platform typography: the two NSH font families the Honba Kao UI is
//! built on, exposed as **family handles**.
//!
//! All static faces (Exo 2 400/600/700/800 and Roboto Mono Regular/Bold)
//! ship via `assets!()`; a NSH `FontAsset` carries the family, not a
//! face, so call sites hold one handle per family and select the weight
//! with `.font_weight(FontWeight::…)`.
//!
//! > **Design note (weight handles vs. family handles):** a NSH `FontAsset`
//! > carries the **family**, not a face — `asset!` reads `"Exo 2"` from the
//! > file at compile time, so per-weight flat constants would expose the
//! > same family under six names and invite passing the semibold handle
//! > while expecting semibold rendering. All faces (400/600/700/800) ship
//! > via `assets!()` and NSH picks the face per
//! > `.font_weight(FontWeight::SEMIBOLD)`; call sites hold **one family
//! > handle per family** (the nsh docs' own pattern:
//! > `const INTER: FontAsset = asset!("fonts/Inter-Regular.ttf");`).

use nsh_core::element::{FontAsset, asset};

/// The primary UI typeface: **Exo 2** (Open Font License). Family handle
/// (`asset!` reads `"Exo 2"` from the file at compile time). The static
/// faces (400 / 600 / 700 / 800) ship via `assets!()`; select a face with
/// `.font_weight(FontWeight::SEMIBOLD)` — the engine maps the requested
/// weight onto the nearest registered face.
pub const EXO2: FontAsset = asset!("fonts/Exo2-Regular.ttf");

/// The financial monospace: **Roboto Mono** (OFL) for order book levels,
/// trade tape sizes, and PnL readouts. Ships Regular + Bold faces;
/// bold PnL digits select the 700 face via `.font_weight(FontWeight::BOLD)`.
pub const MONO: FontAsset = asset!("fonts/RobotoMono-Regular.ttf");

/// Font family name string constants for Tailwind theme (`nsh.json`) and
/// CSS authoring. `tests/assets_test.rs` pins them to the `.family` field
/// of each handle, so a re-export whose file family drifted fails the build.
pub const FAMILY_EXO2: &str = "Exo 2";
pub const FAMILY_MONO: &str = "Roboto Mono";

// The noFX / longbridge-terminal web faces (`Inter`, `IBM Plex Mono`,
// `JetBrains Mono` — the nsh default theme's recommended `font-mono`). A
// NSH `asset!` cannot consume `woff2`/`woff` (native `FontAsset` supports
// `.ttf`/`.otf` only), so these ship as plain bundled files under
// `assets/fonts/web/` for the web targets and are registered to the
// browser via the generated `assets/fonts/web/fonts.css` @font-face layer
// (spec §4.6 — the web font layer lands in a follow-up slice; nothing in
// this module references it). Native desktop targets use the `EXO2` /
// `MONO` TTFs instead.
