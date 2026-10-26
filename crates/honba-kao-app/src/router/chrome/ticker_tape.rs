//! Horizontal ticker tape under the header bar: one scrolling line of quotes
//! for the mock symbol universe. Reads [`MarketFeed`] (provided by
//! [`crate::router::app_chrome::AppChrome`]), so each row shows the mock row
//! until the feed delivers a live quote, and keeps the last quote when the
//! socket goes offline.

use leptos::prelude::*;

use crate::shared::command::symbols::{MOCK_SYMBOLS, MockSymbol};
use crate::shared::stream::feed::MarketFeed;

#[component]
pub fn TickerTape() -> impl IntoView {
    let feed = expect_context::<MarketFeed>();

    // The track holds two copies so the CSS loop (-50%) is seamless; the
    // second copy is hidden from assistive tech.
    view! {
        <div class="ticker-tape" role="region" aria-label="Market quotes">
            <div class="ticker-tape__track">
                <div class="ticker-tape__copy">
                    {MOCK_SYMBOLS.iter().copied().map(|s| ticker_item(feed, s)).collect_view()}
                </div>
                <div class="ticker-tape__copy" aria-hidden="true">
                    {MOCK_SYMBOLS.iter().copied().map(|s| ticker_item(feed, s)).collect_view()}
                </div>
            </div>
        </div>
    }
}

/// One symbol cell: ticker, LTP and signed change, coloured by sign.
fn ticker_item(feed: MarketFeed, symbol: MockSymbol) -> impl IntoView {
    let quote = feed.quote(symbol.ticker);
    let current = move || {
        quote
            .get()
            .map(|q| (q.ltp, q.change_pct))
            .unwrap_or_else(|| (symbol.ltp.to_string(), symbol.change_pct.to_string()))
    };
    let ltp = move || current().0;
    let change = move || signed_pct(&current().1);
    let tone = move || tone_of(&current().1);

    view! {
        <span class="ticker-tape__item">
            <span class="ticker-tape__symbol">{symbol.ticker}</span>
            <span class="ticker-tape__ltp">{ltp}</span>
            <span
                class="ticker-tape__change"
                class:ticker-tape__change--up=move || tone() == Tone::Up
                class:ticker-tape__change--down=move || tone() == Tone::Down
            >
                {change}
            </span>
        </span>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tone {
    Up,
    Down,
    Flat,
}

/// Sign of a decimal change string, read from its text (no float parsing).
fn tone_of(change: &str) -> Tone {
    let change = change.trim();
    if change.starts_with('-') {
        Tone::Down
    } else if change.chars().all(|c| matches!(c, '0' | '.' | '+')) {
        Tone::Flat
    } else {
        Tone::Up
    }
}

/// Display form of a change percentage, e.g. `+1.4%` or `-0.3%`.
fn signed_pct(change: &str) -> String {
    let value = change.trim().trim_start_matches('+');
    match tone_of(value) {
        Tone::Up => format!("+{value}%"),
        Tone::Down | Tone::Flat => format!("{value}%"),
    }
}
