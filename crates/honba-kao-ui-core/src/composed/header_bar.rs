//! HfHeaderBar — logo, nav, mode toggle, theme, notifications.
//! Mirrors bfx-hf-ui-core Navbar component.

use leptos::prelude::*;
use crate::{HfIconButton, HfButton, HfButtonVariant, HfButtonSize};
use crate::icons::outline::{chart, settings, bell};

#[component]
pub fn HfHeaderBar(
    /// Current active page for nav highlighting.
    #[prop(optional)]
    _current_page: Option<&'static str>,
    /// Brand logo text or icon path.
    #[prop(optional, into)]
    brand: Option<String>,
    /// Whether to show the trading mode toggle.
    #[prop(optional)]
    show_trading_mode: Option<bool>,
    /// Whether to show the theme selector.
    #[prop(optional)]
    show_theme: Option<bool>,
    /// Whether to show notifications bell.
    #[prop(optional)]
    show_notifications: Option<bool>,
    children: Children,
) -> impl IntoView {
    let show_trading_mode = show_trading_mode.unwrap_or(true);
    let show_theme = show_theme.unwrap_or(true);
    let show_notifications = show_notifications.unwrap_or(true);

    // Convert brand Option<String> to a renderable view
    let brand_view = move || {
        brand.clone().map(|b| view! { <span class="hf-header-bar__brand-text">{b}</span> }.into_any())
            .unwrap_or_else(|| view! { <HfIconButton icon=chart() size=16 /> }.into_any())
    };

    view! {
        <header class="hf-header-bar">
            <div class="hf-header-bar__brand">
                {brand_view}
            </div>

            <nav class="hf-header-bar__nav">
                {children()}
            </nav>

            <div class="hf-header-bar__actions">
                {move || if show_trading_mode {
                    view! { <HfButton variant=HfButtonVariant::Green size=HfButtonSize::Small>"LIVE NSE"</HfButton> }.into_any()
                } else { ().into_any() }}

                {move || if show_theme {
                    view! { <HfIconButton icon=settings() size=16 /> }.into_any()
                } else { ().into_any() }}

                {move || if show_notifications {
                    view! { <HfIconButton icon=bell() size=16 /> }.into_any()
                } else { ().into_any() }}
            </div>
        </header>
    }
}