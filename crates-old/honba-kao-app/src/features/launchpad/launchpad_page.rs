//! Launchpad mosaic: one outer horizontal split, with the left column split
//! vertically. Three placeholder slots, each with a pop-out button.

use honba_kao_ui_core::{
    HfButton, HfButtonSize, HfButtonVariant, HfPageHeader, HfPanel, HfPanelHeader,
    HfResizableDirection, HfResizableHandle, HfResizablePanel, HfResizablePanelGroup,
};
use leptos::prelude::*;

/// Pop-out window size, passed to `window.open` as a feature string.
const POPOUT_FEATURES: &str = "popup,width=800,height=600";

/// Open `/equity/launchpad/popout?panel=<slug>` in a small popup window.
/// A blocked or failed popup is ignored, as the page has no fallback.
fn pop_out(slug: &str) {
    if let Some(window) = web_sys::window() {
        let url = format!("/equity/launchpad/popout?panel={slug}");
        window
            .open_with_url_and_target_and_features(&url, "_blank", POPOUT_FEATURES)
            .ok();
    }
}

#[component]
fn Slot(slug: &'static str) -> impl IntoView {
    let title = super::panel_title(slug).unwrap_or(slug);

    view! {
        <HfPanel class="launchpad__slot">
            <HfPanelHeader title=title>
                <HfButton
                    size=HfButtonSize::Small
                    variant=HfButtonVariant::Ghost
                    on_click=Callback::new(move |_| pop_out(slug))
                >
                    "Pop out"
                </HfButton>
            </HfPanelHeader>
            <div class="launchpad__body placeholder__note">
                {format!("{title} · mock data")}
            </div>
        </HfPanel>
    }
}

#[component]
pub fn LaunchpadPage() -> impl IntoView {
    view! {
        <div class="console launchpad">
            <HfPageHeader
                title="Launchpad"
                subtitle="Mosaic workspace · pop out any panel · mock data"
            />

            <HfResizablePanelGroup direction=HfResizableDirection::Horizontal class="launchpad__group">
                <HfResizablePanel default_size=60.0 min_size=20.0>
                    <HfResizablePanelGroup direction=HfResizableDirection::Vertical class="launchpad__group">
                        <HfResizablePanel default_size=55.0 min_size=20.0>
                            <Slot slug="chart" />
                        </HfResizablePanel>
                        <HfResizableHandle with_handle=true />
                        <HfResizablePanel default_size=45.0 min_size=20.0>
                            <Slot slug="watchlist" />
                        </HfResizablePanel>
                    </HfResizablePanelGroup>
                </HfResizablePanel>
                <HfResizableHandle with_handle=true />
                <HfResizablePanel default_size=40.0 min_size=20.0>
                    <Slot slug="news" />
                </HfResizablePanel>
            </HfResizablePanelGroup>
        </div>
    }
}
