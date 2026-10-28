//! Side navigation rail. The console's information architecture lives here,
//! top to bottom:
//!
//!   Portfolio · Screener · Workbench · Strategy · Simulator · Tradebook
//!
//! with a **single** account entry pinned to the bottom-left: the initials
//! avatar whose popover carries the account destinations — Portfolio, Palette
//! studio, Settings, Funds, sign out (the header hosts the trading-mode
//! indicator instead of a second user menu).
//!
//! Behaviours:
//!   * one icon per workspace, hover shows a floating label;
//!   * the active route is lit in brand-soft (`<A>`'s `aria-current` drives
//!     the cell highlight through CSS `:has()`);
//!   * signed-out clicks on auth-gated destinations open the
//!     `LoginRequiredOverlay` instead of routing;
//!   * icons are procedural outline glyphs from `honba-kao-ui-core` — no
//!     inline SVG in the chrome.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use honba_kao_ui_core::HfButton;
use honba_kao_ui_core::hooks::use_click_outside;
use honba_kao_ui_core::icons::outline::{self, SvgIcon};

use crate::router::paths::{Page, ROUTES};
use crate::shared::contexts::{use_auth, use_language};

/// One rail destination.
struct RailItem {
    href: &'static str,
    key: &'static str,
    page: Page,
    glyph: &'static str,
}

const RAIL_ITEMS: &[RailItem] = &[
    RailItem {
        href: ROUTES::HOME,
        key: "nav.home",
        page: Page::Home,
        glyph: outline::home(),
    },
    RailItem {
        href: ROUTES::PORTFOLIO,
        key: "nav.portfolio",
        page: Page::Portfolio,
        glyph: outline::briefcase(),
    },
    RailItem {
        href: ROUTES::SCREENER,
        key: "nav.screener",
        page: Page::Screener,
        glyph: outline::search_check(),
    },
    RailItem {
        href: ROUTES::WORKBENCH,
        key: "nav.workbench",
        page: Page::Workbench,
        glyph: outline::terminal(),
    },
    RailItem {
        href: ROUTES::STRATEGY,
        key: "nav.strategy",
        page: Page::Strategy,
        glyph: outline::sliders(),
    },
    RailItem {
        href: ROUTES::SIMULATOR,
        key: "nav.simulator",
        page: Page::Simulator,
        glyph: outline::play_circle(),
    },
    RailItem {
        href: ROUTES::TRADEBOOK,
        key: "nav.tradebook",
        page: Page::Tradebook,
        glyph: outline::book(),
    },
];

#[component]
pub fn NavRail(
    #[prop(into, default = Page::Home.into())] current_page: Signal<Page>,
    on_login_required: Callback<String>,
    /// The chrome-level palette studio flag; the account menu opens it.
    #[prop(into)]
    palette_open: RwSignal<bool>,
) -> impl IntoView {
    let auth = use_auth();
    let language = use_language();

    view! {
        <nav class="nav-rail" aria-label="Primary">
            {move || {
                RAIL_ITEMS
                    .iter()
                    .map(|item| {
                        let key = item.key;
                        let href = item.href;
                        let gate = item.page.label();
                        let glyph = item.glyph;
                        let page = item.page;

                        // Signed in: route directly; signed out: login overlay.
                        if auth.is_authenticated() {
                            view! {
                                <div
                                    class="nav-rail__item"
                                    data-label=move || language.t(key).to_string()
                                >
                                    <A href=href>
                                        <SvgIcon path=glyph size=18 />
                                    </A>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <button
                                    class="nav-rail__item"
                                    data-label=move || language.t(key).to_string()
                                    aria-current=move || if current_page.get() == page {
                                        "page"
                                    } else {
                                        ""
                                    }
                                    on:click=move |_| on_login_required.run(gate.to_string())
                                >
                                    <SvgIcon path=glyph size=18 />
                                </button>
                            }
                                .into_any()
                        }
                    })
                    .collect_view()
            }}

            <span class="nav-rail__spacer" />

            <AccountCluster on_login_required palette_open />
        </nav>
    }
}

/// The single account entry: initials avatar pinned to the rail's bottom-left.
/// Guests get the sign-in trigger; hosts get the merged account popover
/// (identity, Portfolio, Palette studio, Settings, Funds, sign out).
#[component]
fn AccountCluster(
    #[prop(into)] on_login_required: Callback<String>,
    /// The chrome-level palette studio flag; the account menu opens it.
    #[prop(into)]
    palette_open: RwSignal<bool>,
) -> impl IntoView {
    let auth = use_auth();
    let language = use_language();
    let navigate = StoredValue::new(use_navigate());
    let open = RwSignal::new(false);
    let cluster = NodeRef::<leptos::html::Div>::new();

    // Clicking anywhere outside the cluster (trigger + popover) closes it.
    use_click_outside(cluster, move || open.set(false));

    view! {
        <div class="nav-rail__user" node_ref=cluster>
            {move || if auth.is_authenticated() {
                let initials = crate::shared::utils::text::initials(&auth.display_name());
                view! {
                    <button
                        class="nav-rail__item nav-rail__user-trigger"
                        data-label=move || language.t("nav.user").to_string()
                        aria-haspopup="menu"
                        aria-expanded=move || open.get().to_string()
                        aria-label="Account menu"
                        on:click=move |_| open.update(|value| *value = !*value)
                    >
                        <span class="nav-rail__avatar">{initials}</span>
                    </button>
                    <Show when=move || open.get()>
                        <AccountMenu
                            palette_open
                            on_close=Callback::new(move |_| open.set(false))
                            on_sign_out=Callback::new(move |_| {
                                open.set(false);
                                auth.logout();
                                navigate.with_value(|nav| nav(ROUTES::HOME, Default::default()));
                            })
                        />
                    </Show>
                }
                    .into_any()
            } else {
                view! {
                    <button
                        class="nav-rail__item"
                        data-label=move || language.t("auth.sign_in").to_string()
                        aria-haspopup="menu"
                        on:click=move |_| on_login_required.run("Workspace".to_string())
                    >
                        <SvgIcon path=outline::user() size=18 />
                    </button>
                }
                    .into_any()
            }}
        </div>
    }
}

/// The merged account popover: identity header, then Portfolio · Palette
/// studio · Settings · Funds, a divider, and sign out.
#[component]
fn AccountMenu(
    on_close: Callback<()>,
    on_sign_out: Callback<()>,
    /// Signal into the chrome-hosted palette studio modal.
    #[prop(into)]
    palette_open: RwSignal<bool>,
) -> impl IntoView {
    let auth = use_auth();
    let navigate = StoredValue::new(use_navigate());

    let go = move |route: &'static str| {
        on_close.run(());
        navigate.with_value(|nav| nav(route, Default::default()));
    };

    let initials = crate::shared::utils::text::initials(&auth.display_name());
    let email = auth
        .user
        .get()
        .map(|user| user.email)
        .unwrap_or_else(|| "signed out".to_string());

    view! {
        <div class="rail-user-menu" role="menu" aria-label="User">
            <div class="rail-user-menu__head">
                <span class="nav-rail__avatar nav-rail__avatar--lg">{initials}</span>
                <div class="rail-user-menu__identity">
                    <div class="rail-user-menu__name">{move || auth.display_name()}</div>
                    <div class="rail-user-menu__email">{email}</div>
                </div>
            </div>

            <div class="rail-user-menu__sep" />

            <RailMenuItem
                glyph=outline::briefcase()
                label="Portfolio"
                on_click=Callback::new(move |_| go(ROUTES::PORTFOLIO))
            />
            <RailMenuItem
                glyph=outline::settings()
                label="Settings"
                on_click=Callback::new(move |_| go(ROUTES::SETTINGS))
            />
            <RailMenuItem
                glyph=outline::wallet()
                label="Funds"
                on_click=Callback::new(move |_| go(ROUTES::FUNDS))
            />
            <RailMenuItem
                glyph=outline::layout_grid()
                label="Palette studio"
                on_click=Callback::new(move |_| {
                    on_close.run(());
                    palette_open.set(true);
                })
            />

            <div class="rail-user-menu__sep" />

            <button
                class="rail-user-menu__item rail-user-menu__item--danger"
                role="menuitem"
                on:click=move |_| on_sign_out.run(())
            >
                <SvgIcon path=outline::log_out() size=15 />
                "Sign out"
            </button>
        </div>
    }
}

/// One popover row: outline glyph + label.
#[component]
fn RailMenuItem(
    /// Path data from `honba_kao_ui_core::icons::outline`.
    #[prop(into)]
    glyph: &'static str,
    #[prop(into)] label: &'static str,
    on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <HfButton unstyled=true role="menuitem" on_click=Callback::new(move |_| on_click.run(()))>
            <SvgIcon path=glyph size=15 />
            {label}
        </HfButton>
    }
}
