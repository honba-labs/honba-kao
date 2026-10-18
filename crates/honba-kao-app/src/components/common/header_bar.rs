//! Top navigation bar. Mirrors `web/src/components/common/HeaderBar.tsx`.
//!
//! Active-link styling comes from `<A>` setting `aria-current="page"`, so the
//! stylesheet targets `a[aria-current="page"]` rather than threading an active
//! class through props.

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::components::common::BrandLogo;
use crate::components::modals::PaletteStudio;
use crate::constants::branding;
use crate::contexts::{use_auth, use_language};
use crate::router::paths::{Page, ROUTES};
use crate::theme::{Theme, ThemeContext};

struct NavItem {
    href: &'static str,
    key: &'static str,
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        href: ROUTES::TRADERS,
        key: "nav.traders",
    },
    NavItem {
        href: ROUTES::DASHBOARD,
        key: "nav.dashboard",
    },
    NavItem {
        href: ROUTES::STRATEGY,
        key: "nav.strategy",
    },
    NavItem {
        href: ROUTES::COMPETITION,
        key: "nav.competition",
    },
    NavItem {
        href: ROUTES::PLANNING,
        key: "nav.planning",
    },
    NavItem {
        href: ROUTES::DATA,
        key: "nav.data",
    },
    NavItem {
        href: ROUTES::FAQ,
        key: "nav.faq",
    },
];

#[component]
pub fn HeaderBar(
    #[prop(default = Page::Home)] current_page: Page,
    on_login_required: Callback<String>,
) -> impl IntoView {
    let auth = use_auth();
    let language = use_language();
    let theme = expect_context::<ThemeContext>();
    let is_palette_open = RwSignal::new(false);

    view! {
        <header class="header-bar">
            <div class="header-bar__brand">
                <BrandLogo />
                <div class="header-bar__brand-text">
                    <span class="header-bar__brand-name">{branding::APP_NAME}</span>
                    <span class="header-bar__brand-sub">{branding::APP_NAME_LATIN}</span>
                </div>
            </div>

            <nav class="header-bar__nav">
                {NAV_ITEMS
                    .iter()
                    .map(|item| {
                        view! {
                            <A href=item.href>{move || language.t(item.key)}</A>
                        }
                    })
                    .collect_view()}
            </nav>

            <span class="header-bar__crumb">{current_page.label()}</span>

            <div class="header-bar__spacer" />

            <button
                class="header-bar__btn"
                title="Toggle language"
                on:click=move |_| language.toggle()
            >
                {move || language.language.get().label()}
            </button>

            <button class="header-bar__btn" title="Toggle theme" on:click=move |_| theme.toggle()>
                {move || match theme.theme.get() {
                    Theme::Dark => "☀",
                    Theme::Light => "☾",
                }}
            </button>

            <button
                class="header-bar__btn"
                title="256-colour palette studio"
                on:click=move |_| is_palette_open.set(true)
            >
                "🎨"
            </button>

            {move || if auth.is_authenticated() {
                view! {
                    <div class="header-bar__user">
                        <span class="header-bar__avatar">
                            {crate::utils::text::initials(&auth.display_name())}
                        </span>
                        <SignOutButton />
                        <span class="header-bar__cta">
                            <A href=ROUTES::TRADERS>"Open console"</A>
                        </span>
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <button
                        class="header-bar__btn"
                        on:click=move |_| on_login_required.run("Console".to_string())
                    >
                        "Request access"
                    </button>
                    <span class="header-bar__cta">
                        <A href=ROUTES::LOGIN>{move || language.t("auth.sign_in")}</A>
                    </span>
                }
                    .into_any()
            }}

            <PaletteStudio is_open=is_palette_open />
        </header>
    }
}

/// Separate component so `use_navigate` isn't captured inside a re-runnable
/// reactive closure in [`HeaderBar`].
#[component]
fn SignOutButton() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    view! {
        <button
            class="header-bar__btn"
            on:click=move |_| {
                auth.logout();
                navigate(ROUTES::HOME, Default::default());
            }
        >
            "Sign out"
        </button>
    }
}
