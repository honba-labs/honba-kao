//! Procedural outline SVG glyphs with zero external asset dependencies.
//!
//! Each function returns an inline `<svg>` fragment that `leptos` can render
//! directly inside any `Hf*` component.

use leptos::prelude::*;

/// Currency: Rupee (₹)
#[component]
pub fn SvgRupee() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-currency" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M21 6H3v12h15v-6h-6l3-6z" />
        </svg>
    }
}

/// Currency: Dollar ($)
#[component]
pub fn SvgDollar() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-currency" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M12 1a9 9 0 0 0-9 9 9 9 0 0 0 9 9 9 9 0 0 0 9-9 9 9 0 0 0-9-9Zm0 6h-4m4 4h4m-4-8V6m0 12V12m-4 6h4" />
        </svg>
    }
}

/// Currency: Bitcoin (₿)
#[component]
pub fn SvgBtc() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-currency" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M14 8 12 10l2 2M20 12h-8l-2 2-2-2H4l2-2 2 2 2-2h8l2 2z" />
        </svg>
    }
}

/// Currency: Ethereum (Ξ)
#[component]
pub fn SvgEth() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-currency" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M10 14l2-2 2 2M2 12h20M12 2v20" />
        </svg>
    }
}

/// Currency: USDC
#[component]
pub fn SvgUsdc() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-currency" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M12 2a9 9 0 0 0-9 9v2a9 9 0 0 0 9 9 9 9 0 0 0 9-9v-2a9 9 0 0 0-9-9Zm0 6h-4m4 4h4m-4-8v-6m0 12v-6" />
        </svg>
    }
}

/// Sort chevron up / down
#[component]
pub fn SvgSortChevron() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-sort" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="18 6 12 12 6 6" />
            <polyline points="18 18 12 12 6 18" />
        </svg>
    }
}

/// Sort ascending only
#[component]
pub fn SvgSortAsc() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-sort" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="18 6 12 12 6 6" />
        </svg>
    }
}

/// Sort descending only
#[component]
pub fn SvgSortDesc() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-sort" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="18 18 12 12 6 18" />
        </svg>
    }
}

/// Close / dismiss (X)
#[component]
pub fn SvgClose() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-close" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M18 6 6 18M6 6l12 12" />
        </svg>
    }
}

/// External link / open in new
#[component]
pub fn SvgExternalLink() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-external" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M18 13v6a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2v-8M15 3h6v6M10 14L21 3" />
        </svg>
    }
}

/// Chevron right
#[component]
pub fn SvgChevronRight() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="9 18 15 12 9 6" />
        </svg>
    }
}

/// Chevron left
#[component]
pub fn SvgChevronLeft() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="15 18 9 12 15 6" />
        </svg>
    }
}

/// Chevron down
#[component]
pub fn SvgChevronDown() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="6 9 12 15 18 9" />
        </svg>
    }
}

/// Chevron up
#[component]
pub fn SvgChevronUp() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="18 15 12 9 6 15" />
        </svg>
    }
}

/// Plus / add
#[component]
pub fn SvgPlus() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-action" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M12 5v14M5 12h14" />
        </svg>
    }
}

/// Minus / remove
#[component]
pub fn SvgMinus() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-action" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
    }
}

/// Menu / hamburger
#[component]
pub fn SvgMenu() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-action" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M4 6h16M4 12h16M4 18h16" />
        </svg>
    }
}

/// Search / magnifying glass
#[component]
pub fn SvgSearch() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-action" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <circle cx="11" cy="11" r="8" />
            <path d="M21 21l-4.35-4.35" />
        </svg>
    }
}

/// Alert triangle
#[component]
pub fn SvgAlert() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-status" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
    }
}

/// Info circle
#[component]
pub fn SvgInfo() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-status" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 16a4 4 0 0 1-4-4V4a8 8 0 0 1 8 8v4a4 4 0 0 1-4 4Z" />
        </svg>
    }
}

/// Success checkmark
#[component]
pub fn SvgCheck() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-status" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <polyline points="20 6 9 17 4 12" />
        </svg>
    }
}

/// Spinner circle (animated via CSS `hf-spin`)
#[component]
pub fn SvgSpinner() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-spinner animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="3" stroke_linecap="round" aria_hidden="true">
            <path d="M21 12a9 9 0 1 1-6.242-8.127" />
        </svg>
    }
}

/// Order book / ladder icon
#[component]
pub fn SvgOrderBook() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-market" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M3 3v18h18V3H3zM3 9h18M3 15h18" />
        </svg>
    }
}

/// Wallet / account glyph
#[component]
pub fn SvgWallet() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-account" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2M3 3v18h18V3H3z" />
        </svg>
    }
}

/// Lock / security glyph
#[component]
pub fn SvgLock() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-security" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
    }
}

/// Terminal prompt symbol (>_)
#[component]
pub fn SvgPrompt() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-terminal" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="18" y2="13" />
            <line x1="16" y1="17" x2="18" y2="17" />
        </svg>
    }
}

/// Thinking / processing orb
#[component]
pub fn SvgThinkingOrb() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-agent animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" aria_hidden="true">
            <path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zM12 12m-3 0a3 3 0 1 0 6 0a3 3 0 0 0-6 0z" />
        </svg>
    }
}

/// Sentiment bullish (arrow up)
#[component]
pub fn SvgSentimentBullish() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-sentiment" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M5 12h14M12 19l5-5-5-5" />
        </svg>
    }
}

/// Sentiment bearish (arrow down)
#[component]
pub fn SvgSentimentBearish() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-sentiment" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M5 12h14M12 5l5 5-5 5" />
        </svg>
    }
}

/// Neutral sentiment (dash)
#[component]
pub fn SvgSentimentNeutral() -> impl IntoView {
    view! {
        <svg class="hf-icon hf-icon-sentiment" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke_width="2" stroke_linecap="round" stroke_linejoin="round" aria_hidden="true">
            <path d="M5 12h14" />
        </svg>
    }
}
