//! Top navigation bar modeled after bfx-hf-ui-core Navbar.

use crate::theme::ThemeContext;
use crate::ui::{ButtonVariant, HfButton};
use leptos::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NavRoute {
    Dashboard,
    Traders,
    Positions,
    Orders,
    Decisions,
}

#[component]
pub fn Navbar(
    active_route: RwSignal<NavRoute>,
    is_live_mode: RwSignal<bool>,
    on_open_palette: Callback<()>,
) -> impl IntoView {
    let theme_ctx = expect_context::<ThemeContext>();

    let toggle_mode = move |_| {
        is_live_mode.update(|m| *m = !*m);
    };

    let toggle_theme = move |_| {
        theme_ctx.toggle();
    };

    let open_palette = move |_| {
        on_open_palette.run(());
    };

    view! {
        <nav class="hfui-navbar__wrapper">
            <div class="hfui-navbar__logo-group">
                <div class="hfui-navbar__logo">
                    <span>"本場・顔"</span>
                    <span class="hfui-navbar__logo-tag">"TERMINAL"</span>
                </div>

                <ul class="hfui-navbar__main-links">
                    <li
                        class=move || format!("hfui-navbar__link {}", if active_route.get() == NavRoute::Dashboard { "active" } else { "" })
                        on:click=move |_| active_route.set(NavRoute::Dashboard)
                    >
                        "Dashboard"
                    </li>
                    <li
                        class=move || format!("hfui-navbar__link {}", if active_route.get() == NavRoute::Traders { "active" } else { "" })
                        on:click=move |_| active_route.set(NavRoute::Traders)
                    >
                        "Traders"
                    </li>
                    <li
                        class=move || format!("hfui-navbar__link {}", if active_route.get() == NavRoute::Positions { "active" } else { "" })
                        on:click=move |_| active_route.set(NavRoute::Positions)
                    >
                        "Positions"
                    </li>
                    <li
                        class=move || format!("hfui-navbar__link {}", if active_route.get() == NavRoute::Orders { "active" } else { "" })
                        on:click=move |_| active_route.set(NavRoute::Orders)
                    >
                        "Orders"
                    </li>
                    <li
                        class=move || format!("hfui-navbar__link {}", if active_route.get() == NavRoute::Decisions { "active" } else { "" })
                        on:click=move |_| active_route.set(NavRoute::Decisions)
                    >
                        "Decision Log"
                    </li>
                </ul>
            </div>

            <div class="hfui-navbar__right">
                // Mode Toggle (Sandbox vs Live)
                <div
                    class=move || format!("hfui-navbar__mode-toggle {}", if is_live_mode.get() { "live" } else { "sandbox" })
                    on:click=toggle_mode
                    title="Click to toggle Trading Mode"
                >
                    {move || if is_live_mode.get() { "● LIVE NSE" } else { "○ SANDBOX" }}
                </div>

                // Theme Switcher Button
                <HfButton
                    variant=ButtonVariant::Ghost
                    small=true
                    on_click=Callback::new(toggle_theme)
                >
                    {move || match theme_ctx.theme.get() {
                        crate::theme::Theme::Dark => "☀ Light",
                        crate::theme::Theme::Light => "☾ Dark",
                    }}
                </HfButton>

                // 256 Color Palette Studio Button
                <HfButton
                    variant=ButtonVariant::Default
                    small=true
                    on_click=Callback::new(open_palette)
                >
                    "🎨 256 Palette"
                </HfButton>
            </div>
        </nav>
    }
}
