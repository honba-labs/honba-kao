//! Pinned bottom status bar modeled after bfx-hf-ui-core StatusBar.

use crate::theme::ThemeContext;
use crate::ui::{CircleStatus, HfStatusCircle};
use leptos::prelude::*;

#[component]
pub fn StatusBar() -> impl IntoView {
    let theme_ctx = expect_context::<ThemeContext>();

    view! {
        <footer class="hfui-statusbar__wrapper">
            <div class="hfui-statusbar__left">
                <div class="hfui-statusbar__item">
                    <HfStatusCircle status=CircleStatus::Green />
                    <span>"API Connected (/api/health)"</span>
                </div>
                <div class="hfui-statusbar__divider" />
                <div class="hfui-statusbar__item">
                    <HfStatusCircle status=CircleStatus::Green />
                    <span>"WS Connected (14ms)"</span>
                </div>
                <div class="hfui-statusbar__divider" />
                <div class="hfui-statusbar__item">
                    <span>"NSE: "</span>
                    <strong style="color: var(--primary-8); margin-left: 4px;">"REGULAR TRADING"</strong>
                </div>
            </div>

            <div class="hfui-statusbar__right">
                <div class="hfui-statusbar__item">
                    <span>"THEME: "</span>
                    <strong style="color: var(--shade-8); margin-left: 4px;">
                        {move || match theme_ctx.theme.get() {
                            crate::theme::Theme::Dark => "DARK (BFX)",
                            crate::theme::Theme::Light => "LIGHT",
                        }}
                    </strong>
                </div>
                <div class="hfui-statusbar__divider" />
                <div class="hfui-statusbar__item">
                    <span>"honba-kao v0.1.0"</span>
                </div>
            </div>
        </footer>
    }
}
