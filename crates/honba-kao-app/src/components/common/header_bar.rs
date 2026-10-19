//! Top application bar: brand → spacer → actions → avatar menu.
//! Navigation lives in the side icon rail (`nav_rail.rs`), so the bar only
//! carries the brand block, the theme/language controls and account actions.
//!
//! Behaviours:
//!   * sticky, and it lifts with a soft shadow once the page scrolls;
//!   * transparent over the landing hero, solid once scrolled;
//!   * account actions live in one avatar menu instead of separate buttons.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;

use crate::components::common::BrandLogo;
use crate::components::modals::PaletteStudio;
use crate::contexts::{use_auth, use_language};
use crate::router::paths::{Page, ROUTES};
use crate::theme::{Mode, Preference, THEMES, ThemeContext};

#[component]
pub fn HeaderBar(#[prop(default = Page::Home)] current_page: Page) -> impl IntoView {
    let auth = use_auth();
    let language = use_language();
    let theme = expect_context::<ThemeContext>();

    let is_theme_open = RwSignal::new(false);
    let is_palette_open = RwSignal::new(false);
    let scrolled = RwSignal::new(false);

    install_scroll_listener(scrolled);

    // Translucent over the marketing hero, solid as soon as the page moves.
    let transparent = Memo::new(move |_| current_page == Page::Home && !scrolled.get());

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
                            {crate::constants::branding::APP_NAME_LATIN}
                        </span>
                        <span class="header-bar__brand-sub">
                            {crate::constants::branding::APP_NAME}
                        </span>
                    </span>
                </A>
            </div>

            <div class="header-bar__spacer" />

            <div class="header-bar__actions">
                <div class="theme-picker">
                    <button
                        class="icon-button"
                        title="Theme"
                        aria-label="Theme"
                        aria-haspopup="menu"
                        aria-expanded=move || is_theme_open.get().to_string()
                        on:click=move |_| is_theme_open.update(|open| *open = !*open)
                    >
                        {move || {
                            if theme.is_dark() {
                                view! {
                                    <svg
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="1.8"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        aria-hidden="true"
                                    >
                                        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                                    </svg>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <svg
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="1.8"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        aria-hidden="true"
                                    >
                                        <circle cx="12" cy="12" r="4" />
                                        <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
                                    </svg>
                                }
                                    .into_any()
                            }
                        }}
                    </button>

                    {move || {
                        is_theme_open
                            .get()
                            .then(|| {
                                view! {
                                    <div class="theme-menu" role="menu" aria-label="Theme">
                                        <button
                                            class=move || {
                                                if theme.preference.get() == Preference::System {
                                                    "theme-menu__item theme-menu__item--active"
                                                } else {
                                                    "theme-menu__item"
                                                }
                                            }
                                            role="menuitemradio"
                                            on:click=move |_| {
                                                theme.set(Preference::System);
                                                is_theme_open.set(false);
                                            }
                                        >
                                            <span class="theme-menu__swatch theme-menu__swatch--auto" />
                                            <span class="theme-menu__label">"Follow system"</span>
                                        </button>

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
                                                    {THEMES
                                                        .iter()
                                                        .filter(move |spec| spec.mode == mode)
                                                        .map(|spec| {
                                                            let id = spec.id;
                                                            view! {
                                                                <button
                                                                    class=move || {
                                                                        if theme.current().id == id {
                                                                            "theme-menu__item theme-menu__item--active"
                                                                        } else {
                                                                            "theme-menu__item"
                                                                        }
                                                                    }
                                                                    role="menuitemradio"
                                                                    on:click=move |_| {
                                                                        theme.set(Preference::Theme(id));
                                                                        is_theme_open.set(false);
                                                                    }
                                                                >
                                                                    <span class="theme-menu__swatch" data-theme=id />
                                                                    <span class="theme-menu__label">{spec.label}</span>
                                                                </button>
                                                            }
                                                        })
                                                        .collect_view()}
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                }
                            })
                    }}
                </div>

                <button
                    class="icon-button header-bar__lang"
                    title="Language"
                    aria-label="Language"
                    on:click=move |_| language.toggle()
                >
                    {move || language.language.get().label()}
                </button>

                {move || if auth.is_authenticated() {
                    view! { <UserMenu palette_open=is_palette_open /> }.into_any()
                } else {
                    view! {
                        <span class="header-bar__cta">
                            <A href=ROUTES::LOGIN>{move || language.t("auth.sign_in")}</A>
                        </span>
                    }
                        .into_any()
                }}
            </div>

            <PaletteStudio is_open=is_palette_open />
        </header>
    }
}

/// Account menu: avatar trigger plus a panel with identity, settings and exit.
#[component]
fn UserMenu(palette_open: RwSignal<bool>) -> impl IntoView {
    let auth = use_auth();
    // Stored rather than captured so the menu view stays `FnMut`.
    let navigate = StoredValue::new(use_navigate());
    let open = RwSignal::new(false);

    view! {
        <div class="user-menu">
            <button
                class="user-menu__trigger"
                aria-haspopup="menu"
                aria-expanded=move || open.get().to_string()
                aria-label="Account menu"
                on:click=move |_| open.update(|value| *value = !*value)
            >
                <span class="header-bar__avatar">
                    {move || crate::utils::text::initials(&auth.display_name())}
                </span>
                <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                >
                    <path d="m6 9 6 6 6-6" />
                </svg>
            </button>

            {move || {
                open.get()
                    .then(|| {
                        view! {
                            <div class="user-menu__panel" role="menu">
                                <div class="user-menu__head">
                                    <span class="header-bar__avatar">
                                        {crate::utils::text::initials(&auth.display_name())}
                                    </span>
                                    <div class="user-menu__identity">
                                        <div class="user-menu__name">{move || auth.display_name()}</div>
                                        <div class="user-menu__email">
                                            {move || {
                                                auth.user
                                                    .get()
                                                    .map(|user| user.email)
                                                    .unwrap_or_else(|| "signed out".to_string())
                                            }}
                                        </div>
                                    </div>
                                </div>

                                <div class="user-menu__item" on:click=move |_| open.set(false)>
                                    <A href=ROUTES::SETTINGS>"Settings"</A>
                                </div>
                                <div class="user-menu__item" on:click=move |_| open.set(false)>
                                    <A href=ROUTES::PORTFOLIO>"Portfolio"</A>
                                </div>

                                <button
                                    class="user-menu__item"
                                    on:click=move |_| {
                                        open.set(false);
                                        palette_open.set(true);
                                    }
                                >
                                    "Palette studio"
                                </button>

                                <div class="user-menu__sep" />

                                <button
                                    class="user-menu__item user-menu__item--danger"
                                    on:click=move |_| {
                                        auth.logout();
                                        open.set(false);
                                        navigate
                                            .with_value(|nav| {
                                                nav(ROUTES::HOME, Default::default());
                                            });
                                    }
                                >
                                    "Sign out"
                                </button>
                            </div>
                        }
                    })
            }}
        </div>
    }
}

/// Keep `scrolled` in sync with the window, using a passive listener.
fn install_scroll_listener(scrolled: RwSignal<bool>) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let reader = window.clone();
    let on_scroll = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        scrolled.set(reader.scroll_y().unwrap_or(0.0) > 8.0);
    });

    let _ = window.add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
    // The header lives for the whole session; leaking one closure is cheaper
    // than threading an owner through every route.
    on_scroll.forget();
}
