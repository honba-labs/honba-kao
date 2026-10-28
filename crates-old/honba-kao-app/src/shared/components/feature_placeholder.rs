//! `FeaturePlaceholderPage` — the shared structural placeholder used by
//! routes whose real feature work is tracked separately in `plan.md`'s
//! phase checklist. Renders a page header plus a grid of descriptive
//! `HfPanel` cards, built entirely from `honba-kao-ui-core` primitives so
//! new routes never need to hand-roll raw markup (and never trip the
//! design-system lint).

use leptos::prelude::*;

use honba_kao_ui_core::{HfPageHeader, HfPanel, HfPanelHeader};

/// One descriptive card: a heading and a short description of what the
/// finished feature will contain.
pub type PlaceholderSection = (&'static str, &'static str);

#[component]
pub fn FeaturePlaceholderPage(
    title: &'static str,
    #[prop(default = "")] subtitle: &'static str,
    sections: Vec<PlaceholderSection>,
) -> impl IntoView {
    view! {
        <section class="p-6">
            <HfPageHeader title=title subtitle=subtitle />
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2 mt-4">
                {sections
                    .into_iter()
                    .map(|(heading, body)| {
                        view! {
                            <HfPanel class="p-4".to_string()>
                                <HfPanelHeader title=heading />
                                <p class="text-sm text-muted p-4">{body}</p>
                            </HfPanel>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}
