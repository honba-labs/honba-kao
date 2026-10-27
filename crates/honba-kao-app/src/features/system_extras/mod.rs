//! System domain (plan.md §2.1 "System") — the plugins, saved views,
//! account and about screens that round out the domain alongside Settings
//! (handled in `features::account`). Real implementations are tracked
//! separately in `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn PluginsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Plugins"
            subtitle="Installed broker, data-provider and indicator plugins."
            sections=vec![
                ("Installed Plugins", "Enabled broker adapters, data providers and indicator packs."),
                ("Plugin Marketplace", "Browse and install additional plugins."),
            ]
        />
    }
}

#[component]
pub fn SavedViewsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Saved Views"
            subtitle="Saved workspace layouts and mosaic presets."
            sections=vec![
                ("Saved Layouts", "Named mosaic and panel layouts a user can restore in one click."),
                ("Share & Export", "Export a saved view for use on another device or by a teammate."),
            ]
        />
    }
}

#[component]
pub fn AccountHomePage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Account"
            subtitle="Profile, API keys and session management."
            sections=vec![
                ("Profile", "Display name, email and identity details."),
                ("API Keys & Sessions", "Active broker API keys and signed-in session list."),
            ]
        />
    }
}

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="About"
            subtitle="Build version, release notes and acknowledgements."
            sections=vec![
                ("Build Info", "Application version, build commit and target architecture."),
                ("Acknowledgements", "Open-source projects and design inspirations behind honba-kao."),
            ]
        />
    }
}
