#! Top application bar: brand → spacer → actions.
#! Navigation lives in the side icon rail (`nav_rail.rs`) and the account
//! cluster is also rail-owned (bottom-left), so the bar carries only the brand
//! block, the theme/language controls and the trading-mode indicator.
//!
//! Behaviours:
//!   * sticky, and it lifts with a soft shadow once the page scrolls;
//!   * transparent over the landing hero, solid once scrolled;
//!   * the PAPER | LIVE indicator toggles the shared trading-mode store.

use honba_kao_ui_core::{
    HfButton, HfButtonVariant, HfIconButton, HfTradingModeToggle, HfTradingModeType,
};
use leptos::prelude::*;
use leptos_router::components::A;

use crate::shared::components::BrandLogo;
use crate::shared::contexts::{use_auth, use_language};
use crate::router::paths::{Page, ROUTES};
use crate::shared::stores::{TradingMode, trading_mode::use_trading_mode};
use crate::shared::theme::{Mode, Preference, THEMES, ThemeContext};

#[component]
pub fn HeaderBar(
    #[prop(into, default = Page::Home.into())] current_page: Signal<Page>,
) -> impl IntoView {
    let auth = use_auth();
    let language = use_language();
    let theme = expect_context::<ThemeContext>();
    let trading = use_trading_mode();

    let is_theme_open = RwSignal::new(false);
    let scrolled = RwSignal::new(false);

    install_scroll_listener(scrolled);

    // Translucent over the marketing hero, solid as soon as the page moves.
    let transparent = Memo::new(move |_| current_page.get() == Page::Home && !scrolled.get());

    // The store's TradingMode mapped onto the indicator's wire enum.
    let widget_mode = Signal::derive(move || match trading.mode().get() {
        TradingMode::Paper => HfTradingModeType::Sandbox,
        TradingMode::Live => HfTradingModeType::Live,
    });
    let set_widget_mode =
        Callback::new(move |next: HfTradingModeType| match next {
            HfTradingModeType::Sandbox => trading.set(TradingMode::Paper),
            HfTradingModeType::Live => trading.set(TradingMode::Live),
        });

    view! {
        <header
            class="header-bar"
            class:header-bar--scrolled=move || scrolled.get()
            class:header-bar--transparent=move || transparent.get()
        >
            <div class="header-bar__brand">
                <A href=ROUTES::HOME>
                    <BrandLogo />
                    <span class="header-bar__brand-text">
                        <span class="header-bar__brand-name">
                            {crate::shared::constants::branding::APP_NAME_LATIN}
                        </span>
                        <span class="header-bar__brand-sub">
                            {crate::shared::constants::branding::APP_NAME}
                        </span>
                    </span>
                </A>
            </div>

            <div class="header-bar__spacer" />

            <div class="header-bar__actions">
                <div class="theme-picker">
                    <HfIconButton
                        icon=Signal::derive(move || {
                            if theme.is_dark() {
                                honba_kao_ui_core::icons::outline::moon()
                            } else {
                                honba_kao_ui_core::icons::outline::sun()
                            }
                        })
                        size=16
                        title="Theme"
                        aria_expanded=Signal::derive(move || is_theme_open.get())
                        on_click=Callback::new(move |_| {
                            is_theme_open.update(|open| *open = !*open)
                        })
                    />

                    {move || {
                        is_theme_open
                            .get()
                            .then(|| {
                                view! {
                                    <div class="theme-menu" role="menu" aria-label="Theme">
                                        <HfButton
                                            class="theme-menu__item"
                                            role="menuitemradio"
                                            active=Signal::derive(move || {
                                                theme.preference.get() == Preference::System
                                            })
                                            on_click=Callback::new(move |_| {
                                                theme.set(Preference::System);
                                                is_theme_open.set(false);
                                            })
                                        >
                                            <span class="theme-menu__swatch theme-menu__swatch--auto" />
                                            <span class="theme-menu__label">"Follow system"</span>
                                        </HfButton>

                                        {[Mode::Dark, Mode::Light]
                                            .into_iter()
                                            .map(|mode| {
                                                view! {
                                                    <div class="theme-menu__group">
                                                        {match mode {
                                                            Mode::Dark => "Dark",
                                                            Mode::Light => "Light",
                                                        }}
                                                    </div>
                                                }
                                                .collect_view()}
                                            })
                                            .collect_view()}
                                    </div>
                                }
                            })
                            .collect_view()}
                </div>

                <HfIconButton
                    class="header-bar__lang"
                    title="Language"
                    on_click=Callback::new(move |_| language.toggle())
                >
                    {move || language.language.get().label()}
                </HfIconButton>

                {move || if auth.is_authenticated() {
                    // The single user entry point is the rail's bottom-left
                    // account cluster (nav_rail.rs) — the header hosts only
                    // the togglable paper/live book indicator.
                    view! {
                        <HfTradingModeToggle
                            mode=widget_mode
                            on_change=set_widget_mode
                            class="header-bar__trading-mode"
                        />
                    }
                } else {
                    view! {
                        <HfButton
                            class="header-bar__cta"
                            href=ROUTES::LOGIN
                        >
                            {move || language.t("auth.sign_in")}
                        </HfButton>
                    }
                }}
            </div>
        </header>
    }
}

/// Keep `scrolled` in sync with the window scroll position. The listener is
/// removed when the header's owner is cleaned up.
fn install_scroll_listener(scrolled: RwSignal<bool>) {
    let handle = window_event_listener(leptos::ev::scroll, move |_| {
        let y = web_sys::window()
            .and_then(|window| window.scroll_y().ok())
            .unwrap_or(0.0);
        scrolled.set(y > 8.0);
    });
    on_cleanup(move || handle.remove());
}
