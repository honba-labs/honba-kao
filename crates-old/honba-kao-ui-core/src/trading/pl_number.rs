//! Trading primitives — bfx-hf-ui-core trading-specific components.
//! These map to the `components/` layer of bfx-hf-ui-core with `Hf` prefix.

use leptos::prelude::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfPLNumber — P&L with ▲/▼, ₹/%, color-coded (teal/coral)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfPLNumber — profit/loss number with directional indicator and color coding.
///
/// Mirrors bfx-hf-ui-core `.hfui-plnumber`.
///
/// ```rust
/// use leptos::prelude::*;
/// use honba_kao_ui_core::{HfPLNumber, HfPLNumberFormat};
///
/// #[component]
/// fn DayPnl() -> impl IntoView {
///     view! { <HfPLNumber value=1250.50 format=HfPLNumberFormat::Currency /> }
/// }
/// ```
#[component]
pub fn HfPLNumber(
    /// The numeric value (positive = gain, negative = loss, zero = neutral).
    value: f64,
    /// Display format.
    #[prop(optional)]
    format: Option<HfPLNumberFormat>,
    /// Show the ▲/▼ directional indicator.
    #[prop(optional)]
    show_direction: Option<bool>,
    /// Show the sign (+/-).
    #[prop(optional)]
    show_sign: Option<bool>,
    /// Extra CSS classes.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let format = format.unwrap_or(HfPLNumberFormat::Currency);
    let show_direction = show_direction.unwrap_or(true);
    let show_sign = show_sign.unwrap_or(true);

    let is_positive = value > 0.0;
    let is_negative = value < 0.0;

    let color_class = if is_positive {
        "positive"
    } else if is_negative {
        "negative"
    } else {
        "neutral"
    };

    let direction = if is_positive {
        "▲"
    } else if is_negative {
        "▼"
    } else {
        "▬"
    };

    let sign = if is_positive && show_sign {
        "+"
    } else if is_negative {
        "-"
    } else {
        ""
    };

    let abs_value = value.abs();

    let formatted = match format {
        HfPLNumberFormat::Currency => format!("{sign}{abs_value:.2}"),
        HfPLNumberFormat::Percent => format!("{sign}{abs_value:.2}%"),
        HfPLNumberFormat::Rupees => format!("₹{sign}{abs_value:.2}"),
    };

    let full_class = format!("hfui-plnumber {color_class} {class}")
        .trim()
        .to_string();

    view! {
        <span class=full_class>
            {move || if show_direction { view! { <span class="hf-plnumber__direction">{direction}</span> }.into_any() } else { ().into_any() }}
            <span class="hf-plnumber__value">{formatted}</span>
        </span>
    }
}

/// HfPLNumber display format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfPLNumberFormat {
    /// Plain number with 2 decimal places
    Currency,
    /// Percentage with % suffix
    Percent,
    /// Indian Rupees with ₹ prefix
    Rupees,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfStatusCircle — Green/Yellow/Red status dot
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfStatusCircle — connection/status indicator dot.
///
/// Mirrors bfx-hf-ui-core `.hfui-statusbar__statuscircle`. The status text is
/// exposed as the accessible label; visible text sits beside it at the call
/// site.
///
/// ```rust
/// use leptos::prelude::*;
/// use honba_kao_ui_core::{HfStatus, HfStatusCircle};
///
/// #[component]
/// fn FeedStatus() -> impl IntoView {
///     view! { <HfStatusCircle status=HfStatus::Connected /> }
/// }
/// ```
#[component]
pub fn HfStatusCircle(
    /// Connection status.
    status: HfStatus,
    /// Extra CSS classes.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let color_class = match status {
        HfStatus::Connected => "green",
        HfStatus::Connecting | HfStatus::Warning => "yellow",
        HfStatus::Offline => "red",
    };

    let status_text = match status {
        HfStatus::Connected => "Connected",
        HfStatus::Connecting => "Connecting",
        HfStatus::Offline => "Offline",
        HfStatus::Warning => "Warning",
    };

    let full_class = format!("hfui-statusbar__statuscircle {color_class} {class}")
        .trim()
        .to_string();

    view! {
        <span class=full_class role="img" aria-label=status_text title=status_text />
    }
}

/// Connection status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfStatus {
    Connected,
    Connecting,
    Offline,
    Warning,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfSideMarker — row marker dot (green/red) for buy/sell
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfSideMarker — buy/sell side indicator dot used in table rows.
///
/// Mirrors bfx-hf-ui-core `.row-marker`.
#[component]
pub fn HfSideMarker(
    /// Side: Buy (green) or Sell (red).
    side: HfSide,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let color_class = match side {
        HfSide::Buy => "green",
        HfSide::Sell => "red",
    };

    let full_class = format!("row-marker {color_class} {class}")
        .trim()
        .to_string();

    view! {
        <span class=full_class />
    }
}

/// Trade side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfSide {
    Buy,
    Sell,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfOrderBadge — order type (MKT/LMT/SL/SL-M), status, side
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfOrderBadge — order type/status badge with color coding.
#[component]
pub fn HfOrderBadge(
    /// Badge label text.
    label: &'static str,
    /// Badge variant.
    #[prop(optional)]
    variant: Option<HfBadgeVariant>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let variant = variant.unwrap_or(HfBadgeVariant::Default);
    let variant_class = match variant {
        HfBadgeVariant::Default => "hf-order-badge--default",
        HfBadgeVariant::Buy => "hf-order-badge--buy",
        HfBadgeVariant::Sell => "hf-order-badge--sell",
        HfBadgeVariant::Limit => "hf-order-badge--limit",
        HfBadgeVariant::Market => "hf-order-badge--market",
        HfBadgeVariant::Stop => "hf-order-badge--stop",
    };

    let full_class = format!("hf-order-badge {variant_class} {class}")
        .trim()
        .to_string();

    view! {
        <span class=full_class>{label}</span>
    }
}

/// Order badge variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfBadgeVariant {
    Default,
    Buy,
    Sell,
    Limit,
    Market,
    Stop,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfRiskBadge — rejection reason (CIRCUIT_FILTER, MAX_DRAWDOWN, etc.)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfRiskBadge — risk rejection reason badge.
#[component]
pub fn HfRiskBadge(
    /// Rejection reason text.
    reason: &'static str,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let full_class = format!("hf-risk-badge {class}").trim().to_string();

    view! {
        <span class=full_class>{reason}</span>
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfMarketPhase — Pre-open/Regular/Closing/Closed indicator
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfMarketPhase — NSE market phase indicator.
#[component]
pub fn HfMarketPhase(
    /// Market phase.
    phase: HfMarketPhaseType,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let phase_class = match phase {
        HfMarketPhaseType::PreOpen => "hf-market-phase--pre-open",
        HfMarketPhaseType::Regular => "hf-market-phase--regular",
        HfMarketPhaseType::Closing => "hf-market-phase--closing",
        HfMarketPhaseType::Closed => "hf-market-phase--closed",
    };

    let phase_text = match phase {
        HfMarketPhaseType::PreOpen => "Pre-open",
        HfMarketPhaseType::Regular => "Regular",
        HfMarketPhaseType::Closing => "Closing",
        HfMarketPhaseType::Closed => "Closed",
    };

    let full_class = format!("hf-market-phase {phase_class} {class}")
        .trim()
        .to_string();

    view! {
        <div class=full_class>{phase_text}</div>
    }
}

/// NSE market phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfMarketPhaseType {
    PreOpen,
    Regular,
    Closing,
    Closed,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfTradingMode — SANDBOX (yellow) vs LIVE NSE (green)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfTradingMode — trading mode toggle indicator (Sandbox vs Live).
#[component]
pub fn HfTradingMode(
    /// Current trading mode.
    mode: HfTradingModeType,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let mode_class = match mode {
        HfTradingModeType::Sandbox => "hf-trading-mode--sandbox",
        HfTradingModeType::Live => "hf-trading-mode--live",
    };

    let mode_text = match mode {
        HfTradingModeType::Sandbox => "SANDBOX",
        HfTradingModeType::Live => "LIVE NSE",
    };

    let full_class = format!("hf-trading-mode {mode_class} {class}")
        .trim()
        .to_string();

    view! {
        <div class=full_class>{mode_text}</div>
    }
}

/// Trading mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HfTradingModeType {
    Sandbox,
    Live,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HfTradingModeToggle — togglable PAPER / LIVE segmented indicator
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HfTradingModeToggle — the togglable header indicator: a segmented
/// PAPER | LIVE pill whose active side colours (amber for paper, pulsing green
/// for live) state which book the operator's orders hit. Clicking a segment
/// calls `on_change`; the caller owns the mode signal.
///
/// `mode` is reactive: pass a `Signal`/`Memo`/stored so the indicator tracks
/// the shared store live.
#[component]
pub fn HfTradingModeToggle(
    /// Current trading mode (reactive).
    #[prop(into)]
    mode: Signal<HfTradingModeType>,
    /// Fires with the newly selected mode.
    on_change: Callback<HfTradingModeType>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let full_class = format!("hf-trading-mode-toggle {class}").trim().to_string();

    let paper_active = Memo::new(move |_| mode.get() == HfTradingModeType::Sandbox);
    let live_active = Memo::new(move |_| mode.get() == HfTradingModeType::Live);

    view! {
        <div class=full_class role="tablist" aria-label="Trading mode">
            <button
                class=move || {
                    if paper_active.get() {
                        "hf-trading-mode-toggle__option hf-trading-mode-toggle__option--paper is-active"
                    } else {
                        "hf-trading-mode-toggle__option hf-trading-mode-toggle__option--paper"
                    }
                }
                role="tab"
                aria-selected=move || paper_active.get().to_string()
                title="Switch to Paper trading"
                on:click=move |_| on_change.run(HfTradingModeType::Sandbox)
            >
                <span class="hf-trading-mode-toggle__dot" aria-hidden="true" />
                <span class="hf-trading-mode-toggle__label">"PAPER"</span>
            </button>
            <button
                class=move || {
                    if live_active.get() {
                        "hf-trading-mode-toggle__option hf-trading-mode-toggle__option--live is-active"
                    } else {
                        "hf-trading-mode-toggle__option hf-trading-mode-toggle__option--live"
                    }
                }
                role="tab"
                aria-selected=move || live_active.get().to_string()
                title="Switch to Live trading"
                on:click=move |_| on_change.run(HfTradingModeType::Live)
            >
                <span class="hf-trading-mode-toggle__dot" aria-hidden="true" />
                <span class="hf-trading-mode-toggle__label">"LIVE"</span>
            </button>
        </div>
    }
}
