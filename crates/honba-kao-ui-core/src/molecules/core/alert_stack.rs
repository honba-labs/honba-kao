//! `HfAlertBanner` and `HfAlertBannerStack` — contextual notification banners
//! positioned above workspace content (Secured-Finance multi-tier alert hierarchy).

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertSeverity {
    #[default]
    Info,
    Warning,
    Error,
    Success,
}

impl AlertSeverity {
    fn class(self) -> &'static str {
        match self {
            AlertSeverity::Info => "bg-blue-600/20 border-blue-500/30 text-blue-400",
            AlertSeverity::Warning => "bg-amber-600/20 border-amber-500/30 text-amber-400",
            AlertSeverity::Error => "bg-rose-600/20 border-rose-500/30 text-rose-400",
            AlertSeverity::Success => "bg-emerald-600/20 border-emerald-500/30 text-emerald-400",
        }
    }
}

#[component]
pub fn HfAlertBanner(
    #[prop(into, optional)] severity: Option<AlertSeverity>,
    #[prop(into)] message: String,
    #[prop(default = false)] dismissible: bool,
    #[prop(into, optional)] on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let severity = severity.unwrap_or_default();
    let class = severity.class();
    let on_dismiss = on_dismiss.unwrap_or_else(|| Callback::new(move |_| {}));

    view! {
        <div class=format!("hf-alert-banner flex items-center gap-2 px-3 py-2 rounded border {}", class)>
            <span class="hf-alert-icon text-[11px]">
                {match severity {
                    AlertSeverity::Info => "ℹ",
                    AlertSeverity::Warning => "⚠",
                    AlertSeverity::Error => "✖",
                    AlertSeverity::Success => "✔",
                }}
            </span>
            <span class="text-xs text-text-high">{message}</span>
            {dismissible.then(|| view! {
                <button
                    type="button"
                    class="hf-alert-dismiss text-text-muted hover:text-text-high ml-auto"
                    on:click=move |_| on_dismiss.run(())
                    aria-label="Dismiss"
                >
                    <span class="hf-icon hf-icon-close" aria_hidden="true">"✕"</span>
                </button>
            }).into_view()}
        </div>
    }
}

/// Structural notification header stack positioned above workspace content.
#[component]
pub fn HfAlertBannerStack(
    banners: Signal<Vec<(AlertSeverity, String)>>,
) -> impl IntoView {
    view! {
        <div class="hf-alert-banner-stack flex flex-col gap-1 mb-3">
            {move || banners.get().into_iter().enumerate().map(|(_i, (sev, msg))| {
                view! {
                    <HfAlertBanner severity=sev message=msg dismissible=true />
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
