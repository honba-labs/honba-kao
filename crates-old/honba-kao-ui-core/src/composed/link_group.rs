//! Symbol link groups: the colour-tagged cluster a panel joins so that its
//! symbol follows every other panel in the same group.
//!
//! Modelled on OpenTerminalUI `contexts/SymbolLinkContext.tsx`
//! (`none | red | blue | green | yellow`, cycled in that order). The plan
//! names the four groups `A`–`D`; they map to red, blue, green and yellow
//! respectively, and [`LinkGroup::letter`] returns that letter for display.

use leptos::prelude::*;

use crate::{HfButton, HfButtonSize, HfButtonVariant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum LinkGroup {
    #[default]
    None,
    Red,
    Blue,
    Green,
    Yellow,
}

impl LinkGroup {
    /// All linked groups, in cycle order (excludes `None`).
    pub const LINKED: [LinkGroup; 4] = [
        LinkGroup::Red,
        LinkGroup::Blue,
        LinkGroup::Green,
        LinkGroup::Yellow,
    ];

    /// Next group in the cycle: `None -> Red -> Blue -> Green -> Yellow -> None`.
    pub fn cycle(self) -> LinkGroup {
        match self {
            LinkGroup::None => LinkGroup::Red,
            LinkGroup::Red => LinkGroup::Blue,
            LinkGroup::Blue => LinkGroup::Green,
            LinkGroup::Green => LinkGroup::Yellow,
            LinkGroup::Yellow => LinkGroup::None,
        }
    }

    /// Display letter for the plan's A–D naming; `-` for unlinked panels.
    pub fn letter(self) -> &'static str {
        match self {
            LinkGroup::None => "-",
            LinkGroup::Red => "A",
            LinkGroup::Blue => "B",
            LinkGroup::Green => "C",
            LinkGroup::Yellow => "D",
        }
    }

    /// Wire / URL name, matching OpenTerminalUI (`linkGroup=red`).
    pub fn as_str(self) -> &'static str {
        match self {
            LinkGroup::None => "none",
            LinkGroup::Red => "red",
            LinkGroup::Blue => "blue",
            LinkGroup::Green => "green",
            LinkGroup::Yellow => "yellow",
        }
    }

    /// Parses the wire name; unknown values are `None`.
    pub fn from_name(name: &str) -> LinkGroup {
        match name.trim().to_ascii_lowercase().as_str() {
            "red" => LinkGroup::Red,
            "blue" => LinkGroup::Blue,
            "green" => LinkGroup::Green,
            "yellow" => LinkGroup::Yellow,
            _ => LinkGroup::None,
        }
    }

    fn tone(self) -> &'static str {
        self.as_str()
    }
}

/// The link tag shown in a panel header. Clicking cycles to the next group.
#[component]
pub fn HfLinkGroupTag(
    group: Signal<LinkGroup>,
    /// Called with the next group when the tag is clicked.
    on_change: Callback<LinkGroup>,
) -> impl IntoView {
    view! {
        <HfButton
            variant=HfButtonVariant::Ghost
            size=HfButtonSize::Small
            class="hf-link-tag"
            title="Symbol link group. Click to cycle."
            on_click=Callback::new(move |_| on_change.run(group.get_untracked().cycle()))
        >
            <span class=move || format!("hf-link-tag__chip hf-link-tag__chip--{}", group.get().tone())>
                {move || group.get().letter()}
            </span>
        </HfButton>
    }
}
