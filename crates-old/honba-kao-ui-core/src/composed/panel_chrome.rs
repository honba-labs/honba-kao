//! HfPanelChrome — `HfPanel` header with a title, a symbol link-group
//! selector (A–D) and a popout action. Window handling is the caller's job:
//! the popout button only emits `on_popout`.

use leptos::prelude::*;

use crate::composed::link_group::LinkGroup;
use crate::icons::outline;
use crate::primitive::button::{HfButton, HfButtonSize, HfButtonVariant, HfIconButton};
use crate::primitive::panel::HfPanelHeader;

/// Panel header with title, link-group selector and popout button.
#[component]
pub fn HfPanelChrome(
    /// Panel title.
    title: &'static str,
    /// Currently linked group (reactive).
    #[prop(into)]
    group: Signal<LinkGroup>,
    /// Fires with the group the user picked.
    on_group_change: Callback<LinkGroup>,
    /// Fires when the popout button is clicked.
    on_popout: Callback<()>,
) -> impl IntoView {
    let selector = LinkGroup::LINKED
        .into_iter()
        .map(|option| {
            let active = Signal::derive(move || group.get() == option);
            view! {
                <HfButton
                    variant=HfButtonVariant::Ghost
                    size=HfButtonSize::Small
                    active=active
                    class="hf-panel-chrome__group"
                    title=format!("Link group {}", option.letter())
                    on_click=Callback::new(move |_| on_group_change.run(option))
                >
                    {option.letter()}
                </HfButton>
            }
        })
        .collect_view();

    let actions = move || {
        view! {
            <div class="hf-panel-chrome__groups" role="group" aria-label="Symbol link group">
                {selector}
            </div>
            <HfIconButton
                icon=outline::external_link()
                title="Pop out panel"
                class="hf-panel-chrome__popout"
                on_click=Callback::new(move |_| on_popout.run(()))
            />
        }
        .into_any()
    };

    view! {
        <HfPanelHeader title=title children=Box::new(actions) />
    }
}
