//! Bottom status strip for the terminal console (honba-kao flavour; the
//! reference app folds this into the dashboard).

use leptos::prelude::*;

use crate::components::ui::{CircleStatus, HfStatusCircle};
use crate::theme::ThemeContext;

#[component]
pub fn StatusBar() -> impl IntoView {
    let theme = expect_context::<ThemeContext>();

    view! {
        <footer class="hfui-statusbar__wrapper">
            <div class="hfui-statusbar__left">
                <div class="hfui-statusbar__item">
                    <HfStatusCircle status=CircleStatus::Green />
                    <span>"API connected (/api/health)"</span>
                </div>
                <div class="hfui-statusbar__divider" />
                <div class="hfui-statusbar__item">
                    <HfStatusCircle status=CircleStatus::Green />
                    <span>"WS connected (14ms)"</span>
                </div>
                <div class="hfui-statusbar__divider" />
                <div class="hfui-statusbar__item">
                    <span>"NSE: "</span>
                    <strong style="color: var(--primary-8); margin-left: 4px;">
                        "REGULAR TRADING"
                    </strong>
                </div>
            </div>

            <div class="hfui-statusbar__right">
                <div class="hfui-statusbar__item">
                    <span>"THEME: "</span>
                    <strong style="color: var(--shade-8); margin-left: 4px;">
                        {move || theme.current().label.to_uppercase()}
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
