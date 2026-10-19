//! Side navigation rail. The console's information architecture lives here,
//! top to bottom:
//!
//!   Portfolio · Screener · Workbench · Strategy · Simulator · Tradebook
//!
//! plus a User cluster pinned to the bottom (Settings, Funds, sign out).
//!
//! Behaviours:
//!   * one icon per workspace, hover shows a floating label;
//!   * the active route is lit in brand-soft (`<A>`'s `aria-current` drives
//!     the cell highlight through CSS `:has()`);
//!   * signed-out clicks on auth-gated destinations open the
//!     `LoginRequiredOverlay` instead of routing.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::contexts::{use_auth, use_language};
use crate::router::paths::{Page, ROUTES};

/// One rail destination.
struct RailItem {
    href: &'static str,
    key: &'static str,
    page: Page,
    icon: RailIcon,
}

#[derive(Copy, Clone)]
enum RailIcon {
    Portfolio,
    Screener,
    Workbench,
    Strategy,
    Simulator,
    Tradebook,
}

const RAIL_ITEMS: &[RailItem] = &[
    RailItem {
        href: ROUTES::PORTFOLIO,
        key: "nav.portfolio",
        page: Page::Portfolio,
        icon: RailIcon::Portfolio,
    },
    RailItem {
        href: ROUTES::SCREENER,
        key: "nav.screener",
        page: Page::Screener,
        icon: RailIcon::Screener,
    },
    RailItem {
        href: ROUTES::WORKBENCH,
        key: "nav.workbench",
        page: Page::Workbench,
        icon: RailIcon::Workbench,
    },
    RailItem {
        href: ROUTES::STRATEGY,
        key: "nav.strategy",
        page: Page::Strategy,
        icon: RailIcon::Strategy,
    },
    RailItem {
        href: ROUTES::SIMULATOR,
        key: "nav.simulator",
        page: Page::Simulator,
        icon: RailIcon::Simulator,
    },
    RailItem {
        href: ROUTES::TRADEBOOK,
        key: "nav.tradebook",
        page: Page::Tradebook,
        icon: RailIcon::Tradebook,
    },
];

fn icon(icon: RailIcon) -> impl IntoView {
    match icon {
        RailIcon::Portfolio => view! {
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="3" y="7" width="18" height="13" rx="2" />
                <path d="M8 7V5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                <path d="M3 12h18" />
            </svg>
        }
            .into_any(),
        RailIcon::Screener => view! {
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="11" cy="11" r="7" />
                <path d="m21 21-4.5-4.5" />
                <path d="M8 11.5 10 14l4.5-5" />
            </svg>
        }
            .into_any(),
        RailIcon::Workbench => view! {
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="3" y="4" width="18" height="16" rx="2" />
                <path d="m7 9 3 3-3 3" />
                <path d="M12 15h5" />
            </svg>
        }
            .into_any(),
        RailIcon::Strategy => view! {
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M4 21v-7" />
                <path d="M4 10V3" />
                <path d="M12 21v-9" />
                <path d="M12 8V3" />
                <path d="M20 21v-5" />
                <path d="M20 12V3" />
                <path d="M1 14h6" />
                <path d="M9 8h6" />
                <path d="M17 16h6" />
            </svg>
        }
            .into_any(),
        RailIcon::Simulator => view! {
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="12" cy="12" r="9" />
                <path d="m10 8 6 4-6 4z" />
            </svg>
        }
            .into_any(),
        RailIcon::Tradebook => view! {
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
                <path d="M9 7h7" />
                <path d="M9 10.5h5" />
            </svg>
        }
            .into_any(),
    }
}

/// User icon for the bottom cluster.
fn user_icon() -> impl IntoView {
    view! {
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="8" r="4" />
            <path d="M4.5 21c0-3.9 3.4-6.5 7.5-6.5s7.5 2.6 7.5 6.5" />
        </svg>
    }
}

#[component]
pub fn NavRail(
    #[prop(default = Page::Home)] current_page: Page,
    on_login_required: Callback<String>,
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
                        let glyph = icon(item.icon);

                        // Signed in: route directly; signed out: login overlay.
                        if auth.is_authenticated() {
                            view! {
                                <div
                                    class="nav-rail__item"
                                    data-label=move || language.t(key).to_string()
                                >
                                    <A href=href>{glyph}</A>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <button
                                    class="nav-rail__item"
                                    data-label=move || language.t(key).to_string()
                                    aria-current=if current_page == item.page {
                                        "page"
                                    } else {
                                        ""
                                    }
                                    on:click=move |_| on_login_required.run(gate.to_string())
                                >
                                    {icon(item.icon)}
                                </button>
                            }
                                .into_any()
                        }
                    })
                    .collect_view()
            }}

            <span class="nav-rail__spacer" />

            <UserCluster on_login_required />
        </nav>
    }
}

/// Bottom-of-rail user cluster: a User icon whose popover carries the account
/// destinations — Settings, Funds and sign out. Guests get the login overlay.
#[component]
fn UserCluster(#[prop(into)] on_login_required: Callback<String>) -> impl IntoView {
    let auth = use_auth();
    let language = use_language();
    let navigate = StoredValue::new(use_navigate());
    let open = RwSignal::new(false);

    view! {
        <div class="nav-rail__user">
            {move || if auth.is_authenticated() {
                view! {
                    <button
                        class="nav-rail__item"
                        data-label=move || language.t("nav.user").to_string()
                        aria-haspopup="menu"
                        aria-expanded=move || open.get().to_string()
                        on:click=move |_| open.update(|value| *value = !*value)
                    >
                        {user_icon()}
                    </button>
                    <Show when=move || open.get()>
                        <RailUserMenu
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
                        on:click=move |_| on_login_required.run("Workspace".to_string())
                    >
                        {user_icon()}
                    </button>
                }
                    .into_any()
            }}
        </div>
    }
}

/// The popover itself: Settings, Funds, sign out.
#[component]
fn RailUserMenu(
    on_close: Callback<()>,
    on_sign_out: Callback<()>,
) -> impl IntoView {
    let navigate = StoredValue::new(use_navigate());

    let go = move |route: &'static str| {
        on_close.run(());
        navigate.with_value(|nav| nav(route, Default::default()));
    };

    view! {
        <div class="rail-user-menu" role="menu" aria-label="User">
            <button role="menuitem" on:click=move |_| go(ROUTES::SETTINGS)>
                <IconGear />
                "Settings"
            </button>
            <button role="menuitem" on:click=move |_| go(ROUTES::FUNDS)>
                <IconWallet />
                "Funds"
            </button>

            <div class="rail-user-menu__sep" />

            <button
                class="rail-user-menu__item--danger"
                role="menuitem"
                on:click=move |_| on_sign_out.run(())
            >
                <IconLogout />
                "Sign out"
            </button>
        </div>
    }
}

#[component]
fn IconGear() -> impl IntoView {
    view! {
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
    }
}

#[component]
fn IconWallet() -> impl IntoView {
    view! {
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="2" y="6" width="20" height="14" rx="2" />
            <path d="M16 6V4a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h2" />
            <path d="M22 12h-5a2 2 0 0 0 0 4h5z" />
        </svg>
    }
}

#[component]
fn IconLogout() -> impl IntoView {
    view! {
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
            <path d="m16 17 5-5-5-5" />
            <path d="M21 12H9" />
        </svg>
    }
}
