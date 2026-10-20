//! Procedural outline icons — stroke only, fill="none", no static .svg files
//!
//! All icons are generated on-the-fly. Each icon function returns the SVG path
//! `d` data as a `&'static str`. A single `SvgIcon` component renders the SVG
//! wrapper. No static .svg files exist in the repo.
//!
//! CI check: `grep -r "\.svg" crates/honba-kao-app/src --include="*.rs" && exit 1`

use leptos::prelude::*;

/// Renders an outline icon given path data.
///
/// ```rust
/// use leptos::prelude::*;
/// use honba_kao_ui_core::icons::outline::{chart, SvgIcon};
///
/// #[component]
/// fn ChartGlyph() -> impl IntoView {
///     view! { <SvgIcon path=chart() size=16 /> }
/// }
/// ```
#[component]
pub fn SvgIcon(
    /// The SVG path `d` data (from any `outline::*` function).
    path: &'static str,
    /// Size in CSS pixels (16, 20, 24 default, etc.).
    #[prop(optional)]
    size: Option<u32>,
    /// Extra CSS classes.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let size_px = size.unwrap_or(24);
    let size_class = match size {
        Some(16) => "hf-icon--size-16",
        Some(20) => "hf-icon--size-20",
        _ => "hf-icon--size-24",
    };

    let full_class = format!("hf-icon {size_class} {class}").trim().to_string();

    view! {
        <svg
            viewBox="0 0 24 24"
            stroke="currentColor"
            fill="none"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=full_class
            width=size_px.to_string()
            height=size_px.to_string()
        >
            <path d=path />
        </svg>
    }
}

/// HfIcon wrapper — convenience component around `SvgIcon`.
///
/// ```rust
/// use leptos::prelude::*;
/// use honba_kao_ui_core::icons::outline::{chart, HfIcon};
///
/// #[component]
/// fn ChartGlyph() -> impl IntoView {
///     view! { <HfIcon path=chart() size=16 /> }
/// }
/// ```
#[component]
pub fn HfIcon(
    /// The SVG path `d` data (from any `outline::*` function).
    path: &'static str,
    /// Size in pixels (16, 20, 24 default).
    #[prop(optional)]
    size: Option<u32>,
    /// Extra CSS classes.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    view! {
        <SvgIcon path=path size=size.unwrap_or(24) class=class />
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Trading Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const fn chart() -> &'static str { "M13 2L3 14l9-7" }

pub const fn candlestick() -> &'static str { "M9 18h4M5 18h4a2 2 0 012-2v-4a2 2 0 012 2h4a2 2 0 012-2v-4a2 2 0 012-2h4a2 2 0 012 2v14a2 2 0 01-2 2H9a2 2 0 01-2-2v-4a2 2 0 012-2z" }

pub const fn trending_up() -> &'static str { "M23 19L13 9L9 13L5 9L2 12" }

pub const fn trending_down() -> &'static str { "M23 5L13 15L9 11L5 15L2 12" }

pub const fn buy() -> &'static str { "M12 2L2 7l10 5 10-5-10-5zM12 22l-10-5 10-5 10 5-10 5z" }

pub const fn sell() -> &'static str { "M12 22L2 17l10-5 10 5-10 5zM12 2l10 5-10-5-10 5z" }

pub const fn order() -> &'static str { "M3 3h18v18H3zM12 3v18M3 12h18" }

pub const fn position() -> &'static str { "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" }

pub const fn portfolio() -> &'static str { "M3 3h18v2H3zm18 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" }

pub const fn wallet() -> &'static str { "M21 12V7H5a2 2 0 0 1 0-4h14v4M3 5v14a2 2 0 0 0 2 2h16v-5M18 12a2 2 0 0 0 0 4h4v-4z" }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Navigation Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const fn home() -> &'static str { "M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" }

pub const fn settings() -> &'static str { "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 0 0-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 0 0-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 0 0-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 0 0-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 0 0 1.066-2.573c-.94-1.543.826-3.31 2.37-2.37zM12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6" }

pub const fn search() -> &'static str { "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" }

pub const fn menu() -> &'static str { "M3 12h18M3 6h18M3 18h18" }

pub const fn chevron_left() -> &'static str { "M15 18l-6-6 6-6" }

pub const fn chevron_right() -> &'static str { "M9 18l6-6-6-6" }

pub const fn chevron_up() -> &'static str { "M18 15l-6-6-6 6" }

pub const fn chevron_down() -> &'static str { "M6 9l6 6 6-6" }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Action Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const fn plus() -> &'static str { "M12 5v14M5 12h14" }

pub const fn minus() -> &'static str { "M5 12h14" }

pub const fn x_close() -> &'static str { "M18 6L6 18M6 6l12 12" }

pub const fn check() -> &'static str { "M5 13l4 4L19 7" }

pub const fn edit() -> &'static str { "M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" }

pub const fn delete() -> &'static str { "M3 6h18M3 6v14a2 2 0 002 2h12a2 2 0 002-2V6M10 11v6M14 11v6" }

pub const fn copy() -> &'static str { "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v8m-6 0H6m8 0V8m-4 8H8" }

pub const fn download() -> &'static str { "M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" }

pub const fn upload() -> &'static str { "M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l5 5 5-5M12 9V21" }

pub const fn refresh() -> &'static str { "M23 3v10h-4M3 3v10h4M3 16l4-4m0 0l4-4m-4 4v-6" }

pub const fn filter() -> &'static str { "M18 3V6M14 3V6M10 3V6M3 9h18M3 13h18" }

pub const fn sort() -> &'static str { "M7 11V5l5-5 5 5v6zM3 19h18v-6H3z" }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Status Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const fn check_circle() -> &'static str { "M22 11.08V12a10 10 0 11-5.93-9.14" }

pub const fn alert_circle() -> &'static str { "M22 12a10 10 0 1 0-20 0 10 10 0 0 0 20 0M12 8v4M12 16h.01" }

pub const fn alert_triangle() -> &'static str { "M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0zM12 17a2 2 0 100-4 2 2 0 000 4z" }

pub const fn info() -> &'static str { "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }

pub const fn loader() -> &'static str { "M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }

pub const fn wifi() -> &'static str { "M12 20h.01M2 8.82a15 15 0 0 1 20 0M5 12.859a10 10 0 0 1 14 0M8.5 16.429a5 5 0 0 1 7 0" }

pub const fn wifi_off() -> &'static str { "m2 2 20 20M8.5 16.427a5 5 0 0 1 7 0M5 12.859a10 10 0 0 1 5.17-2.69M19 12.859a10 10 0 0 0-2.007-1.523M2 8.82a15 15 0 0 1 4.177-2.643M12 20h.01" }

pub const fn server() -> &'static str { "M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Chart Type Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const fn chart_type_candle() -> &'static str { "M2 3h6a2 2 0 012 2v14a2 2 0 01-2 2H2a2 2 0 01-2-2V5a2 2 0 012-2z" }

pub const fn chart_type_line() -> &'static str { "M3 4v16h18V4zM9 14l6-6m0 0L3 2m6 6l6 6" }

pub const fn chart_type_area() -> &'static str { "M3 4v16h18V4zM21 4a2 2 0 012 2v14a2 2 0 01-2 2H3a2 2 0 01-2-2V6a2 2 0 012-2z" }

pub const fn timeframe_1m() -> &'static str { "M12 2v20M17 5H9.5" }

pub const fn timeframe_1h() -> &'static str { "M12 2v20M17 5H9.5M17 15H9.5" }

pub const fn timeframe_1d() -> &'static str { "M12 2v20M9.5 17H17M17 15H9.5" }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Drawing Tool Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const fn trend_line() -> &'static str { "M4.5 11.5L11.5 4.5M4.5 10.6a1.1 1.1 0 110 2.2 1.1 1.1 0 010-2.2M11.5 2.6a1.1 1.1 0 110 2.2 1.1 1.1 0 010-2.2" }

pub const fn horizontal_line() -> &'static str { "M2 8h12M7 8a1 1 0 112 0 1 1 0 01-2 0" }

pub const fn vertical_line() -> &'static str { "M8 2v12M8 7a1 1 0 110 2 1 1 0 010-2" }

pub const fn ray() -> &'static str { "M3 12L13 4M3 12a1.1 1.1 0 110-2.2 1.1 1.1 0 010 2.2" }

pub const fn rectangle() -> &'static str { "M3 4h10v8H3z" }

pub const fn ellipse() -> &'static str { "M3 8a5 4 0 1110 0a5 4 0 01-10 0" }

pub const fn text() -> &'static str { "M4 4h8M8 4v9" }

pub const fn measure() -> &'static str { "M2 10.5L10.5 2l3.5 3.5L5.5 14zM6 6.5l2 2M8.5 4l2 2M3.5 9l2 2" }

pub const fn fib_retracement() -> &'static str { "M4.5 17.5l16-16M4.5 16.5l16-16M4.5 15.5l16-16M4.5 14.5l16-16M4.5 13.5l16-16M4.5 12.5l16-16" }

pub const fn gann_fan() -> &'static str { "M3 12L21 4M3 12L21 8M3 12L21 12M3 12L21 16M3 12L21 20" }

pub const fn parallel_channel() -> &'static str { "M3 4.5l10 6M3 8.5l10 6M3 12.5l10 6" }

pub const fn regression_channel() -> &'static str { "M4.5 17.5l16-16M4.5 16.5l16-16M4.5 15.5l16-16" }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// UI Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const fn bell() -> &'static str { "M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" }

pub const fn bell_off() -> &'static str { "M13.73 21a2 2 0 01-3.46 0" }

pub const fn eye() -> &'static str { "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8zM12 8c5.5 0 8 5 8 5s-2.5 5-8 5-8-5-8-5s2.5-5 8-5z" }

pub const fn eye_off() -> &'static str { "M1 12s4-8 11-8 11 8-11 8-11-8-11-8z" }

pub const fn lock() -> &'static str { "M5 12v6a2 2 0 002 2h10a2 2 0 002-2v-6M8 12a2 2 0 114 0V4a2 2 0 00-2-2H8a2 2 0 00-2 2z" }

pub const fn unlock() -> &'static str { "M11 12V6a2 2 0 012-2h8a2 2 0 012 2v6a2 2 0 01-2 2H13a2 2 0 01-2-2z" }

pub const fn key() -> &'static str { "M21 2l-2 2M7.7 7.7a2.3 2.3 0 010 3.24 2.3 2.3 0 01-3.24 0L3.51 7.49 3 9.38l-1.32 1.42a2 2 0 00-.7 1.07L1.58 14a2 2 0 000 2l4.33 4.33a2 2 0 001.42.58h2a2 2 0 002-2v-2a2 2 0 00-.58-1.42l1.42-1.42a2 2 0 000-2.83z" }

/// Lucide `user`: head circle (drawn as arc subpath) + shoulders.
pub const fn user() -> &'static str { "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2M12 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8" }

pub const fn log_out() -> &'static str { "M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5m0 0l-5-5m5 5V9" }

pub const fn log_in() -> &'static str { "M15 21h4a2 2 0 002-2V5a2 2 0 00-2-2h-4M7 7l5 5m0 0l-5 5m5-5V3" }

pub const fn sun() -> &'static str { "M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41M12 8a4 4 0 100 8 4 4 0 000-8z" }

pub const fn moon() -> &'static str { "M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" }

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Chrome Icons — the app shell: nav rail destinations and the account cluster
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Lucide `briefcase` — Portfolio rail destination.
pub const fn briefcase() -> &'static str { "M16 6V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2M4 6h16a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2M3 11h18" }

/// Search circle with an inline check — Screener rail destination.
pub const fn search_check() -> &'static str { "M18 11a7 7 0 1 0-14 0 7 7 0 0 0 14 0M21 21l-4.35-4.35M8 11.5 10 14l4.5-5" }

/// Lucide `terminal` — Workbench rail destination.
pub const fn terminal() -> &'static str { "M4 17l6-6-6-6M12 19h8" }

/// Lucide `sliders` — Strategy rail destination.
pub const fn sliders() -> &'static str { "M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6" }

/// Lucide `sliders-horizontal` — horizontal filter controls.
pub const fn sliders_horizontal() -> &'static str { "M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3M14 2v4M8 10v4M16 18v4" }

/// Lucide `circle-play` — Simulator rail destination.
pub const fn play_circle() -> &'static str { "M21 12a9 9 0 1 0-18 0 9 9 0 0 0 18 0M10 8l6 4-6 4z" }

/// Lucide `book` — Tradebook rail destination.
pub const fn book() -> &'static str { "M4 19.5A2.5 2.5 0 0 1 6.5 17H20M6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" }

/// 2×2 grid of swatches — palette studio, layout pickers.
pub const fn layout_grid() -> &'static str { "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z" }

/// Lucide `signal` (vertical bars) — live data / trading-mode indicator.
pub const fn signal() -> &'static str { "M2 20h.01M7 20v-4M12 20v-8M17 20V8M22 20V4" }

/// Lucide `file-text` — paper trade documents / paper mode.
pub const fn file_text() -> &'static str { "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7zM14 2v5h5M9 12h6M9 16h6M9 8h1" }